//! Renaming and moving a file from qurb itself, as the phone's Vault does.
//!
//! The rule that matters: a file keeps its area. A phone files new things
//! privately (decision 0036), and a shared file renamed there must not quietly
//! become private -- vanishing from every other device as if deleted.

use qurb_engine::{Engine, StoreSource};
use qurb_storage::{ChunkKey, Store};
use qurb_sync::Action;
use qurb_watcher::IgnoreRules;
use std::fs;
use std::path::PathBuf;

struct Device {
    _dir: tempfile::TempDir,
    root: PathBuf,
    engine: Engine,
}

impl Device {
    fn new(private: bool) -> Self {
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
        let path = self.root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
        self.engine.reconcile().unwrap();
    }

    fn read(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.root.join(rel)).ok()
    }

    fn private(&self, rel: &str) -> Option<bool> {
        self.engine.store().db().folder_row(rel).unwrap().map(|(_, scope)| scope.is_some())
    }
}

fn sync(a: &mut Device, b: &mut Device) {
    for _ in 0..8 {
        let a_plan = a.engine.plan_with(&b.engine.tree().unwrap(), None).unwrap();
        let b_plan = b.engine.plan_with(&a.engine.tree().unwrap(), None).unwrap();
        let idle = |plan: &[Action]| plan.iter().all(|a| matches!(a, Action::Offer { .. }));
        if idle(&a_plan) && idle(&b_plan) {
            return;
        }
        let stats = a.engine.apply_plan(&a_plan, &mut StoreSource::new(b.engine.store())).unwrap();
        assert!(stats.is_clean(), "{:?}", stats.failures);
        let stats = b.engine.apply_plan(&b_plan, &mut StoreSource::new(a.engine.store())).unwrap();
        assert!(stats.is_clean(), "{:?}", stats.failures);
    }
    panic!("did not settle");
}

#[test]
fn a_shared_file_moved_on_a_phone_stays_shared_and_moves_everywhere() {
    let (mut laptop, mut phone) = (Device::new(false), Device::new(true));
    laptop.write("notes.txt", "for everyone");
    sync(&mut laptop, &mut phone);

    phone.engine.store_mut().rename_file("notes.txt", "work/notes.txt").unwrap();
    assert_eq!(phone.private("work/notes.txt"), Some(false), "a shared file became private");
    sync(&mut laptop, &mut phone);

    assert_eq!(laptop.read("work/notes.txt").as_deref(), Some("for everyone"));
    assert_eq!(laptop.read("notes.txt"), None);
    assert_eq!(phone.read("notes.txt"), None);
}

#[test]
fn a_private_file_renamed_stays_private() {
    let mut phone = Device::new(true);
    phone.write("IMG_0001.jpg", "a photo");
    assert_eq!(phone.private("IMG_0001.jpg"), Some(true));

    phone.engine.store_mut().rename_file("IMG_0001.jpg", "Holiday/beach.jpg").unwrap();
    assert_eq!(phone.private("Holiday/beach.jpg"), Some(true));
    assert_eq!(phone.read("Holiday/beach.jpg").as_deref(), Some("a photo"));
}

#[test]
fn a_name_that_is_taken_or_unsafe_is_refused() {
    let mut phone = Device::new(true);
    phone.write("a.txt", "a");
    phone.write("b.txt", "b");
    let store = phone.engine.store_mut();
    assert!(store.rename_file("a.txt", "b.txt").is_err(), "overwrote another file");
    assert!(store.rename_file("a.txt", "../outside.txt").is_err());
    assert!(store.rename_file("a.txt", ".qurb-sharing/x").is_err());
    assert!(store.rename_file("missing.txt", "c.txt").is_err());
    assert_eq!(phone.read("b.txt").as_deref(), Some("b"));
}
