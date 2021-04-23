use http11::status::StatusCode;
use http11::Http11;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let x = Http11::start();
    x.await;
}
