#[derive(Debug)]
pub enum DatabaseError {
    ConnectionClosed,
    PathNotDirecotry,
    RusqliteError(rusqlite::Error),
    IOError(std::io::Error),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(value: rusqlite::Error) -> Self {
        Self::RusqliteError(value)
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
