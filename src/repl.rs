use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::io::{self, Write};

pub struct Repl {
    interpreter: Interpreter,
    show_tokens: bool,
    show_ast: bool,
    show_result: bool,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            show_tokens: false,
            show_ast: false,
            show_result: true,
        }
    }

    pub fn run(&mut self) {
        println!("Clause REPL v0.1.0");
        println!("Type ':help' for commands, ':quit' to exit");
        println!();

        let mut buffer = String::new();
        let mut in_multiline = false;

        loop {
            // Print prompt
            if in_multiline {
                print!("... ");
            } else {
                print!(">>> ");
            }
            io::stdout().flush().unwrap();

            // Read line
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error reading input: {}", err);
                    continue;
                }
            }

            let line = line.trim_end();

            // Handle REPL commands
            if !in_multiline && line.starts_with(':') {
                match self.handle_command(line) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(err) => eprintln!("{}", err),
                }
                continue;
            }

            // Check for multiline input
            if line.is_empty() && in_multiline {
                // Empty line ends multiline input
                in_multiline = false;
                self.execute(&buffer);
                buffer.clear();
                continue;
            }

            if line.ends_with('{') || in_multiline {
                // Start or continue multiline input
                buffer.push_str(line);
                buffer.push('\n');
                in_multiline = true;
                continue;
            }

            // Single line execution
            buffer.push_str(line);
            self.execute(&buffer);
            buffer.clear();
        }

        println!("Goodbye!");
    }

    fn execute(&mut self, input: &str) {
        if input.trim().is_empty() {
            return;
        }

        // Lexing
        let mut lexer = Lexer::new(input);
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(err) => {
                eprintln!("Lexer error: {}", err);
                return;
            }
        };

        if self.show_tokens {
            println!("\n=== Tokens ===");
            for token in &tokens {
                println!("{}", token);
            }
            println!();
        }

        // Parsing
        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                eprintln!("Parse error: {}", err);
                return;
            }
        };

        if self.show_ast {
            println!("\n=== AST ===");
            println!("{}", program);
        }

        // Interpretation
        match self.interpreter.execute_program(&program) {
            Ok(value) => {
                if self.show_result {
                    println!("{}", value);
                }
            }
            Err(err) => {
                eprintln!("Runtime error: {}", err);
            }
        }
    }

    fn handle_command(&mut self, command: &str) -> Result<bool, String> {
        match command {
            ":quit" | ":q" | ":exit" => Ok(false),
            ":help" | ":h" => {
                self.show_help();
                Ok(true)
            }
            ":tokens on" => {
                self.show_tokens = true;
                println!("Token display enabled");
                Ok(true)
            }
            ":tokens off" => {
                self.show_tokens = false;
                println!("Token display disabled");
                Ok(true)
            }
            ":ast on" => {
                self.show_ast = true;
                println!("AST display enabled");
                Ok(true)
            }
            ":ast off" => {
                self.show_ast = false;
                println!("AST display disabled");
                Ok(true)
            }
            ":result on" => {
                self.show_result = true;
                println!("Result display enabled");
                Ok(true)
            }
            ":result off" => {
                self.show_result = false;
                println!("Result display disabled");
                Ok(true)
            }
            _ => Err(format!(
                "Unknown command: {}. Type ':help' for available commands.",
                command
            )),
        }
    }

    fn show_help(&self) {
        println!("Clause REPL Commands:");
        println!("  :help, :h          Show this help message");
        println!("  :quit, :q, :exit   Exit the REPL");
        println!("  :tokens on/off     Toggle token display");
        println!("  :ast on/off        Toggle AST display");
        println!("  :result on/off     Toggle result display");
        println!();
        println!("Usage:");
        println!("  - Type expressions or statements and press Enter");
        println!("  - Lines ending with '{{' start multiline input");
        println!("  - Empty line ends multiline input");
        println!();
        println!("Examples:");
        println!("  >>> 1 + 2");
        println!("  3");
        println!("  >>> let x = 42");
        println!("  42");
        println!("  >>> x * 2");
        println!("  84");
        println!("  >>> let y = {{");
        println!("  ...   let a = 10");
        println!("  ...   <- a * 2");
        println!("  ... }}");
        println!("  20");
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
