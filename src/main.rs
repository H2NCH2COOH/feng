use std::io::BufReader;
use std::io::Read;

mod lang;

fn main() {
    let mut bytes = BufReader::new(std::io::stdin()).bytes();
    let p = lang::parse("STDIN", &mut bytes).unwrap();
    let r = lang::eval_source(&p).unwrap();
    lang::println(&mut std::io::stdout(), &r).unwrap();
}
