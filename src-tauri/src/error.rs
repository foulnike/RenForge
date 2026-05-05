use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Ошибка ввода/вывода: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ошибка базы данных: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Системная ошибка: {0}")]
    Custom(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}