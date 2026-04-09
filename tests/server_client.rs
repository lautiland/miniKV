use minikv::server::{handler, writer};
use minikv::test_sync::get_lock;
use minikv::KvStore;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc::channel, Arc, RwLock};

fn cleanup() {
    let _ = std::fs::remove_file(".minikv.data");
    let _ = std::fs::remove_file(".minikv.log");
}

fn start_server() -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let store = Arc::new(RwLock::new(KvStore::new()));
    let (writer_tx, writer_rx) = channel();
    let writer_store = Arc::clone(&store);
    std::thread::spawn(move || writer::start(&writer_store, &writer_rx));
    let handle = std::thread::spawn(move || {
        if let Ok((stream, _)) = listener.accept() {
            let _ = handler::handle_client(&stream, &store, &writer_tx);
        }
    });
    (addr.to_string(), handle)
}

fn read_response(reader: &mut BufReader<TcpStream>) -> String {
    let mut buf = String::new();
    reader.read_line(&mut buf).unwrap();
    buf.trim_end().to_string()
}

fn send_line(stream: &mut TcpStream, line: &str) {
    writeln!(stream, "{line}").unwrap();
}

#[test]
fn server_handles_basic_commands() {
    let _guard = get_lock().lock().unwrap();
    cleanup();

    let (addr, handle) = start_server();
    let mut stream = TcpStream::connect(addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    send_line(&mut stream, "set a b");
    assert_eq!(read_response(&mut reader), "OK");

    send_line(&mut stream, "get a");
    assert_eq!(read_response(&mut reader), "b");

    send_line(&mut stream, "length");
    assert_eq!(read_response(&mut reader), "1");

    send_line(&mut stream, "snapshot");
    assert_eq!(read_response(&mut reader), "OK");

    drop(stream);
    let _ = handle.join();
    cleanup();
}

#[test]
fn server_keeps_connection_on_client_errors() {
    let _guard = get_lock().lock().unwrap();
    cleanup();

    let (addr, handle) = start_server();
    let mut stream = TcpStream::connect(addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    send_line(&mut stream, "unknown cmd");
    assert_eq!(read_response(&mut reader), "ERROR \"UNKNOWN COMMAND\"");

    send_line(&mut stream, "get missing");
    assert_eq!(read_response(&mut reader), "ERROR \"NOT FOUND\"");

    send_line(&mut stream, "set a b");
    assert_eq!(read_response(&mut reader), "OK");

    drop(stream);
    let _ = handle.join();
    cleanup();
}
