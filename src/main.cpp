#include <iostream>
#include <string>
#include "clause/lexer/lexer.hpp"
#include "clause/parser/parser.hpp"
#include "clause/interpreter/interpreter.hpp"

int main(int argc, char* argv[]) {
    std::cout << "Clause Interpreter v0.1.0\n";
    std::cout << "Type 'exit' to quit\n\n";

    std::string line;
    while (true) {
        std::cout << "clause> ";
        if (!std::getline(std::cin, line)) {
            break;
        }

        if (line == "exit" || line == "quit") {
            break;
        }

        if (line.empty()) {
            continue;
        }

        // Tokenize the input
        clause::Lexer lexer(line);
        auto tokens = lexer.tokenize();

        // Display any lexer errors
        if (!lexer.errors().empty()) {
            for (const auto& error : lexer.errors()) {
                std::cerr << error.format() << "\n";
            }
            continue;
        }

        // Display tokens
        std::cout << "Tokens:\n";
        for (const auto& token : tokens) {
            if (token.type() != clause::TokenType::EndOfFile) {
                std::cout << "  " << token.to_string() << "\n";
            }
        }
    }

    std::cout << "Goodbye!\n";
    return 0;
}
