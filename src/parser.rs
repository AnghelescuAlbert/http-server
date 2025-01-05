use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    method: String, 
    path: String,
    version: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

pub fn parse_http_request(request: &str) -> Result<HttpRequest, &str> {
    let mut lines = request.lines();
    let request_line = lines.next().ok_or("Error trying to read the request.").unwrap();
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Invalid request line.");
    }
    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();

    let mut headers = HashMap::new();

    while let Some(line) = lines.next() {
        if line.is_empty() {
            break; // Empty signals the start of the body
        }
        let colon_pos = line.find(":").ok_or("Invalid header format.").unwrap();
        let key = line[..colon_pos].trim().to_string();
        let value = line[(colon_pos +1)..].trim().to_string();
        headers.insert(key, value);
    }

    let body = if let Some(body) = lines.next() {
        Some(body.to_string())
    } else {
        None
    };

    Ok(HttpRequest {
        method,
        path,
        version, 
        headers,
        body
    })
}