use minikv::*;
use std::env;

/// Representa un comando ingresado por el usuario, con su tipo y argumentos.
pub struct Comando {
    tipo: TipoDeComando,
    clave: Option<String>,
    valor: Option<String>,
}
impl Comando {
    /// Parsea los argumentos de línea de comandos y construye un Comando.
    /// Valida que la cantidad de argumentos sea válida para cada tipo de comando.
    pub fn new(args: &[String]) -> Result<Comando, String> {
        let nombre_comando: Option<String> = args.get(1).cloned();
        match nombre_comando {
            Some(nombre) => {
                let tipo_de_comando: Option<TipoDeComando> = TipoDeComando::from_str(&nombre);
                match tipo_de_comando {
                    Some(tipo) => {
                        let args_recibidos = args.len() - 2;
                        let args_min = tipo.cantidad_minima_argumentos();
                        let args_max = tipo.cantidad_maxima_argumentos();
                        if args_recibidos < args_min {
                            return Err(Error::MissingArgument.mensaje());
                        } else if args_recibidos > args_max {
                            return Err(Error::ExtraArgument.mensaje());
                        }
                        let clave = args.get(2).cloned();
                        let valor = if tipo == TipoDeComando::Set {
                            if args_recibidos == 1 {
                                None
                            } else {
                                args.get(3).cloned()
                            }
                        } else {
                            None
                        };
                        Ok(Comando { tipo, clave, valor })
                    }
                    _none => Err(Error::UnknownCommand.mensaje()),
                }
            }
            _none => Err(Error::UnknownCommand.mensaje()),
        }
    }
    /// Obtiene la clave del comando, retorna error si no existe.
    pub fn get_clave(&self) -> Result<String, String> {
        match &self.clave {
            Some(clave) => Ok(clave.clone()),
            None => Err(Error::NotFound.mensaje()),
        }
    }
    /// Obtiene el valor del comando, retorna cadena vacía para SET sin valor.
    pub fn get_valor(&self) -> Result<String, String> {
        match &self.valor {
            Some(valor) => Ok(valor.clone()),
            None => {
                if self.tipo == TipoDeComando::Set {
                    Ok("".to_string())
                } else {
                    Err(Error::NotFound.mensaje())
                }
            }
        }
    }
}
/// Define los tipos de comandos disponibles en la aplicación.
/// Cada tipo tiene una cantidad mínima y máxima de argumentos permitida.
#[derive(Debug, PartialEq, Copy, Clone)]
enum TipoDeComando {
    Set,
    Get,
    Length,
    Snapshot,
}
impl TipoDeComando {
    /// Convierte una cadena en un tipo de comando.
    pub fn from_str(s: &str) -> Option<TipoDeComando> {
        match s {
            "set" => Some(TipoDeComando::Set),
            "get" => Some(TipoDeComando::Get),
            "length" => Some(TipoDeComando::Length),
            "snapshot" => Some(TipoDeComando::Snapshot),
            _ => None,
        }
    }
    /// Retorna la cantidad mínima de argumentos requeridos para este comando.
    pub fn cantidad_minima_argumentos(&self) -> usize {
        match self {
            TipoDeComando::Set => 1,
            TipoDeComando::Get => 1,
            TipoDeComando::Length => 0,
            TipoDeComando::Snapshot => 0,
        }
    }
    /// Retorna la cantidad máxima de argumentos permitidos para este comando.
    pub fn cantidad_maxima_argumentos(&self) -> usize {
        match self {
            TipoDeComando::Set => 2,
            TipoDeComando::Get => 1,
            TipoDeComando::Length => 0,
            TipoDeComando::Snapshot => 0,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let comando: Result<Comando, String> = Comando::new(&args);
    match comando {
        Ok(arg) => match (arg.tipo, arg.get_clave(), arg.get_valor()) {
            (TipoDeComando::Set, Ok(clave), Ok(valor)) => execute_set(clave, valor),
            (TipoDeComando::Get, Ok(clave), _) => execute_get(clave),
            (TipoDeComando::Length, _, _) => execute_length(),
            (TipoDeComando::Snapshot, _, _) => execute_snapshot(),
            (_, _, _) => eprintln!("{}", Error::NotFound.mensaje()),
        },
        Err(e) => eprintln!("{}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Argumentos válidos
    #[test]
    fn test01_verificar_comando_set() {
        let args = vec![
            "program".into(),
            "set".into(),
            "clave".into(),
            "valor".into(),
        ];
        let result = Comando::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.tipo, TipoDeComando::Set);
                assert_eq!(cmd.clave, Some("clave".to_string()));
                assert_eq!(cmd.valor, Some("valor".to_string()));
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    #[test]
    fn test02_verificar_comando_get() {
        let args = vec!["program".into(), "get".into(), "clave".into()];
        let result = Comando::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.tipo, TipoDeComando::Get);
                assert_eq!(cmd.clave, Some("clave".to_string()));
                assert_eq!(cmd.valor, None);
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    #[test]
    fn test03_verificar_comando_length() {
        let args = vec!["program".into(), "length".into()];
        let result = Comando::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.tipo, TipoDeComando::Length);
                assert_eq!(cmd.clave, None);
                assert_eq!(cmd.valor, None);
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    #[test]
    fn test04_verificar_comando_snapshot() {
        let args = vec!["program".into(), "snapshot".into()];
        let result = Comando::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.tipo, TipoDeComando::Snapshot);
                assert_eq!(cmd.clave, None);
                assert_eq!(cmd.valor, None);
            }
            Err(_) => panic!("El comando debería haberse creado correctamente."),
        }
    }

    // Argumentos inválidos
    #[test]
    fn test05_verificar_comando_desconocido() {
        let args = vec!["program".into(), "unknown".into()];
        let result = Comando::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test06_verificar_comando_set_sin_parametros() {
        let args = vec!["program".into(), "set".into()];
        let result = Comando::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test07_verificar_comando_set_un_parametro() {
        let args = vec!["program".into(), "set".into(), "clave".into()];
        let result = Comando::new(&args);
        match result {
            Ok(cmd) => {
                assert_eq!(cmd.tipo, TipoDeComando::Set);
                assert_eq!(cmd.clave, Some("clave".to_string()));
                assert_eq!(cmd.valor, None);
            }
            Err(_) => {
                panic!("El comando set con 1 parámetro debería haberse creado correctamente.")
            }
        }
    }

    #[test]
    fn test08_verificar_comando_get_faltan_argumentos() {
        let args = vec!["program".into(), "get".into()];
        let result = Comando::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test09_verificar_comando_sin_argumentos() {
        let args = vec!["program".into()];
        let result = Comando::new(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test10_verificar_comando_set_demasiados_argumentos() {
        let args = vec![
            "program".into(),
            "set".into(),
            "clave".into(),
            "valor".into(),
            "extra".into(),
        ];
        let result = Comando::new(&args);
        assert!(result.is_err());
    }
}
