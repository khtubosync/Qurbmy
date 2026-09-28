//! Recently deleted: a deletion anywhere is recoverable from any device that
//! held the file (decision 0042).
//!
//! Under single-copy storage the file in the folder is the only copy of its
//! bytes on a device. Before this, a deletion on one device removed the file
//! from every other device as it synced, and with it every copy there was.

use qurb_engine::{Engine, StoreSource};
use qurb_storage::{ChunkKey, Store};
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
        fs::write(self.root.join(rel), contents).unwrap();
        self.engine.reconcile().unwrap();
    }

    fn remove(&mut self, rel: &str) {
        fs::remove_file(self.root.join(rel)).unwrap();
        self.engine.reconcile().unwrap();
    }

    fn read(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.root.join(rel)).ok()
    }

    fn deleted(&self) -> Vec<qurb_storage::db::Trashed> {
        self.engine.store().recently_deleted().unwrap()
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

#[test]
fn a_deletion_from_another_device_is_kept_here() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("thesis.txt", "years of work");
    sync(&mut laptop, &mut phone);
    assert_eq!(phone.read("thesis.txt").as_deref(), Some("years of work"));

    laptop.remove("thesis.txt");
    sync(&mut laptop, &mut phone);

    assert_eq!(phone.read("thesis.txt"), None, "the deletion still reaches the folder");
    let kept = phone.deleted();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].path, "thesis.txt");
    assert_eq!(kept[0].size, 13);
    assert_eq!(kept[0].deleted_by, Some(laptop.engine.store().device_id().unwrap()));
}

/// Restored as a change made there, so it comes back on the device that
/// deleted it too -- which had no copy left to restore from.
#[test]
fn restoring_brings_it_back_everywhere() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("thesis.txt", "years of work");
    sync(&mut laptop, &mut phone);
    laptop.remove("thesis.txt");
    sync(&mut laptop, &mut phone);

    let id = phone.deleted()[0].id;
    let at = phone.engine.store_mut().restore_from_trash(id).unwrap();
    assert_eq!(at, "thesis.txt");
    assert!(phone.deleted().is_empty());
    sync(&mut laptop, &mut phone);

    assert_eq!(phone.read("thesis.txt").as_deref(), Some("years of work"));
    assert_eq!(laptop.read("thesis.txt").as_deref(), Some("years of work"));
}

#[test]
fn a_restore_never_overwrites_what_is_there_now() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("plan.txt", "the old plan");
    sync(&mut laptop, &mut phone);
    laptop.remove("plan.txt");
    sync(&mut laptop, &mut phone);
    laptop.write("plan.txt", "a new plan");
    sync(&mut laptop, &mut phone);

    let id = phone.deleted()[0].id;
    let at = phone.engine.store_mut().restore_from_trash(id).unwrap();
    assert_eq!(at, "plan (restored).txt");
    sync(&mut laptop, &mut phone);
    assert_eq!(laptop.read("plan.txt").as_deref(), Some("a new plan"));
    assert_eq!(laptop.read("plan (restored).txt").as_deref(), Some("the old plan"));
}

/// Deleted through qurb itself -- the phone's Vault, a settled conflict -- the
/// device doing it keeps a copy too.
#[test]
fn deleting_through_qurb_keeps_a_copy_here() {
    let mut phone = Device::new();
    phone.write("photo.jpg", "a photo");
    phone.engine.store_mut().delete_to_trash("photo.jpg", None).unwrap();

    assert_eq!(phone.read("photo.jpg"), None);
    assert!(phone.engine.store().db().folder_row("photo.jpg").unwrap().is_none());
    assert_eq!(phone.deleted().len(), 1);
}

#[test]
fn old_entries_expire_and_can_be_deleted_for_good() {
    let mut phone = Device::new();
    phone.write("a.txt", "a");
    phone.write("b.txt", "b");
    phone.engine.store_mut().delete_to_trash("a.txt", None).unwrap();
    phone.engine.store_mut().delete_to_trash("b.txt", None).unwrap();

    // Nothing is thirty days old yet.
    assert_eq!(phone.engine.housekeep(std::time::Duration::from_secs(3600)).unwrap().expired, (0, 0));
    assert_eq!(phone.deleted().len(), 2);

    let id = phone.deleted()[0].id;
    phone.engine.store_mut().forget_deleted(id).unwrap();
    assert_eq!(phone.deleted().len(), 1);
    assert_eq!(phone.engine.store_mut().empty_trash(std::time::Duration::ZERO).unwrap().0, 1);
    assert!(phone.deleted().is_empty());
    assert_eq!(phone.engine.store().usage().unwrap().trash, 0);
}

/// Under a storage limit, deleted files go before any live one.
#[test]
fn the_storage_limit_empties_recently_deleted_first() {
    let mut phone = Device::new();
    phone.write("keep.txt", &"k".repeat(10_000));
    phone.write("gone.txt", &"g".repeat(10_000));
    phone.engine.store_mut().delete_to_trash("gone.txt", None).unwrap();
    let used = phone.engine.store().usage().unwrap().total();

    let stats = phone.engine.enforce_limit(used - 5_000).unwrap();
    assert_eq!(stats.emptied, 10_000);
    assert_eq!(stats.dropped, 0, "a live file went before a deleted one");
    assert_eq!(phone.read("keep.txt").map(|s| s.len()), Some(10_000));
}


/// Found building this, and older than it: a device that deleted a file could
/// not take the same bytes again under any name. Its own tombstone answered
/// "the content is here" with an empty chunk list, the empty list assembled
/// to the wrong bytes, and the file failed as corrupt on every sync.
#[test]
fn content_deleted_here_can_arrive_again() {
    let (mut laptop, mut phone) = (Device::new(), Device::new());
    laptop.write("a.txt", "the same bytes");
    sync(&mut laptop, &mut phone);
    laptop.remove("a.txt");
    sync(&mut laptop, &mut phone);

    phone.write("b.txt", "the same bytes");
    sync(&mut laptop, &mut phone);
    assert_eq!(laptop.read("b.txt").as_deref(), Some("the same bytes"));
}
