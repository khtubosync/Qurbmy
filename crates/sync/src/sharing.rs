//! Which devices a folder in the shared area is shared with (decision 0044).
//!
//! A folder with no rule is shared with every device, which is how the shared
//! area has always behaved. A rule names the devices a folder is shared with;
//! every other device is neither shown what is in it nor accepted as a source
//! of changes to it.
//!
//! Rules are ordinary small files in a hidden directory at the top of the
//! shared area, one per folder, so they reach every device by the same sync
//! that carries everything else and every device enforces the same ones. What
//! is here is their format: where a folder's rule file lives, what is in it,
//! and how a set of rules answers "may this device see this path".

use crate::device::DeviceId;
use std::collections::{BTreeMap, BTreeSet};

/// Where rule files live, at the top of the shared area.
pub const SHARING_DIR: &str = ".qurb-sharing";

/// Whether `path` is a rule file (or anything else in the rules directory).
pub fn is_rule_path(path: &str) -> bool {
    path.strip_prefix(SHARING_DIR).is_some_and(|rest| rest.starts_with('/'))
}

/// The rule file for `folder`: the folder's path with `%` and `/` escaped, so
/// that one flat directory holds every folder's rule.
pub fn rule_path(folder: &str) -> String {
    let escaped = folder.replace('%', "%25").replace('/', "%2F");
    format!("{SHARING_DIR}/{escaped}")
}

/// The folder a rule file is about, or `None` if `path` is not one.
///
/// A conflict copy of a rule file is not a rule: two devices that changed one
/// rule at once settle it the way any conflict is settled, and the name that
/// wins is the rule.
pub fn rule_folder(path: &str) -> Option<String> {
    let name = path.strip_prefix(SHARING_DIR)?.strip_prefix('/')?;
    if name.is_empty() || name.contains('/') || crate::resolve::conflict_origin(path).is_some() {
        return None;
    }
    let folder = name.replace("%2F", "/").replace("%25", "%");
    valid_folder(&folder).then_some(folder)
}

/// Whether `folder` can have a rule: a relative path, not the top of the
/// shared area, and not the rules directory itself.
pub fn valid_folder(folder: &str) -> bool {
    !folder.is_empty()
        && !folder.starts_with('/')
        && !folder.ends_with('/')
        && !is_rule_path(folder)
        && folder != SHARING_DIR
        && crate::path::is_safe_path(folder)
}

/// A rule file's contents for `members`: one device id per line, in hex,
/// sorted, after a comment saying what the file is. Sorted so that the same
/// rule written on two devices is the same bytes, and not a conflict.
pub fn encode_members(members: &BTreeSet<DeviceId>) -> String {
    let mut out = String::from(
        "# The devices this folder is shared with. Written by qurb; see decision 0044.\n",
    );
    for device in members {
        out.push_str(&device.to_hex());
        out.push('\n');
    }
    out
}

/// The members a rule file names. Lines that are not a device id are ignored,
/// so a person editing the file by hand cannot make it unreadable -- only
/// shorter.
pub fn parse_members(text: &str) -> BTreeSet<DeviceId> {
    text.lines().filter_map(|line| DeviceId::from_hex(line.trim())).collect()
}

/// Every rule in force: folder to members.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Rules(BTreeMap<String, BTreeSet<DeviceId>>);

impl Rules {
    pub fn new(rules: BTreeMap<String, BTreeSet<DeviceId>>) -> Self {
        Self(rules)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &BTreeSet<DeviceId>)> {
        self.0.iter()
    }

    /// The folder whose rule covers `path`, and its members. Rules do not
    /// nest, so there is at most one.
    pub fn covering(&self, path: &str) -> Option<(&str, &BTreeSet<DeviceId>)> {
        self.0
            .iter()
            .find(|(folder, _)| {
                path == folder.as_str()
                    || path.strip_prefix(folder.as_str()).is_some_and(|rest| rest.starts_with('/'))
            })
            .map(|(folder, members)| (folder.as_str(), members))
    }

    /// Whether `device` may see and change `path`. Anything no rule covers is
    /// shared with every device.
    pub fn allows(&self, path: &str, device: &DeviceId) -> bool {
        self.covering(path).is_none_or(|(_, members)| members.contains(device))
    }

    /// Whether a rule for `folder` could be added without nesting inside, or
    /// around, one that exists. Replacing a folder's own rule is fine.
    pub fn nests(&self, folder: &str) -> Option<&str> {
        self.0.keys().map(String::as_str).find(|existing| {
            *existing != folder
                && (folder.strip_prefix(existing).is_some_and(|rest| rest.starts_with('/'))
                    || existing.strip_prefix(folder).is_some_and(|rest| rest.starts_with('/')))
        })
    }

    /// Whether `device` may change the rule for `folder`: anybody while there
    /// is none, and afterwards only a device the folder is shared with. A
    /// device left out cannot write itself back in.
    pub fn may_change(&self, folder: &str, device: &DeviceId) -> bool {
        self.0.get(folder).is_none_or(|members| members.contains(device))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(byte: u8) -> DeviceId {
        DeviceId::from_bytes([byte; 32])
    }

    #[test]
    fn a_rule_file_names_its_folder_and_back() {
        for folder in ["Family Photos", "work/clients/acme", "100%/done"] {
            let path = rule_path(folder);
            assert!(is_rule_path(&path));
            assert_eq!(rule_folder(&path).as_deref(), Some(folder), "{path}");
        }
    }

    #[test]
    fn what_is_not_a_rule_is_not_read_as_one() {
        for path in [
            ".qurb-sharing",
            ".qurb-sharingX/Photos",
            "Photos/.qurb-sharing/x",
            ".qurb-sharing/",
            ".qurb-sharing/a/b",
            ".qurb-sharing/Photos.conflict-a1a1a1a1-2026-09-28-101502",
            ".qurb-sharing/..%2Fescape",
        ] {
            assert_eq!(rule_folder(path), None, "{path}");
        }
    }

    #[test]
    fn members_survive_a_round_trip_and_hand_editing() {
        let members: BTreeSet<_> = [device(3), device(1)].into();
        let text = encode_members(&members);
        assert_eq!(parse_members(&text), members);
        let edited = format!("{text}\nnot a device\n\n  {}  \n", device(9).to_hex());
        assert_eq!(parse_members(&edited).len(), 3);
    }

    #[test]
    fn a_rule_covers_its_folder_and_nothing_beside_it() {
        let rules = Rules::new([("Family".to_string(), [device(1)].into())].into());
        assert!(!rules.allows("Family/beach.jpg", &device(2)));
        assert!(!rules.allows("Family", &device(2)));
        assert!(rules.allows("Family/beach.jpg", &device(1)));
        assert!(rules.allows("Family Photos/beach.jpg", &device(2)), "a longer name is another folder");
        assert!(rules.allows("notes.txt", &device(2)));
    }

    #[test]
    fn rules_do_not_nest() {
        let rules = Rules::new([("work/clients".to_string(), [device(1)].into())].into());
        assert_eq!(rules.nests("work"), Some("work/clients"));
        assert_eq!(rules.nests("work/clients/acme"), Some("work/clients"));
        assert_eq!(rules.nests("work/clients"), None, "replacing its own rule");
        assert_eq!(rules.nests("work/other"), None);
    }

    #[test]
    fn only_a_member_changes_a_rule() {
        let rules = Rules::new([("Family".to_string(), [device(1)].into())].into());
        assert!(rules.may_change("Family", &device(1)));
        assert!(!rules.may_change("Family", &device(2)), "left out, writing itself back in");
        assert!(rules.may_change("Work", &device(2)), "no rule yet: anybody may make one");
    }
}
