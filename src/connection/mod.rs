use crate::app::{App, SharedApp};
use crate::request::{Request, RequestBuilder};
use crate::response::Response;
use std::collections::linked_list::{IterMut as LinkedListIterMut, LinkedList};
use std::io::{ErrorKind, Result};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

static FRAME_SIZE: usize = 1024;

pub struct Connection {
    app: SharedApp,
    tcp_stream: TcpStream,
    requests: LinkedList<(Request, Response)>,
}

impl Connection {
    pub fn new(value: TcpStream, app: SharedApp) -> Self {
        let requests = LinkedList::new();

        Connection {
            tcp_stream: value,
            requests,
            app,
        }
    }

    pub async fn process_socket(&mut self) {
        loop {
            let mut has_received_requests = false;
            let response = Response::new();
            let mut request_builder = RequestBuilder::new();
            loop {
                let stream = &mut self.tcp_stream;
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
                        has_received_requests = true;
                        break;
                    }
                    Ok(n) => n,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        return;
                    }
                };

                println!("{}", String::from_utf8_lossy(&buffer[..]));
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

            if has_received_requests {
                break;
            }

            if !request_builder.can_parse_more() {
                let request = request_builder.build();
                self.app
                    .as_ref()
                    .lock()
                    .unwrap()
                    .process_request(&request, &response)
                    .await;
            }
        }
    }

    async fn send_response(&self, request: &Request, response: &Response) {
        let stream = &self.tcp_stream;
        loop {
            stream.writable().await;
            // Try to write data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match stream.try_write(
                b"HTTP/1.1 200 Ok\r\ncontent-length: 3\r\n\r\nabc\r\n",
            ) {
                Ok(n) => {
                    break;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    // return Err(e.into());
                }
            }
        }
    }
}
