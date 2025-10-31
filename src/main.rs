mod ast;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod token;
mod value;

use repl::Repl;

fn main() {
    let mut repl = Repl::new();
    repl.run();
}
