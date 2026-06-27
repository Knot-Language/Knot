mod types;
mod handler;

use handler::LspHandler;
use std::io::{BufRead, BufReader, Read, Write};

pub fn run_lsp() {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = std::io::stdout();
    let mut handler = LspHandler::new();
    let mut line = String::new();

    loop {
        // Read headers until empty line
        let mut content_length: Option<usize> = None;
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => return,
                Ok(_) => {}
                Err(_) => return,
            }
            if line.trim().is_empty() {
                break;
            }
            if let Some(val) = line.strip_prefix("Content-Length: ") {
                content_length = val.trim().parse::<usize>().ok();
            }
        }

        let len = match content_length {
            Some(l) => l,
            None => continue,
        };

        let mut body = vec![0u8; len];
        if reader.read_exact(&mut body).is_err() {
            return;
        }
        let body_str = String::from_utf8_lossy(&body);

        let response = handler.handle(&body_str);

        if let Some(resp) = response {
            let resp_str = serde_json::to_string(&resp).unwrap_or_default();
            let header = format!("Content-Length: {}\r\n\r\n{}", resp_str.len(), resp_str);
            let _ = stdout.write_all(header.as_bytes());
            let _ = stdout.flush();
        }
    }
}
