use chumsky::prelude::*;

use percival::{codegen::compile, parser::parser};

fn main() {
    let parser = parser();
    let prog = parser
        .parse("tc(x, y) :- tc(x, y: z), edge(x: z, y).")
        .unwrap();
    match compile(&prog) {
        Ok(js) => println!("{}", js),
        Err(err) => eprintln!("Error: {}", err),
    }
}
