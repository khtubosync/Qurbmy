//! Which paths another device may name.
//!
//! A path in a tree is a *logical* path: relative to the synced folder,
//! components separated by `/`. Every device that receives one joins it onto
//! a directory of its own and writes there. So a path is not just a name -- it
//! is an instruction about where on this disk to write, or what to delete, and
//! it comes from another machine.
//!
//! Pairing says who that machine is. It says nothing about whether it is
//! telling the truth: a device that is stolen or compromised keeps its pinned
//! identity. A path of `../../.bashrc`, or `/etc/cron.d/x`, or one that
//! reaches into qurb's own store, must therefore be refused wherever one
//! arrives, not trusted because the sender was.

/// Whether `path` names somewhere inside the folder, and nowhere else.
///
/// Refused: empty paths, absolute paths, empty components (`a//b`, a trailing
/// `/`), `.` and `..` components, and NUL, which no filesystem this runs on
/// allows in a name and which some APIs treat as the end of the string.
///
/// Deliberately *not* refused: backslashes, which are ordinary characters in
/// a Linux filename. A device on Windows would read one as a separator; that
/// platform is not supported yet, and supporting it means revisiting this.
pub fn is_safe_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains('\0') {
        return false;
    }
    path.split('/').all(|component| !matches!(component, "" | "." | ".."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_paths_are_safe() {
        for path in ["notes.txt", "photos/2026/beach.jpg", ".bashrc", "a..b", "...", "café/ü.txt", "back\\slash"] {
            assert!(is_safe_path(path), "{path:?} should be allowed");
        }
    }

    #[test]
    fn paths_that_leave_the_folder_are_not() {
        for path in [
            "",
            "/etc/passwd",
            "../escape.txt",
            "photos/../../escape.txt",
            "..",
            ".",
            "./notes.txt",
            "photos//beach.jpg",
            "photos/",
            "nul\0byte",
        ] {
            assert!(!is_safe_path(path), "{path:?} should be refused");
        }
    }
}
