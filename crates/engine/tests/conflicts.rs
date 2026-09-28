//! Conflicts a person settles (brief §24).
//!
//! Two devices change one file without either seeing the other; both versions
//! are kept, one under a conflict name (decision 0005). These are about what
//! happens next: every device recognising the conflict, and a choice made on
//! one of them reaching the rest -- without losing the version not chosen.

use qurb_engine::{Engine, StoreSource};
use qurb_storage::{ChunkKey, Keep, Store};
use qurb_watcher::IgnoreRules;
use std::fs;
use std::path::PathBuf;

struct Device {
    _dir: tempfile::TempDir,
    root: PathBuf,
    engine: Engine,
}

impl Device {
    fn new() -> Self {
        Self::with_privacy(false)
    }

    /// A phone: files new things privately (decision 0036), which is what
    /// qurb writing a file itself must not fall into.
    fn phone() -> Self {
        Self::with_privacy(true)
    }

    fn with_privacy(private: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("sync");
        fs::create_dir_all(&root).unwrap();
        let store_dir = root.join(".qurb");
        let mut store = Store::open(&store_dir, ChunkKey::from_bytes([42; 32])).unwrap().in_tree(&root);
        store.set_new_files_private(private);
        let ignore = IgnoreRules::new().with_store_dir(&store_dir);
        Self { _dir: dir, root: root.clone(), engine: Engine::new(root, store, ignore) }
    }

    fn write(&mut self, rel: &str, contents: &str) {
        fs::write(self.root.join(rel), contents).unwrap();
        self.engine.reconcile().unwrap();
    }

    fn read(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.root.join(rel)).ok()
    }

    fn files(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.root)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n != ".qurb")
            .collect();
        names.sort();
        names
    }
}

fn sync(a: &mut Device, b: &mut Device) {
    for _ in 0..8 {
        let a_plan = a.engine.plan_against(&b.engine.tree().unwrap()).unwrap();
        let b_plan = b.engine.plan_against(&a.engine.tree().unwrap()).unwrap();
        if a_plan.is_empty() && b_plan.is_empty() {
            return;
        }
        let stats = a.engine.apply_plan(&a_plan, &mut StoreSource::new(b.engine.store())).unwrap();
        assert!(stats.is_clean(), "{:?}", stats.failures);
        let stats = b.engine.apply_plan(&b_plan, &mut StoreSource::new(a.engine.store())).unwrap();
        assert!(stats.is_clean(), "{:?}", stats.failures);
    }
    panic!("did not converge");
}

/// A laptop and a phone that both changed `plan.txt`, synced: one conflict.
/// The phone files new things privately, as a real one does.
fn conflicted() -> (Device, Device) {
    let (mut laptop, mut phone) = (Device::new(), Device::phone());
    laptop.write("plan.txt", "the first plan");
    sync(&mut laptop, &mut phone);
    laptop.write("plan.txt", "the laptop's plan");
    phone.write("plan.txt", "the phone's plan");
    sync(&mut laptop, &mut phone);
    (laptop, phone)
}

#[test]
fn both_devices_recognise_the_conflict() {
    let (laptop, phone) = conflicted();
    for device in [&laptop, &phone] {
        let found = device.engine.store().conflicts().unwrap();
        assert_eq!(found.len(), 1, "{:?}", device.files());
        assert_eq!(found[0].original_path, "plan.txt");
        assert!(found[0].original.is_some());
        assert!(found[0].copy.here);
    }
}

fn settle(device: &mut Device, keep: Keep) -> String {
    let copy = device.engine.store().conflicts().unwrap().remove(0).copy.path;
    device.engine.store_mut().settle_conflict(&copy, keep, "SM-S911B").unwrap()
}

#[test]
fn keeping_the_named_version_settles_it_everywhere() {
    let (mut laptop, mut phone) = conflicted();
    let kept_text = laptop.read("plan.txt").unwrap();
    settle(&mut phone, Keep::Original);
    sync(&mut laptop, &mut phone);

    for device in [&laptop, &phone] {
        assert!(device.engine.store().conflicts().unwrap().is_empty());
        assert_eq!(device.files(), vec!["plan.txt"]);
        assert_eq!(device.read("plan.txt"), Some(kept_text.clone()));
    }
    // And the version not kept is not gone.
    let deleted = phone.engine.store().recently_deleted().unwrap();
    assert_eq!(deleted.len(), 1);
    assert!(deleted[0].why.as_deref().unwrap().contains("not kept"));
}

#[test]
fn keeping_the_other_version_puts_it_under_the_name() {
    let (mut laptop, mut phone) = conflicted();
    let copy = phone.engine.store().conflicts().unwrap().remove(0).copy.path;
    let other_text = phone.read(&copy).unwrap();
    settle(&mut phone, Keep::Copy);
    sync(&mut laptop, &mut phone);

    for device in [&laptop, &phone] {
        assert!(device.engine.store().conflicts().unwrap().is_empty());
        assert_eq!(device.files(), vec!["plan.txt"]);
        assert_eq!(device.read("plan.txt"), Some(other_text.clone()));
    }
    assert_eq!(phone.engine.store().recently_deleted().unwrap().len(), 1);
}

#[test]
fn keeping_both_gives_the_other_a_name_a_person_can_read() {
    let (mut laptop, mut phone) = conflicted();
    let at = settle(&mut phone, Keep::Both);
    assert_eq!(at, "plan (SM-S911B).txt");
    sync(&mut laptop, &mut phone);

    for device in [&laptop, &phone] {
        assert!(device.engine.store().conflicts().unwrap().is_empty());
        assert_eq!(device.files(), vec!["plan (SM-S911B).txt", "plan.txt"]);
    }
    assert!(phone.engine.store().recently_deleted().unwrap().is_empty(), "nothing was discarded");
}

/// A device name is chosen by that device; it must not make a path.
#[test]
fn a_hostile_label_does_not_escape_the_folder() {
    let (_, mut phone) = conflicted();
    let copy = phone.engine.store().conflicts().unwrap().remove(0).copy.path;
    let at = phone.engine.store_mut().settle_conflict(&copy, Keep::Both, "../../evil\n").unwrap();
    assert_eq!(at, "plan (....evil).txt");
}

#[test]
fn a_file_that_only_looks_like_a_conflict_is_left_alone() {
    let mut laptop = Device::new();
    laptop.write("plan.conflict-notes.txt", "my own notes");
    assert!(laptop.engine.store().conflicts().unwrap().is_empty());
    assert!(laptop
        .engine
        .store_mut()
        .settle_conflict("plan.conflict-notes.txt", Keep::Original, "x")
        .is_err());
    assert_eq!(laptop.read("plan.conflict-notes.txt").as_deref(), Some("my own notes"));
}
