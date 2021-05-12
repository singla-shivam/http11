use crate::status::StatusCode;

pub struct Response {
    is_response_ready: bool,
    is_sent: bool,
}

impl Response {
    pub fn new() -> Self {
        Response {
            is_sent: false,
            is_response_ready: false,
        }
    }

    pub fn send_code(&mut self, code: StatusCode) {}
}
