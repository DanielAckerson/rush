use std::io::{self, Write};
use std::process::{self, Command};

fn main() {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break,
            Ok(_) => if let Some((path, args)) = eval_input(input.trim()) {
                exec_input(path, args);
            },
            Err(_) => process::exit(1),
        }
        input.clear();
    }
    println!("Bye!");
}


fn eval_input(input: &str) -> Option<(&str, Vec<&str>)> {
    let mut expr_iter = input.split_whitespace();
    if let Some(path) = expr_iter.next() {
        Some((path, expr_iter.collect()))
    } else {
        None
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
