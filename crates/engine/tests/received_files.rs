//! A file somebody sent you is yours, and stays private.
//!
//! The folder holds two kinds of file: content shared with every device, and
//! content sent to *this* device. Both are written into the same folder,
//! because a person asked for a file and should find a file — but only one of
//! them is anybody else's business.
//!
//! Everything that walks the folder has to know that. This is the test that
//! says so, and it was written after watching a phone receive a file, index it
//! as shared on its next scan, and advertise it straight back to the laptop
//! that had sent it — which then wrote it into the user's synced folder.

use qurb_engine::{Engine, Error, PlanStats, StoreSource};
use qurb_storage::{ChunkKey, Store};
use qurb_sync::{Content, FileVersion, VersionVector};
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
        let store = Store::open(&store_dir, ChunkKey::from_bytes([31; 32])).unwrap();
        let ignore = IgnoreRules::new().with_store_dir(&store_dir);
        Self { _dir: dir, root: root.clone(), engine: Engine::new(root, store, ignore) }
    }

    /// What this device would tell a peer with nothing waiting for it: the
    /// shared area, and only that.
    fn advertises(&self) -> Vec<String> {
        self.engine.tree().unwrap().into_iter().map(|v| v.path).collect()
    }

    /// Take whatever the peer's shared area says, the way a sync would.
    fn sync_from(&mut self, peer: &Device) -> PlanStats {
        let tree = peer.engine.tree().unwrap();
        let plan = self.engine.plan_against(&tree).unwrap();
        let reader = Store::open(&peer.root.join(".qurb"), ChunkKey::from_bytes([31; 32]))
            .unwrap()
            .in_tree(&peer.root);
        let mut content = StoreSource::new(&reader);
        self.engine.apply_plan(&plan, &mut content).unwrap()
    }

    /// Every live path in the index, shared or private.
    fn live(&self) -> Vec<String> {
        self.engine
            .store()
            .tree()
            .unwrap()
            .into_iter()
            .filter(|v| !v.is_deleted())
            .map(|v| v.path)
            .collect()
    }

    /// Where each live row at this path sits, shared first.
    ///
    /// Asked of the table directly rather than by path through `scope_of`,
    /// which answers for one row and so cannot tell a received file from a
    /// shared namesake -- the very case several of these tests are about.
    fn scopes(&self, path: &str) -> Vec<String> {
        let me = self.engine.store().device_id().unwrap();
        let db = self.engine.store().db();
        let mut stmt = db
            .conn()
            .prepare(
                "SELECT scope FROM files WHERE path = ?1 AND deleted_at IS NULL
                  ORDER BY scope IS NOT NULL",
            )
            .unwrap();
        stmt.query_map([path], |r| r.get::<_, Option<Vec<u8>>>(0))
            .unwrap()
            .map(|scope| match scope.unwrap() {
                None => "shared".to_string(),
                Some(bytes) if bytes == me.as_bytes() => "mine".to_string(),
                Some(_) => "someone else's".to_string(),
            })
            .collect()
    }
}

/// Somebody sent this device a file: written into the folder, scoped to this
/// device, and adopted the way a delivery is.
fn receive(device: &mut Device, name: &str, contents: &[u8]) -> FileVersion {
    let sender = qurb_sync::DeviceId::from_bytes([0xCC; 32]);
    let mut vector = VersionVector::new();
    vector.increment(sender);

    let version = FileVersion {
        path: name.to_string(),
        content: Content::File { hash: *blake3::hash(contents).as_bytes(), size: contents.len() as u64 },
        vector,
        modified_by: sender,
        modified_at: 1_790_000_000,
        area: qurb_sync::Area::Sent,
    };

    // Written where a delivery writes it, then adopted privately — the two
    // halves `Engine::take` performs.
    let path = device.root.join(name);
    fs::write(&path, contents).unwrap();
    device.engine.store_mut().adopt_file_privately(&version, &path, 0).unwrap();
    version
}

#[test]
fn a_received_file_is_not_advertised_to_anybody() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");

    assert!(
        device.advertises().is_empty(),
        "a file sent to this device was advertised: {:?}",
        device.advertises()
    );
}

/// The one that was found on hardware.
///
/// The file is in the folder, so the next scan of the folder finds it. Before
/// this, the scan looked only at shared rows, concluded the file was new, and
/// indexed it as shared — at which point it was advertised to every device,
/// including the one that had sent it.
#[test]
fn a_scan_does_not_turn_a_received_file_into_a_shared_one() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");

    device.engine.reconcile().unwrap();

    assert_eq!(
        device.scopes("holiday.jpg"),
        vec!["mine"],
        "a scan re-filed a received file"
    );
    assert!(
        device.advertises().is_empty(),
        "a scan made a received file public: {:?}",
        device.advertises()
    );
}

/// Repeatedly, because the daemon reconciles at startup and after every dropped
/// event, and a bug that needs two passes is still a bug.
#[test]
fn it_survives_being_scanned_again_and_again() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");

    for _ in 0..3 {
        device.engine.reconcile().unwrap();
    }

    assert_eq!(device.scopes("holiday.jpg"), vec!["mine"]);
    assert!(device.advertises().is_empty());
}

/// Deleting a received file is the recipient's business, and has to work.
/// Before, the scan could not see the row at all, so the deletion went
/// unnoticed and the file stayed in the index for ever.
#[test]
fn deleting_a_received_file_is_noticed() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");
    device.engine.reconcile().unwrap();

    fs::remove_file(device.root.join("holiday.jpg")).unwrap();
    device.engine.reconcile().unwrap();

    let live = device.live();
    assert!(live.is_empty(), "a deleted received file is still live: {live:?}");
}

/// The phone saves a copy out of its store, and a file sent to it is the
/// likeliest thing to be saved. Before, reading one by name answered "not
/// found", because the read looked only at the shared area.
#[test]
fn a_received_file_can_be_read_back() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");

    assert_eq!(
        device.engine.store().read_file("holiday.jpg").unwrap(),
        b"for you and nobody else"
    );
}

/// Deleting one must not leave the index claiming bytes nobody has. The file
/// in the folder *was* the payload store for it, so once the file is gone its
/// chunk references have to go too, exactly as for a shared file.
#[test]
fn deleting_a_received_file_leaves_nothing_dangling() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");

    // The way the phone deletes: the file, then the store told directly.
    fs::remove_file(device.root.join("holiday.jpg")).unwrap();
    device.engine.store_mut().delete_file("holiday.jpg").unwrap();

    assert!(device.live().is_empty(), "still live: {:?}", device.live());
    let report = device.engine.store().verify(false).unwrap();
    assert!(report.is_healthy(), "deleting a received file damaged the store: {report:?}");
}

/// Editing a file somebody sent you does not publish it. A file that became
/// shared because it was opened and saved would be the worse of the two
/// mistakes.
#[test]
fn an_edited_received_file_stays_private() {
    let mut device = Device::new();
    receive(&mut device, "holiday.jpg", b"for you and nobody else");
    device.engine.reconcile().unwrap();

    fs::write(device.root.join("holiday.jpg"), b"cropped, and still only for you").unwrap();
    device.engine.reconcile().unwrap();

    assert_eq!(device.scopes("holiday.jpg"), vec!["mine"]);
    assert!(device.advertises().is_empty(), "an edit published it: {:?}", device.advertises());
    assert_eq!(
        device.engine.store().read_file("holiday.jpg").unwrap(),
        b"cropped, and still only for you"
    );
}

/// A shared file of the same name was deleted before this one arrived, so the
/// index holds a shared tombstone and a live private row at one path. Anything
/// that asks "what is in the folder at this path" has to find the live one;
/// finding the tombstone made the received file impossible to delete.
#[test]
fn a_received_file_where_a_shared_one_was_deleted() {
    let mut device = Device::new();
    fs::write(device.root.join("holiday.jpg"), b"everybody's").unwrap();
    device.engine.reconcile().unwrap();
    fs::remove_file(device.root.join("holiday.jpg")).unwrap();
    device.engine.reconcile().unwrap();

    receive(&mut device, "holiday.jpg", b"for you and nobody else");
    device.engine.reconcile().unwrap();
    // The shared tombstone is still advertised, so that other devices learn of
    // the deletion. Nothing live is.
    let shared_live: Vec<String> = device
        .engine
        .tree()
        .unwrap()
        .into_iter()
        .filter(|v| !v.is_deleted())
        .map(|v| v.path)
        .collect();
    assert!(shared_live.is_empty(), "a received file was published: {shared_live:?}");
    assert_eq!(device.live(), vec!["holiday.jpg".to_string()]);

    fs::remove_file(device.root.join("holiday.jpg")).unwrap();
    device.engine.reconcile().unwrap();

    assert!(device.live().is_empty(), "a deleted received file is still live: {:?}", device.live());
    let report = device.engine.store().verify(false).unwrap();
    assert!(report.is_healthy(), "{report:?}");
}

/// A shared file is still a shared file. The whole point of the fix is to tell
/// the two apart, so the ordinary case has to keep working.
#[test]
fn an_ordinary_file_is_still_shared() {
    let mut device = Device::new();
    fs::write(device.root.join("notes.txt"), b"everybody").unwrap();
    device.engine.reconcile().unwrap();

    assert_eq!(device.scopes("notes.txt"), vec!["shared"]);
    assert_eq!(device.advertises(), vec!["notes.txt".to_string()]);
}

/// Another device deleted a shared file of the same name. That deletion is
/// about the shared area, and the file here is not in it.
///
/// Before, the deletion removed whatever was at the path on disk -- the file
/// somebody had sent -- and the next scan then recorded it as deleted by the
/// person who had just been given it.
#[test]
fn a_deletion_elsewhere_leaves_a_received_namesake_alone() {
    let mut peer = Device::new();
    fs::write(peer.root.join("notes.txt"), b"everybody's").unwrap();
    peer.engine.reconcile().unwrap();
    fs::remove_file(peer.root.join("notes.txt")).unwrap();
    peer.engine.reconcile().unwrap();

    let mut device = Device::new();
    receive(&mut device, "notes.txt", b"for you and nobody else");

    let stats = device.sync_from(&peer);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);

    assert_eq!(
        fs::read(device.root.join("notes.txt")).unwrap(),
        b"for you and nobody else",
        "a shared deletion removed a private file"
    );
    device.engine.reconcile().unwrap();
    assert_eq!(device.live(), vec!["notes.txt".to_string()]);
    assert_eq!(
        device.engine.store().read_file("notes.txt").unwrap(),
        b"for you and nobody else"
    );
}

/// A shared file arriving where a received one already has the name. The
/// received file's bytes live nowhere else here, so writing over it would lose
/// it. Refused instead, and the shared file arrives once the name is free.
#[test]
fn a_shared_file_waits_for_a_received_namesake() {
    let mut peer = Device::new();
    fs::write(peer.root.join("notes.txt"), b"everybody's").unwrap();
    peer.engine.reconcile().unwrap();

    let mut device = Device::new();
    receive(&mut device, "notes.txt", b"for you and nobody else");

    let stats = device.sync_from(&peer);
    assert_eq!(stats.failures.len(), 1, "the clash should have been refused");
    assert!(
        matches!(stats.failures[0].error, Error::TakenPrivately { .. }),
        "got {:?}",
        stats.failures[0].error
    );
    assert_eq!(
        fs::read(device.root.join("notes.txt")).unwrap(),
        b"for you and nobody else",
        "the received file was overwritten"
    );

    // The person deletes the received file; the shared one is not lost.
    fs::remove_file(device.root.join("notes.txt")).unwrap();
    device.engine.reconcile().unwrap();
    let stats = device.sync_from(&peer);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);
    assert_eq!(fs::read(device.root.join("notes.txt")).unwrap(), b"everybody's");
}

/// A shared version from another device, adopted at a path where a received
/// file already lives, stays shared.
///
/// The rule that keeps a received file private when it is edited applies to
/// changes made *here*. Applied to a version from elsewhere, it re-filed the
/// shared file into this device's vault -- taking it out of the shared area on
/// this device alone, and overwriting the received file's record with it.
#[test]
fn a_version_from_elsewhere_is_not_refiled_as_private() {
    let mut device = Device::new();
    receive(&mut device, "notes.txt", b"for you and nobody else");

    let peer = qurb_sync::DeviceId::from_bytes([0xDD; 32]);
    let mut vector = VersionVector::new();
    vector.increment(peer);
    let shared = FileVersion {
        path: "notes.txt".to_string(),
        content: Content::File { hash: *blake3::hash(b"everybody's").as_bytes(), size: 11 },
        vector,
        modified_by: peer,
        modified_at: 1_790_000_000,
        area: qurb_sync::Area::Shared,
    };
    device.engine.store_mut().adopt(&shared, Some(b"everybody's"), 0).unwrap();

    assert_eq!(device.scopes("notes.txt"), vec!["shared", "mine"]);
    assert_eq!(
        fs::read(device.root.join("notes.txt")).unwrap(),
        b"for you and nobody else",
        "the received file on disk was touched"
    );
}

/// A received file, deleted, stays deleted -- past the retention window too.
///
/// The sender goes on offering what it sent for as long as it keeps the entry.
/// The only record of having taken it was the file's own row, and a deleted
/// file's row is a tombstone that garbage collection expires; the daemon keeps
/// tombstones for seven days. So a week after somebody deleted a file they had
/// been sent, it arrived again.
#[test]
fn a_deleted_delivery_is_not_offered_again_after_collection() {
    let mut device = Device::new();
    let version = receive(&mut device, "holiday.jpg", b"for you and nobody else");

    fs::remove_file(device.root.join("holiday.jpg")).unwrap();
    device.engine.reconcile().unwrap();
    // No retention at all: every tombstone expires now, as it would a week on.
    device.engine.store_mut().gc(std::time::Duration::ZERO).unwrap();

    let plan = device.engine.plan_against(&[version]).unwrap();
    assert!(plan.is_empty(), "a deleted delivery was offered again: {plan:?}");
}
