//! What sending some files and folders to a device actually sends.
//!
//! A person picks things — three photos, or a folder of them — and the
//! recipient should find the same things: the photos under their own names,
//! the folder as a folder. The store sends one file at a time, so this turns
//! what was picked into a list of files and the names the recipient will see.
//!
//! Only the filesystem is read here, never the store, so the caller can send
//! each file separately. The window relies on that: it holds a lock while it
//! stores, and a folder of a thousand files must not hold it for the whole
//! folder.

use qurb_sync::is_safe_path;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// The files to send, and what could not be.
#[derive(Debug, Default)]
pub struct Plan {
    /// The name the recipient will see, and where the bytes are here.
    pub files: Vec<(String, PathBuf)>,
    /// What was picked and will not be sent, with why, in words a person can
    /// act on. Sending the rest is still right: one unreadable file is not a
    /// reason to send none of a folder.
    pub skipped: Vec<(PathBuf, String)>,
}

/// Work out what sending `picked` means.
///
/// A file is sent under its own name. A folder is sent whole, under its own
/// name, with everything beneath it: `Photos/2026/beach.jpg`. Links inside a
/// folder are not followed — a link can point anywhere, including back up the
/// tree — and a qurb store inside one is never sent, because it holds this
/// device's keys. Something picked directly is followed, since that is what
/// the person chose.
pub fn plan(picked: &[PathBuf]) -> Plan {
    let mut plan = Plan::default();
    let mut taken = HashSet::new();

    for item in picked {
        let meta = match std::fs::metadata(item) {
            Ok(meta) => meta,
            Err(e) => {
                plan.skipped.push((item.clone(), unreadable(&e)));
                continue;
            }
        };
        let Some(name) = item.file_name().and_then(|n| n.to_str()) else {
            plan.skipped.push((item.clone(), "its name cannot be read as text".into()));
            continue;
        };

        if meta.is_file() {
            add(&mut plan, &mut taken, name.to_string(), item.clone());
        } else if meta.is_dir() {
            walk(&mut plan, &mut taken, item, name.to_string());
        } else {
            plan.skipped.push((item.clone(), "not a file or a folder".into()));
        }
    }
    plan
}

fn walk(plan: &mut Plan, taken: &mut HashSet<String>, dir: &Path, sent_as: String) {
    if dir.file_name().is_some_and(|n| n == ".qurb") {
        plan.skipped.push((dir.to_path_buf(), "a qurb store, which holds this device's keys".into()));
        return;
    }

    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(e) => {
            plan.skipped.push((dir.to_path_buf(), unreadable(&e)));
            return;
        }
    };
    // In name order, so the recipient's files arrive in an order a person
    // would recognise, and the same folder always sends the same way.
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            plan.skipped.push((path, "its name cannot be read as text".into()));
            continue;
        };
        let Ok(kind) = entry.file_type() else {
            plan.skipped.push((path, "could not be read".into()));
            continue;
        };
        let inside = format!("{sent_as}/{name}");

        if kind.is_symlink() {
            plan.skipped.push((path, "a link, which is not followed inside a folder".into()));
        } else if kind.is_dir() {
            walk(plan, taken, &path, inside);
        } else if kind.is_file() {
            add(plan, taken, inside, path);
        }
    }
}

/// Add one file, under a name nothing else in this send has used.
///
/// Two things picked together can share a name — `notes.txt` from two
/// different folders. Sent under one name, the second would replace the first
/// before either arrived, and the recipient would get one file where two were
/// sent. The second is sent as `notes (2).txt` instead.
fn add(plan: &mut Plan, taken: &mut HashSet<String>, wanted: String, source: PathBuf) {
    let wanted = qurb_watcher::normalize(&wanted);
    if !is_safe_path(&wanted) {
        plan.skipped.push((source, "its name is not one another device can use".into()));
        return;
    }

    let mut name = wanted.clone();
    let mut n = 2;
    while !taken.insert(name.clone()) {
        name = numbered(&wanted, n);
        n += 1;
    }
    plan.files.push((name, source));
}

/// `photos/notes.txt` as `photos/notes (2).txt`: the number before the
/// extension, so the file still opens in whatever it belongs to.
fn numbered(path: &str, n: usize) -> String {
    let (dir, file) = match path.rfind('/') {
        Some(i) => (&path[..=i], &path[i + 1..]),
        None => ("", path),
    };
    match file[1..].rfind('.').map(|i| i + 1) {
        Some(dot) => format!("{dir}{} ({n}){}", &file[..dot], &file[dot..]),
        None => format!("{dir}{file} ({n})"),
    }
}

fn unreadable(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::NotFound => "it is not there any more".into(),
        std::io::ErrorKind::PermissionDenied => "qurb is not allowed to read it".into(),
        _ => format!("could not be read: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn names(plan: &Plan) -> Vec<&str> {
        plan.files.iter().map(|(name, _)| name.as_str()).collect()
    }

    #[test]
    fn files_go_under_their_own_names() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("beach.jpg");
        let b = dir.path().join("notes.txt");
        fs::write(&a, b"a").unwrap();
        fs::write(&b, b"b").unwrap();

        let plan = plan(&[a, b]);
        assert_eq!(names(&plan), vec!["beach.jpg", "notes.txt"]);
        assert!(plan.skipped.is_empty());
    }

    #[test]
    fn a_folder_goes_whole_under_its_own_name() {
        let dir = tempfile::tempdir().unwrap();
        let photos = dir.path().join("Photos");
        fs::create_dir_all(photos.join("2026/summer")).unwrap();
        fs::write(photos.join("cover.jpg"), b"c").unwrap();
        fs::write(photos.join("2026/summer/beach.jpg"), b"b").unwrap();
        fs::write(photos.join("2026/index.txt"), b"i").unwrap();

        let plan = plan(&[photos]);
        assert_eq!(
            names(&plan),
            vec!["Photos/2026/index.txt", "Photos/2026/summer/beach.jpg", "Photos/cover.jpg"]
        );
    }

    /// A store holds the keys to everything. A person sending the folder that
    /// has one inside it means the files, not the store.
    #[test]
    fn a_qurb_store_is_never_sent() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("qurb");
        fs::create_dir_all(folder.join(".qurb")).unwrap();
        fs::write(folder.join(".qurb/key"), b"secret").unwrap();
        fs::write(folder.join("notes.txt"), b"n").unwrap();

        let plan = plan(&[folder]);
        assert_eq!(names(&plan), vec!["qurb/notes.txt"]);
        assert_eq!(plan.skipped.len(), 1);
    }

    #[test]
    fn links_inside_a_folder_are_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("share");
        fs::create_dir_all(&folder).unwrap();
        fs::write(dir.path().join("elsewhere.txt"), b"e").unwrap();
        fs::write(folder.join("real.txt"), b"r").unwrap();
        let elsewhere = dir.path().join("elsewhere.txt");
        std::os::unix::fs::symlink(&elsewhere, folder.join("link.txt")).unwrap();
        // And one that would loop for ever if followed.
        std::os::unix::fs::symlink(&folder, folder.join("again")).unwrap();

        let plan = plan(&[folder]);
        assert_eq!(names(&plan), vec!["share/real.txt"]);
        assert_eq!(plan.skipped.len(), 2);
    }

    /// Sent under one name, the second would replace the first before either
    /// arrived.
    #[test]
    fn two_things_with_one_name_are_both_sent() {
        let dir = tempfile::tempdir().unwrap();
        for sub in ["a", "b", "c"] {
            fs::create_dir_all(dir.path().join(sub)).unwrap();
            fs::write(dir.path().join(sub).join("notes.txt"), sub.as_bytes()).unwrap();
        }
        let picked: Vec<PathBuf> =
            ["a", "b", "c"].iter().map(|s| dir.path().join(s).join("notes.txt")).collect();

        let plan = plan(&picked);
        assert_eq!(names(&plan), vec!["notes.txt", "notes (2).txt", "notes (3).txt"]);
    }

    #[test]
    fn something_missing_is_reported_and_the_rest_still_sent() {
        let dir = tempfile::tempdir().unwrap();
        let here = dir.path().join("here.txt");
        fs::write(&here, b"h").unwrap();

        let plan = plan(&[dir.path().join("gone.txt"), here]);
        assert_eq!(names(&plan), vec!["here.txt"]);
        assert_eq!(plan.skipped.len(), 1);
        assert_eq!(plan.skipped[0].1, "it is not there any more");
    }

    #[test]
    fn numbering_goes_before_the_extension() {
        assert_eq!(numbered("notes.txt", 2), "notes (2).txt");
        assert_eq!(numbered("Photos/beach.jpg", 3), "Photos/beach (3).jpg");
        assert_eq!(numbered("Makefile", 2), "Makefile (2)");
        assert_eq!(numbered(".bashrc", 2), ".bashrc (2)");
        assert_eq!(numbered("archive.tar.gz", 2), "archive.tar (2).gz");
    }
}
