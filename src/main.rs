use std::io::Read;

mod lang;

fn main() {
    println!("{:?}", lang::parse(std::io::stdin().bytes()));
}
