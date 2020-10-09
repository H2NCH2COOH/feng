use std::io::Read;

mod lang;

fn main() {
    let mut bytes = std::io::stdin().bytes();
    println!("{:?}", lang::parse(&mut bytes));
}
