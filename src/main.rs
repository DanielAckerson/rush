extern crate pom;

mod parse;

use std::io::{self, Write};
use std::process::{self, Command};
use std::collections::HashMap;

// TODO: instead of working directly with path and args after parsing, generate a 
// graph of tasks to execute and then execute them
fn main() {
    let mut input = String::new();
    let env_vars: HashMap<String, String> = std::env::vars().collect();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => if let Ok((path, args)) = parse::parse(&input, &env_vars) {
                exec_input(&path, args.iter().map(AsRef::as_ref).collect());
            },
            Err(_) => process::exit(1),
        }
        input.clear();
    }
    println!("Bye!");
}


fn exec_input(path: &str, args: Vec<&str>) {
    match Command::new(path).args(args).spawn() {
        Ok(mut child) => if let Ok(exit_status) = child.wait() {
            println!("process exited with code {}", exit_status.code().unwrap_or(0));
        },
        Err(e) => {
            eprintln!("{}", e);
        },
    }
}
