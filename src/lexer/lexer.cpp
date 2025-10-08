#include "clause/lexer/lexer.hpp"

#include <ranges>
#include <utility>
#include <unordered_map>

namespace clause {
    Lexer::Lexer(std::string source, std::string filename)
        : source_(std::move(source)), filename_(std::move(filename)) {
    }

    std::vector<Token> Lexer::tokenize() {
        std::vector<Token> tokens;

        while (!is_at_end()) {
            token_start_ = current_;
            token_start_line_ = line_;
            token_start_column_ = column_;
            if (auto token = next_token()) {
                tokens.push_back(std::move(*token));
            }
        }

        tokens.push_back(make_token(TokenType::EndOfFile));
        return tokens;
    }

    std::optional<Token> Lexer::next_token() {
        skip_whitespace();

        if (is_at_end()) {
            return std::nullopt;
        }

        token_start_ = current_;
        token_start_line_ = line_;
        token_start_column_ = column_;
        const char c = advance();

        // Numbers
        if (std::isdigit(c)) {
            current_--;
            column_--;
            return number();
        }

        // Identifiers and keywords
        if (std::isalpha(c) || c == '_') {
            current_--;
            column_--;
            return identifier();
        }

        // String literals
        if (c == '"') {
            return string();
        }

        // Comments
        if (c == '/' && peek() == '/') {
            skip_comment();
            return next_token();
        }

        // Two-character operators
        // ReSharper disable once CppDefaultCaseNotHandledInSwitchStatement
        switch (c) { // NOLINT(*-multiway-paths-covered)
            case '=':
                if (peek() == '=') {
                    advance();
                    return make_token(TokenType::EqualEqual);
                }
                return make_token(TokenType::Equal);
            case '!':
                if (peek() == '=') {
                    advance();
                    return make_token(TokenType::BangEqual);
                }
                return make_token(TokenType::Bang);
            case '<':
                if (peek() == '=') {
                    advance();
                    return make_token(TokenType::LessEqual);
                }
                return make_token(TokenType::Less);
            case '>':
                if (peek() == '=') {
                    advance();
                    return make_token(TokenType::GreaterEqual);
                }
                return make_token(TokenType::Greater);
            case '-':
                if (peek() == '>') {
                    advance();
                    return make_token(TokenType::Arrow);
                }
                return make_token(TokenType::Minus);
            case '.':
                if (peek() == '.') {
                    advance();
                    return make_token(TokenType::DotDot);
                }
                return make_token(TokenType::Dot);
            case '&':
                if (peek() == '&') {
                    advance();
                    return make_token(TokenType::And);
                }
                return make_token(TokenType::Ampersand);
            case '|':
                if (peek() == '|') {
                    advance();
                    return make_token(TokenType::Or);
                }
                return make_token(TokenType::Pipe);
        }

        // Single-character tokens
        // ReSharper disable once CppDefaultCaseNotHandledInSwitchStatement
        switch (c) { // NOLINT(*-multiway-paths-covered)
            case '+': return make_token(TokenType::Plus);
            case '*': return make_token(TokenType::Star);
            case '/': return make_token(TokenType::Slash);
            case '(': return make_token(TokenType::LeftParen);
            case ')': return make_token(TokenType::RightParen);
            case '{': return make_token(TokenType::LeftBrace);
            case '}': return make_token(TokenType::RightBrace);
            case '[': return make_token(TokenType::LeftBracket);
            case ']': return make_token(TokenType::RightBracket);
            case ',': return make_token(TokenType::Comma);
            case ';': return make_token(TokenType::Semicolon);
            case ':': return make_token(TokenType::Colon);
        }

        add_error("Unexpected character: " + std::string(1, c));
        return make_token(TokenType::Invalid);
    }

    char Lexer::peek() const {
        if (is_at_end()) return '\0';
        return source_[current_];
    }

    char Lexer::peek_next() const {
        if (current_ + 1 >= source_.length()) return '\0';
        return source_[current_ + 1];
    }

    char Lexer::advance() {
        const char c = source_[current_++];
        if (c == '\n') {
            line_++;
            column_ = 1;
        } else {
            column_++;
        }
        return c;
    }

    bool Lexer::is_at_end() const {
        return current_ >= source_.length();
    }

    void Lexer::skip_whitespace() {
        while (!is_at_end() && std::isspace(peek())) {
            advance();
        }
    }

    void Lexer::skip_comment() {
        // Skip until end of line
        while (!is_at_end() && peek() != '\n') {
            advance();
        }
    }

    Token Lexer::make_token(TokenType type, TokenValue value) const {
        return {type, std::move(value), SourceLocation(filename_, token_start_line_, token_start_column_)};
    }

    Token Lexer::number() {
        const size_t start = current_;

        // Consume all digits
        while (std::isdigit(peek())) {
            this->advance();
        }

        // Check for decimal point
        bool is_float = false;
        if (peek() == '.' && std::isdigit(peek_next())) {
            is_float = true;
            this->advance(); // consume '.'

            // Consume fractional digits
            while (std::isdigit(peek())) {
                this->advance();
            }
        }

        // Check if number is followed by an identifier character (letter or underscore)
        // This would indicate an invalid identifier starting with a digit
        if (std::isalpha(peek()) || peek() == '_') {
            this->add_error("Identifiers cannot start with a digit");
            // Consume the rest of the invalid identifier
            while (std::isalnum(peek()) || peek() == '_') {
                this->advance();
            }
            return make_token(TokenType::Invalid);
        }

        const std::string number_str = source_.substr(start, current_ - start);

        if (is_float) {
            double value = std::stod(number_str);
            return make_token(TokenType::Float, value);
        }
        int64_t value = std::stoll(number_str);
        return make_token(TokenType::Integer, value);
    }

    Token Lexer::string() {
        std::string value;

        // String already started (opening quote consumed)
        while (!is_at_end() && peek() != '"') {
            if (peek() == '\n') {
                this->add_error("Unterminated string");
                return make_token(TokenType::Invalid);
            }

            // Handle escape sequences
            if (peek() == '\\') {
                this->advance();
                if (is_at_end()) {
                    this->add_error("Unterminated string");
                    return make_token(TokenType::Invalid);
                }

                switch (char escaped = this->advance()) {
                    case 'n': value += '\n';
                        break;
                    case 't': value += '\t';
                        break;
                    case 'r': value += '\r';
                        break;
                    case '\\': value += '\\';
                        break;
                    case '"': value += '"';
                        break;
                    default:
                        value += escaped;
                        break;
                }
            } else {
                value += this->advance();
            }
        }

        if (is_at_end()) {
            this->add_error("Unterminated string");
            return make_token(TokenType::Invalid);
        }

        // Consume closing quote
        this->advance();
        return make_token(TokenType::String, value);
    }

    Token Lexer::identifier() {
        const size_t start = current_;

        // Consume identifier characters
        while (std::isalnum(peek()) || peek() == '_') {
            this->advance();
        }

        std::string text = source_.substr(start, current_ - start);

        // Check for keywords
        static const std::unordered_map<std::string, TokenType> keywords = {
            {"let", TokenType::Let},
            {"const", TokenType::Const},
            {"fn", TokenType::Fn},
            {"return", TokenType::Return},
            {"if", TokenType::If},
            {"else", TokenType::Else},
            {"while", TokenType::While},
            {"for", TokenType::For},
            {"class", TokenType::Class},
            {"where", TokenType::Where},
            {"in", TokenType::In}
        };

        if (const auto it = keywords.find(text); it != keywords.end()) {
            return make_token(it->second);
        }

        return make_token(TokenType::Identifier, text);
    }

    void Lexer::add_error(const std::string &message) {
        errors_.emplace_back(ErrorType::Lexical, message, SourceLocation(filename_, line_, column_));
    }
} // namespace clause
