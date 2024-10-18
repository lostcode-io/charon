// use crate::Args;
// use log::info;
//
// pub fn run(_args: Args, server: String, token: String) {
//     info!("Running client");
//     info!("Connecting to server: {}, with token: {}", server, token);
// }

use crate::Args;
use log::{error, info};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn connect_to_server(server: &str) -> TcpStream {
    info!("Connecting to server: {}", server);
    match TcpStream::connect(server).await {
        Ok(stream) => stream,
        Err(e) => {
            error!("Failed to connect to server: {}", e);
            std::process::exit(1);
        }
    }
}

pub fn run(_args: Args, server: String, local_server: String) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut stream = connect_to_server(&server).await;

        // Send a post request to the server /add/
        let data = "POST / HTTP/1.1\r\n";
        if let Err(e) = stream.write_all(data.as_bytes()).await {
            error!("Failed to write to server: {}", e);
            return;
        }

        info!("Listening for data from server");

        loop {
            // Read from the server
            let mut data = [0; 1024];
            let n = stream.read(&mut data).await.unwrap();
            let data = String::from_utf8(data[..n].to_vec()).unwrap();

            // Send the buffer to the local server
            info!("Sending data to local server {}: \n{}", local_server, data);
            let mut local_stream = connect_to_server(&local_server).await;
            if let Err(e) = local_stream.write_all(data.as_bytes()).await {
                error!("Failed to write to local server: {}", e);
                return;
            }

            // Read the response from the local server
            let mut response = [0; 4024];
            let n = local_stream.read(&mut response).await.unwrap();
            let response = String::from_utf8(response[..n].to_vec()).unwrap();
            info!("Received response from local server: \n{}", response);

            // Send the response back to the server
            stream.set_nodelay(true).unwrap();
            if let Err(e) = stream.write(response.as_bytes()).await {
                error!("Failed to write to server: {}", e);
                return;
            }
            stream.flush().await.unwrap();

            info!("Sent response to server");
        }
    });
}
