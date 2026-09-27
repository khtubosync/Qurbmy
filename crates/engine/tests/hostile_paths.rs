//! A peer that names somewhere it should not.
//!
//! A path in a tree is joined onto this device's folder and written to, or
//! deleted from. Pairing proves who the peer is and nothing about whether what
//! it says is safe to act on: a stolen or compromised device keeps its pinned
//! identity. So a version naming `../../.bashrc`, an absolute path, or qurb's
//! own store inside the folder must be refused before anything touches the
//! disk -- for a file, a deletion and a delivery alike.
//!
//! The wire refuses these on decoding. These tests build the peer's tree in
//! process, so they exercise the engine's own check, which is what stands if
//! a path ever arrives some other way.

use qurb_engine::{ContentSource, Engine, Error};
use qurb_storage::{ChunkKey, Store};
use qurb_sync::{Content, DeviceId, FileVersion, VersionVector};
use qurb_watcher::IgnoreRules;
use std::fs;
use std::path::PathBuf;

struct Device {
    dir: tempfile::TempDir,
    root: PathBuf,
    engine: Engine,
}

impl Device {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("sync");
        fs::create_dir_all(&root).unwrap();
        let store_dir = root.join(".qurb");
        let store = Store::open(&store_dir, ChunkKey::from_bytes([5; 32])).unwrap();
        let ignore = IgnoreRules::new().with_store_dir(&store_dir);
        Self { dir, root: root.clone(), engine: Engine::new(root, store, ignore) }
    }
}

/// Supplies whatever is asked for, so that a refusal is the engine's doing and
/// not a fetch failing.
struct Anything(Vec<u8>);

impl ContentSource for Anything {
    fn fetch(&mut self, _hash: &[u8; 32], _size: u64) -> qurb_engine::Result<Vec<u8>> {
        Ok(self.0.clone())
    }
}

fn from_peer(path: &str, content: Content, private: bool) -> FileVersion {
    let peer = DeviceId::from_bytes([0xEE; 32]);
    let mut vector = VersionVector::new();
    vector.increment(peer);
    FileVersion {
        path: path.to_string(),
        content,
        vector,
        modified_by: peer,
        modified_at: 1_790_000_000,
        area: if private { qurb_sync::Area::Sent } else { qurb_sync::Area::Shared },
    }
}

fn a_file(bytes: &[u8]) -> Content {
    Content::File { hash: *blake3::hash(bytes).as_bytes(), size: bytes.len() as u64 }
}

#[test]
fn nothing_is_written_outside_the_folder_or_into_the_store() {
    let mut device = Device::new();
    let bytes = b"#!/bin/sh\necho owned\n".to_vec();

    let hostile: Vec<FileVersion> = [
        "../escaped.txt",
        "../../escaped-further.txt",
        "photos/../../escaped-sideways.txt",
        ".qurb/config",
    ]
    .iter()
    .map(|path| from_peer(path, a_file(&bytes), false))
    .chain([from_peer(
        &device.dir.path().join("absolute.txt").display().to_string(),
        a_file(&bytes),
        false,
    )])
    // And the same, as something sent to this device privately.
    .chain([from_peer("../delivered-outside.txt", a_file(&bytes), true)])
    .collect();

    let config_before = fs::read(device.root.join(".qurb/config")).ok();

    let plan = device.engine.plan_against(&hostile).unwrap();
    let stats = device.engine.apply_plan(&plan, &mut Anything(bytes.clone())).unwrap();

    assert_eq!(stats.failures.len(), hostile.len(), "every hostile path should have been refused");
    assert!(
        stats.failures.iter().all(|f| matches!(f.error, Error::UnsafePath { .. })),
        "{:?}",
        stats.failures
    );

    for escaped in ["escaped.txt", "absolute.txt", "delivered-outside.txt"] {
        assert!(!device.dir.path().join(escaped).exists(), "{escaped} was written outside the folder");
    }
    assert!(!device.dir.path().parent().unwrap().join("escaped-further.txt").exists());
    assert_eq!(fs::read(device.root.join(".qurb/config")).ok(), config_before, "the store was written to");
}

#[test]
fn nothing_outside_the_folder_is_deleted() {
    let mut device = Device::new();
    let victim = device.dir.path().join("victim.txt");
    fs::write(&victim, b"not qurb's to delete").unwrap();

    let hostile = vec![from_peer("../victim.txt", Content::Deleted, false)];
    let plan = device.engine.plan_against(&hostile).unwrap();
    let stats = device.engine.apply_plan(&plan, &mut Anything(Vec::new())).unwrap();

    assert_eq!(fs::read(&victim).unwrap(), b"not qurb's to delete", "a peer deleted a file outside the folder");
    assert!(stats.failures.iter().all(|f| matches!(f.error, Error::UnsafePath { .. })));
}
