use nix::unistd::{fork, ForkResult};

fn main() {
    match fork() {
        Ok(ForkResult::Parent { child, ..}) => {
            println!("Continuing execution in parent process, new child has pid: {}", child);
        }
        Ok(ForkResult::Child) => println!("I'm a new child process"),
        Err(_) => println!("Fork failed"),
    }
}
