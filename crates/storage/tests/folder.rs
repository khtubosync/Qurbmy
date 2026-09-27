//! Looking into a device's folder by directory, by name, and by search.
//!
//! What a file browser on the phone asks the index, instead of walking the
//! directory: which files are beneath a folder, what one file is and where its
//! bytes are, and which files match some text. The index is flat, so "a
//! folder" is only the paths that begin with it -- and a folder name containing
//! `_` or `%` must not turn into a pattern.

use qurb_storage::db::Availability;
use qurb_storage::{ChunkKey, Store};

struct Folder {
    dir: tempfile::TempDir,
    store: Store,
}

impl Folder {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("folder");
        std::fs::create_dir_all(&root).unwrap();
        let store = Store::open(&root.join(".qurb"), ChunkKey::from_bytes([21; 32]))
            .unwrap()
            .in_tree(&root);
        Self { dir, store }
    }

    fn add(&mut self, path: &str) {
        let on_disk = self.dir.path().join("folder").join(path);
        std::fs::create_dir_all(on_disk.parent().unwrap()).unwrap();
        std::fs::write(&on_disk, path.as_bytes()).unwrap();
        self.store.put_file(path, &on_disk).unwrap();
    }

    fn under(&self, dir: &str) -> Vec<String> {
        self.store.db().folder_entries_under(dir).unwrap().into_iter().map(|e| e.path).collect()
    }
}

#[test]
fn a_folder_is_the_paths_beneath_it() {
    let mut folder = Folder::new();
    for path in ["top.txt", "album/one.jpg", "album/deep/two.jpg", "albumen.txt"] {
        folder.add(path);
    }

    assert_eq!(folder.under(""), ["album/deep/two.jpg", "album/one.jpg", "albumen.txt", "top.txt"]);
    assert_eq!(folder.under("album"), ["album/deep/two.jpg", "album/one.jpg"], "albumen.txt is not in album");
    assert_eq!(folder.under("/album/"), ["album/deep/two.jpg", "album/one.jpg"]);
    assert_eq!(folder.under("album/deep"), ["album/deep/two.jpg"]);
    assert!(folder.under("nowhere").is_empty());
}

/// `_` and `%` are wildcards to LIKE. Unescaped, a folder called `a_b` also
/// held everything in `axb`, and `100%` held everything beginning `100`.
#[test]
fn wildcards_in_a_folder_name_are_only_characters() {
    let mut folder = Folder::new();
    for path in ["a_b/in.txt", "axb/out.txt", "100%/in.txt", "1000/out.txt"] {
        folder.add(path);
    }

    assert_eq!(folder.under("a_b"), ["a_b/in.txt"]);
    assert_eq!(folder.under("100%"), ["100%/in.txt"]);

    let db = folder.store.db();
    assert_eq!(db.folder_paths_under("a_b").unwrap(), ["a_b/in.txt"]);
    let listed: Vec<String> =
        db.listing(Some("a_b"), 50, 0).unwrap().into_iter().map(|l| l.path).collect();
    assert_eq!(listed, ["a_b/in.txt"], "the desktop's Files screen had the same slip");
}

#[test]
fn one_file_and_where_its_bytes_are() {
    let mut folder = Folder::new();
    folder.add("album/one.jpg");

    let entry = folder.store.db().folder_entry("album/one.jpg").unwrap().unwrap();
    assert_eq!(entry.size, "album/one.jpg".len() as u64);
    assert_eq!(entry.availability, Availability::OnlyHere, "nobody else has it");
    assert!(folder.store.db().folder_entry("album").unwrap().is_none(), "a folder is not a file");
    assert!(folder.store.db().folder_entry("missing.jpg").unwrap().is_none());
}

#[test]
fn search_matches_anywhere_in_the_path_and_ignores_case() {
    let mut folder = Folder::new();
    for path in ["Holiday/Beach.jpg", "holiday.txt", "work/notes.txt", "100%_done.txt"] {
        folder.add(path);
    }
    let found = |text: &str| -> Vec<String> {
        folder.store.db().folder_search(text, 50).unwrap().into_iter().map(|e| e.path).collect()
    };

    assert_eq!(found("HOLIDAY"), ["Holiday/Beach.jpg", "holiday.txt"]);
    assert_eq!(found("beach"), ["Holiday/Beach.jpg"]);
    assert_eq!(found("%_"), ["100%_done.txt"], "a wildcard typed into search is a character");
    assert_eq!(folder.store.db().folder_search("t", 2).unwrap().len(), 2, "the limit holds");
}
