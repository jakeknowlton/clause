grammar Clause;

// ============================================================================
// Parser Rules
// ============================================================================

// Entry point for the program
program
    : statement* EOF
    ;

// Statements
statement
    : variableDeclaration
    | yieldStatement
    | expressionStatement
    ;

variableDeclaration
    : ('let' | 'fix') IDENTIFIER (':' type)? '=' expression
    ;

expressionStatement
    : expression
    ;

yieldStatement
    : YIELD expression
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
    | primary
    ;

primary
    : literal
    | IDENTIFIER
    | block
    | '(' expression ')'
    ;

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
    ;

type
    : 'I8' | 'I16' | 'I32' | 'I64'
    | 'U8' | 'U16' | 'U32' | 'U64'
    | 'F16' | 'F32' | 'F64'
    | 'Bool'
    | 'Void'
    ;

// ============================================================================
// Lexer Rules
// ============================================================================

// Keywords
LET         : 'let';
FIX         : 'fix';
VOID_LITERAL: 'void';

// Built-in types (already covered in parser rule 'type')

// Literals
BOOLEAN_LITERAL
    : 'true'
    | 'false'
    ;

INTEGER_LITERAL
    : DecimalLiteral
    | HexLiteral
    | BinaryLiteral
    | OctalLiteral
    ;

fragment DecimalLiteral
    : [0-9]+
    ;

fragment HexLiteral
    : '0' [xX] [0-9a-fA-F]+
    ;

fragment BinaryLiteral
    : '0' [bB] [01]+
    ;

fragment OctalLiteral
    : '0' [oO] [0-7]+
    ;

FLOAT_LITERAL
    : [0-9]+ '.' [0-9]+ Exponent?
    | [0-9]+ Exponent
    ;

fragment Exponent
    : [eE] [+\-]? [0-9]+
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
COLON       : ':';

// Comments
LINE_COMMENT
    : '//' ~[\r\n]* -> skip
    ;

// Whitespace
WHITESPACE
    : [ \t\r\n]+ -> skip
    ;
