//! Crate containing code for the `percival-cli` binary.

use std::{
    fs::read_to_string,
    io::{self, Read, Write},
    path::PathBuf,
    process::{self, Command, Stdio},
};

use chumsky::prelude::*;
use structopt::StructOpt;

use percival::{
    codegen::compile,
    parser::{format_errors, parser},
};

/// Convenience CLI for testing the Percival language compiler.
#[derive(StructOpt, Debug)]
#[structopt(name = "Percival")]
struct Opt {
    /// Input file (default: read from stdin).
    #[structopt(name = "FILE", parse(from_os_str))]
    input: Option<PathBuf>,

    /// Runs prettier and bat on the output.
    #[structopt(short, long)]
    format: bool,
}

/// Run the main program.
fn main() {
    let opt = Opt::from_args();

    let src = match opt.input {
        Some(path) => read_to_string(path).unwrap(),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }
    };

    let parser = parser();
    let prog = match parser.parse(&src[..]) {
        Ok(prog) => prog,
        Err(errors) => {
            eprintln!("{}", format_errors(&src, errors));
            process::exit(1);
        }
    };

    match compile(&prog) {
        Ok(js) => {
            if !opt.format {
                println!("{}", js);
            } else {
                let mut child = Command::new("prettier")
                    .args(["--parser", "babel"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();
                {
                    let child_stdin = child.stdin.as_mut().unwrap();
                    child_stdin.write_all(js.as_bytes()).unwrap();
                }
                let output = child.wait_with_output().unwrap();

                let mut child = Command::new("bat")
                    .args(["--plain", "--paging", "never", "--language", "js"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .spawn()
                    .unwrap();
                {
                    let child_stdin = child.stdin.as_mut().unwrap();
                    child_stdin.write_all(&output.stdout).unwrap();
                }
                child.wait().unwrap();
            }
        }
        Err(err) => eprintln!("Error: {}", err),
    }
}
