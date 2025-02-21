use crate::database::{prelude::*, tests::init_test_database};

#[test]
pub fn get_all_audio_files() {
    use crate::database::prelude::*;

    let handle: Database = init_test_database();

    assert_eq!(
        handle.get_all_audio_files().unwrap(),
        vec![
            AudioFile::new(
                0,
                "Audio File 0".to_string(),
                0,
                0.,
                0.,
                0.,
                0.,
                Some(0.),
                Some(0.),
                0.
            ),
            AudioFile::new(
                1,
                "Audio File 1".to_string(),
                1,
                0.,
                0.,
                0.,
                0.,
                Some(0.),
                Some(0.),
                0.
            ),
        ]
    );
}

#[test]
pub fn get_child_audio_files() {
    use crate::database::prelude::*;

    let handle: Database = init_test_database();

    assert_eq!(
        handle.get_child_audio_files(0).unwrap(),
        vec![AudioFile::new(
            0,
            "Audio File 0".to_string(),
            0,
            0.,
            0.,
            0.,
            0.,
            Some(0.),
            Some(0.),
            0.
        )],
    );
}
