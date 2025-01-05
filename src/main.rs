use std::{io::Read, net::{TcpListener, TcpStream}};

use parser::parse_http_request;

mod parser;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]).into_owned();
    let trimmed = request.trim_end_matches("\0");
    let req_parse  = parse_http_request(trimmed);
    println!("{:?}", req_parse);
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server running on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => handle_client(s),
            Err(e) => eprintln!("Unexpected error, {}", e)
        }
    }

    Ok(())
}
