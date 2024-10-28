#[cfg(test)]
pub mod utils {
    use crate::utils::read_from_socket;

    use std::net::TcpListener;
    use std::io::Write;
    use tokio::net::TcpStream as TokioTcpStream;

    #[tokio::test]
    async fn test_read_from_socket() {
        const MESSAGE: &str = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nHello, world!";

        // Bind a TCP listener to a random available port
        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind listener");
        let addr = listener.local_addr().unwrap();

        // Spawn the server side of the test
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept() {
                stream.write_all(MESSAGE.as_bytes()).expect("Could not write to stream");
            }
        });

        // Connect as a client and test read_from_socket
        let mut client = TokioTcpStream::connect(addr).await.expect("Could not connect to server");

        // Run the tested function
        let result = read_from_socket(false, &mut client).await;

        // Assert that the response is what we expect
        assert_eq!(result, Some(MESSAGE.to_string()));
    }
}
