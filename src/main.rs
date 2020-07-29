#[macro_use]
extern crate lazy_static;

mod parse;

use std::io::{self, Write};
use std::process::{self, Command};


// TODO: instead of working directly with path and args after parsing, generate a 
// graph of tasks to execute and then execute them
fn main() {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => if let Some((path, args)) = eval_input(input.trim()) {
                exec_input(&path, args.iter().map(AsRef::as_ref).collect());
            },
            Err(_) => process::exit(1),
        }
        input.clear();
    }
    println!("Bye!");
}


fn eval_input(input: &str) -> Option<(String, Vec<String>)> {
    match parse::parse(input) {
        Ok(mut expr) => {
            let args = expr
                .split_off(1)
                .iter()
                .map(|x| x.unwrap())
                .collect();
            let path = expr[0].unwrap();

            Some((path, args))
        },
        Err(msg) => {
            eprintln!("error: {}", msg);
            None
        },
    }
}


fn exec_input(path: &str, args: Vec<&str>) {
    match Command::new(path).args(args).spawn() {
        Ok(mut child) => if let Ok(exit_status) = child.wait() {
            println!("process exited with code {}", exit_status.code().unwrap_or(0));
        },
        Err(e) => {
            println!("{}", e);
        },
    }
}
