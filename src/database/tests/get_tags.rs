use crate::database::{prelude::*, tests::init_test_database};

#[test]
pub fn get_all_tags() {
    use crate::database::prelude::*;

    let handle: Database = init_test_database();

    assert_eq!(
        handle.get_all_tags().unwrap(),
        vec![
            Tag::new(0, "Tag 0".to_string(), "f00".to_string(), 0),
            Tag::new(1, "Tag 1".to_string(), "0f0".to_string(), 0),
            Tag::new(2, "Tag 2".to_string(), "00f".to_string(), 0),
        ]
    );
}

#[test]
pub fn get_tags_from_audio_file() {
    use crate::database::prelude::*;

    let handle: Database = init_test_database();

    assert_eq!(
        handle.get_tags_for_audio_file(0).unwrap(),
        vec![Tag::new(0, "Tag 0".to_string(), "f00".to_string(), 0),]
    );

    assert_eq!(
        handle.get_tags_for_audio_file(1).unwrap(),
        vec![
            Tag::new(1, "Tag 1".to_string(), "0f0".to_string(), 0),
            Tag::new(2, "Tag 2".to_string(), "00f".to_string(), 0),
        ]
    );
}
