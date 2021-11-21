use chumsky::prelude::*;

use percival::{
    codegen::compile,
    parser::{format_errors, parser},
};

fn main() {
    let src = "
edge(x: 2, y: 3).
edge(x: 3, y: 4).
tc(x, y) :- edge(x, y).
tc(x, y) :- tc(x, y: z), edge(x: z, y).
"
    .trim();

    let parser = parser();
    let prog = match parser.parse(src) {
        Ok(prog) => prog,
        Err(errors) => {
            eprintln!("{}", format_errors(src, errors));
            return;
        }
    };
    match compile(&prog) {
        Ok(js) => println!("{}", js),
        Err(err) => eprintln!("Error: {}", err),
    }
}
