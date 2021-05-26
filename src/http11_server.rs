use crate::app::App;
use crate::connection::Connection;
use crate::{Request, RequestBuilder};
use std::io::{ErrorKind, Result};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

pub struct Http11Server {}

impl Http11Server {
    pub async fn start<'a>(app: App) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        let app = Arc::new(Mutex::new(App {}));

        loop {
            let (stream, _) = listener.accept().await?;
            let app = app.clone();
            let mut connection = Connection::new(stream, app);
            connection.process_socket().await;
            println!("connection closed");
        }
    }
}
