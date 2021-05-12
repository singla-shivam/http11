use crate::{Request, RequestBuilder};
use std::io::{ErrorKind, Result};
// use std::mem::MaybeUninit;
use crate::connection::Connection;
use tokio::net::{TcpListener, TcpStream};

pub struct Http11 {}

impl Http11 {
    pub async fn start() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let connection = Connection::from(stream);
            connection.process_socket();
        }
    }
}
