use minikv::{protocol, Error};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

const READ_TIMEOUT_MS: u64 = 2_000;
const WRITE_TIMEOUT_MS: u64 = 2_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(addr) = args.get(1) else {
        println!("{}", protocol::error(Error::InvalidArgs));
        return;
    };
    let stream = match connect(addr) {
        Ok(stream) => stream,
        Err(err) => {
            println!("{}", protocol::error(err));
            return;
        }
    };
    if let Err(err) = run_client(stream) {
        println!("{}", protocol::error(err));
    }
}

fn connect(addr: &str) -> Result<TcpStream, Error> {
    let stream = TcpStream::connect(addr).map_err(|_| Error::ClientSocketBinding)?;
    configure_timeouts(&stream).map_err(|_| Error::ClientSocketBinding)?;
    Ok(stream)
}

fn configure_timeouts(stream: &TcpStream) -> std::io::Result<()> {
    let read_timeout = std::time::Duration::from_millis(READ_TIMEOUT_MS);
    let write_timeout = std::time::Duration::from_millis(WRITE_TIMEOUT_MS);
    stream.set_read_timeout(Some(read_timeout))?;
    stream.set_write_timeout(Some(write_timeout))?;
    Ok(())
}

fn run_client(mut stream: TcpStream) -> Result<(), Error> {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stream.try_clone().map_err(|_| Error::ClientSocketBinding)?);
    for line in stdin.lock().lines() {
        let line = line.map_err(|_| Error::ConnectionClosed)?;
        if line.trim().is_empty() {
            continue;
        }
        if writeln!(stream, "{line}").is_err() {
            return Err(Error::ConnectionClosed);
        }
        let mut response = String::new();
        match reader.read_line(&mut response) {
            Ok(0) => return Err(Error::ConnectionClosed),
            Ok(_) => {
                print!("{response}");
                let _ = std::io::stdout().flush();
            }
            Err(err) => {
                if err.kind() == std::io::ErrorKind::TimedOut {
                    return Err(Error::Timeout);
                }
                return Err(Error::ConnectionClosed);
            }
        }
    }
    Ok(())
}
