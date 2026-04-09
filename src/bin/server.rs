use minikv::server::listener;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let addr = if args.len() > 1 {
        &args[1]
    } else {
        "127.0.0.1:7878"
    };
    if let Err(e) = listener::start(addr) {
        eprintln!("Error al iniciar el servidor: {e}");
    }
}
