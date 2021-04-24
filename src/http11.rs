use crate::RequestBuilder;
use std::io::{ErrorKind, Result};
use std::mem::MaybeUninit;
use tokio::net::{TcpListener, TcpStream};

pub struct Http11 {}

impl Http11 {
    pub async fn start() -> Result<()> {
        println!("here1");
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            println!("here2");
            let (socket, _) = listener.accept().await?;
            Http11::process_socket(socket).await?;
        }
    }

    async fn process_socket(stream: TcpStream) -> Result<()> {
        let mut buffer: [u8; 1024] = unsafe { MaybeUninit::uninit().assume_init() };

        loop {
            stream.readable().await?;
            match stream.try_read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    // msg.truncate(n);
                    break;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        let _r = RequestBuilder::<String>::parse(&buffer);

        Ok(())
    }
}
