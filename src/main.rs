mod parser;

use parser::HttpRequest;
use parser::HttpResponse;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use parser::parse_http_request;

const HTML_EXTENSION: &str = ".html";
const ROUTES_DIR: &str = "./src/routes";
const ROUTE_404: &str = "./src/routes/404.html";

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

async fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..]).into_owned();

    let trimmed = request.trim_end_matches("\0");

    let req_parse  = parse_http_request(trimmed)?;

    println!("{:?}", req_parse);

    let response = get_response_html(&req_parse).unwrap();
    println!("{:?}", &response);
    response.send(stream).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    println!("Server running on http://127.0.0.1:8080...");
    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling client: {:?}", e);
            }
        });
    }
}
