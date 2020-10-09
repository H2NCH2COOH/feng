use std::io::Read;

mod lang;

fn main() {
    let mut bytes = std::io::stdin().bytes();
    let p = lang::parse(&mut bytes).unwrap();
    for ref v in p {
        lang::print(&mut std::io::stdout(), v).unwrap();
    }
}
