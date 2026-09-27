//! What a device believes about one file.

use crate::clock::VersionVector;
use crate::device::DeviceId;

/// What a path currently holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    File {
        /// BLAKE3 of the whole file, which is also what identifies the version.
        hash: [u8; 32],
        size: u64,
    },
    /// A tombstone.
    ///
    /// Deletion has to be recorded rather than inferred. The absence of a file
    /// is not self-describing: without a tombstone a peer cannot tell "deleted"
    /// from "not received yet", and would helpfully restore it.
    Deleted,
}

impl Content {
    pub fn is_deleted(&self) -> bool {
        matches!(self, Content::Deleted)
    }

    pub fn hash(&self) -> Option<&[u8; 32]> {
        match self {
            Content::File { hash, .. } => Some(hash),
            Content::Deleted => None,
        }
    }
}

/// Where a version belongs, as the device receiving it should file it.
///
/// Carried on the version rather than inferred, because the receiver cannot
/// tell from the path and gets exactly one chance to file it correctly. A
/// private version adopted as shared content would be advertised to every
/// other device on the next sync -- silent, immediate and irreversible.
///
/// Four rather than two because holding a vault has a direction. Once two
/// devices can each hold the other's, "this is a vault entry" cannot say
/// whose. See decisions 0029, 0030 and 0036.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Area {
    /// The shared area: everything the product had before vaults existed.
    #[default]
    Shared,
    /// Sent into the receiver's vault by the device it came from (0030).
    Sent,
    /// The receiver's own file, which the device it came from holds for it.
    Held,
    /// The sending device's own file, for the receiver to hold for it.
    Hold,
}

impl Area {
    /// Anything but the shared area: nobody else's business.
    pub fn is_private(self) -> bool {
        self != Area::Shared
    }

    /// The byte this is carried as. The first two are the values the old
    /// `private` flag had, which is not compatibility -- the protocol version
    /// changed with this -- but does keep the common cases recognisable.
    pub fn to_byte(self) -> u8 {
        match self {
            Area::Shared => 0,
            Area::Sent => 1,
            Area::Held => 2,
            Area::Hold => 3,
        }
    }

    pub fn from_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            0 => Area::Shared,
            1 => Area::Sent,
            2 => Area::Held,
            3 => Area::Hold,
            _ => return None,
        })
    }
}

/// One device's view of one path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileVersion {
    pub path: String,
    pub content: Content,
    pub vector: VersionVector,
    /// Which device made this change. Used for conflict filenames and for
    /// explaining to a user where a version came from.
    pub modified_by: DeviceId,
    /// Wall-clock time of the change, in unix seconds.
    ///
    /// **Never used to order versions.** Device clocks disagree, sometimes by
    /// a lot, and ordering by wall clock would let a device with a wrong clock
    /// win or lose every conflict systematically. Ordering is the version
    /// vector's job. This is for display and for conflict filenames only.
    pub modified_at: i64,
    /// Where the receiving device should file it. See [`Area`].
    pub area: Area,
}

impl FileVersion {
    pub fn file(
        path: impl Into<String>,
        hash: [u8; 32],
        size: u64,
        vector: VersionVector,
        modified_by: DeviceId,
        modified_at: i64,
    ) -> Self {
        Self {
            path: path.into(),
            content: Content::File { hash, size },
            vector,
            modified_by,
            modified_at,
            area: Area::Shared,
        }
    }

    pub fn tombstone(
        path: impl Into<String>,
        vector: VersionVector,
        modified_by: DeviceId,
        modified_at: i64,
    ) -> Self {
        Self {
            path: path.into(),
            content: Content::Deleted,
            vector,
            modified_by,
            modified_at,
            area: Area::Shared,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.content.is_deleted()
    }

    /// Whether two versions hold the same bytes, regardless of history.
    ///
    /// Two devices can reach identical content independently — the same file
    /// copied into place on each. That is not a conflict even though the
    /// vectors are concurrent.
    pub fn same_content(&self, other: &Self) -> bool {
        self.content == other.content
    }

    /// The same version, filed in `area`.
    pub fn in_area(mut self, area: Area) -> Self {
        self.area = area;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: DeviceId = DeviceId::from_bytes([0xA1; 32]);

    #[test]
    fn a_tombstone_is_deleted_and_has_no_hash() {
        let v = FileVersion::tombstone("a.txt", VersionVector::new(), A, 0);
        assert!(v.is_deleted());
        assert_eq!(v.content.hash(), None);
    }

    #[test]
    fn identical_bytes_compare_equal_even_from_different_devices() {
        let b = DeviceId::from_bytes([0xB2; 32]);
        let mut v1 = VersionVector::new();
        v1.increment(A);
        let mut v2 = VersionVector::new();
        v2.increment(b);

        let left = FileVersion::file("a.txt", [7; 32], 100, v1, A, 111);
        let right = FileVersion::file("a.txt", [7; 32], 100, v2, b, 222);
        assert!(left.same_content(&right), "history differs, content does not");
    }

    #[test]
    fn different_sizes_are_different_content() {
        let v = VersionVector::new();
        let left = FileVersion::file("a.txt", [7; 32], 100, v.clone(), A, 0);
        let right = FileVersion::file("a.txt", [7; 32], 200, v, A, 0);
        assert!(!left.same_content(&right));
    }

    #[test]
    fn two_tombstones_are_the_same_content() {
        let a = FileVersion::tombstone("a.txt", VersionVector::new(), A, 1);
        let b = FileVersion::tombstone("a.txt", VersionVector::new(), A, 2);
        assert!(a.same_content(&b));
    }
}
