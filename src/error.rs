//! Tipos de error para el servidor y cliente de `MiniKV`.
//!
//! Este módulo centraliza los códigos de error definidos en la consigna
//! y su clasificación por categoría.

/// Categorías de error según la consigna.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    Client,
    Communication,
    Server,
}

/// Errores que pueden ocurrir en el sistema.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    // Errores de cliente
    NotFound,
    ExtraArgument,
    MissingArgument,
    UnknownCommand,
    // Errores de comunicación
    Timeout,
    ConnectionClosed,
    ClientSocketBinding,
    // Errores del servidor
    InvalidArgs,
    ServerSocketBinding,
    InvalidDataFile,
    InvalidLogFile,
}

impl Error {
    /// Retorna el código de error exacto según la consigna.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Error::NotFound => "NOT FOUND",
            Error::ExtraArgument => "EXTRA ARGUMENT",
            Error::MissingArgument => "MISSING ARGUMENT",
            Error::UnknownCommand => "UNKNOWN COMMAND",
            Error::Timeout => "TIMEOUT",
            Error::ConnectionClosed => "CONNECTION CLOSED",
            Error::ClientSocketBinding => "CLIENT SOCKET BINDING",
            Error::InvalidArgs => "INVALID ARGS",
            Error::ServerSocketBinding => "SERVER SOCKET BINDING",
            Error::InvalidDataFile => "INVALID DATA FILE",
            Error::InvalidLogFile => "INVALID LOG FILE",
        }
    }

    /// Retorna la categoría del error.
    #[must_use]
    pub fn category(self) -> ErrorCategory {
        match self {
            Error::NotFound
            | Error::ExtraArgument
            | Error::MissingArgument
            | Error::UnknownCommand => ErrorCategory::Client,
            Error::Timeout | Error::ConnectionClosed | Error::ClientSocketBinding => {
                ErrorCategory::Communication
            }
            Error::InvalidArgs
            | Error::ServerSocketBinding
            | Error::InvalidDataFile
            | Error::InvalidLogFile => ErrorCategory::Server,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::NotFound.code(), "NOT FOUND");
        assert_eq!(Error::ExtraArgument.code(), "EXTRA ARGUMENT");
        assert_eq!(Error::MissingArgument.code(), "MISSING ARGUMENT");
        assert_eq!(Error::UnknownCommand.code(), "UNKNOWN COMMAND");
        assert_eq!(Error::Timeout.code(), "TIMEOUT");
        assert_eq!(Error::ConnectionClosed.code(), "CONNECTION CLOSED");
        assert_eq!(Error::ClientSocketBinding.code(), "CLIENT SOCKET BINDING");
        assert_eq!(Error::InvalidArgs.code(), "INVALID ARGS");
        assert_eq!(Error::ServerSocketBinding.code(), "SERVER SOCKET BINDING");
        assert_eq!(Error::InvalidDataFile.code(), "INVALID DATA FILE");
        assert_eq!(Error::InvalidLogFile.code(), "INVALID LOG FILE");
    }
}
