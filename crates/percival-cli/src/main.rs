//! Crate containing code for the `percival-cli` binary.

use std::error::Error;
use std::{
    fs::read_to_string,
    io::{self, Read, Write},
    path::PathBuf,
    process::{self, Command, Stdio},
};

use clap::{ArgEnum, Parser};

use percival::{
    codegen_js::compile as compile_js, codegen_json::compile as compile_json,
    errors::format_errors, parser::Grammar,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum, Debug)]
enum Emitter {
    JS,
    JSON,
}

/// Convenience CLI for testing the Percival language compiler.
#[derive(Parser, Debug)]
#[clap(name = "Percival")]
struct Opt {
    /// Input file (default: read from stdin).
    #[clap(name = "FILE", parse(from_os_str))]
    input: Option<PathBuf>,

    /// Runs prettier and bat on the output.
    #[clap(short, long)]
    format: bool,

    #[clap(short, long, arg_enum, default_value_t = Emitter::JS)]
    emit: Emitter,
}

/// Run the main program.
fn main() {
    let opt = Opt::parse();

    let mut src = match opt.input {
        Some(path) => read_to_string(path).unwrap(),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }
    };
    if !src.ends_with('\n') {
        src += "\n";
    }

    let grammar = Grammar::new();
    let prog = match grammar.parse(&src[..]) {
        Ok(prog) => prog,
        Err(errors) => {
            eprintln!("{}", format_errors(&src, errors));
            process::exit(1);
        }
    };

    let emitted: Result<String, Box<dyn Error>> = match opt.emit {
        Emitter::JS => compile_js(&prog).map_err(|err| err.into()),
        Emitter::JSON => compile_json(&prog).map_err(|err| err.into()),
    };

    match emitted {
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
