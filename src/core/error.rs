use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    InvalidConfiguration(String),
    UnsupportedDatabaseEngine(String),
    UnsupportedType(String),
    MissingRequiredField(String),
    Database(String),
    Process(String),
    NotImplemented(&'static str),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(message) => write!(f, "configuracion invalida: {message}"),
            Self::UnsupportedDatabaseEngine(engine) => {
                write!(f, "motor no soportado: {engine}")
            }
            Self::UnsupportedType(db_type) => write!(f, "tipo no soportado: {db_type}"),
            Self::MissingRequiredField(field) => write!(f, "campo requerido faltante: {field}"),
            Self::Database(message) => write!(f, "error de base de datos: {message}"),
            Self::Process(message) => write!(f, "error de proceso externo: {message}"),
            Self::NotImplemented(feature) => write!(f, "pendiente de implementar: {feature}"),
        }
    }
}

impl Error for AppError {}
