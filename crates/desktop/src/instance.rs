//! One qurb per person, and a second launch showing the first.
//!
//! Closing the window leaves qurb running, because syncing is the point of it
//! and a laptop that stops syncing whenever its window is closed only syncs
//! while somebody is looking. Which means opening qurb from the applications
//! menu while it is already running must show the window that is there, not
//! start another -- a second one would find the folder's daemon lock taken and
//! could only say so.
//!
//! A Unix socket in the runtime directory, rather than a plugin: the first
//! instance listens, a second connects, says "show" and leaves. The runtime
//! directory is the user's own, mode 0700, so nobody else can ask; and the
//! only thing anyone could ask is for a window to be shown.
//!
//! Two launches at the very same moment can both become "first". The folder's
//! lock still lets only one of them sync, and the other says so in its window.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

/// Where the running instance listens.
pub fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    // SAFETY: getuid cannot fail and has no preconditions.
    let uid = unsafe { libc::getuid() };
    dir.join(format!("qurb-desktop-{uid}.sock"))
}

pub enum Instance {
    /// Nobody else is running: this is qurb, and listens here.
    First(UnixListener),
    /// Another qurb is running and has been asked to show its window.
    AskedToShow,
}

/// Become the one instance, or ask the one already running to show itself.
pub fn claim(path: &Path) -> std::io::Result<Instance> {
    if let Ok(mut running) = UnixStream::connect(path) {
        running.write_all(b"show\n")?;
        return Ok(Instance::AskedToShow);
    }
    // Nothing answered: no socket, or one left by a qurb that did not exit
    // cleanly. Either way the path is ours now.
    let _ = std::fs::remove_file(path);
    Ok(Instance::First(UnixListener::bind(path)?))
}

/// Call `show` each time another launch asks, for the life of the process.
pub fn listen(listener: UnixListener, show: impl Fn() + Send + 'static) {
    std::thread::spawn(move || {
        for mut asking in listener.incoming().flatten() {
            let mut said = [0u8; 8];
            if asking.read(&mut said).is_ok_and(|n| said[..n].starts_with(b"show")) {
                show();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn a_second_launch_asks_the_first_to_show_itself() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("qurb.sock");

        let Instance::First(listener) = claim(&path).unwrap() else { panic!("nobody was running") };
        let (shown, asked) = mpsc::channel();
        listen(listener, move || shown.send(()).unwrap());

        assert!(matches!(claim(&path).unwrap(), Instance::AskedToShow));
        asked.recv_timeout(Duration::from_secs(5)).expect("the running one was not asked");
        assert!(matches!(claim(&path).unwrap(), Instance::AskedToShow), "only the first time");
        asked.recv_timeout(Duration::from_secs(5)).expect("asked again, and not shown again");
    }

    /// A qurb that crashed leaves its socket behind, with nobody listening.
    /// That must not stop qurb ever starting again.
    #[test]
    fn a_socket_left_by_a_crash_is_taken_over() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("qurb.sock");
        drop(UnixListener::bind(&path).unwrap());
        assert!(path.exists(), "the leftover this test is about");

        assert!(matches!(claim(&path).unwrap(), Instance::First(_)));
    }
}
