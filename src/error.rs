//! Tipos de error para el almacén clave-valor `MiniKV`.
//!
//! Este módulo define todos los errores posibles que pueden ocurrir durante
//! el parseo de comandos, validación de datos y operaciones de archivos.

/// Errores que pueden ocurrir en el sistema.
///
/// # Ejemplo
/// ```
/// use minikv::Error;
/// let error = Error::NotFound;
/// assert_eq!(error.msg(), "ERROR: NOT FOUND");
/// ```
#[derive(Debug, PartialEq)]
pub enum Error {
    NotFound,
    ExtraArgument,
    InvalidDataFile,
    InvalidLogFile,
    MissingArgument,
    UnknownCommand,
}

impl Error {
    /// Retorna el mensaje de error formateado.
    /// Todos los mensajes de error siguen el formato `"ERROR: <DESCRIPCIÓN>"`.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::Error;
    /// assert_eq!(Error::NotFound.msg(), "ERROR: NOT FOUND");
    /// assert_eq!(Error::MissingArgument.msg(), "ERROR: MISSING ARGUMENT");
    /// ```
    #[must_use]
    pub fn msg(&self) -> String {
        match self {
            Error::NotFound => "ERROR: NOT FOUND".to_string(),
            Error::ExtraArgument => "ERROR: EXTRA ARGUMENT".to_string(),
            Error::InvalidDataFile => "ERROR: INVALID DATA FILE".to_string(),
            Error::InvalidLogFile => "ERROR: INVALID LOG FILE".to_string(),
            Error::MissingArgument => "ERROR: MISSING ARGUMENT".to_string(),
            Error::UnknownCommand => "ERROR: UNKNOWN COMMAND".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_not_found_msg() {
        assert_eq!(Error::NotFound.msg(), "ERROR: NOT FOUND");
    }

    #[test]
    fn test_error_extra_argument_msg() {
        assert_eq!(Error::ExtraArgument.msg(), "ERROR: EXTRA ARGUMENT");
    }

    #[test]
    fn test_error_missing_argument_msg() {
        assert_eq!(Error::MissingArgument.msg(), "ERROR: MISSING ARGUMENT");
    }

    #[test]
    fn test_error_unknown_command_msg() {
        assert_eq!(Error::UnknownCommand.msg(), "ERROR: UNKNOWN COMMAND");
    }

    #[test]
    fn test_error_invalid_data_file_msg() {
        assert_eq!(Error::InvalidDataFile.msg(), "ERROR: INVALID DATA FILE");
    }

    #[test]
    fn test_error_invalid_log_file_msg() {
        assert_eq!(Error::InvalidLogFile.msg(), "ERROR: INVALID LOG FILE");
    }
}
