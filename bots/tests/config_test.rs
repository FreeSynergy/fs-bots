use fs_bots::config::{FsLevel, MessengerAccess};

#[test]
fn fs_level_default_is_member() {
    assert!(matches!(FsLevel::default(), FsLevel::Member));
}

#[test]
fn messenger_access_default_is_everyone() {
    assert!(matches!(
        MessengerAccess::default(),
        MessengerAccess::Everyone
    ));
}

#[test]
fn fs_level_variants_are_distinct() {
    assert!(!matches!(FsLevel::Public, FsLevel::Member));
    assert!(!matches!(FsLevel::Admin, FsLevel::Operator));
}
