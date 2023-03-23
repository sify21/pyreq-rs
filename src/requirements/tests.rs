use std::cmp::Ordering;

use crate::requirements::LocalVersionPart;

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
