mod parser;

use parser::HttpRequest;
use parser::HttpResponse;
use parser::parse_http_request;

use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio_rustls::TlsAcceptor;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;


use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

const HTML_EXTENSION: &str = ".html";
const ROUTES_DIR: &str = "./src/routes";
const ROUTE_404: &str = "./src/routes/404.html";

const CERT_FILE: &str = "./cert.pem";
const KEY_FILE: &str = "./decrypted_private_key.pem";

fn response_sv_error() -> Result<HttpResponse, &'static str> {
    let mut headers = HashMap::new();
    headers.insert(String::from("Content-Type"), String::from("text/plain"));
    headers.insert(String::from("Connection"), String::from("close"));
    let status_line = String::from("500 Internal Server Error");
    let content = String::from("Something went wrong.");
    return Ok(HttpResponse {
        status_line,
        headers,
        body: Some(content)
    })
}

fn get_response_html(request: &HttpRequest) -> Result<HttpResponse, &str> {

    let mut path = request.path[..].to_owned();

    path += HTML_EXTENSION;
    path.insert_str(0, ROUTES_DIR);

    let mut headers = HashMap::new();

    headers.insert(String::from("Content-Type"), String::from("text/html; charset=UTF-8"));
    headers.insert(String::from("Connection"), String::from("close"));

    if let Ok(mut file) = File::open(path) {
        let mut content = String::new();
        if let Err(_e) = file.read_to_string(&mut content) {
            return response_sv_error();
        } else {
            headers.insert(String::from("Content-Length"), content.len().to_string());
            let status_line = String::from("200 OK");
            return Ok(HttpResponse {
                status_line,
                headers,
                body: Some(content),
            });
        }
    } else {
        if let Ok(mut file) = File::open(ROUTE_404) {
            let mut content = String::new();
            if let Err(_e) = file.read_to_string(&mut content) {
                return response_sv_error();
            } else {
                headers.insert(String::from("Content-Length"), content.len().to_string());
                let status_line = String::from("404 NOT FOUND");
                return Ok(HttpResponse {
                    status_line,
                    headers,
                    body: Some(content)
                })
            }
        } else {
            return response_sv_error();
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    acceptor: TlsAcceptor
    ) -> Result<(), Box<dyn Error>> {

    let mut tls_stream: tokio_rustls::server::TlsStream<TcpStream> = match acceptor.accept(stream).await {
        Ok(stream) => stream,
        Err(err) => return Err(Box::new(err)),
    };

    let mut buffer = [0; 1024];

    tls_stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..]).into_owned();

    let trimmed = request.trim_end_matches("\0");

    let req_parse  = parse_http_request(trimmed)?;

    let response = get_response_html(&req_parse).unwrap();

    response.send(tls_stream).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let certs: Vec<CertificateDer> = CertificateDer::pem_file_iter(CERT_FILE)?
            .map(|cert| cert.unwrap())
            .collect();
    
    let private_key = PrivateKeyDer::from_pem_file(KEY_FILE)?;
    
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Server running on http://127.0.0.1:8080...");
    loop {
        let (stream, _) = listener.accept().await?;

        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, acceptor).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}
