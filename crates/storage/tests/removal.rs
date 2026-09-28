//! Removing a device.
//!
//! Removal stops trust; it cannot reach into the removed device, and does not
//! pretend to (brief §35). What it must get right is everything *here* that
//! went on counting on that device: sends waiting for it, what this device
//! keeps for it, and -- the one that loses data -- copies it was known to hold,
//! which freeing space treats as the reason it is safe to drop a local copy.

use qurb_storage::{ChunkKey, Store};
use qurb_sync::{Area, Content, DeviceId, FileVersion, VersionVector};

struct Device {
    dir: tempfile::TempDir,
    store: Store,
}

impl Device {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("folder");
        std::fs::create_dir_all(&root).unwrap();
        let store = Store::open(&root.join(".qurb"), ChunkKey::from_bytes([21; 32]))
            .unwrap()
            .in_tree(&root);
        Self { dir, store }
    }

    fn add(&mut self, name: &str, bytes: &[u8]) {
        let path = self.dir.path().join("folder").join(name);
        std::fs::write(&path, bytes).unwrap();
        self.store.put_file(name, &path).unwrap();
    }

    fn send(&mut self, name: &str, bytes: &[u8], to: &DeviceId) {
        let loose = self.dir.path().join(format!("loose-{name}"));
        std::fs::write(&loose, bytes).unwrap();
        self.store.send_to_vault(name, &loose, to).unwrap();
    }

    fn hold(&mut self, owner: &DeviceId, name: &str, bytes: &[u8]) {
        let mut vector = VersionVector::new();
        vector.increment(*owner);
        let version = FileVersion {
            path: name.to_string(),
            content: Content::File { hash: *blake3::hash(bytes).as_bytes(), size: bytes.len() as u64 },
            vector,
            modified_by: *owner,
            modified_at: 1_790_000_000,
            area: Area::Hold,
        };
        let staged = self.dir.path().join("staged");
        std::fs::write(&staged, bytes).unwrap();
        self.store.hold_file(&version, owner, &staged).unwrap();
    }

    fn outgoing(&self) -> Vec<String> {
        self.store.pending_deliveries().unwrap().into_iter().map(|(p, _, _)| p).collect()
    }
}

fn paired(device: &Device, name: &str, byte: u8) -> DeviceId {
    let other = DeviceId::from_bytes([byte; 32]);
    device.store.db().trust_peer(&other, &[byte; 32], name).unwrap();
    other
}

#[test]
fn it_is_no_longer_trusted_and_the_history_says_so_by_name() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "SM-S911B", 7);

    let plan = laptop.store.remove_device(&phone, "SM-S911B", false).unwrap();
    assert_eq!(plan, Default::default(), "nothing here depended on it");

    assert!(laptop.store.db().trusted_peers().unwrap().is_empty());
    let last = laptop.store.db().activity(1, None).unwrap().remove(0);
    assert_eq!(last.kind, qurb_storage::db::Event::Removed);
    assert_eq!(last.detail.as_deref(), Some("SM-S911B"));

    // And the history of it still says who, not a short id.
    assert_eq!(laptop.store.db().device_names().unwrap().get(&phone).map(String::as_str), Some("SM-S911B"));
}

/// Nothing can collect a send to a device that is no longer trusted, so it is
/// cancelled rather than left waiting for ever.
#[test]
fn a_send_it_never_collected_is_cancelled() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "phone", 7);
    laptop.send("report.pdf", b"the report", &phone);

    let plan = laptop.store.removal_plan(&phone).unwrap();
    assert_eq!(plan.waiting, vec![("report.pdf".to_string(), 10)]);

    laptop.store.remove_device(&phone, "phone", false).unwrap();
    assert!(laptop.outgoing().is_empty());
}

/// One it did collect stays collected: removal must not make it look as if
/// it is waiting again.
#[test]
fn a_send_it_collected_does_not_start_waiting_again() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "phone", 7);
    laptop.send("report.pdf", b"the report", &phone);
    laptop.store.note_replica_in_vault(&blake3::hash(b"the report"), &phone).unwrap();
    assert!(laptop.outgoing().is_empty());

    laptop.store.remove_device(&phone, "phone", false).unwrap();
    assert!(laptop.outgoing().is_empty(), "{:?}", laptop.outgoing());
}

/// Its own files, kept here for it, may exist nowhere else. They stay unless
/// the person removing it says to delete them.
#[test]
fn what_this_device_keeps_for_it_stays_unless_asked() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "phone", 7);
    laptop.hold(&phone, "IMG_0001.jpg", b"a photo");
    laptop.store.db().add_holder(&phone).unwrap();

    let plan = laptop.store.removal_plan(&phone).unwrap();
    assert_eq!(plan.kept_for_it, vec![("IMG_0001.jpg".to_string(), 7)]);

    laptop.store.remove_device(&phone, "phone", false).unwrap();
    assert_eq!(laptop.store.db().vault_contents(&phone).unwrap().len(), 1);
    assert!(laptop.store.db().holders().unwrap().is_empty(), "it still keeps this device's files");
}

#[test]
fn what_this_device_keeps_for_it_goes_when_asked() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "phone", 7);
    laptop.hold(&phone, "IMG_0001.jpg", b"a photo");

    laptop.store.remove_device(&phone, "phone", true).unwrap();
    assert!(laptop.store.db().vault_contents(&phone).unwrap().is_empty());
}

/// The one that loses data. A copy on the removed device can no longer be
/// asked for, so it must stop being the reason freeing a local copy is safe.
#[test]
fn a_copy_on_it_stops_counting_as_a_copy() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "phone", 7);
    laptop.add("thesis.pdf", b"years of work");
    laptop.store.note_replica(&blake3::hash(b"years of work"), &phone).unwrap();

    laptop.store.remove_device(&phone, "phone", false).unwrap();
    assert!(
        laptop.store.free_local("thesis.pdf").is_err(),
        "the only copy was freed on the strength of a device nothing will connect to again"
    );
}

/// And a file already freed on its strength is named before removing it,
/// because afterwards it has nowhere to come back from.
#[test]
fn a_file_only_it_keeps_is_named_first() {
    let mut laptop = Device::new();
    let phone = paired(&laptop, "phone", 7);
    let tablet = paired(&laptop, "tablet", 9);
    laptop.add("only-phone.txt", b"on the phone");
    laptop.add("both.txt", b"on both");
    laptop.store.note_replica(&blake3::hash(b"on the phone"), &phone).unwrap();
    laptop.store.note_replica(&blake3::hash(b"on both"), &phone).unwrap();
    laptop.store.note_replica(&blake3::hash(b"on both"), &tablet).unwrap();
    laptop.store.free_local("only-phone.txt").unwrap();
    laptop.store.free_local("both.txt").unwrap();

    let plan = laptop.store.removal_plan(&phone).unwrap();
    assert_eq!(plan.only_there, vec!["only-phone.txt".to_string()]);
}
