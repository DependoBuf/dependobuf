use dbuf_core::parser::parse;
use std::io::{self, Read};

fn main() {
    let mut input = String::new();

    io::stdin()
        .read_to_string(&mut input)
        .expect("failed reading");

    let res = parse(&input);
    match res.into_result() {
        Ok(expr) => println!("{:#?}\n {:?}", &expr, &expr),
        Err(err) => println!("error: {:?}", err),
    }
}
