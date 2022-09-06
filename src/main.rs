use std::io::Read;

mod lang;

fn main() {
    let mut bytes = std::io::stdin().bytes();
    //let mut bytes = "1".bytes().map(|b| Ok(b));
    let p = lang::parse("STDIN", &mut bytes).unwrap();
    for v in &p {
        lang::println(&mut std::io::stdout(), &v.into()).unwrap();
    }
    let r = lang::eval_source(&p).unwrap();
    lang::println(&mut std::io::stdout(), &r).unwrap();
}
