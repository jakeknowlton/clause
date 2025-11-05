mod analysis;
mod cli;
mod error;
mod frontend;
mod runtime;

use cli::Repl;

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
