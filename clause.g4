grammar Clause;

// ============================================================================
// Parser Rules
// ============================================================================

// Entry point for the program
program
    : declaration* EOF
    ;

// Top-level declarations (only functions and fix constants allowed)
declaration
    : functionDeclaration
    | topLevelConstant
    ;

functionDeclaration
    : FUN IDENTIFIER '(' parameterList? ')' (':' type)? block
    ;

parameterList
    : parameter (',' parameter)*
    ;

parameter
    : IDENTIFIER ':' type
    ;

topLevelConstant
    : FIX IDENTIFIER ':' type '=' expression ';'
    ;

// Statements
statement
    : variableDeclaration
    | returnStatement
    | yieldStatement
    | breakStatement
    | expressionStatement
    | blockStatement
    | ifStatement
    | whileStatement
    ;

variableDeclaration
    : (LET | FIX) IDENTIFIER (':' type)? '=' expression ';'
    ;

returnStatement
    : RETURN expression? ';'
    ;

expressionStatement
    : expression ';'
    ;

yieldStatement
    : YIELD expression? ';'
    ;

breakStatement
    : BREAK expression? ';'
    ;

// Block-based statements (no semicolon required)
// These use the same productions as the expression forms
blockStatement
    : blockExpression
    ;

ifStatement
    : ifExpression
    ;

whileStatement
    : whileExpression
    ;

// Expressions with precedence (lowest to highest precedence at bottom)
expression
    : assignment
    ;

assignment
    : logicalOr (assignmentOp logicalOr)*
    ;

assignmentOp
    : '=' | '+=' | '-=' | '*=' | '/=' | '%='
    | '&=' | '|=' | '^=' | '<<=' | '>>='
    ;

logicalOr
    : logicalAnd ('||' logicalAnd)*
    ;

logicalAnd
    : bitwiseOr ('&&' bitwiseOr)*
    ;

bitwiseOr
    : bitwiseXor ('|' bitwiseXor)*
    ;

bitwiseXor
    : bitwiseAnd ('^' bitwiseAnd)*
    ;

bitwiseAnd
    : equality ('&' equality)*
    ;

equality
    : relational (('==' | '!=') relational)*
    ;

relational
    : bitwiseShift (('<' | '<=' | '>' | '>=') bitwiseShift)*
    ;

bitwiseShift
    : additive (('<<' | '>>') additive)*
    ;

additive
    : multiplicative (('+' | '-') multiplicative)*
    ;

multiplicative
    : unary (('*' | '/' | '%') unary)*
    ;

unary
    : ('+' | '-' | '!' | '~') unary
    | postfix
    ;

postfix
    : primary (postfixOp)*
    ;

postfixOp
    : '[' expression ']'           // Array indexing
    | '(' argumentList? ')'        // Function call
    ;

argumentList
    : expression (',' expression)*
    ;

primary
    : literal
    | IDENTIFIER
    | blockExpression
    | ifExpression
    | whileExpression
    | '(' expression ')'
    ;

// Block/if/while as expressions (can be used in expression context)
blockExpression
    : block
    ;

ifExpression
    : IF expression block (ELSE IF expression block)* (ELSE block)?
    ;

whileExpression
    : WHILE expression block (ELSE block)?
    ;

// Block definition (shared by statements and expressions)
block
    : '{' blockBody '}'
    ;

blockBody
    : statement*
    ;

literal
    : INTEGER_LITERAL
    | FLOAT_LITERAL
    | BOOLEAN_LITERAL
    | VOID_LITERAL
    | arrayLiteral
    ;

arrayLiteral
    : '[' (expression (',' expression)*)? ']'
    ;

type
    : 'I8' | 'I16' | 'I32' | 'I64'
    | 'U8' | 'U16' | 'U32' | 'U64'
    | 'F16' | 'F32' | 'F64'
    | 'Bool'
    | 'Void'
    | '[' type ',' INTEGER_LITERAL ']'
    ;

// ============================================================================
// Lexer Rules
// ============================================================================

// Keywords
LET         : 'let';
FIX         : 'fix';
FUN         : 'fun';
RETURN      : 'return';
IF          : 'if';
ELSE        : 'else';
WHILE       : 'while';
BREAK       : 'break';
VOID_LITERAL: 'void';

// Built-in types (already covered in parser rule 'type')

// Literals
BOOLEAN_LITERAL
    : 'true'
    | 'false'
    ;

INTEGER_LITERAL
    : HexLiteral
    | BinaryLiteral
    | OctalLiteral
    | DecimalLiteral
    ;

fragment DecimalLiteral
    : [0-9] ([0-9_])*
    ;

fragment HexLiteral
    : '0' [xX] [0-9a-fA-F] ([0-9a-fA-F_])*
    ;

fragment BinaryLiteral
    : '0' [bB] [01] ([01_])*
    ;

fragment OctalLiteral
    : '0' [oO] [0-7] ([0-7_])*
    ;

FLOAT_LITERAL
    : [0-9] ([0-9_])* '.' [0-9] ([0-9_])* Exponent?
    | [0-9] ([0-9_])* Exponent
    ;

fragment Exponent
    : [eE] [+\-]? [0-9] ([0-9_])*
    ;

// Identifiers
IDENTIFIER
    : [a-zA-Z_][a-zA-Z0-9_]*
    ;

// Operators (single and multi-character)
// Assignment operators
ASSIGN      : '=';
PLUS_ASSIGN : '+=';
MINUS_ASSIGN: '-=';
MULT_ASSIGN : '*=';
DIV_ASSIGN  : '/=';
MOD_ASSIGN  : '%=';
AND_ASSIGN  : '&=';
OR_ASSIGN   : '|=';
XOR_ASSIGN  : '^=';
LSHIFT_ASSIGN: '<<=';
RSHIFT_ASSIGN: '>>=';

// Comparison operators
EQ          : '==';
NE          : '!=';
LT          : '<';
LE          : '<=';
GT          : '>';
GE          : '>=';

// Logical operators
LAND        : '&&';
LOR         : '||';
NOT         : '!';

// Bitwise operators
AND         : '&';
OR          : '|';
XOR         : '^';
BNOT        : '~';
LSHIFT      : '<<';
RSHIFT      : '>>';

// Arithmetic operators
PLUS        : '+';
MINUS       : '-';
MULT        : '*';
DIV         : '/';
MOD         : '%';

// Yield operator
YIELD       : '<-';

// Delimiters
LPAREN      : '(';
RPAREN      : ')';
LBRACE      : '{';
RBRACE      : '}';
LBRACKET    : '[';
RBRACKET    : ']';
COLON       : ':';
COMMA       : ',';
SEMICOLON   : ';';

// Comments
LINE_COMMENT
    : '//' ~[\r\n]* -> skip
    ;

// Whitespace
WHITESPACE
    : [ \t\r\n]+ -> skip
    ;
