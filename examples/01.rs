use http11::Http11;
use http11::StatusCode;

fn main() {
    println!("abc");
    // Http11::start();
    let x = StatusCode::CONTINUE;
    println!("{:?}", x);
    println!("{:?}", StatusCode::reason(100));
}
