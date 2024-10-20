use crate::utils::read_from_socket;
use crate::Args;

use log::{error, info};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_postgres::NoTls;

struct Client {
    addr: String,
    stream: TcpStream,
}

type ClientList = Arc<Mutex<Vec<Client>>>;

pub fn run(args: Args, port: u16) {
    let debug = args.debug;
    info!("Running server on port: {}", port);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let clients: ClientList = Arc::new(Mutex::new(Vec::new()));

    rt.block_on(async {
        let (psql, connection) = tokio_postgres::connect("host=localhost user=charon", NoTls)
            .await
            .expect("Failed to connect to database");

        let clients_clone = clients.clone();

        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        info!("Listening on port: {}", port);

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Connection error: {}", e);
                std::process::exit(1);
            }
        });

        loop {
            print!("\n");

            let (mut socket, _) = match listener.accept().await {
                Ok(socket) => socket,
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                    continue;
                }
            };
            info!("Accepted connection from: {}", socket.peer_addr().unwrap());

            // Read the data from the socket
            let data = match read_from_socket(debug, &mut socket).await {
                Some(data) => data,
                None => {
                    error!("Failed to read from socket");
                    continue;
                }
            };
            if debug {
                info!("Recieved data: \n{}", data);
            }
            let path = data.split_whitespace().nth(1).unwrap_or("/").to_string();

            // Chech if the path is just a slash (Add client)
            if path == "/" {
                handle_client(debug, &psql, clients_clone.clone(), socket, data).await;
            } else {
                handle_proxy(debug, clients_clone.clone(), socket, path, data).await;
            }
        }
    });
}

async fn handle_client(debug: bool, psql: &tokio_postgres::Client, clients_clone: ClientList, mut socket: TcpStream, data: String) {
    if debug {
        info!("Client connection request recieved");
    }

    let token = data.split("\n").skip(3).collect::<Vec<&str>>().join("\n");
    let rows = match psql.query("SELECT * FROM tokens", &[]).await {
        Ok(client_row) => client_row,
        Err(e) => {
            error!("Failed to get client row: {}", e);
            return;
        }
    };
    let mut addr: String = String::new();
    let mut valid = false;
    for row in rows {
        let row_token: String = row.get("token");
        if row_token == token {
            addr = row.get("addr");
            valid = true;
            break;
        }
    }

    if !valid {
        let response = "HTTP/1.1 401 Unauthorized\r\n\r\n";
        info!("Invalid token, rejecting connection");
        if let Err(e) = socket.write_all(response.as_bytes()).await {
            error!("Failed to send response to listener: {}", e);
            return;
        }
        return;
    }

    {
        let mut clients = clients_clone.lock().await;
        if let Some(pos) = clients.iter().position(|client| client.addr == addr) {
            info!("Client already exists, deleting");
            clients.remove(pos);
        }

        let response = format!("HTTP/1.1 200 OK\r\n{}: {}\r\n\r\n{}", "Content-Length", addr.len(), addr);
        if let Err(e) = socket.write_all(response.as_bytes()).await {
            error!("Failed to send response to listener: {}", e);
            return;
        }

        clients.push(Client {
            addr: addr.clone(),
            stream: socket,
        });
    }
    
    info!("Client added: {}", addr);
}

async fn handle_proxy(debug: bool, clients_clone: ClientList, mut socket: TcpStream, path: String, data: String) {
    // Check if path starts with a number which is an id of a client
    let id = match path.split('/').nth(1) {
        Some(id) => id,
        None => {
            error!("Failed to get id from path");
            return
        }
    };

    // Check if the client exists
    let mut clients = clients_clone.lock().await;
    let client = match clients.iter_mut().find(|c| c.addr == id) {
        Some(client) => client,
        None => {
            error!("Client not found");
            return;
        }
    };

    if debug {
        info!("Data from listener: \n{}", data);
    }

    // Send the request to the client without the id part
    let data_without_id = data.replace(&format!("/{}", id), "");

    if debug {
        info!(
            "Received request from listener: {}, data: \n{}",
            id, data_without_id
        );
    }

    // Send the request to the client
    if let Err(e) = client.stream.write_all(data_without_id.as_bytes()).await {
        error!("Failed to send request to client: {}", e);
        let response = "HTTP/1.1 500 Internal Server Error\n\n";
        if let Err(e) = socket.write_all(response.as_bytes()).await {
            error!("Failed to send response to listener: {}", e);
            return;
        }
    }

    if debug {
        info!(
            "Sent request to client: {}, data: \n{}",
            id, data_without_id
        );
    }

    // Read the response from the client
    let response = match read_from_socket(debug, &mut client.stream).await {
        Some(response) => response,
        None => {
            error!("Failed to read from client");
            return;
        }
    };

    // Send the response back to the listener
    if let Err(e) = socket.write_all(response.as_bytes()).await {
        error!("Failed to send response to listener: {}", e);
        return;
    }

    info!("Sent response to listener");
}
