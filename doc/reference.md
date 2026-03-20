## Basic Types

```tat
// booleans
bool, bN

// integers
iN, int, intptr
uN, uint, uintptr

// endian specific integers
iNle, uNle  // little endian
iNbe, uNbe  // big endian

// floating point numbers
f32, f64

// endian specific floating point numbers
f32le, f64le  // little endian
f32be, f64be  // big endian

// complex numbers
complex32, complex64

// quaternion numbers
quat32, quat64

// dual quaternion numbers
dquat32, dquat64

// unicode code point
char

// strings
string cstring

// raw pointer type
rawptr
```

## Pointers
## Slices
## Arrays
## Vectors
## Matrices

## struct {#sec-struct}
## enum
## union
## opaque
## namespace

## Operators

### Precedence

```
x!() x() x[] x.y x->y x.& x.* x.! x.?
x...
x{}
!x -x -%x ~x ?x ...x
* / % *% *| ||
+ - ++ +% -% +| -|
<< >> <<|
& ^ | or_else catch
== != < > <= >= <=>
and
or
x..y x..=y x.. ..y ..=y 
= *= *%= *|= /= %= += +%= +|= -= -%= -|= <<= <<|= >>= &= ^= |=
```

## Traits

TODO

### Builtin Traits

TODO

#### Callable trait

TODO

#### Pointer trait

TODO

#### Variadic Tuple

TODO

```tat
pub sum :: fn(args: (...)) -> int {
    val := int(0);
    for inline (args) |arg| val += arg;
    return val;
}

pub main :: fn() void {
    #dbgPrint(sum((1, 2.0, 3)));
    #dbgPrint(sum((1, 2.0, 3, 4.0, 5)));
}
```

#### Variadic Homogenous Tuple

TODO

```tat
pub sum :: fn(args: int...) -> int {
    val := int(0);
    for inline (args) |arg| val += arg;
    return val;
}

pub main :: fn() void {
    #dbgPrint(sum((1, 2, 3)));
    #dbgPrint(sum((1, 2, 3, 4, 5)));
}
```

### User-definable Traits

TODO

## Assembly

```tat
pub main :: fn() no_return {
    msg :: "hello world\n";
    _ = syscall3(SYS_write, STDOUT_FILENO, uint(msg), msg.len);
    _ = syscall1(SYS_exit, 0);
    unreachable;
}

pub SYS_write :: 1;
pub SYS_exit :: 60;

pub STDOUT_FILENO :: 1;

pub syscall1 :: fn(number: uint, arg1: uint) -> uint {
    return asm[number, arg1](
        number: "{rax}",
        arg1: "{rdi}",
        .{ .rcx = true, .r11 = true }
    ) volatile -> (_: uint "={rax}") { 
        "syscall" 
    };
}

pub syscall3 :: fn(number: uint, arg1: uint, arg2: uint, arg3: uint) -> uint {
    return asm[number, arg1, arg2, arg3](
        number: "{rax}",
        arg1: "{rdi}",
        arg2: "{rsi}",
        arg3: "{rdx}",
        .{ .rcx = true, .r11 = true }
    ) volatile -> (_: uint "={rax}") { 
        "syscall" 
    };
}
```

## Attributes

### `#![struct]`, `#![struct const]`, `#![struct($expr)]`, `#![struct($expr) const]`

May be applied to the file root, in which case the file is parsed as a struct instead of a namespace.
See [struct](#sec-struct) for more information regarding struct layouts and `const` structs.

```tat
// Vec2.tat

#![struct]

a: f32, 
b: f32,

Vec2 :: Self;

pub translate :: fn(self, other: Vec2) -> Vec2 {
    return .{.a = self.a + other.a, .b = self.b + other.b};
}
```

## Annotations

## Builtin Functions

Builtin functions are provided by the compiler and are prefixed with `#`. Unlike normal functions, they are considered immediate-functions, i.e. they can not be coerced to `fnptr` values.

### `#divExact`
### `#divFloor`
### `#divTrunc`
### `#hasDecl`
### `#inConst`
### `#infoOf`
### `#intFromBool`
### `#intFromEnum`
### `#intFromFloat`
### `#intFromPtr`
### `#isCallable`
### `#mod`
### `#rem`
### `#shlExact`
### `#shrExact`
### `#typeOf`
### `#typeOfCtx`

## Grammar

```
Root <- skip InnerAttributeRoot* BlockContents eof

BlockContents <- InnerAttribute* Statement*

Block <- LBRACE BlockContents RBRACE

*** Statements ***

Statement 
    <- KEYWORD_const BlockExpr
     / KEYWORD_run BlockExpr
     / KEYWORD_defer BlockExprStatement
     / KEYWORD_cont_defer BlockExprStatement
     / KEYWORD_err_defer BlockExprStatement
     / IfStatement
     / LabeledStatement
     / DeclStatement
     / ExprStatement

BlockExprStatement
    <- BlockExpr
     / ExprStatement

BlockExpr <- BlockLabel? Block

IfStatement <- TODO

LabeledStatement <- BlockLabel? (Block / LoopStatement / SwitchExpr)

LoopStatement <- ForStatement / WhileStatement

ForStatement <- TODO

WhileStatement <- TODO

DeclStatement
    <- OuterDocComment* (ConstDeclStatement / GlobalDeclStatement) SEMICOLON

ConstDeclStatement 
    <- DeclProto (COMMA DeclProto)* (COLON TypeExpr? COLON / COLON2) Expr (COMMA Expr)*

GlobalDeclStatement
    <- (KEYWORD_thread_local / KEYWORD_static)? LocalDeclStatement

LocalDeclStatement
    <- DeclProto (COMMA DeclProto)* COLON TypeExpr? EQUAL Expr (COMMA Expr)*

DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAnnotation*

ExprStatement <- AssignExpr SEMICOLON

# *** Attributes ***

InnerAttributeRoot
    <- InnerAttribute
     / POUNDEXCLAMATIONMARK LBRACKET RootAttribute RBRACKET
     
InnerAttribute <- InnerDocComment / InnerAnnotation

RootAttribute <- KEYWORD_struct (LPAREN Expr RPAREN)? KEYWORD_const?

InnerAnnotation <- POUNDEXCLAMATIONMARK LBRACKET EQUAL Expr RBRACKET

OuterAnnotation <- POUND LBRACKET EQUAL Expr RBRACKET

# *** Expressions ***

AssignExpr <- Expr (AssignOp Expr / (COMMA Expr)+ EQUAL Expr (COMMA Expr)*)?

Expr <- RangeExpr

RangeExpr <- BoolOrExpr (DOT2 BoolOrExpr? / DOT2EQUAL BoolOrExpr) / (DOT2 / DOT2EQUAL) BoolOrExpr

BoolOrExpr <- BoolAndExpr (KEYWORD_or BoolAndExpr)*

BoolAndExpr <- CompareExpr (KEYWORD_and CompareExpr)*

CompareExpr <- BitwiseExpr (CompareOp BitwiseExpr)?

BitwiseExpr <- BitShiftExpr (BitwiseOp BitShiftExpr)*

BitShiftExpr <- AdditionExpr (BitShiftOp AdditionExpr)*

AdditionExpr <- MultiplyExpr (AdditionOp MultiplyExpr)*

MultiplyExpr <- PrefixExpr (MultiplyOp PrefixExpr)*

PrefixExpr <- PrefixOp* PrimaryExpr

PrimaryExpr
    <- AsmExpr
     / IfExpr
     / KEYWORD_break BreakLabel? Expr?
     / KEYWORD_const Expr
     / KEYWORD_continue BreakLabel? Expr?
     / KEYWORD_return Expr?
     / KEYWORD_throw Expr?
     / BlockLabel? LoopExpr / TryExpr
     / FnExpr
     / Block
     / CurlySuffixExpr

AsmExpr <- KEYWORD_asm AsmCaptures? LPAREN (AsmInput COMMA)? Expr COMMA? RPAREN KEYWORD_volatile? AsmOutput? AsmBlock

IfExpr <- TODO

LoopExpr <- ForExpr / WhileExpr

ForExpr <- ForPrefix Expr (KEYWORD_else Expr)?

WhileExpr <- WhilePrefix Expr (KEYWORD_else Payload? Expr)?

TryExpr <- KEYWORD_try Block

FnExpr <- KEYWORD_fn FnCaptures? FnArgs FnModifier? FnCallConv? FnReturn? FnWhere? Block

CurlySuffixExpr <- TypeExpr InitList?

# *** Type Expressions ***

TypeExpr <- ErrorUnionExpr

ErrorUnionExpr <- PackTypeExpr (EXCLAMATIONMARK PackTypeExpr)?

PackTypeExpr <- SingleTypeExpr DOT3?

SingleTypeExpr <- PrefixTypeOp* PrimaryTypeExpr (SuffixOp / MacroCallArguments / FnCallArguments)*

PrimaryTypeExpr 
    <- DOT IDENTIFIER
     / DOT InitList
     / IfTypeExpr
     / LabeledTypeExpr
     / GroupedTypeExpr
     / IDENTIFIER
     / CORE_IDENTIFIER
     / BUILTIN_IDENTIFIER
     / KEYWORD_const TypeExpr
     / KEYWORD_where TypeExpr
     / KEYWORD_Self
     / KEYWORD_self
     / KEYWORD_unreachable
     / CHAR_LITERAL
     / FLOAT_LITERAL
     / INT_LITERAL
     / STRING_LITERAL
     / RAW_STRING_LITERAL
     / ContainerTypeExpr
     / FnptrTypeExpr

IfTypeExpr <- TODO

LabeledTypeExpr 
    <- BlockLabel Block
     / BlockLabel? LoopTypeExpr
     / BlockLabel? SwitchExpr

LoopTypeExpr <- ForTypeExpr / WhileTypeExpr

ForTypeExpr <- ForPrefix TypeExpr (KEYWORD_else TypeExpr)?

WhileTypeExpr <- WhilePrefix TypeExpr (KEYWORD_else Payload? TypeExpr)?

SwitchExpr <- TODO

GroupedTypeExpr <- TODO

ContainerTypeExpr <- ContainerKind (LPAREN Expr RPAREN)? KEYWORD_const? Block

ContainerKind <- KEYWORD_enum / KEYWORD_namespace / KEYWORD_opaque / KEYWORD_primitive / KEYWORD_struct / KEYWORD_union

FnptrTypeExpr <- TODO

# *** Operators ***

AssignOp
    <- ASTERISKEQUAL
     / ASTERISKPIPEEQUAL
     / SLASHEQUAL
     / PERCENTEQUAL
     / PLUSEQUAL
     / PLUSPIPEEQUAL
     / MINUSEQUAL
     / MINUSPIPEEQUAL
     / LARROW2EQUAL
     / LARROW2PIPEEQUAL
     / RARROW2EQUAL
     / AMPERSANDEQUAL
     / CARETEQUAL
     / PIPEEQUAL
     / ASTERISKPERCENTEQUAL
     / PLUSPERCENTEQUAL
     / MINUSPERCENTEQUAL
     / EQUAL

CompareOp
    <- EQUALEQUAL
     / EXCLAMATIONMARKEQUAL
     / LARROW
     / RARROW
     / LARROWEQUAL
     / RARROWEQUAL
     / LARROWEQUALRARROW

BitwiseOp
    <- AMPERSAND
     / CARET
     / PIPE
     / KEYWORD_or_else
     / KEYWORD_catch Payload?

BitShiftOp
    <- LARROW2
     / RARROW2
     / LARROW2PIPE

AdditionOp
    <- PLUS
     / MINUS
     / PLUS2
     / PLUSPERCENT
     / MINUSPERCENT
     / PLUSPIPE
     / MINUSPIPE

MultiplyOp
    <- ASTERISK
     / SLASH
     / PERCENT
     / ASTERISKPERCENT
     / ASTERISKPIPE

PrefixOp
    <- EXCLAMATIONMARK
     / MINUS
     / TILDE
     / MINUSPERCENT
     / DOT3

PrefixTypeOp
    <- QUESTIONMARK
     / SliceTypeStart
     / PtrTypeStart (ByteAlign / QUESTIONMARK? KEYWORD_var / QUESTIONMARK? KEYWORD_volatile)*
     / VectorTypeStart
     / MatrixTypeStart
     / ArrayTypeStart

SuffixOp
    <- LBRACKET Expr RBRACKET
     / DOT IDENTIFIER
     / MINUSARROW IDENTIFIER
     / DOTAMPERSAND
     / DOTASTERISK
     / DOTEXCLAMATIONMARK
     / DOTQUESTIONMARK

MacroCallArguments
    <- EXCLAMATIONMARK LPAREN TokenSequence RPAREN
     / EXCLAMATIONMARK LBRACKET TokenSequence RBRACKET
     / EXCLAMATIONMARK LBRACE TokenSequence RBRACE

FnCallArguments <- LPAREN ExprList RPAREN

# *** Assembly ***

AsmCaptures <- LBRACKET (AsmCaptureItem COMMA)* AsmCaptureItem? COMMA? RBRACKET

AsmCaptureItem <- IDENTIFIER (COLON TypeExpr? EQUAL Expr)?

AsmInput <- (AsmInputItem COMMA)* AsmInputItem?

AsmInputItem <- IDENTIFIER COLON STRING_LITERAL

AsmOutput <- MINUSARROW LPAREN (AsmOutputItem COMMA)* AsmOutputItem? RPAREN

AsmOutputItem <- IDENTIFIER COLON TypeExpr STRING_LITERAL

AsmBlock <- LBRACE Expr RBRACE

# *** Functions ***

FnCaptures 
    <- LBRACKET FnCapturesContext (COMMA (FnCaptureItem COMMA)* (FnCaptureItem COMMA?)?)? RBRACKET
     / LBRACKET (FnCaptureItem COMMA)* (FnCaptureItem COMMA?)? RBRACKET

FnCapturesContext <- QUESTIONMARK KEYWORD_context / KEYWORD_context (COLON TypeExpr)?

FnCaptureItem <- IDENTIFIER (COLON TypeExpr? EQUAL Expr)?

FnArgs 
    <- LPAREN FnReceiver (COMMA (FnArgsItem COMMA)* (FnArgsItem COMMA?)?)? RPAREN
     / LPAREN (FnArgsItem COMMA)* (FnArgsItem COMMA?) RPAREN

FnArgModifier <- KEYWORD_const / KEYWORD_no_alias

FnReceiver <- FnArgModifier? ((COMMA? KEYWORD_var / COMMA? KEYWORD_volatile)* ASTERISK)? KEYWORD_self

FnArgsItem <- FnArgModifier? IDENTIFIER COLON TypeExpr (EQUAL Expr)?

FnModifier <- KEYWORD_const / KEYWORD_inline / KEYWORD_no_inline

FnCallConv <- KEYWORD_callconv LPAREN Expr RPAREN

FnReturn <- MINUSARROW TypeExpr

FnWhere <- KEYWORD_where LPAREN Expr RPAREN

# *** Pointers ***

ArrayTypeStart <- LBRACKET Expr (COLON Expr)? RBRACKET

VectorTypeStart <- LBRACKET LBRACKET Expr RBRACKET RBRACKET

MatrixTypeStart <- LBRACKET LBRACKET Expr COMMA Expr RBRACKET (COLON Expr)? RBRACKET

PtrTypeStart 
    <- ASTERISK
     / LBRACKET ASTERISK (COLON Expr)? RBRACKET

SliceTypeStart <- LBRACKET (COLON Expr)? RBRACKET

# *** Control flow prefixes ***

IfPrefix <- KEYWORD_if KEYWORD_const? LPAREN Expr RPAREN PtrPayload

ForPrefix <- KEYWORD_for LPAREN ExprList RPAREN PtrListPayload KEYWORD_inline?

WhilePrefix <- KEYWORD_while LPAREN Expr RPAREN PtrPayload? KEYWORD_inline?

# *** Payloads ***

Payload <- PIPE IDENTIFIER PIPE

PtrPayload <- PIPE ASTERISK? IDENTIFIER PIPE

PtrIndexPayload <- PIPE ASTERISK? IDENTIFIER (COMMA IDENTIFIER)? PIPE

PtrListPayload <- PIPE ASTERISK? IDENTIFIER (COMMA ASTERISK? IDENTIFIER)* COMMA? PIPE

# *** Token sequences ***

TokenSequence <- TODO

# *** Helper Grammar ***

BreakLabel <- COLON IDENTIFIER

BlockLabel <- IDENTIFIER COLON

FieldInit <- IDENTIFIER COLON EQUAL Expr

InitList
    <- LBRACE FieldInit (COMMA FieldInit)* COMMA? RBRACE
     / LBRACE Expr (COMMA Expr)* COMMA? RBRACE
     / LBRACE RBRACE

ByteAlign <- KEYWORD_align LPAREN Expr RPAREN

# *** Lists ***

ExprList <- (Expr COMMA)* Expr?

# *** Tokens ***

eof <- !.
bin <- [01]
bin_ <- '_'* bin
oct <- [0-7]
oct_ <- '_'* oct
hex <- [0-9a-fA-F]
hex_ <- '_'* hex
dec <- [0-9]
dec_ <- '_'* dec

bin_int <- bin bin_*
oct_int <- oct oct_*
dec_int <- dec dec_*
hex_int <- hex hex_*

ox80_oxBF <- [\200-\277]
oxF4 <- '\364'
ox80_ox8F <- [\200-\217]
oxF1_oxF3 <- [\361-\363]
oxF0 <- '\360'
ox90_0xBF <- [\220-\277]
oxEE_oxEF <- [\356-\357]
oxED <- '\355'
ox80_ox9F <- [\200-\237]
oxE1_oxEC <- [\341-\354]
oxE0 <- '\340'
oxA0_oxBF <- [\240-\277]
oxC2_oxDF <- [\302-\337]

# From https://lemire.me/blog/2018/05/09/how-quickly-can-you-check-that-a-string-is-valid-unicode-utf-8/
# First Byte      Second Byte     Third Byte      Fourth Byte
# [0x00,0x7F]
# [0xC2,0xDF]     [0x80,0xBF]
#    0xE0         [0xA0,0xBF]     [0x80,0xBF]
# [0xE1,0xEC]     [0x80,0xBF]     [0x80,0xBF]
#    0xED         [0x80,0x9F]     [0x80,0xBF]
# [0xEE,0xEF]     [0x80,0xBF]     [0x80,0xBF]
#    0xF0         [0x90,0xBF]     [0x80,0xBF]     [0x80,0xBF]
# [0xF1,0xF3]     [0x80,0xBF]     [0x80,0xBF]     [0x80,0xBF]
#    0xF4         [0x80,0x8F]     [0x80,0xBF]     [0x80,0xBF]

multibyte_utf8 <-
       oxF4      ox80_ox8F ox80_oxBF ox80_oxBF
     / oxF1_oxF3 ox80_oxBF ox80_oxBF ox80_oxBF
     / oxF0      ox90_0xBF ox80_oxBF ox80_oxBF
     / oxEE_oxEF ox80_oxBF ox80_oxBF
     / oxED      ox80_ox9F ox80_oxBF
     / oxE1_oxEC ox80_oxBF ox80_oxBF
     / oxE0      oxA0_oxBF ox80_oxBF
     / oxC2_oxDF ox80_oxBF

non_control_ascii <- [\040-\176]

char_escape
    <- "\\x" hex hex
     / "\\u{" hex+ "}"
     / "\\" [nr\\t'"]
char_char
    <- multibyte_utf8
     / char_escape
     / ![\\'\n] non_control_ascii

string_char
    <- multibyte_utf8
     / char_escape
     / ![\\"\n] non_control_ascii

InnerDocComment <- ('//!' [^\n]* [ \n]* skip)+
OuterDocComment <- ('///' [^\n]* [ \n]* skip)+
line_comment <- '//' ![!/][^\n]*
line_string <- ('\\\\' [^\n]* [ \n]*)+
skip <- ([ \n] / line_comment)*

CHAR_LITERAL <- ['] char_char ['] skip
FLOAT_LITERAL 
    <- '0x' hex_int '.' hex_int ([pP] [-+]? dec_int)? skip
     /      dec_int '.' dec_int ([eE] [-+]? dec_int)? skip
     / '0x' hex_int [pP] [-+]? dec_int skip
     /      dec_int [eE] [-+]? dec_int skip
INTEGER_LITERAL
    <- '0b' bin_int skip
     / '0o' oct_int skip
     / '0x' hex_int skip
     /      dec_int skip
RAW_STRING_LITERAL <- (line_string     skip)+
STRING_LITERAL <- ["] string_char* ["] skip

IDENTIFIER
    <- !Keyword [A-Za-z_] [A-Za-z0-9_]* skip
     / POUND STRING_LITERAL
BUILTIN_IDENTIFIER <- '@' !Keyword [A-Za-z_][A-Za-z0-9_]* skip
CORE_IDENTIFIER <- !Keyword POUND [A-Za-z_][A-Za-z0-9_]* skip

AMPERSAND                <- '&'      ![=!]     skip
AMPERSANDEQUAL           <- '&='               skip
ASTERISK                 <- '*'      ![%=|]    skip
ASTERISKEQUAL            <- '*='               skip
ASTERISKPERCENT          <- '*%'     ![=]      skip
ASTERISKPERCENTEQUAL     <- '*%='              skip
ASTERISKPIPE             <- '*|'     ![=]      skip
ASTERISKPIPEEQUAL        <- '*|='              skip
CARET                    <- '^'      ![=]      skip
CARETEQUAL               <- '^='               skip
COLON                    <- ':'      ![:]      skip
COLON2                   <- '::'               skip
COMMA                    <- ','                skip
DOT                      <- '.'      ![*.&?!]  skip
DOT2                     <- '..'     ![.=]     skip
DOT2EQUAL                <- '..='              skip
DOT3                     <- '...'              skip
DOTAMPERSAND             <- '.&'               skip
DOTASTERISK              <- '.*'               skip
DOTEXCLAMATIONMARK       <- '.!'               skip
DOTQUESTIONMARK          <- '.?'               skip
EQUAL                    <- '='      ![>=]     skip
EQUALEQUAL               <- '=='               skip
EQUALRARROW              <- '=>'               skip
EXCLAMATIONMARK          <- '!'      ![=]      skip
EXCLAMATIONMARKEQUAL     <- '!='               skip
LARROW                   <- '<'      ![<=]     skip
LARROW2                  <- '<<'     ![=|]     skip
LARROW2EQUAL             <- '<<='              skip
LARROW2PIPE              <- '<<|'    ![=]      skip
LARROW2PIPEEQUAL         <- '<<|='             skip
LARROWEQUAL              <- '<='     ![>]      skip
LARROWEQUALRARROW        <- '<=>'               skip
LBRACE                   <- '{'                skip
LBRACKET                 <- '['                skip
LPAREN                   <- '('                skip
MINUS                    <- '-'      ![%=>|]   skip
MINUSEQUAL               <- '-='               skip
MINUSPERCENT             <- '-%'     ![=]      skip
MINUSPERCENTEQUAL        <- '-%='              skip
MINUSPIPE                <- '-|'     ![=]      skip
MINUSPIPEEQUAL           <- '-|='              skip
MINUSARROW               <- '->'               skip
PERCENT                  <- '%'      ![=]      skip
PERCENTEQUAL             <- '%='               skip
PIPE                     <- '|'      ![|=]     skip
PIPEEQUAL                <- '|='               skip
PLUS                     <- '+'      ![%+=|]   skip
PLUS2                    <- '++'               skip
PLUSEQUAL                <- '+='               skip
PLUSPERCENT              <- '+%'     ![=]      skip
PLUSPERCENTEQUAL         <- '+%='              skip
PLUSPIPE                 <- '+|'     ![=]      skip
PLUSPIPEEQUAL            <- '+|='              skip
QUESTIONMARK             <- '?'                skip
RARROW                   <- '>'      ![>=]     skip
RARROW2                  <- '>>'     ![=]      skip
RARROW2EQUAL             <- '>>='              skip
RARROWEQUAL              <- '>='               skip
RBRACE                   <- '}'                skip
RBRACKET                 <- ']'                skip
RPAREN                   <- ')'                skip
SEMICOLON                <- ';'                skip
SLASH                    <- '/'      ![=]      skip
SLASHEQUAL               <- '/='               skip
TILDE                    <- '~'                skip

end_of_word <- ![a-zA-Z0-9_] skip
KEYWORD_align         <-       'align'         end_of_word
KEYWORD_and           <-       'and'           end_of_word
KEYWORD_asm           <-       'asm'           end_of_word
KEYWORD_break         <-       'break'         end_of_word
KEYWORD_callconv      <-       'callconv'      end_of_word
KEYWORD_catch         <-       'catch'         end_of_word
KEYWORD_const         <-       'const'         end_of_word
KEYWORD_context       <-       'context'       end_of_word
KEYWORD_con_defer     <-       'con_defer'     end_of_word
KEYWORD_continue      <-       'continue'      end_of_word
KEYWORD_defer         <-       'defer'         end_of_word
KEYWORD_else          <-       'else'          end_of_word
KEYWORD_enum          <-       'enum'          end_of_word
KEYWORD_err_defer     <-       'err_defer'     end_of_word
KEYWORD_fn            <-       'fn'            end_of_word
KEYWORD_fnptr         <-       'fnptr'         end_of_word
KEYWORD_for           <-       'for'           end_of_word
KEYWORD_if            <-       'if'            end_of_word
KEYWORD_inline        <-       'inline'        end_of_word
KEYWORD_no_alias      <-       'no_alias'      end_of_word
KEYWORD_no_inline     <-       'no_inline'     end_of_word
KEYWORD_opaque        <-       'opaque'        end_of_word
KEYWORD_or            <-       'or'            end_of_word
KEYWORD_or_else       <-       'or_else'       end_of_word
KEYWORD_primitive     <- POUND 'primitive'     end_of_word
KEYWORD_pub           <-       'pub'           end_of_word
KEYWORD_return        <-       'return'        end_of_word
KEYWORD_Self          <-       'Self'          end_of_word
KEYWORD_self          <-       'self'          end_of_word
KEYWORD_static        <-       'static'        end_of_word
KEYWORD_struct        <-       'struct'        end_of_word
KEYWORD_switch        <-       'switch'        end_of_word
KEYWORD_thread_local  <-       'thread_local'  end_of_word
KEYWORD_throw         <-       'throw'         end_of_word
KEYWORD_try           <-       'try'           end_of_word
KEYWORD_union         <-       'union'         end_of_word
KEYWORD_unreachable   <-       'unreachable'   end_of_word
KEYWORD_var           <-       'var'           end_of_word
KEYWORD_volatile      <-       'volatile'      end_of_word
KEYWORD_where         <-       'where'         end_of_word
KEYWORD_while         <-       'while'         end_of_word

Keyword 
    <- KEYWORD_align / KEYWORD_and / KEYWORD_asm
     / KEYWORD_break / KEYWORD_callconv / KEYWORD_catch
     / KEYWORD_const / KEYWORD_context / KEYWORD_con_defer
     / KEYWORD_continue / KEYWORD_defer / KEYWORD_else
     / KEYWORD_enum / KEYWORD_err_defer / KEYWORD_fn
     / KEYWORD_fnptr / KEYWORD_for / KEYWORD_if
     / KEYWORD_inline / KEYWORD_noalias / KEYWORD_no_inline
     / KEYWORD_opaque / KEYWORD_or / KEYWORD_or_else
     / KEYWORD_primitive / KEYWORD_pub / KEYWORD_return
     / KEYWORD_Self / KEYWORD_self / KEYWORD_static
     / KEYWORD_struct / KEYWORD_switch / KEYWORD_thread_local
     / KEYWORD_throw / KEYWORD_try / KEYWORD_union
     / KEYWORD_unreachable / KEYWORD_var / KEYWORD_volatile
     / KEYWORD_where / KEYWORD_while
```
