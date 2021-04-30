use http11::status::StatusCode;
use http11::Http11;
use regex::Regex;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // let x = Http11::start();
    // x.await;

    let re = Regex::new(r"([\s\t]+)").unwrap();
    let str = "  \tabc  \t  ssdfsdf  sfdf\t\tsdf";
    let r = re.replace_all(str, " ");
    println!("{}", r);
    println!("{}", "\t\tssdf\t\t".trim());
}
