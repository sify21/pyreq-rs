use std::cmp::Ordering;

use crate::{
    parser::{requirement_specifier::version_one, version::version_scheme},
    requirements::LocalVersionPart,
};

use super::Version;

#[test]
fn test_local_version_part_ordering() {
    assert_eq!(
        LocalVersionPart::Num(2).cmp(&LocalVersionPart::LowerStr("abc".to_string())),
        Ordering::Greater
    );
    assert_eq!(
        LocalVersionPart::LowerStr("abc".to_string()).cmp(&LocalVersionPart::Num(2)),
        Ordering::Less
    );
    assert_eq!(
        LocalVersionPart::Num(2).cmp(&LocalVersionPart::Num(2)),
        Ordering::Equal
    );
    assert_eq!(
        LocalVersionPart::Num(2).cmp(&LocalVersionPart::Num(3)),
        Ordering::Less
    );
    assert_eq!(
        LocalVersionPart::Num(2).cmp(&LocalVersionPart::Num(1)),
        Ordering::Greater
    );
    assert_eq!(
        LocalVersionPart::LowerStr("abc".to_string())
            .cmp(&LocalVersionPart::LowerStr("abc".to_string())),
        Ordering::Equal
    );
    assert_eq!(
        LocalVersionPart::LowerStr("abc".to_string())
            .cmp(&LocalVersionPart::LowerStr("efg".to_string())),
        Ordering::Less
    );
    assert_eq!(
        LocalVersionPart::LowerStr("efg".to_string())
            .cmp(&LocalVersionPart::LowerStr("abc".to_string())),
        Ordering::Greater
    );
}

#[test]
fn test_version_ordering() {
    // https://github.com/pypa/packaging/blob/main/tests/test_version.py
    let versions = [
        "1.0.dev456",
        "1.0a1",
        "1.0a2.dev456",
        "1.0a12.dev456",
        "1.0a12",
        "1.0b1.dev456",
        "1.0b2",
        "1.0b2.post345.dev456",
        "1.0b2.post345",
        "1.0b2-346",
        "1.0c1.dev456",
        "1.0c1",
        "1.0rc2",
        "1.0c3",
        "1.0",
        "1.0.post456.dev34",
        "1.0.post456",
        "1.1.dev1",
        "1.2+123abc",
        "1.2+123abc456",
        "1.2+abc",
        "1.2+abc123",
        "1.2+abc123def",
        "1.2+1234.abc",
        "1.2+123456",
        "1.2.r32+123456",
        "1.2.rev33+123456",
        "1!1.0.dev456",
        "1!1.0a1",
        "1!1.0a2.dev456",
        "1!1.0a12.dev456",
        "1!1.0a12",
        "1!1.0b1.dev456",
        "1!1.0b2",
        "1!1.0b2.post345.dev456",
        "1!1.0b2.post345",
        "1!1.0b2-346",
        "1!1.0c1.dev456",
        "1!1.0c1",
        "1!1.0rc2",
        "1!1.0c3",
        "1!1.0",
        "1!1.0.post456.dev34",
        "1!1.0.post456",
        "1!1.1.dev1",
        "1!1.2+123abc",
        "1!1.2+123abc456",
        "1!1.2+abc",
        "1!1.2+abc123",
        "1!1.2+abc123def",
        "1!1.2+1234.abc",
        "1!1.2+123456",
        "1!1.2.r32+123456",
        "1!1.2.rev33+123456",
    ];
    let vers: Vec<Version> = versions
        .iter()
        .map(|&s| version_scheme(s).unwrap().1)
        .collect();
    for i in 0..vers.len() {
        for j in i + 1..vers.len() {
            assert!(vers[i] < vers[j]);
        }
    }
    for i in 0..vers.len() {
        for j in i..vers.len() {
            assert!(vers[i] <= vers[j]);
        }
    }
    for i in 0..vers.len() {
        assert!(vers[i] == vers[i]);
    }
    for i in 0..vers.len() {
        for j in 0..vers.len() {
            if i != j {
                assert!(vers[i] != vers[j]);
            }
        }
    }
    for i in 0..vers.len() {
        for j in 0..i + 1 {
            assert!(vers[i] >= vers[j]);
        }
    }
    for i in 0..vers.len() {
        for j in 0..i {
            assert!(vers[i] > vers[j]);
        }
    }
    for i in 0..vers.len() {
        for j in 0..i + 1 {
            assert!(!(vers[i] < vers[j]));
        }
    }
    for i in 0..vers.len() {
        for j in 0..i {
            assert!(!(vers[i] <= vers[j]));
        }
    }
    for i in 0..vers.len() {
        for j in 0..vers.len() {
            if i != j {
                assert!(!(vers[i] == vers[j]));
            }
        }
    }
    for i in 0..vers.len() {
        assert!(!(vers[i] != vers[i]));
    }
    for i in 0..vers.len() {
        for j in i + 1..vers.len() {
            assert!(!(vers[i] >= vers[j]));
        }
    }
    for i in 0..vers.len() {
        for j in i..vers.len() {
            assert!(!(vers[i] > vers[j]));
        }
    }
}

#[test]
fn test_spec_contains_version() {
    let contains = [
        ("2.0", "==2"),
        ("2.0", "==2.0"),
        ("2.0", "==2.0.0"),
        ("2.0+deadbeef", "==2"),
        ("2.0+deadbeef", "==2.0"),
        ("2.0+deadbeef", "==2.0.0"),
        ("2.0+deadbeef", "==2+deadbeef"),
        ("2.0+deadbeef", "==2.0+deadbeef"),
        ("2.0+deadbeef", "==2.0.0+deadbeef"),
        ("2.0+deadbeef.0", "==2.0.0+deadbeef.00"),
        ("2.dev1", "==2.*"),
        ("2a1", "==2.*"),
        ("2a1.post1", "==2.*"),
        ("2b1", "==2.*"),
        ("2b1.dev1", "==2.*"),
        ("2c1", "==2.*"),
        ("2c1.post1.dev1", "==2.*"),
        ("2c1.post1.dev1", "==2.0.*"),
        ("2rc1", "==2.*"),
        ("2rc1", "==2.0.*"),
        ("2", "==2.*"),
        ("2", "==2.0.*"),
        ("2", "==0!2.*"),
        ("0!2", "==2.*"),
        ("2.0", "==2.*"),
        ("2.0.0", "==2.*"),
        ("2.1+local.version", "==2.1.*"),
        ("2.1", "!=2"),
        ("2.1", "!=2.0"),
        ("2.0.1", "!=2"),
        ("2.0.1", "!=2.0"),
        ("2.0.1", "!=2.0.0"),
        ("2.0", "!=2.0+deadbeef"),
        ("2.0", "!=3.*"),
        ("2.1", "!=2.0.*"),
        ("2.0", ">=2"),
        ("2.0", ">=2.0"),
        ("2.0", ">=2.0.0"),
        ("2.0.post1", ">=2"),
        ("2.0.post1.dev1", ">=2"),
        ("2.0", "<=2"),
        ("2.0", "<=2.0"),
        ("2.0", "<=2.0.0"),
        ("2.0.dev1", "<=2"),
        ("2.0a1", "<=2"),
        ("2.0a1.dev1", "<=2"),
        ("2.0b1", "<=2"),
        ("2.0b1.post1", "<=2"),
        ("2.0c1", "<=2"),
        ("2.0c1.post1.dev1", "<=2"),
        ("2.0rc1", "<=2"),
        ("1", "<=2"),
        ("3", ">2"),
        ("2.1", ">2.0"),
        ("2.0.1", ">2"),
        ("2.1.post1", ">2"),
        ("2.1+local.version", ">2"),
        ("1", "<2"),
        ("2.0", "<2.1"),
        ("2.0.dev0", "<2.1"),
        ("1", "~=1.0"),
        ("1.0.1", "~=1.0"),
        ("1.1", "~=1.0"),
        ("1.9999999", "~=1.0"),
        ("1.1", "~=1.0a1"),
        ("2022.01.01", "~=2022.01.01"),
        ("2!1.0", "~=2!1.0"),
        ("2!1.0", "==2!1.*"),
        ("2!1.0", "==2!1.0"),
        ("2!1.0", "!=1.0"),
        ("2!1.0.0", "==2!1.0.*"),
        ("2!1.0.0", "==2!1.*"),
        ("1.0", "!=2!1.0"),
        ("1.0", "<=2!0.1"),
        ("2!1.0", ">=2.0"),
        ("1.0", "<2!0.1"),
        ("2!1.0", ">2.0"),
        ("2.0.5", ">2.0dev"),
    ];
    let not_contains = [
        ("2.1", "==2"),
        ("2.1", "==2.0"),
        ("2.1", "==2.0.0"),
        ("2.0", "==2.0+deadbeef"),
        ("2.0", "==3.*"),
        ("2.1", "==2.0.*"),
        ("2.0", "!=2"),
        ("2.0", "!=2.0"),
        ("2.0", "!=2.0.0"),
        ("2.0+deadbeef", "!=2"),
        ("2.0+deadbeef", "!=2.0"),
        ("2.0+deadbeef", "!=2.0.0"),
        ("2.0+deadbeef", "!=2+deadbeef"),
        ("2.0+deadbeef", "!=2.0+deadbeef"),
        ("2.0+deadbeef", "!=2.0.0+deadbeef"),
        ("2.0+deadbeef.0", "!=2.0.0+deadbeef.00"),
        ("2.dev1", "!=2.*"),
        ("2a1", "!=2.*"),
        ("2a1.post1", "!=2.*"),
        ("2b1", "!=2.*"),
        ("2b1.dev1", "!=2.*"),
        ("2c1", "!=2.*"),
        ("2c1.post1.dev1", "!=2.*"),
        ("2c1.post1.dev1", "!=2.0.*"),
        ("2rc1", "!=2.*"),
        ("2rc1", "!=2.0.*"),
        ("2", "!=2.*"),
        ("2", "!=2.0.*"),
        ("2.0", "!=2.*"),
        ("2.0.0", "!=2.*"),
        ("2.0.dev1", ">=2"),
        ("2.0a1", ">=2"),
        ("2.0a1.dev1", ">=2"),
        ("2.0b1", ">=2"),
        ("2.0b1.post1", ">=2"),
        ("2.0c1", ">=2"),
        ("2.0c1.post1.dev1", ">=2"),
        ("2.0rc1", ">=2"),
        ("1", ">=2"),
        ("2.0.post1", "<=2"),
        ("2.0.post1.dev1", "<=2"),
        ("3", "<=2"),
        ("1", ">2"),
        ("2.0.dev1", ">2"),
        ("2.0a1", ">2"),
        ("2.0a1.post1", ">2"),
        ("2.0b1", ">2"),
        ("2.0b1.dev1", ">2"),
        ("2.0c1", ">2"),
        ("2.0c1.post1.dev1", ">2"),
        ("2.0rc1", ">2"),
        ("2.0", ">2"),
        ("2.0.post1", ">2"),
        ("2.0.post1.dev1", ">2"),
        ("2.0+local.version", ">2"),
        ("2.0.dev1", "<2"),
        ("2.0a1", "<2"),
        ("2.0a1.post1", "<2"),
        ("2.0b1", "<2"),
        ("2.0b2.dev1", "<2"),
        ("2.0c1", "<2"),
        ("2.0c1.post1.dev1", "<2"),
        ("2.0rc1", "<2"),
        ("2.0", "<2"),
        ("2.post1", "<2"),
        ("2.post1.dev1", "<2"),
        ("3", "<2"),
        ("2.0", "~=1.0"),
        ("1.1.0", "~=1.0.0"),
        ("1.1.post1", "~=1.0.0"),
        ("1.0", "~=2!1.0"),
        ("2!1.0", "~=1.0"),
        ("2!1.0", "==1.0"),
        ("1.0", "==2!1.0"),
        ("2!1.0", "==1.*"),
        ("1.0", "==2!1.*"),
        ("2!1.0", "!=2!1.0"),
    ];
    for (ver, spec) in contains {
        let (_, s) = version_one(spec).unwrap();
        assert!(s.contains(ver));
    }
    for (ver, spec) in not_contains {
        let (_, s) = version_one(spec).unwrap();
        assert!(!s.contains(ver));
    }
}
