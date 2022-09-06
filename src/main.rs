use std::io::Read;

mod lang;

fn main() {
    let mut bytes = std::io::stdin().bytes();
    let p = lang::parse("STDIN", &mut bytes).unwrap();
    let r = lang::eval_source(&p).unwrap();
    lang::println(&mut std::io::stdout(), &r).unwrap();
}
