mod frontend;
mod analysis;
mod runtime;
mod cli;
mod error;

use cli::Repl;

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
