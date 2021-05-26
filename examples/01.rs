use http11::{App, Http11Server};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let x = Http11Server::start(App::new());
    let _z = x.await;
}
