use crate::request::{Request, RequestBuilder};
use crate::response::Response;
use std::collections::LinkedList;
use std::convert::TryFrom;
use std::io::{ErrorKind, Result};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

static FRAME_SIZE: usize = 1024;

pub struct Connection {
    tcp_stream: TcpStream,
    requests: LinkedList<(Request, Response)>,
}

impl Connection {
    pub async fn process_socket(&self) {
        let stream = &self.tcp_stream;
        loop {
            let response = Response::new();
            let mut request_builder = RequestBuilder::new();
            loop {
                if !request_builder.can_parse_more() {
                    break;
                }
                stream.readable().await;
                let mut buffer = Vec::with_capacity(FRAME_SIZE);
                unsafe {
                    buffer.set_len(FRAME_SIZE);
                }
                let bytes_read = match stream.try_read(&mut buffer) {
                    Ok(0) => {
                        break;
                    }
                    // Limit the number of bytes read
                    // Ok(n) => (),
                    Ok(n) => n,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return;
                    }
                };

                let result = request_builder.parse(buffer, bytes_read);
                match result {
                    Ok(_) => (),
                    Err(error) => {
                        // TODO
                        // error while parsing
                        // send appropriate response and
                        // close the connection
                    }
                }
            }

            let request = request_builder.build();
            // TODO
            break;
        }
    }

    fn send_response(&self, request: &Request, response: &Response) {}

    fn add_new_request(&mut self, request: Request) {
        self.requests.push_back((request, Response::new()));
    }
}

impl From<TcpStream> for Connection {
    fn from(value: TcpStream) -> Self {
        Connection {
            tcp_stream: value,
            requests: LinkedList::new(),
        }
    }
}
