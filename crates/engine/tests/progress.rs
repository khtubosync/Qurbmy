//! What a transfer in flight reports.
//!
//! A window showing "photo.jpg — 1.2 GB of 4.0 GB" needs to hear about bytes as
//! they arrive, and must not be told about work that is not happening: content
//! already on this device under another name costs a lookup, not a transfer.

use qurb_engine::{Engine, Progress, StoreSource};
use qurb_storage::{ChunkKey, Store};
use qurb_watcher::IgnoreRules;
use std::fs;
use std::path::PathBuf;

const KEY: [u8; 32] = [61; 32];

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
        let store = Store::open(&store_dir, ChunkKey::from_bytes(KEY)).unwrap();
        let ignore = IgnoreRules::new().with_store_dir(&store_dir);
        Self { _dir: dir, root: root.clone(), engine: Engine::new(root, store, ignore) }
    }

    fn write(&mut self, name: &str, contents: &[u8]) {
        fs::write(self.root.join(name), contents).unwrap();
        self.engine.reconcile().unwrap();
    }

    fn sync_from(&mut self, peer: &Device, progress: &mut dyn Progress) {
        let plan = self.engine.plan_against(&peer.engine.tree().unwrap()).unwrap();
        let reader = Store::open(&peer.root.join(".qurb"), ChunkKey::from_bytes(KEY))
            .unwrap()
            .in_tree(&peer.root);
        let stats = self
            .engine
            .apply_plan_reporting(&plan, &mut StoreSource::new(&reader), progress)
            .unwrap();
        assert!(stats.failures.is_empty(), "{:?}", stats.failures);
    }
}

#[derive(Default)]
struct Recorded {
    started: Vec<(String, u64)>,
    advanced: u64,
    finished: Vec<String>,
}

impl Progress for Recorded {
    fn started(&mut self, path: &str, size: u64) {
        self.started.push((path.to_string(), size));
    }
    fn advanced(&mut self, bytes: u64) {
        self.advanced += bytes;
    }
    fn finished(&mut self, path: &str) {
        self.finished.push(path.to_string());
    }
}

fn noisy(size: usize) -> Vec<u8> {
    let mut out = vec![0u8; size];
    let mut x: u32 = 0x2545_f491;
    for byte in out.iter_mut() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *byte = x as u8;
    }
    out
}

#[test]
fn a_file_that_crosses_is_reported_byte_for_byte() {
    let mut phone = Device::new();
    phone.write("video.mp4", &noisy(3 << 20));
    let mut laptop = Device::new();

    let mut seen = Recorded::default();
    laptop.sync_from(&phone, &mut seen);

    assert_eq!(seen.started, vec![("video.mp4".to_string(), 3 << 20)]);
    assert_eq!(seen.advanced, 3 << 20, "every byte written should have been counted");
    assert_eq!(seen.finished, vec!["video.mp4".to_string()]);
}

/// A copy of something this device already has moves no data, so nothing is
/// shown moving.
#[test]
fn content_already_here_is_not_a_transfer() {
    let bytes = noisy(1 << 20);
    let mut phone = Device::new();
    phone.write("original.bin", &bytes);
    let mut laptop = Device::new();
    laptop.sync_from(&phone, &mut Recorded::default());

    phone.write("copy.bin", &bytes);
    let mut seen = Recorded::default();
    laptop.sync_from(&phone, &mut seen);

    assert!(seen.started.is_empty(), "a local copy was reported as a transfer: {:?}", seen.started);
    assert_eq!(seen.advanced, 0);
    assert_eq!(fs::read(laptop.root.join("copy.bin")).unwrap(), bytes);
}
