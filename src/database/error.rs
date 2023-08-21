#[derive(Debug)]
pub enum DatabaseError {
    ConnectionClosed,
    PathNotDirectory,
    RusqliteError(rusqlite::Error),
    IOError(std::io::Error),
    NotifyError(notify::Error),
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

impl From<notify::Error> for DatabaseError {
    fn from(value: notify::Error) -> Self {
        Self::NotifyError(value)
    }
}
