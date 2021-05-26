use crate::request::Request;
use crate::response::Response;
use std::sync::{Arc, Mutex};

type MiddleWare = dyn Fn(&Request, &Response) -> ();

pub type SharedApp = Arc<Mutex<App>>;

pub struct App {}

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub(crate) async fn process_request(
        &self,
        request: &Request,
        response: &Response,
    ) {
    }

    pub fn get(&self, callback: fn(&Request, &Response) -> ()) {
        unimplemented!();
    }
}
