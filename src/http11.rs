use crate::{Request, RequestBuilder};
use std::io::{ErrorKind, Result};
use std::mem::MaybeUninit;
use tokio::net::{TcpListener, TcpStream};

pub struct Http11 {}

impl Http11 {
    pub async fn start() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            let (stream, _) = listener.accept().await?;
            let request = Http11::process_socket(stream).await?;
            println!("request: {:?}", request);
        }
    }

    async fn process_socket(stream: TcpStream) -> Result<Request> {
        let mut request_builder = RequestBuilder::new();
        // let mut buffer: [u8; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut buffer: [u8; 1024] = [48; 1024];

        loop {
            if !request_builder.can_parse_more() {
                break;
            }

            stream.readable().await?;

            match stream.try_read(&mut buffer) {
                Ok(0) => {
                    println!("0 byte");
                    break;
                }
                // Limit the number of bytes read
                // Ok(n) => (),
                Ok(n) => println!("{}", n),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
            request_builder.parse(&buffer);
        }

        let request = request_builder.build()?;
        Ok(request)
    }
}
