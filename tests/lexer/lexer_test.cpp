#include <gtest/gtest.h>
#include "clause/lexer/lexer.hpp"
#include "clause/lexer/token.hpp"

namespace clause {
    namespace {
        // Helper function to verify token properties
        void ExpectToken(const Token &token, const TokenType expected_type) {
            EXPECT_EQ(token.type(), expected_type);
        }

        void ExpectToken(const Token &token, const int64_t expected_value) {
            EXPECT_EQ(token.type(), TokenType::Integer);
            ASSERT_TRUE(std::holds_alternative<int64_t>(token.value()));
            EXPECT_EQ(std::get<int64_t>(token.value()), expected_value);
        }

        void ExpectToken(const Token &token, const double expected_value) {
            EXPECT_EQ(token.type(), TokenType::Float);
            ASSERT_TRUE(std::holds_alternative<double>(token.value()));
            EXPECT_DOUBLE_EQ(std::get<double>(token.value()), expected_value);
        }

        void ExpectToken(const Token &token, const TokenType expected_type, const std::string &expected_value) {
            EXPECT_EQ(token.type(), expected_type);
            ASSERT_TRUE(std::holds_alternative<std::string>(token.value()));
            EXPECT_EQ(std::get<std::string>(token.value()), expected_value);
        }

        // Test fixture for lexer tests
        class LexerTest : public ::testing::Test {
        protected:
            static std::vector<Token> Tokenize(const std::string &source) {
                Lexer lexer(source, "test.clause");
                return lexer.tokenize();
            }

            static Lexer CreateLexer(const std::string &source) {
                return Lexer(source, "test.clause");
            }
        };

        // =============================================================================================================
        // Integer Literal Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesSingleDigitInteger) {
            const auto tokens = Tokenize("5");
            ASSERT_EQ(tokens.size(), 2); // Integer + EOF
            ExpectToken(tokens[0], 5LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesMultiDigitInteger) {
            const auto tokens = Tokenize("12345");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 12345LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesZero) {
            const auto tokens = Tokenize("0");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 0LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesLargeInteger) {
            const auto tokens = Tokenize("9223372036854775807"); // Max int64_t
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 9223372036854775807LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Float Literal Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesSimpleFloat) {
            const auto tokens = Tokenize("3.14");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 3.14);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesFloatWithZeroIntegerPart) {
            const auto tokens = Tokenize("0.5");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 0.5);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesFloatWithMultipleDecimalDigits) {
            const auto tokens = Tokenize("123.456789");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 123.456789);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DoesNotTokenizeTrailingDotAsFloat) {
            const auto tokens = Tokenize("42.");
            ASSERT_GE(tokens.size(), 2);
            // Should be integer 42 followed by something else (not a float)
            ExpectToken(tokens[0], 42LL);
        }

        // =============================================================================================================
        // String Literal Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesEmptyString) {
            const auto tokens = Tokenize("\"\"");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::String, std::string(""));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesSimpleString) {
            const auto tokens = Tokenize("\"hello\"");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::String, std::string("hello"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesStringWithSpaces) {
            const auto tokens = Tokenize("\"hello world\"");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::String, std::string("hello world"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesStringWithEscapeSequences) {
            const auto tokens = Tokenize(R"("hello\nworld")");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::String, std::string("hello\nworld"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesStringWithAllEscapeSequences) {
            const auto tokens = Tokenize(R"("\n\t\r\\\"")");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::String, std::string("\n\t\r\\\""));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, ReportsErrorForUnterminatedString) {
            Lexer lexer = CreateLexer("\"unterminated");
            const auto tokens = lexer.tokenize();
            EXPECT_FALSE(lexer.errors().empty());
            EXPECT_EQ(lexer.errors()[0].type(), ErrorType::Lexical);
        }

        TEST_F(LexerTest, ReportsErrorForStringWithNewline) {
            Lexer lexer = CreateLexer("\"line1\nline2\"");
            const auto tokens = lexer.tokenize();
            EXPECT_FALSE(lexer.errors().empty());
        }

        // =============================================================================================================
        // Identifier Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesSingleLetterIdentifier) {
            const auto tokens = Tokenize("x");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Identifier, std::string("x"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesMultiLetterIdentifier) {
            const auto tokens = Tokenize("variable");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Identifier, std::string("variable"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesIdentifierWithUnderscore) {
            const auto tokens = Tokenize("my_variable");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Identifier, std::string("my_variable"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesIdentifierWithNumbers) {
            const auto tokens = Tokenize("var123");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Identifier, std::string("var123"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesIdentifierStartingWithUnderscore) {
            const auto tokens = Tokenize("_private");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Identifier, std::string("_private"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, ReportsErrorForIdentifierStartingWithNumber) {
            Lexer lexer = CreateLexer("123var");
            const auto tokens = lexer.tokenize();
            EXPECT_FALSE(lexer.errors().empty());
            EXPECT_EQ(lexer.errors()[0].type(), ErrorType::Lexical);
        }

        // =============================================================================================================
        // Keyword Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesLetKeyword) {
            const auto tokens = Tokenize("let");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Let);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesConstKeyword) {
            const auto tokens = Tokenize("const");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Const);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesFnKeyword) {
            const auto tokens = Tokenize("fn");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Fn);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesReturnKeyword) {
            const auto tokens = Tokenize("return");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Return);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesControlFlowKeywords) {
            const auto tokens = Tokenize("if else while for");
            ASSERT_EQ(tokens.size(), 5);
            ExpectToken(tokens[0], TokenType::If);
            ExpectToken(tokens[1], TokenType::Else);
            ExpectToken(tokens[2], TokenType::While);
            ExpectToken(tokens[3], TokenType::For);
            ExpectToken(tokens[4], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesClassKeyword) {
            const auto tokens = Tokenize("class");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Class);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesWhereKeyword) {
            const auto tokens = Tokenize("where");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Where);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesInKeyword) {
            const auto tokens = Tokenize("in");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::In);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DistinguishesInKeywordFromIdentifier) {
            const auto tokens = Tokenize("input");
            ASSERT_EQ(tokens.size(), 2);
            // "input" starts with "in" but is not the keyword
            ExpectToken(tokens[0], TokenType::Identifier, std::string("input"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DistinguishesKeywordFromIdentifier) {
            const auto tokens = Tokenize("letter");
            ASSERT_EQ(tokens.size(), 2);
            // "letter" starts with "let" but is not the keyword
            ExpectToken(tokens[0], TokenType::Identifier, std::string("letter"));
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Operator Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesArithmeticOperators) {
            const auto tokens = Tokenize("+ - * /");
            ASSERT_EQ(tokens.size(), 5);
            ExpectToken(tokens[0], TokenType::Plus);
            ExpectToken(tokens[1], TokenType::Minus);
            ExpectToken(tokens[2], TokenType::Star);
            ExpectToken(tokens[3], TokenType::Slash);
            ExpectToken(tokens[4], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesEqualOperator) {
            const auto tokens = Tokenize("=");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Equal);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesEqualEqualOperator) {
            const auto tokens = Tokenize("==");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::EqualEqual);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesBangEqualOperator) {
            const auto tokens = Tokenize("!=");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::BangEqual);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesComparisonOperators) {
            const auto tokens = Tokenize("< <= > >=");
            ASSERT_EQ(tokens.size(), 5);
            ExpectToken(tokens[0], TokenType::Less);
            ExpectToken(tokens[1], TokenType::LessEqual);
            ExpectToken(tokens[2], TokenType::Greater);
            ExpectToken(tokens[3], TokenType::GreaterEqual);
            ExpectToken(tokens[4], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesArrowOperator) {
            const auto tokens = Tokenize("->");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Arrow);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DistinguishesMinusFromArrow) {
            const auto tokens = Tokenize("- >");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], TokenType::Minus);
            ExpectToken(tokens[1], TokenType::Greater);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesAmpersandOperator) {
            const auto tokens = Tokenize("&");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Ampersand);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesAndOperator) {
            const auto tokens = Tokenize("&&");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::And);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesPipeOperator) {
            const auto tokens = Tokenize("|");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Pipe);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesOrOperator) {
            const auto tokens = Tokenize("||");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::Or);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DistinguishesAmpersandFromAnd) {
            const auto tokens = Tokenize("& &&");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], TokenType::Ampersand);
            ExpectToken(tokens[1], TokenType::And);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DistinguishesPipeFromOr) {
            const auto tokens = Tokenize("| ||");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], TokenType::Pipe);
            ExpectToken(tokens[1], TokenType::Or);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Delimiter Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesParentheses) {
            const auto tokens = Tokenize("()");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], TokenType::LeftParen);
            ExpectToken(tokens[1], TokenType::RightParen);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesBraces) {
            const auto tokens = Tokenize("{}");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], TokenType::LeftBrace);
            ExpectToken(tokens[1], TokenType::RightBrace);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesBrackets) {
            const auto tokens = Tokenize("[]");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], TokenType::LeftBracket);
            ExpectToken(tokens[1], TokenType::RightBracket);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesPunctuation) {
            const auto tokens = Tokenize(",;:");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], TokenType::Comma);
            ExpectToken(tokens[1], TokenType::Semicolon);
            ExpectToken(tokens[2], TokenType::Colon);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesDotDotDelimiter) {
            const auto tokens = Tokenize("..");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], TokenType::DotDot);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesDotDotInRange) {
            const auto tokens = Tokenize("1..10");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 1LL);
            ExpectToken(tokens[1], TokenType::DotDot);
            ExpectToken(tokens[2], 10LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesDotDotWithSpaces) {
            const auto tokens = Tokenize("1 .. 10");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 1LL);
            ExpectToken(tokens[1], TokenType::DotDot);
            ExpectToken(tokens[2], 10LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesDotDotInParentheses) {
            const auto tokens = Tokenize("(0..100)");
            ASSERT_EQ(tokens.size(), 6);
            ExpectToken(tokens[0], TokenType::LeftParen);
            ExpectToken(tokens[1], 0LL);
            ExpectToken(tokens[2], TokenType::DotDot);
            ExpectToken(tokens[3], 100LL);
            ExpectToken(tokens[4], TokenType::RightParen);
            ExpectToken(tokens[5], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Comment Tests
        // =============================================================================================================

        TEST_F(LexerTest, SkipsSingleLineComment) {
            const auto tokens = Tokenize("// this is a comment\n42");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, SkipsCommentAtEndOfFile) {
            const auto tokens = Tokenize("42 // comment");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, SkipsMultipleComments) {
            const auto tokens = Tokenize("// comment 1\n42\n// comment 2\n43");
            ASSERT_EQ(tokens.size(), 3);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], 43LL);
            ExpectToken(tokens[2], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, DoesNotConfuseSlashWithComment) {
            const auto tokens = Tokenize("10 / 2");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 10LL);
            ExpectToken(tokens[1], TokenType::Slash);
            ExpectToken(tokens[2], 2LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Whitespace Tests
        // =============================================================================================================

        TEST_F(LexerTest, SkipsLeadingWhitespace) {
            const auto tokens = Tokenize("   42");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, SkipsTrailingWhitespace) {
            const auto tokens = Tokenize("42   ");
            ASSERT_EQ(tokens.size(), 2);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, SkipsWhitespaceBetweenTokens) {
            const auto tokens = Tokenize("42   +   10");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::Plus);
            ExpectToken(tokens[2], 10LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, HandlesTabs) {
            const auto tokens = Tokenize("\t42\t+\t10\t");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::Plus);
            ExpectToken(tokens[2], 10LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, HandlesNewlines) {
            const auto tokens = Tokenize("42\n+\n10");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 42LL);
            ExpectToken(tokens[1], TokenType::Plus);
            ExpectToken(tokens[2], 10LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Complex Expression Tests
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesSimpleExpression) {
            const auto tokens = Tokenize("2 + 3");
            ASSERT_EQ(tokens.size(), 4);
            ExpectToken(tokens[0], 2LL);
            ExpectToken(tokens[1], TokenType::Plus);
            ExpectToken(tokens[2], 3LL);
            ExpectToken(tokens[3], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesComplexExpression) {
            const auto tokens = Tokenize("(10 + 20) * 3.14");
            ASSERT_EQ(tokens.size(), 8);
            ExpectToken(tokens[0], TokenType::LeftParen);
            ExpectToken(tokens[1], 10LL);
            ExpectToken(tokens[2], TokenType::Plus);
            ExpectToken(tokens[3], 20LL);
            ExpectToken(tokens[4], TokenType::RightParen);
            ExpectToken(tokens[5], TokenType::Star);
            ExpectToken(tokens[6], 3.14);
            ExpectToken(tokens[7], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesVariableDeclaration) {
            const auto tokens = Tokenize("let x = 42;");
            ASSERT_EQ(tokens.size(), 6);
            ExpectToken(tokens[0], TokenType::Let);
            ExpectToken(tokens[1], TokenType::Identifier, std::string("x"));
            ExpectToken(tokens[2], TokenType::Equal);
            ExpectToken(tokens[3], 42LL);
            ExpectToken(tokens[4], TokenType::Semicolon);
            ExpectToken(tokens[5], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesFunctionSignature) {
            const auto tokens = Tokenize("fn add(a, b) -> c");
            ASSERT_EQ(tokens.size(), 10);
            ExpectToken(tokens[0], TokenType::Fn);
            ExpectToken(tokens[1], TokenType::Identifier, std::string("add"));
            ExpectToken(tokens[2], TokenType::LeftParen);
            ExpectToken(tokens[3], TokenType::Identifier, std::string("a"));
            ExpectToken(tokens[4], TokenType::Comma);
            ExpectToken(tokens[5], TokenType::Identifier, std::string("b"));
            ExpectToken(tokens[6], TokenType::RightParen);
            ExpectToken(tokens[7], TokenType::Arrow);
            ExpectToken(tokens[8], TokenType::Identifier, std::string("c"));
            ExpectToken(tokens[9], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesConditionalStatement) {
            const auto tokens = Tokenize("if x == 5 { return true; }");
            ASSERT_EQ(tokens.size(), 10);
            ExpectToken(tokens[0], TokenType::If);
            ExpectToken(tokens[1], TokenType::Identifier, std::string("x"));
            ExpectToken(tokens[2], TokenType::EqualEqual);
            ExpectToken(tokens[3], 5LL);
            ExpectToken(tokens[4], TokenType::LeftBrace);
            ExpectToken(tokens[5], TokenType::Return);
            ExpectToken(tokens[6], TokenType::Identifier, std::string("true"));
            ExpectToken(tokens[7], TokenType::Semicolon);
            ExpectToken(tokens[8], TokenType::RightBrace);
            ExpectToken(tokens[9], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesConstraintSyntax) {
            const auto tokens = Tokenize("where self in (3..8)");
            ASSERT_EQ(tokens.size(), 9);
            ExpectToken(tokens[0], TokenType::Where);
            ExpectToken(tokens[1], TokenType::Identifier, std::string("self"));
            ExpectToken(tokens[2], TokenType::In);
            ExpectToken(tokens[3], TokenType::LeftParen);
            ExpectToken(tokens[4], 3LL);
            ExpectToken(tokens[5], TokenType::DotDot);
            ExpectToken(tokens[6], 8LL);
            ExpectToken(tokens[7], TokenType::RightParen);
            ExpectToken(tokens[8], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Error Handling Tests
        // =============================================================================================================

        TEST_F(LexerTest, ReportsErrorForInvalidCharacter) {
            Lexer lexer = CreateLexer("@");
            const auto tokens = lexer.tokenize();
            EXPECT_FALSE(lexer.errors().empty());
            EXPECT_EQ(lexer.errors()[0].type(), ErrorType::Lexical);
        }

        TEST_F(LexerTest, ContinuesAfterError) {
            Lexer lexer = CreateLexer("42 @ 43");
            const auto tokens = lexer.tokenize();
            EXPECT_FALSE(lexer.errors().empty());
            // Should still tokenize the valid parts
            ASSERT_GE(tokens.size(), 2);
            ExpectToken(tokens[0], 42LL);
        }

        // =============================================================================================================
        // Edge Cases
        // =============================================================================================================

        TEST_F(LexerTest, TokenizesEmptySource) {
            const auto tokens = Tokenize("");
            ASSERT_EQ(tokens.size(), 1);
            ExpectToken(tokens[0], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesOnlyWhitespace) {
            const auto tokens = Tokenize("   \n\t  ");
            ASSERT_EQ(tokens.size(), 1);
            ExpectToken(tokens[0], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, TokenizesOnlyComment) {
            const auto tokens = Tokenize("// just a comment");
            ASSERT_EQ(tokens.size(), 1);
            ExpectToken(tokens[0], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, HandlesConsecutiveOperators) {
            const auto tokens = Tokenize("+-*/");
            ASSERT_EQ(tokens.size(), 5);
            ExpectToken(tokens[0], TokenType::Plus);
            ExpectToken(tokens[1], TokenType::Minus);
            ExpectToken(tokens[2], TokenType::Star);
            ExpectToken(tokens[3], TokenType::Slash);
            ExpectToken(tokens[4], TokenType::EndOfFile);
        }

        TEST_F(LexerTest, HandlesAdjacentDelimiters) {
            const auto tokens = Tokenize("()[]{}");
            ASSERT_EQ(tokens.size(), 7);
            ExpectToken(tokens[0], TokenType::LeftParen);
            ExpectToken(tokens[1], TokenType::RightParen);
            ExpectToken(tokens[2], TokenType::LeftBracket);
            ExpectToken(tokens[3], TokenType::RightBracket);
            ExpectToken(tokens[4], TokenType::LeftBrace);
            ExpectToken(tokens[5], TokenType::RightBrace);
            ExpectToken(tokens[6], TokenType::EndOfFile);
        }

        // =============================================================================================================
        // Source Location Tests
        // =============================================================================================================

        TEST_F(LexerTest, TracksLineNumbers) {
            const auto tokens = Tokenize("let\nx\n=\n42");
            ASSERT_EQ(tokens.size(), 5);
            EXPECT_EQ(tokens[0].location().line, 1);
            EXPECT_EQ(tokens[1].location().line, 2);
            EXPECT_EQ(tokens[2].location().line, 3);
            EXPECT_EQ(tokens[3].location().line, 4);
        }

        TEST_F(LexerTest, IncludesFilename) {
            Lexer lexer("42", "my_file.clause");
            const auto tokens = lexer.tokenize();
            ASSERT_GE(tokens.size(), 1);
            EXPECT_EQ(tokens[0].location().filename, "my_file.clause");
        }
    } // namespace
} // namespace clause
