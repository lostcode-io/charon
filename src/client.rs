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

pub fn run(_args: Args, server: String, local_server: String, token: String) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut stream = connect_to_server(&server).await;

        // Send a post request to the server /add/
        let data = format!("POST / HTTP/1.1\r\n{}: {}\r\n\r\n{}", "Content-Length", token.len(), token);
        info!("Data sent to server: \n{}", data);
        if let Err(e) = stream.write(data.as_bytes()).await {
            error!("Failed to write to server: {}", e);
            return;
        }

        // Read the response from the server
        let response = match read_from_socket(&mut stream).await {
            Some(response) => response,
            None => {
                error!("Failed to read from server");
                return;
            }
        };
        
        info!("Response from server: \n{}", response);

        if response.starts_with("HTTP/1.1 401") {
            error!("Wrong token");
            std::process::exit(1);
        } else if response.starts_with("HTTP/1.1 200") {
            let body = response
                .split("\n")
                .skip(2)
                .collect::<Vec<&str>>()
                .join("\n");
            info!("Running on http://{}/{}/", server, body);
        }

        loop {
            print!("\n");

            // Read from the server
            let data = match read_from_socket(&mut stream).await {
                Some(data) => data,
                None => {
                    error!("Failed to read from server");
                    return;
                }
            };

            info!("Data from server: \n{}", data);

            // Send the buffer to the local server
            let mut local_stream = connect_to_server(&local_server).await;
            local_stream.set_nodelay(true).unwrap();
            if let Err(e) = local_stream.write_all(data.as_bytes()).await {
                error!("Failed to write to local server: {}", e);
                return;
            }
            local_stream.flush().await.unwrap();
            info!("Data sent to local server: \n{}", data);

            // Read the response from the local server
            let response = match read_from_socket(&mut local_stream).await {
                Some(response) => response,
                None => {
                    error!("Failed to read from local server");
                    return;
                }
            };
            info!("Response from local server");

            // Send the response back to the server
            stream.set_nodelay(true).unwrap();
            if let Err(e) = stream.write(response.as_bytes()).await {
                error!("Failed to write to server: {}", e);
                return;
            }
            stream.flush().await.unwrap();

            info!("Data sent to server");
        }
    });
}
