//! Parseo y validación de comandos para `MiniKV`.
//!
//! Este módulo maneja el parseo de comandos de texto y argumentos de
//! línea de comandos, validando la cantidad de parámetros.

use crate::Error;

/// Representa un comando ingresado por el usuario, con su tipo y argumentos.
///
/// # Ejemplo
/// ```
/// use minikv::Command;
///
/// let args = vec!["program".into(), "set".into(), "clave".into(), "valor".into()];
/// let cmd: Command = Command::new(&args).expect("command válido");
/// assert_eq!(cmd.get_key(), Ok("clave"));
/// assert_eq!(cmd.get_value(), Ok(Some("valor")));
/// ```
pub struct Command {
    cmd_type: CommandType,
    key: Option<String>,
    value: Option<String>,
}

impl Command {
    /// Parsea los argumentos de línea de comandos y construye un Comando.
    /// Valida que la cantidad de argumentos sea válida para cada tipo de comando.
    ///
    /// # Errors
    /// Retorna error si el comando es desconocido o tiene argumentos inválidos.
    pub fn new(args: &[String]) -> Result<Command, Error> {
        let Some(name) = args.get(1) else {
            return Err(Error::UnknownCommand);
        };
        let Some(cmd_type) = CommandType::parse(name) else {
            return Err(Error::UnknownCommand);
        };

        let received_args = args.len() - 2;
        if received_args < cmd_type.min_argument_count() {
            return Err(Error::MissingArgument);
        }
        if received_args > cmd_type.max_argument_count() {
            return Err(Error::ExtraArgument);
        }

        let key = args.get(2).cloned();
        let value = get_set_value(cmd_type, args, received_args);

        Ok(Command {
            cmd_type,
            key,
            value,
        })
    }

    /// Parsea un comando a partir de una cadena de texto.
    /// Permite argumentos entre comillas y escapes con `\"`.
    /// Valida que el comando tenga la cantidad correcta de argumentos.
    ///
    /// # Errors
    /// Retorna error si el comando es desconocido o tiene argumentos inválidos.
    pub fn parse_from_string(line: &str) -> Result<Command, Error> {
        let parts = parse_parts(line)?;
        let Some(cmd_text) = parts.first() else {
            return Err(Error::UnknownCommand);
        };
        let Some(cmd_type) = CommandType::parse(cmd_text) else {
            return Err(Error::UnknownCommand);
        };
        let key = parts.get(1).cloned();
        let value = parts.get(2).cloned();
        if parts.len() > 3 {
            return Err(Error::ExtraArgument);
        }
        let received_args = key.as_ref().map_or(0, |_| 1) + value.as_ref().map_or(0, |_| 1);
        if received_args < cmd_type.min_argument_count() {
            return Err(Error::MissingArgument);
        }
        if received_args > cmd_type.max_argument_count() {
            return Err(Error::ExtraArgument);
        }
        Ok(Command {
            cmd_type,
            key,
            value,
        })
    }

    /// Obtiene la clave del comando, retorna error si no existe.
    ///
    /// # Errors
    /// Retorna error si el comando no tiene clave.
    pub fn get_key(&self) -> Result<&str, Error> {
        match &self.key {
            Some(key) => Ok(key.as_str()),
            None => Err(Error::NotFound),
        }
    }

    /// Obtiene el valor del comando, devuelve `None` para SET sin valor.
    ///
    /// # Errors
    /// Retorna error si el comando no tiene valor y no es SET.
    pub fn get_value(&self) -> Result<Option<&str>, Error> {
        match &self.value {
            Some(value) => Ok(Some(value.as_str())),
            None => {
                if self.cmd_type == CommandType::Set {
                    Ok(None)
                } else {
                    Err(Error::NotFound)
                }
            }
        }
    }

    /// Retorna el tipo de comando.
    #[must_use]
    pub fn get_type(&self) -> CommandType {
        self.cmd_type
    }
}

/// Define los tipos de comandos disponibles en la aplicación.
/// Cada tipo tiene una cantidad mínima y máxima de argumentos permitida.
///
/// # Comandos Soportados
/// - `Set`: Almacena o actualiza un par clave-valor (1-2 args)
/// - `Get`: Recupera un valor por clave (1 arg)
/// - `Length`: Obtiene la cantidad de claves almacenadas (0 args)
/// - `Snapshot`: Guarda el estado actual a disco (0 args)
///
/// # Ejemplo
/// ```
/// use minikv::CommandType;
/// let cmd = CommandType::parse("set").unwrap();
/// assert_eq!(cmd.min_argument_count(), 1);
/// assert_eq!(cmd.max_argument_count(), 2);
/// ```
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum CommandType {
    Set,
    Get,
    Length,
    Snapshot,
}

impl CommandType {
    /// Convierte una cadena en un tipo de comando.
    ///
    /// # Ejemplo
    /// ```
    /// use minikv::CommandType;
    /// assert_eq!(CommandType::parse("set"), Some(CommandType::Set));
    /// assert_eq!(CommandType::parse("get"), Some(CommandType::Get));
    /// assert_eq!(CommandType::parse("length"), Some(CommandType::Length));
    /// assert_eq!(CommandType::parse("snapshot"), Some(CommandType::Snapshot));
    /// assert_eq!(CommandType::parse("unknown"), None);
    /// ```
    #[must_use]
    pub fn parse(s: &str) -> Option<CommandType> {
        match s.to_lowercase().as_str() {
            "set" => Some(CommandType::Set),
            "get" => Some(CommandType::Get),
            "length" => Some(CommandType::Length),
            "snapshot" => Some(CommandType::Snapshot),
            _ => None,
        }
    }

    /// Retorna la cantidad mínima de argumentos requeridos para este comando.
    ///
    /// # Ejemplo
    ///
    /// ```
    /// use::minikv::CommandType;
    /// assert_eq!(CommandType::Set.min_argument_count(), 1);
    /// assert_eq!(CommandType::Length.min_argument_count(), 0);
    /// ```
    #[must_use]
    pub fn min_argument_count(self) -> usize {
        match self {
            CommandType::Set | CommandType::Get => 1,
            CommandType::Length | CommandType::Snapshot => 0,
        }
    }

    /// Retorna la cantidad máxima de argumentos permitidos para este comando.
    ///
    /// # Ejemplos
    ///
    /// ```
    /// use minikv::CommandType;
    ///
    /// assert_eq!(CommandType::Set.max_argument_count(), 2);
    /// assert_eq!(CommandType::Get.max_argument_count(), 1);
    /// ```
    #[must_use]
    pub fn max_argument_count(self) -> usize {
        match self {
            CommandType::Set => 2,
            CommandType::Get => 1,
            CommandType::Length | CommandType::Snapshot => 0,
        }
    }
}

/// Obtiene el valor para el comando SET si corresponde.
fn get_set_value(cmd_type: CommandType, args: &[String], received_args: usize) -> Option<String> {
    if cmd_type == CommandType::Set && received_args >= 2 {
        args.get(3).cloned()
    } else {
        None
    }
}

fn parse_parts(line: &str) -> Result<Vec<String>, Error> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escaped = false;
    for c in line.chars() {
        let action = parse_char(c, in_quotes, escaped);
        apply_char_action(
            &mut parts,
            &mut current,
            action,
            &mut in_quotes,
            &mut escaped,
        );
    }
    finish_parts(&mut parts, current, in_quotes)
}

#[derive(Copy, Clone)]
enum CharAction {
    Push(char),
    ToggleQuotes,
    BeginEscape,
    Split,
}

fn parse_char(c: char, in_quotes: bool, escaped: bool) -> CharAction {
    if escaped {
        return CharAction::Push(c);
    }
    if c == '\\' && in_quotes {
        return CharAction::BeginEscape;
    }
    if c == '"' {
        return CharAction::ToggleQuotes;
    }
    if c.is_whitespace() && !in_quotes {
        return CharAction::Split;
    }
    CharAction::Push(c)
}

fn apply_char_action(
    parts: &mut Vec<String>,
    current: &mut String,
    action: CharAction,
    in_quotes: &mut bool,
    escaped: &mut bool,
) {
    match action {
        CharAction::Push(c) => {
            current.push(c);
            *escaped = false;
        }
        CharAction::ToggleQuotes => *in_quotes = !*in_quotes,
        CharAction::BeginEscape => *escaped = true,
        CharAction::Split => {
            if !current.is_empty() {
                parts.push(std::mem::take(current));
            }
            *escaped = false;
        }
    }
}

fn finish_parts(
    parts: &mut Vec<String>,
    current: String,
    in_quotes: bool,
) -> Result<Vec<String>, Error> {
    if in_quotes {
        return Err(Error::ExtraArgument);
    }
    if !current.is_empty() {
        parts.push(current);
    }
    Ok(std::mem::take(parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Argumentos válidos
    #[test]
    fn test01_verify_command_set() {
        let args = vec![
            "program".into(),
            "set".into(),
            "clave".into(),
            "valor".into(),
        ];
        let cmd = Command::new(&args).unwrap();
        assert_eq!(cmd.get_type(), CommandType::Set);
        assert_eq!(cmd.get_key(), Ok("clave"));
        assert_eq!(cmd.get_value(), Ok(Some("valor")));
    }

    #[test]
    fn test02_verify_command_get() {
        let args = vec!["program".into(), "get".into(), "clave".into()];
        let cmd = Command::new(&args).unwrap();
        assert_eq!(cmd.get_type(), CommandType::Get);
        assert_eq!(cmd.get_key(), Ok("clave"));
    }

    #[test]
    fn test03_verify_command_length() {
        let args = vec!["program".into(), "length".into()];
        let cmd = Command::new(&args).unwrap();
        assert_eq!(cmd.get_type(), CommandType::Length);
    }

    #[test]
    fn test04_verify_command_snapshot() {
        let args = vec!["program".into(), "snapshot".into()];
        let cmd = Command::new(&args).unwrap();
        assert_eq!(cmd.get_type(), CommandType::Snapshot);
    }

    // Argumentos inválidos
    #[test]
    fn test05_verify_command_unknown() {
        let args = vec!["program".into(), "unknown".into()];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::UnknownCommand)));
    }

    #[test]
    fn test06_verify_command_set_without_parameters() {
        let args = vec!["program".into(), "set".into()];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::MissingArgument)));
    }

    #[test]
    fn test07_verify_command_set_one_parameter() {
        let args = vec!["program".into(), "set".into(), "clave".into()];
        let cmd = Command::new(&args).unwrap();
        assert_eq!(cmd.get_type(), CommandType::Set);
        assert_eq!(cmd.get_key(), Ok("clave"));
    }

    #[test]
    fn test08_verify_command_get_missing_arguments() {
        let args = vec!["program".into(), "get".into()];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::MissingArgument)));
    }

    #[test]
    fn test09_verify_command_no_arguments() {
        let args = vec!["program".into()];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::UnknownCommand)));
    }

    #[test]
    fn test10_verify_command_set_too_many_arguments() {
        let args = vec![
            "program".into(),
            "set".into(),
            "clave".into(),
            "valor".into(),
            "extra".into(),
        ];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::ExtraArgument)));
    }

    #[test]
    fn test11_verify_command_get_too_many_arguments() {
        let args = vec![
            "program".into(),
            "get".into(),
            "clave".into(),
            "extra".into(),
        ];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::ExtraArgument)));
    }

    #[test]
    fn test12_verify_command_length_with_arguments() {
        let args = vec!["program".into(), "length".into(), "extra".into()];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::ExtraArgument)));
    }

    #[test]
    fn test13_verify_command_snapshot_with_arguments() {
        let args = vec!["program".into(), "snapshot".into(), "extra".into()];
        let result = Command::new(&args);
        assert!(matches!(result, Err(Error::ExtraArgument)));
    }

    #[test]
    fn test14_get_key_returns_key() {
        let args = vec![
            "program".into(),
            "set".into(),
            "mi_clave".into(),
            "mi_valor".into(),
        ];
        let cmd = Command::new(&args).expect("Debería crear comando");
        assert_eq!(cmd.get_key(), Ok("mi_clave"));
    }

    #[test]
    fn test15_get_value_returns_value() {
        let args = vec![
            "program".into(),
            "set".into(),
            "clave".into(),
            "valor".into(),
        ];
        let cmd = Command::new(&args).expect("Debería crear comando");
        assert_eq!(cmd.get_value(), Ok(Some("valor")));
    }

    #[test]
    fn test16_get_value_for_set_without_value_returns_empty() {
        let args = vec!["program".into(), "set".into(), "clave".into()];
        let cmd = Command::new(&args).expect("Debería crear comando");
        assert_eq!(cmd.get_value(), Ok(None));
    }

    #[test]
    fn test17_cmd_type_min_argument_count() {
        assert_eq!(CommandType::Set.min_argument_count(), 1);
        assert_eq!(CommandType::Get.min_argument_count(), 1);
        assert_eq!(CommandType::Length.min_argument_count(), 0);
        assert_eq!(CommandType::Snapshot.min_argument_count(), 0);
    }

    #[test]
    fn test18_cmd_type_max_argument_count() {
        assert_eq!(CommandType::Set.max_argument_count(), 2);
        assert_eq!(CommandType::Get.max_argument_count(), 1);
        assert_eq!(CommandType::Length.max_argument_count(), 0);
        assert_eq!(CommandType::Snapshot.max_argument_count(), 0);
    }

    #[test]
    fn test19_parse_from_string_valid() {
        let cmd = Command::parse_from_string("set clave valor").unwrap();
        assert_eq!(cmd.get_type(), CommandType::Set);
        assert_eq!(cmd.get_key(), Ok("clave"));
        assert_eq!(cmd.get_value(), Ok(Some("valor")));
    }

    #[test]
    fn test20_parse_from_string_invalid() {
        let result = Command::parse_from_string("unknown cmd");
        assert!(matches!(result, Err(Error::UnknownCommand)));
    }

    #[test]
    fn test21_parse_missing_argument() {
        let result = Command::parse_from_string("get");
        assert!(matches!(result, Err(Error::MissingArgument)));
    }

    #[test]
    fn test22_parse_extra_argument() {
        let result = Command::parse_from_string("get clave extra");
        assert!(matches!(result, Err(Error::ExtraArgument)));
    }

    #[test]
    fn test23_parse_mayus_minus() {
        let cmd = Command::parse_from_string("SeT clave valor").unwrap();
        assert_eq!(cmd.get_type(), CommandType::Set);
        assert_eq!(cmd.get_key(), Ok("clave"));
        assert_eq!(cmd.get_value(), Ok(Some("valor")));
    }
}
