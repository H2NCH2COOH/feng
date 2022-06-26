use std::io::Read;
use std::io::Write;

mod lang;

fn main() {
    let mut bytes = std::io::stdin().bytes();
    let p = lang::parse("STDIN", &mut bytes);

    if let Err(e) = p {
        println!("{}", e);
        return;
    }

    for ref v in p.unwrap() {
        lang::print(&mut std::io::stdout(), &v.into()).unwrap();
        std::io::stdout().write_all(b"\n").unwrap();
    }
}
