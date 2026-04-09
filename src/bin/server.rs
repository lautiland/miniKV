use minikv::{protocol, server::listener, Error};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(addr) = args.get(1) else {
        println!("{}", protocol::error(Error::InvalidArgs));
        return;
    };
    if let Err(e) = listener::start(addr) {
        let err = if e.to_string().contains(Error::ServerSocketBinding.code()) {
            Error::ServerSocketBinding
        } else if e.to_string().contains(Error::InvalidDataFile.code()) {
            Error::InvalidDataFile
        } else if e.to_string().contains(Error::InvalidLogFile.code()) {
            Error::InvalidLogFile
        } else {
            Error::ServerSocketBinding
        };
        println!("{}", protocol::error(err));
    }
}
