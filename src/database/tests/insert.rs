#[test]
fn insert_from_directory() {
    use crate::database::Database;
    use std::path::Path;

    let handle = Database::from_directory(Path::new("test_files/").to_path_buf()).unwrap();

    for col in handle.get_all_collections().unwrap() {
        println!("{:?}", col)
    }

    for audio_file in handle.get_all_audio_files().unwrap() {
        println!("{:?}", audio_file)
    }
}
