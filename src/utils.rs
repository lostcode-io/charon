use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn, error, debug_span};

pub async fn read_from_socket(socket: &mut TcpStream) -> Option<String> {
    let span = debug_span!("read_from_socket");
    let _enter = span.enter();

    let mut result = String::new();

    let mut headers = String::new();
    let mut buffer = [0; 1];
    let mut headers_end = false;

    while !headers_end {
        match socket.read(&mut buffer).await {
            Ok(0) => {
                continue;
            }
            Ok(_) => {
                let c = std::str::from_utf8(&buffer).unwrap();
                result.push_str(c);
                headers.push_str(c);
                if result.ends_with("\r\n\r\n") {
                    headers_end = true;
                }
            }
            Err(e) => {
                error!("Failed to read from socket: {}", e);
                return None;
            }
        }
    }

    let mut content_length = 0;
    for line in headers.lines() {
        if line.to_lowercase().starts_with("content-length") {
            content_length = line
                .split(": ")
                .collect::<Vec<&str>>()[1]
                .parse::<usize>()
                .unwrap();
        }
    }

    debug!("Content-Length: {}", content_length);

    if content_length > 1024 * 1024 {
        warn!("Invalid content length: {}", content_length);
        return None;
    }

    if content_length > 0 {
        let mut buffer = vec![0; content_length];
        match timeout(Duration::from_secs(5), socket.read_exact(&mut buffer)).await {
            Ok(_) => {
                result.push_str(std::str::from_utf8(&buffer).unwrap());
            }
            Err(_) => {
                return None;
            }
        }
    }

    debug!("Data from socket: \n{}", result);

    Some(result)
}
