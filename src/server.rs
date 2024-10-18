// use crate::Args;
// use tokio::sync::Mutex;
// use tokio::net::{TcpListener, TcpStream};
// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use log::{info, error};
//
// pub fn run(_args: Args, port: u16) {
//     info!("Running server on port: {}", port);
//
//     let rt = tokio::runtime::Runtime::new().unwrap();
//
//     rt.block_on(async {
//         let (client, connection) = tokio_postgres::connect("host=localhost user=charon", NoTls)
//             .await
//             .expect("Failed to connect to database");
//
//         let soket_clients: Mutex<Vec<TcpStream>> = Mutex::new(Vec::new());
//
//         tokio::spawn(async move {
//             if let Err(e) = connection.await {
//                 eprintln!("connection error: {}", e);
//             }
//         });
//
//         // Create a tcp server which listens on the given port, and sends a message to the client
//     });
// }

use crate::Args;
use log::{info, error};
// use rocket::{get, post, routes, Config, State, Request};
// use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

type ClientList = Arc<Mutex<Vec<TcpStream>>>;

pub fn run(_args: Args, port: u16) {
    info!("Running server on port: {}", port);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let clients: ClientList = Arc::new(Mutex::new(Vec::new()));

    rt.block_on(async {
        // Move clients into the async block
        let clients_clone = clients.clone(); // Clone the Arc

        // tokio::spawn(async move {
        let clients_clone = clients_clone.clone(); // Clone the Arc again

        // Create a tcp server which listens on the given port, and sends a message to the client
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        info!("Listening on port: {}", port);

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            info!("Accepted connection from: {}", socket.peer_addr().unwrap());

            // Read the data from the socket
            let mut data = [0; 1024];
            let n = socket.read(&mut data).await.unwrap();
            let data = String::from_utf8(data[..n].to_vec()).unwrap();

            // Get the path from the request
            let path = data.split_whitespace().nth(1).unwrap_or("/");

            // Chech if the path is just a slash
            if path == "/" || path == "" {
                // Add the client to the list
                let mut clients = clients_clone.lock().await;
                let id = clients.len();
                info!("Adding client with id: {}", id);
                clients.push(socket);
            } else {
                // Check if path starts with a number which is an id of a client
                let id = match path.split('/').nth(1) {
                    Some(id) => match id.parse::<usize>() {
                        Ok(id) => id,
                        Err(e) => {
                            error!("Failed to parse id: {}", e);
                            continue;
                        }
                    },
                    None => {
                        error!("Failed to get id from path");
                        continue;
                    }
                };

                // Check if the client exists
                let mut clients = clients_clone.lock().await;
                let client = match clients.get_mut(id) {
                    Some(client) => client,
                    None => {
                        error!("Client not found");
                        continue;
                    }
                };

                // Send the request to the client without the id part and get the response
                let data_without_id = data.replace(&format!("/{}", id), "");
                info!("Sending request to client: \n{}", data_without_id);

                // Send the request to the client
                if let Err(e) = client.write_all(data_without_id.as_bytes()).await {
                    error!("Failed to send request to client: {}", e);
                    let response = "HTTP/1.1 500 Internal Server Error\n\n";
                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        error!("Failed to send response to listener: {}", e);
                        continue;
                    }
                }

                info!("Sent request to client: {}", id);

                // Read the response from the client
                let mut buffer = vec![0; 1024]; // Adjust size as needed
                let bytes_read = client.read(&mut buffer).await.unwrap();
                let response = String::from_utf8(buffer[..bytes_read].to_vec()).unwrap();

                info!("Sending response to listener: \n{}", response);

                // Send the response back to the listener
                if let Err(e) = socket.write_all(response.as_bytes()).await {
                    error!("Failed to send response to listener: {}", e);
                    continue;
                }

                info!("Sent response to listener");
            }
        }
        // });

        // // Start the Rocket server
        // rocket::build()
        //     .manage(clients.clone()) // You can still clone the Arc here
        //     .mount("/", routes![get_client_message, post_client_message])
        //     .configure(Config::figment().merge(("port", port + 1)))
        //     .launch()
        //     .await
        //     .expect("Failed to launch Rocket server");
    });
}

// #[get("/<id>/<path..>", data="<body>")]
// async fn get_client_message(id: usize, path: PathBuf, body: String, clients: &State<ClientList>) -> String {
//     // Lock the client list and fetch the corresponding client
//     let mut clients = clients.lock().await;
//     let client: &mut tokio::net::TcpStream = match clients.get_mut(id) {
//         Some(client) => client,
//         None => return "Client not found".to_string(),
//     };
//
//     // Construct the request to send to the client
//     let request = format!("{} {}\n{}\n", id, path.display(), body);
//     
//     // Send the request to the client
//     if let Err(e) = client.write_all(request.as_bytes()).await {
//         return format!("Failed to send request: {}", e);
//     }
//
//     // Read the response from the client
//     let mut response_buffer = Vec::new();
//     if let Err(e) = client.read_to_end(&mut response_buffer).await {
//         return format!("Failed to read response: {}", e);
//     }
//
//     // Convert the response buffer to a string
//     let response = String::from_utf8_lossy(&response_buffer).to_string();
//
//     // Return the response back
//     response
// }
//
// #[post("/<id>/<path..>", data = "<body>")]
// async fn post_client_message(id: usize, path: PathBuf, body: String, req: Request<'_>, clients: &State<ClientList>) -> String {
//     // Lock the client list and fetch the corresponding client
//     let mut clients = clients.lock().await;
//     let client: &mut tokio::net::TcpStream = match clients.get_mut(id) {
//         Some(client) => client,
//         None => return "Client not found".to_string(),
//     };
//
//     // Construct the request to send to the client
//     let method = req.method();
//     let path = req.uri().path();
//     let query = req.uri().query().unwrap();
//
//     // Collect headers
//     let mut headers_string = String::new();
//     for (key, value) in req.headers().iter() {
//         headers_string.push_str(&format!("{}: {}\n", key.as_str(), value));
//     }
//
//     // Read the request body
//     let mut body = String::new();
//     if let Some(mut body_stream) = req.data().await {
//         let _ = body_stream.read_to_string(&mut body).await;
//     }
//
//     // Construct the raw request string
//     let raw_request = format!(
//         "{} {}?{} HTTP/1.1\n{}\n{}\n",
//         method,
//         path,
//         query,
//         headers_string,
//         body
//     );
//
//     // Send the request to the client
//     if let Err(e) = client.write_all(raw_request.as_bytes()).await {
//         return format!("Failed to send request: {}", e);
//     }
//
//     // Read the response from the client
//     let mut response_buffer = Vec::new();
//     if let Err(e) = client.read_to_end(&mut response_buffer).await {
//         return format!("Failed to read response: {}", e);
//     }
//
//     // Convert the response buffer to a string
//     let response = String::from_utf8_lossy(&response_buffer).to_string();
//
//     // Return the response back
//     response
// }
//
