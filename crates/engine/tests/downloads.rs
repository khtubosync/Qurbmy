//! A file sent to a desktop is an ordinary file in Downloads.
//!
//! On a device that names a downloads directory, a delivery is written there
//! and qurb stops tracking it: it is not in the folder, not in the index as
//! content, not counted against the storage limit, and deleting it is the
//! person tidying their Downloads. What qurb keeps is the record that it took
//! the delivery, so the sender offering it again changes nothing. See
//! `docs/decisions/0037-a-file-sent-to-a-desktop-is-an-ordinary-file.md`.

use qurb_engine::{Engine, PlanStats, StoreSource};
use qurb_storage::db::{Audience, Event};
use qurb_storage::{ChunkKey, Store};
use qurb_watcher::IgnoreRules;
use std::fs;
use std::path::PathBuf;

const KEY: [u8; 32] = [23; 32];

struct Device {
    _dir: tempfile::TempDir,
    root: PathBuf,
    downloads: PathBuf,
    engine: Engine,
}

impl Device {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("sync");
        let downloads = dir.path().join("Downloads").join("qurb");
        fs::create_dir_all(&root).unwrap();
        let store_dir = root.join(".qurb");
        let store = Store::open(&store_dir, ChunkKey::from_bytes(KEY)).unwrap();
        let ignore = IgnoreRules::new().with_store_dir(&store_dir);
        let engine = Engine::new(root.clone(), store, ignore);
        Self { _dir: dir, root, downloads, engine }
    }

    /// The same device, filing deliveries in Downloads as a desktop does.
    fn desktop() -> Self {
        let mut device = Self::new();
        device.engine.set_downloads(Some(device.downloads.clone()));
        device
    }

    fn id(&self) -> qurb_sync::DeviceId {
        self.engine.store().device_id().unwrap()
    }

    /// Send `contents` to `to`, the way `qurb send` does.
    fn send(&mut self, name: &str, contents: &[u8], to: &Device) {
        let flat = name.replace('/', "_");
        let loose = self.root.parent().unwrap().join(format!("outgoing-{flat}"));
        fs::write(&loose, contents).unwrap();
        self.engine.store_mut().send_to_vault(name, &loose, &to.id()).unwrap();
    }

    /// Take whatever `peer` offers this device, as a sync would.
    fn sync_from(&mut self, peer: &Device) -> PlanStats {
        let me = self.id();
        let offered = peer.engine.store().tree_for(Audience::Device(&me)).unwrap();
        let plan = self.engine.plan_against(&offered).unwrap();
        let reader = Store::open(&peer.root.join(".qurb"), ChunkKey::from_bytes(KEY))
            .unwrap()
            .in_tree(&peer.root);
        let mut content = StoreSource::new(&reader);
        self.engine.apply_plan(&plan, &mut content).unwrap()
    }

    /// Every live path in the index, in any area.
    fn indexed(&self) -> Vec<String> {
        self.engine
            .store()
            .tree()
            .unwrap()
            .into_iter()
            .filter(|v| !v.is_deleted())
            .map(|v| v.path)
            .collect()
    }
}

#[test]
fn a_delivery_lands_in_downloads_and_nowhere_else() {
    let mut phone = Device::new();
    let mut desktop = Device::desktop();
    phone.send("photo.jpg", b"a photo from the phone", &desktop);

    let stats = desktop.sync_from(&phone);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);

    assert_eq!(fs::read(desktop.downloads.join("photo.jpg")).unwrap(), b"a photo from the phone");
    assert!(!desktop.root.join("photo.jpg").exists(), "it was also put in the folder");
    assert!(desktop.indexed().is_empty(), "it entered the index: {:?}", desktop.indexed());
    assert!(desktop.engine.tree().unwrap().is_empty(), "it is being advertised");

    // No partial file left behind, and the folder's scan does not see it.
    let leftovers: Vec<_> = fs::read_dir(&desktop.downloads)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(leftovers, vec!["photo.jpg"]);
    desktop.engine.reconcile().unwrap();
    assert!(desktop.indexed().is_empty());

    // Activity says where it went, which is what the window needs to offer
    // "Open folder".
    let arrived = desktop.engine.store().db().activity(10, None).unwrap();
    let received = arrived.iter().find(|a| a.kind == Event::Received).expect("no activity recorded");
    assert!(
        received.detail.as_deref().unwrap_or("").contains("Downloads"),
        "{:?}",
        received.detail
    );
}

#[test]
fn deleting_it_from_downloads_does_not_bring_it_back() {
    let mut phone = Device::new();
    let mut desktop = Device::desktop();
    phone.send("photo.jpg", b"a photo from the phone", &desktop);
    desktop.sync_from(&phone);

    fs::remove_file(desktop.downloads.join("photo.jpg")).unwrap();
    desktop.engine.reconcile().unwrap();
    // The retention window passing changes nothing either: there is no
    // tombstone to expire, only the record that it was taken.
    desktop.engine.store_mut().gc(std::time::Duration::ZERO).unwrap();

    let stats = desktop.sync_from(&phone);
    assert_eq!(stats.adopted, 0, "the delivery was taken again");
    assert!(!desktop.downloads.join("photo.jpg").exists());
}

#[test]
fn nothing_already_in_downloads_is_overwritten() {
    let mut phone = Device::new();
    let mut desktop = Device::desktop();
    fs::create_dir_all(&desktop.downloads).unwrap();
    fs::write(desktop.downloads.join("report.pdf"), b"the person's own report").unwrap();

    phone.send("report.pdf", b"the phone's report", &desktop);
    let stats = desktop.sync_from(&phone);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);

    assert_eq!(fs::read(desktop.downloads.join("report.pdf")).unwrap(), b"the person's own report");
    let filed: Vec<_> = fs::read_dir(&desktop.downloads)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|name| name != "report.pdf")
        .collect();
    assert_eq!(filed.len(), 1, "{filed:?}");
    assert!(filed[0].starts_with("report.from-") && filed[0].ends_with(".pdf"), "{filed:?}");
    assert_eq!(fs::read(desktop.downloads.join(&filed[0])).unwrap(), b"the phone's report");
}

/// A crash after the file was moved into place and before the delivery was
/// recorded. The next sync finds the file already there, byte for byte, and
/// records it -- rather than filing a second copy under another name.
#[test]
fn an_interrupted_delivery_is_recognised_not_duplicated() {
    let mut phone = Device::new();
    let mut desktop = Device::desktop();
    fs::create_dir_all(&desktop.downloads).unwrap();
    fs::write(desktop.downloads.join("photo.jpg"), b"a photo from the phone").unwrap();

    phone.send("photo.jpg", b"a photo from the phone", &desktop);
    let stats = desktop.sync_from(&phone);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);
    assert_eq!(fs::read_dir(&desktop.downloads).unwrap().count(), 1, "a duplicate was filed");

    let again = desktop.sync_from(&phone);
    assert_eq!(again.adopted, 0, "not recorded as taken");
}

/// Without a downloads directory -- a phone -- nothing changes: the delivery
/// is filed privately in the folder.
#[test]
fn a_phone_still_files_it_in_the_folder() {
    let mut desktop = Device::new();
    let mut phone = Device::new();
    desktop.send("project.zip", b"a project", &phone);

    let stats = phone.sync_from(&desktop);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);
    assert_eq!(fs::read(phone.root.join("project.zip")).unwrap(), b"a project");
    assert_eq!(phone.indexed(), vec!["project.zip".to_string()]);
    assert!(phone.engine.tree().unwrap().is_empty(), "it is being advertised");
    assert!(!phone.downloads.exists());
}

/// A folder sent is a folder received, on a desktop and on a phone alike.
#[test]
fn a_folder_arrives_as_a_folder() {
    let mut sender = Device::new();
    let mut desktop = Device::desktop();
    let mut phone = Device::new();
    for (name, bytes) in [
        ("Photos/cover.jpg", &b"cover"[..]),
        ("Photos/2026/summer/beach.jpg", &b"beach"[..]),
    ] {
        sender.send(name, bytes, &desktop);
        sender.send(name, bytes, &phone);
    }

    for receiver in [&mut desktop, &mut phone] {
        let stats = receiver.sync_from(&sender);
        assert!(stats.failures.is_empty(), "{:?}", stats.failures);
    }

    let base = desktop.downloads.clone();
    assert_eq!(fs::read(base.join("Photos/cover.jpg")).unwrap(), b"cover");
    assert_eq!(fs::read(base.join("Photos/2026/summer/beach.jpg")).unwrap(), b"beach");
    assert_eq!(fs::read(phone.root.join("Photos/2026/summer/beach.jpg")).unwrap(), b"beach");
}

/// A send taken back before it is collected never arrives, however long the
/// other device was away. One already collected cannot be taken back.
#[test]
fn a_cancelled_send_never_arrives() {
    let mut sender = Device::new();
    let mut desktop = Device::desktop();
    sender.send("wrong-file.pdf", b"not meant for them", &desktop);
    sender.send("right-file.pdf", b"meant for them", &desktop);

    sender.engine.store_mut().cancel_send("wrong-file.pdf", &desktop.id()).unwrap();
    let waiting: Vec<String> =
        sender.engine.store().pending_deliveries().unwrap().into_iter().map(|(p, _, _)| p).collect();
    assert_eq!(waiting, vec!["right-file.pdf".to_string()]);

    let stats = desktop.sync_from(&sender);
    assert!(stats.failures.is_empty(), "{:?}", stats.failures);
    assert!(!desktop.downloads.join("wrong-file.pdf").exists(), "a cancelled send arrived");
    assert_eq!(fs::read(desktop.downloads.join("right-file.pdf")).unwrap(), b"meant for them");

    // Collected is final. The receiver says so the way it does over the
    // network, and the sender records it.
    sender
        .engine
        .store()
        .note_replica_in_vault(&blake3::hash(b"meant for them"), &desktop.id())
        .unwrap();
    let refused = sender.engine.store_mut().cancel_send("right-file.pdf", &desktop.id());
    assert!(
        matches!(refused, Err(qurb_storage::Error::AlreadyCollected { .. })),
        "{refused:?}"
    );
}
