//! The key: our group id <-> the Kanidm group name.

use fs_identity_kanidm::naming::{PREFIX, group_id, group_name};
use fs_objects::ObjectId;

#[test]
fn the_name_is_derived_from_the_id_and_reads_back() {
    let id = ObjectId::new();
    let name = group_name(id);
    assert_eq!(name, format!("fsg-{id}"));
    assert_eq!(group_id(&name), Some(id));
    assert_eq!(
        group_id(&format!("{name}@idm.example.org")),
        Some(id),
        "an SPN reads back too"
    );
}

#[test]
fn the_name_is_deterministic() {
    let id = ObjectId::new();
    assert_eq!(group_name(id), group_name(id));
}

#[test]
fn the_name_obeys_kanidms_name_rule() {
    // server/lib/src/value.rs INAME_RE: ^[a-z][a-z0-9-_.]{0,63}$
    for _ in 0..50 {
        let name = group_name(ObjectId::new());
        assert!(name.len() <= 64, "{name}");
        assert!(name.starts_with(|c: char| c.is_ascii_lowercase()));
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-_.".contains(c)),
            "{name}"
        );
    }
}

#[test]
fn names_that_are_not_ours_do_not_read_as_ours() {
    for other in [
        "admin",
        "idm_admins@idm.example.org",
        "fsg-",
        "fsg-not-a-uuid",
        "xfsg-0198a6f0-0000-7000-8000-000000000000",
    ] {
        assert_eq!(group_id(other), None, "{other}");
    }
    assert_eq!(PREFIX, "fsg-");
}
