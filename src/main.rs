mod ast;
mod lexer;
mod parser;
mod repl;
mod token;

use repl::Repl;

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
