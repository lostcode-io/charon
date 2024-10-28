use crate::utils::read_from_socket;
use crate::Args;

use log::{error, info};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn connect_to_server(server: &str) -> TcpStream {
    match TcpStream::connect(server).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            std::process::exit(1);
        }
    }
}

pub async fn send_add_request(debug: bool, stream: &mut TcpStream, token: &str) -> Result<String, String> {
    let data = format!("POST / HTTP/1.1\r\n{}: {}\r\n\r\n{}", "Content-Length", token.len(), token);
    if debug {
        info!("Data to send (authorization): \n{}", data);
    }
    if let Err(e) = stream.write(data.as_bytes()).await {
        return Err(format!("Failed to write to server: {}", e));
    }

    // Read the response from the server
    let response = match read_from_socket(debug, stream).await {
        Some(response) => response,
        None => {
            return Err("Failed to read from server".to_string());
        }
    };
    
    if debug {
        info!("Response from server: \n{}", response);
    }

    if response.starts_with("HTTP/1.1 401") {
        return Err("Wrong token".to_string());
    }

    let body = response
        .split("\n")
        .skip(2)
        .collect::<Vec<&str>>()
        .join("\n");

    return Ok(body);
}

pub fn run(args: Args, server: String, local_server: String, token: String) {
    let debug = args.debug;
    let rt = tokio::runtime::Runtime::new().unwrap();
    info!("Running client. Forwarding from: {}, to: {}", server, local_server);

    rt.block_on(async {
        let mut stream = match TcpStream::connect(&server).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to connect to server: {}. Is local server on?", e);
                std::process::exit(1);
            }
        };

        let addr = match send_add_request(debug, &mut stream, &token).await {
            Ok(addr) => addr,
            Err(e) => {
                error!("Failed to send add request: {}", e);
                return;
            }
        };
        info!("Running on http://{}/{}/", server, addr);

        loop {
            print!("\n");

            // Read from the server
            let data = match read_from_socket(debug, &mut stream).await {
                Some(data) => data,
                None => {
                    error!("Failed to read from server");
                    return;
                }
            };

            info!("Accepted data from server");
            let start = std::time::Instant::now();

            if debug {
                info!("Data from server: \n{}", data);
            }

            // Send the buffer to the local server
            let mut local_stream = connect_to_server(&local_server).await;
            local_stream.set_nodelay(true).unwrap();
            if let Err(e) = local_stream.write_all(data.as_bytes()).await {
                error!("Failed to write to local server: {}", e);
                return;
            }
            local_stream.flush().await.unwrap();
            if debug {
                info!("Data sent to local server: \n{}", data);
            }

            // Read the response from the local server
            let response = match read_from_socket(debug, &mut local_stream).await {
                Some(response) => response,
                None => {
                    error!("Failed to read from local server");
                    return;
                }
            };
            if debug {
                info!("Response from local server: \n{}", response);
            }

            // Send the response back to the server
            stream.set_nodelay(true).unwrap();
            if let Err(e) = stream.write(response.as_bytes()).await {
                error!("Failed to write to server: {}", e);
                return;
            }
            stream.flush().await.unwrap();

            let elapsed = start.elapsed();
            info!("Data sent to the server in {}µs", elapsed.as_micros());
        }
    });
}
