# 0045 — A folder kept only remotely

**Status:** Accepted
**Date:** 2026-09-28 (product plan §4.8, brief §29)

## Decision

On any device, a folder can be **kept here** — the default, every file
downloaded as it changes — or **kept remotely**:

- **Its files stay listed.** Each shows as not on this device, with Fetch on
  the desktop and "downloads when opened" in the phone's file picker.
- **A new or changed file arriving from another device is recorded, not
  downloaded.** Its version, size and hash go in the index, and the device
  that made it is noted as having it, which is the same state freeing a file
  leaves behind.
- **Local copies are freed only where another device keeps them.** Freeing the
  only copy would be deleting it, so those stay, and the person is told which
  and why (brief §29: "refuse it and explain why").
- **A file already here is kept up to date.** Kept remotely is about what
  arrives; a file somebody opened does not go stale.
- **Asking for a file downloads it**, as Fetch already did.
- **Keeping the folder here again** asks for everything in it, and it arrives
  at the next sync.

This is the device's own choice: kept in its index (`remote_folders`), never
synced. It is not sharing ([0044](0044-sharing-with-chosen-devices.md)), which
decides whether a device has a folder *at all*.

Desktop: the Folders section of Files — "Free space here" / "Keep on this
computer". Phone: Settings → Folders. Command line: `qurb keep <folder>
here|remote`.

## Why this shape

The storage cap already frees local copies safely, and never the only one
([0025](0025-a-storage-cap-that-cannot-lose-data.md)). What it cannot do is
stop them coming back: a file evicted to stay under the limit was downloaded
again the next time another device changed it. A folder kept remotely is the
one place that is exactly what somebody wants, so the new behaviour is there,
at the point a new version is taken — recorded instead of downloaded.

**Listed rather than left out.** The replica's `PinSet` leaves unpinned paths
out of the index altogether. For a person's own laptop or phone that would make
a folder they chose to keep remotely look deleted, and give them nothing to
fetch.

## Not done

- No placeholder files on Linux: a file kept remotely is not in the folder at
  all, only in qurb's own listing (product plan §9, "Linux has no placeholder
  filesystem API"). On Android the system file picker lists it and downloads
  it when opened.
- A folder kept remotely on every device is kept nowhere but where it was
  made; nothing warns about that yet beyond each file's "only here" state.
- By top-level folder in the screens; the engine takes any folder.
