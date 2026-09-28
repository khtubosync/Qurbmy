//! A folder this device keeps only remotely (decision 0045, brief §29).
//!
//! Listed, not downloaded; a local copy freed only where another device keeps
//! it; brought back when asked for. The safety rule underneath is the storage
//! cap's: the only copy of anything is never the one freed.

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
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("sync");
        fs::create_dir_all(&root).unwrap();
        let store_dir = root.join(".qurb");
        let store = Store::open(&store_dir, ChunkKey::from_bytes([42; 32])).unwrap().in_tree(&root);
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

    fn here(&self, rel: &str) -> Option<bool> {
        self.engine.store().is_materialised(rel).unwrap()
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
fn keeping_a_folder_remotely_frees_only_what_another_device_has() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("Photos/shared.jpg", "on both");
    sync(&mut laptop, &mut phone);
    laptop.write("Photos/only-here.jpg", "nowhere else");
    // What the network would have told the laptop: the phone has it.
    laptop.engine.store().note_replica(&blake3::hash(b"on both"), &phone.engine.store().device_id().unwrap()).unwrap();

    let (freed, _, kept) = laptop.engine.store_mut().keep_remotely("Photos").unwrap();
    assert_eq!(freed, 1);
    assert_eq!(kept, vec!["Photos/only-here.jpg"], "the only copy was freed");
    assert_eq!(laptop.read("Photos/shared.jpg"), None);
    assert_eq!(laptop.here("Photos/shared.jpg"), Some(false), "still listed");
    assert_eq!(laptop.read("Photos/only-here.jpg").as_deref(), Some("nowhere else"));
}

#[test]
fn a_new_file_there_is_listed_and_not_downloaded() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("Photos/first.jpg", "first");
    sync(&mut laptop, &mut phone);
    phone.engine.store().db().set_kept_remotely("Photos", true).unwrap();

    laptop.write("Photos/new.jpg", "taken on the laptop");
    laptop.write("notes.txt", "elsewhere");
    sync(&mut laptop, &mut phone);

    assert_eq!(phone.read("Photos/new.jpg"), None, "downloaded into a folder kept remotely");
    assert_eq!(phone.here("Photos/new.jpg"), Some(false));
    assert_eq!(phone.read("notes.txt").as_deref(), Some("elsewhere"), "other folders are untouched");

    // Asked for, it comes.
    assert!(phone.engine.store().db().want("Photos/new.jpg").unwrap());
    sync(&mut laptop, &mut phone);
    assert_eq!(phone.read("Photos/new.jpg").as_deref(), Some("taken on the laptop"));
}

/// A file already here stays up to date: remote-only is about what arrives,
/// not a reason to let a file somebody opened go stale.
#[test]
fn a_file_already_here_is_kept_up_to_date() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("Photos/open.jpg", "version one");
    sync(&mut laptop, &mut phone);
    phone.engine.store().db().set_kept_remotely("Photos", true).unwrap();

    laptop.write("Photos/open.jpg", "version two");
    sync(&mut laptop, &mut phone);
    assert_eq!(phone.read("Photos/open.jpg").as_deref(), Some("version two"));
}

#[test]
fn keeping_it_locally_again_brings_everything_back() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("Photos/a.jpg", "a");
    sync(&mut laptop, &mut phone);
    phone.engine.store().db().set_kept_remotely("Photos", true).unwrap();
    laptop.write("Photos/b.jpg", "b");
    sync(&mut laptop, &mut phone);
    assert_eq!(phone.read("Photos/b.jpg"), None);

    assert_eq!(phone.engine.store_mut().keep_locally("Photos").unwrap(), 1);
    sync(&mut laptop, &mut phone);
    assert_eq!(phone.read("Photos/b.jpg").as_deref(), Some("b"));
    laptop.write("Photos/c.jpg", "c");
    sync(&mut laptop, &mut phone);
    assert_eq!(phone.read("Photos/c.jpg").as_deref(), Some("c"), "new files arrive again");
}
