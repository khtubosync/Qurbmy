//! Opening an index written by another build (decision 0047).

use qurb_storage::db::Db;

/// Migrations only go forward, so before one runs the index is copied as it
/// was: the way back, if an upgrade has to be undone.
#[test]
fn an_index_about_to_be_migrated_is_copied_first() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("index.db");
    drop(Db::open(&path).unwrap());
    // As an older build would have left it.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "user_version", (qurb_storage::db::SCHEMA_VERSION - 1) as i64).unwrap();
    drop(conn);

    drop(Db::open(&path).unwrap());
    let copy = dir.path().join(format!("index.before-schema-{}.db", qurb_storage::db::SCHEMA_VERSION));
    assert!(copy.exists(), "no copy was taken before migrating");
    let version: i64 = rusqlite::Connection::open(&copy)
        .unwrap()
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version as usize, qurb_storage::db::SCHEMA_VERSION - 1, "the copy is not as it was");
}

/// And an index from a newer build is refused, not guessed at.
#[test]
fn an_index_from_a_newer_build_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("index.db");
    drop(Db::open(&path).unwrap());
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.pragma_update(None, "user_version", (qurb_storage::db::SCHEMA_VERSION + 1) as i64).unwrap();
    drop(conn);

    let refused = Db::open(&path).err().expect("opened an index from the future");
    assert!(refused.to_string().contains("newer qurb"), "{refused}");
}
