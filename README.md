# charon
A utility to forward foreign requests to a local server.

## Usage
Install cargo and rustc from [here](https://www.rust-lang.org/tools/install).
Both server and client can be ran with `-d` option to debug the request data.

### Server
Server is required to be running in the web. Multiple clients can use one server at a time. Server is responsible for forwarding requests to clients using sockets.

To run server, use the following command:
```bash
cargo run --release -- server -p PORT
```
where `PORT` is the port number on which server will listen for incoming requests.

### Client
Client is the part running locally on the user machine and forwarding messages from the server to the locally ran application.

To run client, use the following command:
```bash
cargo run --release -- client -p PORT -s SERVER_IP:PORT -t TOKEN
```
where `PORT` is the port number on which client will listen for incoming requests, `SERVER_IP:PORT` is the IP address and port number of the server, and `TOKEN` is the token to authenticate the client with the server.

## Database
Server uses Postgres database to store tokens and arrdesses of clients. To setup the database.

To setup the database, follow these steps:
1. Install Postgres from [here](https://www.postgresql.org/download/).
2. Create a user called `charon` in the psql datapase:
```bash
echo "CREATE USER charon" | psql
```
3. Create a database called `charon` in the psql database:
```bash
echo "CREATE DATABASE charon WITH OWNER charon" | psql
```
4. Run the migrations to create the tables:
```bash
make migrate
```
5. For testing purposes, you can add a token to the database:
```bash
make fixtures
```
That will add a token `foo`, with address `some_kind_of_an_address` to the database. Altho for production, you should add proper tokens and adresses from outside charon.

Now you can run the server and client as described above.
