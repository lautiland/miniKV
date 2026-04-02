use minikv::*;
use std::env;

/// Representa un comando ingresado por el usuario, con su tipo y argumentos.
pub struct Command {
    command_type: CommandType,
    key: Option<String>,
    value: Option<String>,
}
impl Command {
    /// Parsea los argumentos de línea de comandos y construye un Comando.
    /// Valida que la cantidad de argumentos sea válida para cada tipo de comando.
    pub fn new(args: &[String]) -> Result<Command, String> {
        let Some(name) = args.get(1) else {
            return Err(Error::UnknownCommand.msg());
        };
        let Some(command_type) = CommandType::from_str(name) else {
            return Err(Error::UnknownCommand.msg());
        };

        let received_args = args.len() - 2;
        if received_args < command_type.min_argument_count() {
            return Err(Error::MissingArgument.msg());
        }
        if received_args > command_type.max_argument_count() {
            return Err(Error::ExtraArgument.msg());
        }

        let key = args.get(2).cloned();
        let value = get_set_value(&command_type, args, received_args);

        Ok(Command {
            command_type,
            key,
            value,
        })
    }
    /// Obtiene la clave del comando, retorna error si no existe.
    pub fn get_key(&self) -> Result<String, String> {
        match &self.key {
            Some(key) => Ok(key.clone()),
            None => Err(Error::NotFound.msg()),
        }
    }
    /// Obtiene el valor del comando, retorna cadena vacía para SET sin valor.
    pub fn get_value(&self) -> Result<String, String> {
        match &self.value {
            Some(value) => Ok(value.clone()),
            None => {
                if self.command_type == CommandType::Set {
                    Ok("".to_string())
                } else {
                    Err(Error::NotFound.msg())
                }
            }
        }
    }
}
/// Define los tipos de comandos disponibles en la aplicación.
/// Cada tipo tiene una cantidad mínima y máxima de argumentos permitida.
#[derive(Debug, PartialEq, Copy, Clone)]
enum CommandType {
    Set,
    Get,
    Length,
    Snapshot,
}
impl CommandType {
    /// Convierte una cadena en un tipo de comando.
    pub fn from_str(s: &str) -> Option<CommandType> {
        match s {
            "set" => Some(CommandType::Set),
            "get" => Some(CommandType::Get),
            "length" => Some(CommandType::Length),
            "snapshot" => Some(CommandType::Snapshot),
            _ => None,
        }
    }
    /// Retorna la cantidad mínima de argumentos requeridos para este comando.
    pub fn min_argument_count(&self) -> usize {
        match self {
            CommandType::Set | CommandType::Get => 1,
            CommandType::Length | CommandType::Snapshot => 0,
        }
    }
    /// Retorna la cantidad máxima de argumentos permitidos para este comando.
    pub fn max_argument_count(&self) -> usize {
        match self {
            CommandType::Set => 2,
            CommandType::Get => 1,
            CommandType::Length | CommandType::Snapshot => 0,
        }
    }
}

/// Obtiene el valor para el comando SET si corresponde.
fn get_set_value(
    command_type: &CommandType,
    args: &[String],
    received_args: usize,
) -> Option<String> {
    if *command_type == CommandType::Set && received_args >= 2 {
        args.get(3).cloned()
    } else {
        None
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command: Result<Command, String> = Command::new(&args);
    match command {
        Ok(arg) => match (arg.command_type, arg.get_key(), arg.get_value()) {
            (CommandType::Set, Ok(clave), Ok(valor)) => execute_set(clave, valor),
            (CommandType::Get, Ok(clave), _) => execute_get(clave),
            (CommandType::Length, _, _) => execute_length(),
            (CommandType::Snapshot, _, _) => execute_snapshot(),
            (_, _, _) => eprintln!("{}", Error::NotFound.msg()),
        },
        Err(e) => eprintln!("{}", e),
    }
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
        let result = Command::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.command_type, CommandType::Set);
                assert_eq!(cmd.key, Some("clave".to_string()));
                assert_eq!(cmd.value, Some("valor".to_string()));
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    #[test]
    fn test02_verify_command_get() {
        let args = vec!["program".into(), "get".into(), "clave".into()];
        let result = Command::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.command_type, CommandType::Get);
                assert_eq!(cmd.key, Some("clave".to_string()));
                assert_eq!(cmd.value, None);
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    #[test]
    fn test03_verify_command_length() {
        let args = vec!["program".into(), "length".into()];
        let result = Command::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.command_type, CommandType::Length);
                assert_eq!(cmd.key, None);
                assert_eq!(cmd.value, None);
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    #[test]
    fn test04_verify_command_snapshot() {
        let args = vec!["program".into(), "snapshot".into()];
        let result = Command::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.command_type, CommandType::Snapshot);
                assert_eq!(cmd.key, None);
                assert_eq!(cmd.value, None);
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    // Argumentos inválidos
    #[test]
    fn test05_verify_command_unknown() {
        let args = vec!["program".into(), "unknown".into()];
        let result = Command::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test06_verify_command_set_without_parameters() {
        let args = vec!["program".into(), "set".into()];
        let result = Command::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test07_verify_command_set_one_parameter() {
        let args = vec!["program".into(), "set".into(), "clave".into()];
        let result = Command::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.command_type, CommandType::Set);
                assert_eq!(cmd.key, Some("clave".to_string()));
                assert_eq!(cmd.value, None);
            }
            Err(_) => {
                panic!("El comando set con 1 parámetro debería haberse creado correctamente.")
            }
        }
    }

    #[test]
    fn test08_verify_command_get_missing_arguments() {
        let args = vec!["program".into(), "get".into()];
        let result = Command::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test09_verify_command_no_arguments() {
        let args = vec!["program".into()];
        let result = Command::new(&args);
        assert!(result.is_err());
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
        assert!(result.is_err());
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
        assert!(result.is_err());
    }

    #[test]
    fn test12_verify_command_length_with_arguments() {
        let args = vec!["program".into(), "length".into(), "extra".into()];
        let result = Command::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test13_verify_command_snapshot_with_arguments() {
        let args = vec!["program".into(), "snapshot".into(), "extra".into()];
        let result = Command::new(&args);
        assert!(result.is_err());
    }
}
