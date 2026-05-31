# Clause — Grammar (living draft)

EBNF, built incrementally during the syntax design sessions. Conventions:
`x*` zero-or-more, `x+` one-or-more, `x?` optional, `|` alternation, `( )`
grouping, `'lit'` terminal, UPPERCASE lexical tokens.

This grammar realizes the semantics in [`DESIGN.md`](../DESIGN.md). Where a
syntactic choice has a rationale, it is noted in a `// note:` comment.

---

## 1. Program, statements, blocks

```ebnf
program        = declaration* EOF ;

// A block is an expression; its value is produced by an explicit yield (`<-`),
// never by an implicit trailing expression. Semicolons DELIMIT statements.
block          = '{' statement* '}' ;

statement      = ';'                         // empty statement
               | lineStatement ';'           // statements that require a terminator
               | blockStatement ;            // block-form, no terminator needed

lineStatement  = variableDecl
               | yieldStatement
               | breakStatement
               | continueStatement
               | returnStatement
               | expression ;                // expression statement (value discarded)

blockStatement = ifExpr | whileExpr | forExpr | matchExpr | block ;

// Three parallel value-exits, one per construct (each takes a bare value):
//   `<- v`     evaluates a BLOCK     `break v`  evaluates a LOOP
//   `return v` evaluates a FUNCTION
// `<-` never produces a loop's value (a loop body's per-iteration value is
// discarded); a loop's value comes from `break v` or its `else` completion block.
yieldStatement    = '<-' expression? ;       // value of the enclosing block
breakStatement    = 'break' expression? ;    // value of the enclosing loop (and exits it)
continueStatement = 'continue' ;
returnStatement   = 'return' expression? ;   // value of the enclosing function
```

Function bodies may be a `block` or an expression-body `= expression` (see §3).

## 2. Bindings & `where` clauses

```ebnf
variableDecl   = ( 'fix' | 'let' ) IDENTIFIER ( ':' typeExpr )? whereClause* '=' expression ;

// Each constraint is its own `where`; they conjoin. The binder is the declared
// name (here, the IDENTIFIER). In anonymous positions the pronoun `it` is used.
// A `whereClause` carries the same `constraint` as the set-builder form (§4).
whereClause      = 'where' constraint ;
constraint       = verificationMode? predicate ;
verificationMode = 'dynamic' ;              // 'static' is the default (omitted)
predicate        = expression ;             // must be Bool-typed AND runtime-evaluable
                                            //   (bounded quantifiers only) — semantic check
```

Examples:
```clause
fix x = 5;
fix x: I32 where x > 0 = 5;
fix x where x > 0 = 5;                       // type inferred, binder = x
let count: U32 where count < limit = 0;
fix p: I32 where p > 0
              where dynamic isPrime(p) = next_prime();
```

## 3. Functions & measures

```ebnf
funcDecl     = funcModifier* 'fun' IDENTIFIER genericParams? '(' paramList? ')'
               whereClause*                        // preconditions (scope: params)
               ( '->' typeExpr whereClause* )?     // return type + postcondition (binder 'it')
               funcBody ;

measureDecl  = visibility? 'measure' IDENTIFIER genericParams? '(' paramList? ')'
               whereClause*                        // preconditions (e.g. domain restriction)
               '->' typeExpr
               funcBody ;          // body restricted to the total sublanguage (semantic check)

funcModifier = visibility | 'pure' ;
visibility   = 'public' | 'internal' ;            // absence = private (module-only)

funcBody     = block | '=' expression ';' ;

paramList    = param ( ',' param )* ;
param        = IDENTIFIER ':' typeExpr whereClause* ;   // per-param precondition (inline sugar)

// Preconditions: the pre-`->` `whereClause*` (scope: params, and `self` in a
// method) holds relational/self preconditions; per-param `where` is inline sugar
// for a single-parameter one. The post-return `whereClause*` is the postcondition
// (binder `it`). Position disambiguates: before `->` ⇒ precondition; after the
// return type ⇒ postcondition.

genericParams  = '<' genericParam ( ',' genericParam )* '>' ;
genericParam   = IDENTIFIER ( ':' interfaceBound )? ;
interfaceBound = typePath ( '+' typePath )* ;     // e.g.  T: Ord + Hash
```

## 4. Type expressions

```ebnf
typeExpr      = funcType | postfixType ;

funcType      = '(' ( typeExpr ( ',' typeExpr )* )? ')' '->' typeExpr ;
postfixType   = atomType '?'* ;               // T?  ==  Option<T>  (may stack: T?? )

atomType      = primitiveType
              | typePath genericArgs?
              | arrayType
              | tupleType
              | groupType
              | refinementType ;

arrayType     = '[' typeExpr ( ',' constExpr )? ']' ;   // [T, N] array | [T] slice (== Slice<T>)
tupleType     = '(' typeExpr ',' typeExpr ( ',' typeExpr )* ')' ;   // 2+ elements
groupType     = '(' typeExpr ')' ;
refinementType = '{' IDENTIFIER ':' typeExpr '|' constraintList '}' ; // {v: T | c1, c2}
constraintList = constraint ( ',' constraint )* ;   // same `constraint` as a whereClause (§2)

genericArgs   = '<' typeExpr ( ',' typeExpr )* '>' ;
typePath      = IDENTIFIER ( '::' IDENTIFIER )* ;
primitiveType = 'I8'|'I16'|'I32'|'I64'|'U8'|'U16'|'U32'|'U64'
              | 'F16'|'F32'|'F64'|'Bool'|'Void'|'Char' ;
              // String/Option/Result/Ptr/List are prelude types (a `path`), not reserved
```

Notes:
- A parenthesized type list is parsed, then classified: followed by `->` ⇒
  `funcType`; otherwise ≥2 elements ⇒ tuple, exactly 1 ⇒ grouping. (`Void` is the
  unit type; there is no `()` type literal.)
- A refinement is **base type + a comma-separated list of moded constraints**,
  expressed identically in `where`-form and set-builder form (they share the
  `constraint` production). `,` separates constraints; `&&` combines within a
  single constraint's predicate. Modes **are** carried in the type
  representation. However, **subtyping / precondition-satisfaction uses only the
  logical predicates** — a value of type `{v | p}` satisfies a requirement for
  `{p' | p}` regardless of either side's modes. Modes instead drive:
  well-formedness (a `static` predicate may use only `measure` functions; a
  `dynamic` predicate any `pure` function) and the code emitted at establishment
  / discharge sites. Every constraint is therefore either *proven* (`static`) or
  *runtime-guaranteed* (`dynamic`) — nothing is merely trusted.
- `constExpr` is a compile-time-constant expression (semantic restriction).

## 5. Type declarations

```ebnf
typeDecl     = structDecl | classDecl | enumDecl | interfaceDecl | implDecl | aliasDecl ;

// Invariant lives in the header (whereClause*), contract-first, for struct & class.

// STRUCT — body is a pure record: public fields only, no mutability keyword
// (a struct's mutability comes from its binding), no `init`. Behavior via `impl`.
structDecl   = visibility? 'struct' IDENTIFIER genericParams? whereClause*
               '{' ( structField ( ',' structField )* ','? )? '}' ;
structField  = IDENTIFIER ':' typeExpr whereClause* ;

// CLASS — fields (private default, `fix` default), inits, and inherent methods
// live in the body. Interface conformance lives in `impl` blocks.
classDecl    = visibility? 'class' IDENTIFIER genericParams? whereClause*
               '{' classMember* '}' ;
classMember  = fieldDecl | initDecl | methodDecl ;
fieldDecl    = visibility? ( 'fix' | 'let' )? IDENTIFIER ':' typeExpr whereClause* ';' ; // default fix
initDecl     = visibility? 'init' '(' paramList? ')' block ;     // dispatched by base-type sig

methodDecl   = methodModifier* 'fun' IDENTIFIER genericParams? '(' paramList? ')'
               whereClause*                            // preconditions (scope: self + params)
               ( '->' typeExpr whereClause* )? funcBody ;
methodModifier = funcModifier | 'static' ;             // instance is default (implicit `self`);
                                                       // 'static' ⇒ associated fn (no `self`)

// ENUM — tuple-style and struct-style payloads.
enumDecl     = visibility? 'enum' IDENTIFIER genericParams? '{' enumVariant ( ',' enumVariant )* ','? '}' ;
enumVariant  = IDENTIFIER ( '(' typeExpr ( ',' typeExpr )* ')'
                          | '{' structField ( ',' structField )* '}' )? ;

// INTERFACE — method signatures and/or default methods; `Self` = implementor.
interfaceDecl   = visibility? 'interface' IDENTIFIER genericParams? '{' interfaceMember* '}' ;
interfaceMember = methodSig | methodDecl ;
methodSig       = 'static'? 'fun' IDENTIFIER genericParams? '(' paramList? ')'
                  whereClause* ( '->' typeExpr whereClause* )? ';' ;

// IMPL — interface conformance (orphan rule: in the interface's or type's package,
// semantic check). `impl T { ... }` (no `for`) adds inherent behavior to any type.
implDecl     = 'impl' ( typePath genericArgs? 'for' )? typeExpr '{' ( initDecl | methodDecl )* '}' ;

// TYPE ALIAS — including refined aliases.
aliasDecl    = visibility? 'type' IDENTIFIER genericParams? '=' typeExpr whereClause* ';' ;
```

Notes:
- `self` is a keyword-expression of type `Self`; fields are accessed `self.field`
  (qualified — no implicit field scope, so params never shadow fields). The
  header invariant references fields unqualified (`where balance >= 0`), as it has
  no parameters to shadow.
- `Self` is the implementing type inside `interface`/`impl`/method bodies.
- Instance methods are the **default** and have an *implicit* `self` (accessed
  as `self.field`; no `self` parameter). **`static fun`** marks an associated
  function (no `self` in scope; may name `Self`). `init` likewise has an implicit
  `self` (the object under construction, assigned `self.field`). A method's
  receiver-state precondition goes in its pre-`->` `where` zone (`where len(self)
  > 0`).
- Call convention: **`Type::assoc(...)`** for static/associated items (type path,
  `::`), **`value.method(...)`** / **`value.field`** for instance members (`.`),
  **`Type(...)`** to run an `init`.

## 6. Expressions & operators

Precedence low → high. Each level's operands are the next level up.

```ebnf
expression   = assignment ;
assignment   = range ( assignOp assignment )? ;          // right-assoc; type Void
assignOp     = '=' | '+=' | '-=' | '*=' | '/=' | '%='
             | '&=' | '|=' | '^=' | '<<=' | '>>=' ;
range        = logicalOr ( ( '..' | '..=' ) logicalOr )? ;   // non-assoc; '..' excl, '..=' incl
logicalOr    = logicalAnd ( '||' logicalAnd )* ;
logicalAnd   = bitOr     ( '&&' bitOr )* ;
bitOr        = bitXor    ( '|'  bitXor )* ;
bitXor       = bitAnd    ( '^'  bitAnd )* ;
bitAnd       = equality  ( '&'  equality )* ;
equality     = relational ( ( '==' | '!=' ) relational )* ;
relational   = shift   ( ( '<' | '<=' | '>' | '>=' ) shift )* ;
shift        = additive ( ( '<<' | '>>' ) additive )* ;
additive     = multiplicative ( ( '+' | '-' | '+%' | '-%' ) multiplicative )* ;
multiplicative = cast ( ( '*' | '/' | '%' | '*%' ) cast )* ;
cast         = unary ( 'as' typeExpr )* ;                // binds tighter than arithmetic
unary        = ( '+' | '-' | '!' | '~' ) unary | postfix ;
postfix      = primary postfixOp* ;
postfixOp    = '.' IDENTIFIER            // field access / method name
             | '.' INTEGER_LITERAL       // tuple element (t.0)
             | '(' argList? ')'          // call
             | '[' expression ']'        // index
             | '?' ;                     // error propagation (Result)
argList      = expression ( ',' expression )* ;

primary      = literal
             | path                       // names, module/type-qualified (incl. Type::assoc)
             | 'self'
             | '(' expression ')'         // grouping
             | tupleLit | structLit | arrayLit
             | lambda                     // §7
             | uncheckedExpr              // §9
             | block | ifExpr | whileExpr | forExpr | matchExpr ;
uncheckedExpr = 'unchecked' block ;       // value-producing; only place for extern/raw-deref

path         = IDENTIFIER ( '::' IDENTIFIER )* turbofish? ;
turbofish    = '::<' typeExpr ( ',' typeExpr )* '>' ;     // explicit type args
structLit    = path '{' ( fieldInit ( ',' fieldInit )* ','? )? '}' ;
fieldInit    = IDENTIFIER ':' expression | IDENTIFIER ;    // shorthand { x } == { x: x }
tupleLit     = '(' expression ',' expression ( ',' expression )* ')' ;   // 2+ elements
arrayLit     = '[' ( expression ( ',' expression )* ','? )? ']' ;
literal      = INTEGER_LITERAL | FLOAT_LITERAL | BOOLEAN_LITERAL
             | STRING_LITERAL  | CHAR_LITERAL  | 'void' ;
```

Notes:
- **Struct-literal disambiguation:** a bare `path { … }` is *not* allowed at the
  top level of an `if`/`while`/`for`/`match` scrutinee; parenthesize it there
  (`if (Point { x: 0 }).is_origin() { … }`). Elsewhere it is unambiguous.
- `void` is the unit value (type `Void`). Assignment expressions have type `Void`.

## 7. Closures, match, patterns

```ebnf
lambda       = lambdaParams '=>' ( expression | block ) ;
lambdaParams = IDENTIFIER | '(' ( lambdaParam ( ',' lambdaParam )* )? ')' ;
lambdaParam  = IDENTIFIER ( ':' typeExpr )? ;

matchExpr    = 'match' expression '{' matchArm ( ',' matchArm )* ','? '}' ;
                                  // scrutinee: bare struct-literal must be parenthesized
matchArm     = armLeft '=>' ( expression | block ) ;
armLeft      = condition | pattern ;
condition    = expression ;       // a Bool expression mentioning `it` (the scrutinee);
                                  // the presence of `it` marks a condition arm, not a pattern

pattern        = orPattern ;
orPattern      = primaryPattern ( '|' primaryPattern )* ;
primaryPattern = '_'                                   // wildcard — the ONLY catch-all
               | literal                               // literal pattern
               | rangePattern                          // a..b | a..=b
               | IDENTIFIER                             // lowercase ⇒ binding; resolves-to-variant/const ⇒ that
               | path ( '(' ( pattern ( ',' pattern )* )? ')' )?    // variant / tuple-variant
               | path '{' fieldPat ( ',' fieldPat )* ( ',' '..' )? '}'  // struct-variant
               | '(' pattern ( ',' pattern )* ')'      // tuple pattern
               | slicePattern ;
rangePattern   = literal ( '..' | '..=' ) literal ;
slicePattern   = '[' ( patternElem ( ',' patternElem )* )? ']' ;
patternElem    = pattern | '..' IDENTIFIER? ;          // rest: `..name` or `..`
fieldPat       = IDENTIFIER ( ':' pattern )? ;         // shorthand `{ x }` binds field x
```

Notes:
- **No `where`/`if` guards.** An arm-left containing `it` is a *condition arm*
  (Bool over the scrutinee); otherwise it is a *pattern*. The catch-all is `_`;
  reach the scrutinee in any body via `it` (no bare-identifier catch-all).
- **Exhaustiveness:** structural matches by closed-variant coverage; condition
  matches when the solver proves the arms total, else a `_` arm is required.
- **Binding vs variant:** a lowercase `IDENTIFIER` binds; one that resolves to a
  variant/constant matches it (Capitalized-variants convention). No `@`-bindings.

## 8. Control flow

`if`/`while`/`for` are expressions. The scrutinee/condition may not be a bare
struct-literal (parenthesize). A loop's value comes from `break v` (early) or its
`else` completion block (normal completion).

```ebnf
ifExpr     = 'if' expression block ( 'else' 'if' expression block )* ( 'else' block )? ;
whileExpr  = 'while' expression block ( 'else' block )? ;        // else = completion value
forExpr    = 'for' pattern 'in' expression block ( 'else' block )? ;
```

Notes:
- `for pattern in iterable` desugars to the `Iterator` interface; the loop
  variable is a full `pattern` (so `for (k, v) in pairs` destructures).
- The `else` on a loop is the **completion value** — it runs (and supplies the
  loop-expression's value) when the loop ends *without* `break`. Not Python's
  "else": it is what makes a non-breaking loop usable as a value.
- Loop labels (labeled `break`/`continue`) are deferred post-v1.

## 9. Modules, imports, FFI, attributes

```ebnf
declaration   = attribute* declBody ;
declBody      = importDecl | moduleDecl | funcDecl | measureDecl
              | typeDecl | topLevelConst | externBlock ;

importDecl    = 'import' path '::' '{' importItem ( ',' importItem )* '}' ';'
              | 'import' path ( 'as' IDENTIFIER )? ';' ;     // no glob imports
importItem    = IDENTIFIER ( 'as' IDENTIFIER )? ;

moduleDecl    = visibility? 'module' IDENTIFIER '{' declaration* '}' ;

topLevelConst = visibility? 'fix' IDENTIFIER ':' typeExpr whereClause* '=' expression ';' ;

externBlock   = 'extern' '{' externItem* '}' ;     // C ABI (the only ABI in v1; no ABI string)
externItem    = 'fun' IDENTIFIER '(' paramList? ')' ( '->' typeExpr )? ';'
              | 'let' IDENTIFIER ':' typeExpr ';' ;          // extern global

attribute     = '@' IDENTIFIER ( '(' argList? ')' )? ;       // e.g. @derive(Eq, Ord, Hash)
```

## 10. Lexical grammar

```ebnf
IDENTIFIER      = [A-Za-z_] [A-Za-z0-9_]* ;   // ASCII (v1); Capitalized ⇒ type/variant by convention

INTEGER_LITERAL = ( DEC | HEX | OCT | BIN ) INT_SUFFIX? ;
DEC             = [0-9] [0-9_]* ;
HEX             = '0' [xX] [0-9a-fA-F] [0-9a-fA-F_]* ;
OCT             = '0' [oO] [0-7] [0-7_]* ;
BIN             = '0' [bB] [01] [01_]* ;
INT_SUFFIX      = ( 'i' | 'u' ) ( '8' | '16' | '32' | '64' ) ;     // 255u8, 1_000i64

FLOAT_LITERAL   = DEC '.' DEC EXPONENT? FLOAT_SUFFIX?
                | DEC EXPONENT FLOAT_SUFFIX? ;
EXPONENT        = [eE] [+\-]? DEC ;
FLOAT_SUFFIX    = 'f' ( '16' | '32' | '64' ) ;                     // 3.0f32

BOOLEAN_LITERAL = 'true' | 'false' ;
STRING_LITERAL  = '"' ( ESCAPE | ~["\\] )* '"' ;                   // UTF-8
CHAR_LITERAL    = "'" ( ESCAPE | ~['\\] ) "'" ;                    // Unicode scalar → Char
ESCAPE          = '\\' ( ['"\\nrt0] | 'u' '{' HEX_DIGIT+ '}' ) ;
HEX_DIGIT       = [0-9a-fA-F] ;

LINE_COMMENT    = '//'  ~[\n]*  -> skip ;
DOC_COMMENT     = '///' ~[\n]*  -> attach to next declaration ;     // lexed before LINE_COMMENT
BLOCK_COMMENT   = '/*' ( BLOCK_COMMENT | . )*? '*/'  -> skip ;      // nestable
WHITESPACE      = [ \t\r\n]+    -> skip ;
```

Reserved keywords:
```
fix let fun measure pure static public internal struct class enum interface impl
type init self Self if else while for in match break continue return where
dynamic as import module extern unchecked true false void it
I8 I16 I32 I64 U8 U16 U32 U64 F16 F32 F64 Bool Void Char
```
(Raw/multiline strings and Unicode identifiers are deferred post-v1.)
