use std::io::Read;
use std::io::Write;

mod lang;

fn main() {
    let mut bytes = std::io::stdin().bytes();
    let p = lang::parse(Some("Stdin"), &mut bytes).unwrap();
    for ref v in p {
        lang::print(&mut std::io::stdout(), v).unwrap();
        std::io::stdout().write(b"\n");
    }
}
