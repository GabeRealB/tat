use std::fmt::Display;
use std::marker::ConstParamTy;
use std::ops::Range;
use std::sync::Arc;
use std::{collections::BTreeMap, sync::OnceLock};

#[cfg(test)]
use std::ffi::{CStr, CString};

#[derive(Debug, Clone)]
pub struct Span {
    buffer: Arc<[u8]>,
    range: Range<usize>,
}

impl Span {
    pub fn buffer(&self) -> &[u8] {
        &self.buffer[self.byte_range()]
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn combine_range(&self, other: &Self) -> Self {
        assert!(self.buffer.as_ptr() == other.buffer.as_ptr());
        let start = self.range.start.min(other.range.start);
        let end = self.range.end.max(other.range.end);
        Self {
            buffer: self.buffer.clone(),
            range: start..end,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.buffer()))
    }
}

#[derive(ConstParamTy, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tag {
    Invalid,
    EndOfFile,
    Ident,
    IdentCore,
    IdentBuiltin,
    LitChar,
    LitString,
    LitRawString,
    LitInt,
    LitFloat,
    InnerDocComment,
    OuterDocComment,
    SymAmpersand,            // &
    SymAmpersandEqual,       // &=
    SymAsterisk,             // *
    SymAsteriskEqual,        // *=
    SymAsteriskPercent,      // *%
    SymAsteriskPercentEqual, // *%=
    SymAsteriskPipe,         // *|
    SymAsteriskPipeEqual,    // *|=
    SymCaret,                // ^
    SymCaretEqual,           // ^=
    SymColon,                // :
    SymColon2,               // ::
    SymComma,                // ,
    SymDot,                  // .
    SymDot2,                 // ..
    SymDot2Equal,            // ..=
    SymDot3,                 // ...
    SymDotAmpersand,         // .&
    SymDotAsterisk,          // .*
    SymDotExclamationmark,   // .!
    SymDotQuestionmark,      // .?
    SymEqual,                // =
    SymEqualEqual,           // ==
    SymEqualArrow,           // =>
    SymExclamationmark,      // !
    SymExclamationmarkEqual, // !=
    SymLArrow,               // <
    SymLArrow2,              // <<
    SymLArrow2Equal,         // <<=
    SymLArrow2Pipe,          // <<|
    SymLArrow2PipeEqual,     // <<|=
    SymLArrowEqual,          // <=
    SymLArrowEqualRArrow,    // <=>
    SymLBrace,               // {
    SymLBracket,             // [
    SymLParen,               // (
    SymMinus,                // -
    SymMinusEqual,           // -=
    SymMinusPercent,         // -%
    SymMinusPercentEqual,    // -%=
    SymMinusPipe,            // -|
    SymMinusPipeEqual,       // -|=
    SymMinusArrow,           // ->
    SymPercent,              // %
    SymPercentEqual,         // %=
    SymPipe,                 // |
    SymPipeEqual,            // |=
    SymPlus,                 // +
    SymPlus2,                // ++
    SymPlusEqual,            // +=
    SymPlusPercent,          // +%
    SymPlusPercentEqual,     // +%=
    SymPlusPipe,             // +|
    SymPlusPipeEqual,        // +|=
    SymPound,                // #
    SymPoundExclamationmark, // #!
    SymQuestionmark,         // ?
    SymRArrow,               // >
    SymRArrow2,              // >>
    SymRArrow2Equal,         // >>=
    SymRArrowEqual,          // >=
    SymRBrace,               // }
    SymRBracket,             // ]
    SymRParen,               // )
    SymSemicolon,            // ;
    SymSlash,                // /
    SymSlashEqual,           // /=
    SymTilde,                // ~
    KwAlign,                 // align
    KwAnd,                   // and
    KwAsm,                   // asm
    KwBreak,                 // break
    KwCallconv,              // callconv
    KwCatch,                 // catch
    KwConst,                 // const
    KwContext,               // context
    KwContDefer,             // cont_defer
    KwContinue,              // continue
    KwDefer,                 // defer
    KwElse,                  // else
    KwEnum,                  // enum
    KwErrDefer,              // err_defer
    KwFn,                    // fn
    KwFnptr,                 // fnptr
    KwFor,                   // for
    KwIf,                    // if
    KwImpl,                  // impl
    KwInline,                // inline
    KwNamespace,             // namespace
    KwNoAlias,               // no_alias
    KwNoInline,              // no_inline
    KwOpaque,                // opaque
    KwOr,                    // or
    KwOrElse,                // or_else
    KwPrimitive,             // #primitive
    KwPub,                   // pub
    KwReturn,                // return
    KwSelf,                  // Self
    KwSelfIdent,             // self
    KwStruct,                // struct
    KwSwitch,                // switch
    KwThreadLocal,           // thread_local
    KwThrow,                 // throw
    KwTry,                   // try
    KwUnion,                 // union
    KwUnreachable,           // unreachable
    KwVar,                   // var
    KwVolatile,              // volatile
    KwWhere,                 // where
    KwWhile,                 // while
    KwWith,                  // with
}

impl Tag {
    #[cfg(test)]
    const LEXEMES: &[(&str, Self)] = &[
        ("&", Self::SymAmpersand),
        ("&=", Self::SymAmpersandEqual),
        ("*", Self::SymAsterisk),
        ("*=", Self::SymAsteriskEqual),
        ("*%", Self::SymAsteriskPercent),
        ("*%=", Self::SymAsteriskPercentEqual),
        ("*|", Self::SymAsteriskPipe),
        ("*|=", Self::SymAsteriskPipeEqual),
        ("^", Self::SymCaret),
        ("^=", Self::SymCaretEqual),
        (":", Self::SymColon),
        ("::", Self::SymColon2),
        (",", Self::SymComma),
        (".", Self::SymDot),
        ("..", Self::SymDot2),
        ("..=", Self::SymDot2Equal),
        ("...", Self::SymDot3),
        (".&", Self::SymDotAmpersand),
        (".*", Self::SymDotAsterisk),
        (".!", Self::SymDotExclamationmark),
        (".?", Self::SymDotQuestionmark),
        ("=", Self::SymEqual),
        ("==", Self::SymEqualEqual),
        ("=>", Self::SymEqualArrow),
        ("!", Self::SymExclamationmark),
        ("!=", Self::SymExclamationmarkEqual),
        ("<", Self::SymLArrow),
        ("<<", Self::SymLArrow2),
        ("<<=", Self::SymLArrow2Equal),
        ("<<|", Self::SymLArrow2Pipe),
        ("<<|=", Self::SymLArrow2PipeEqual),
        ("<=", Self::SymLArrowEqual),
        ("<=>", Self::SymLArrowEqualRArrow),
        ("{", Self::SymLBrace),
        ("[", Self::SymLBracket),
        ("(", Self::SymLParen),
        ("-", Self::SymMinus),
        ("-=", Self::SymMinusEqual),
        ("-%", Self::SymMinusPercent),
        ("-%=", Self::SymMinusPercentEqual),
        ("-|", Self::SymMinusPipe),
        ("-|=", Self::SymMinusPipeEqual),
        ("->", Self::SymMinusArrow),
        ("%", Self::SymPercent),
        ("%=", Self::SymPercentEqual),
        ("|", Self::SymPipe),
        ("|=", Self::SymPipeEqual),
        ("+", Self::SymPlus),
        ("++", Self::SymPlus2),
        ("+=", Self::SymPlusEqual),
        ("+%", Self::SymPlusPercent),
        ("+%=", Self::SymPlusPercentEqual),
        ("+|", Self::SymPlusPipe),
        ("+|=", Self::SymPlusPipeEqual),
        ("#", Self::SymPound),
        ("#!", Self::SymPoundExclamationmark),
        ("?", Self::SymQuestionmark),
        (">", Self::SymRArrow),
        (">>", Self::SymRArrow2),
        (">>=", Self::SymRArrow2Equal),
        (">=", Self::SymRArrowEqual),
        ("}", Self::SymRBrace),
        ("]", Self::SymRBracket),
        (")", Self::SymRParen),
        (";", Self::SymSemicolon),
        ("/", Self::SymSlash),
        ("/=", Self::SymSlashEqual),
        ("~", Self::SymTilde),
        ("align", Self::KwAlign),
        ("and", Self::KwAnd),
        ("asm", Self::KwAsm),
        ("break", Self::KwBreak),
        ("callconv", Self::KwCallconv),
        ("catch", Self::KwCatch),
        ("const", Self::KwConst),
        ("context", Self::KwContext),
        ("cont_defer", Self::KwContDefer),
        ("continue", Self::KwContinue),
        ("defer", Self::KwDefer),
        ("else", Self::KwElse),
        ("enum", Self::KwEnum),
        ("err_defer", Self::KwErrDefer),
        ("fn", Self::KwFn),
        ("fnptr", Self::KwFnptr),
        ("for", Self::KwFor),
        ("if", Self::KwIf),
        ("impl", Self::KwImpl),
        ("inline", Self::KwInline),
        ("namespace", Self::KwNamespace),
        ("no_alias", Self::KwNoAlias),
        ("no_inline", Self::KwNoInline),
        ("opaque", Self::KwOpaque),
        ("or", Self::KwOr),
        ("or_else", Self::KwOrElse),
        ("#primitive", Self::KwPrimitive),
        ("pub", Self::KwPub),
        ("return", Self::KwReturn),
        ("Self", Self::KwSelf),
        ("self", Self::KwSelfIdent),
        ("struct", Self::KwStruct),
        ("switch", Self::KwSwitch),
        ("thread_local", Self::KwThreadLocal),
        ("throw", Self::KwThrow),
        ("try", Self::KwTry),
        ("union", Self::KwUnion),
        ("unreachable", Self::KwUnreachable),
        ("var", Self::KwVar),
        ("volatile", Self::KwVolatile),
        ("where", Self::KwWhere),
        ("while", Self::KwWhile),
        ("with", Self::KwWith),
    ];

    const KEYWORDS: &[(&str, Self)] = &[
        ("align", Self::KwAlign),
        ("and", Self::KwAnd),
        ("asm", Self::KwAsm),
        ("break", Self::KwBreak),
        ("callconv", Self::KwCallconv),
        ("catch", Self::KwCatch),
        ("const", Self::KwConst),
        ("context", Self::KwContext),
        ("cont_defer", Self::KwContDefer),
        ("continue", Self::KwContinue),
        ("defer", Self::KwDefer),
        ("else", Self::KwElse),
        ("enum", Self::KwEnum),
        ("err_defer", Self::KwErrDefer),
        ("fn", Self::KwFn),
        ("fnptr", Self::KwFnptr),
        ("for", Self::KwFor),
        ("if", Self::KwIf),
        ("impl", Self::KwImpl),
        ("inline", Self::KwInline),
        ("namespace", Self::KwNamespace),
        ("no_alias", Self::KwNoAlias),
        ("no_inline", Self::KwNoInline),
        ("opaque", Self::KwOpaque),
        ("or", Self::KwOr),
        ("or_else", Self::KwOrElse),
        ("#primitive", Self::KwPrimitive),
        ("pub", Self::KwPub),
        ("return", Self::KwReturn),
        ("Self", Self::KwSelf),
        ("self", Self::KwSelfIdent),
        ("struct", Self::KwStruct),
        ("switch", Self::KwSwitch),
        ("thread_local", Self::KwThreadLocal),
        ("throw", Self::KwThrow),
        ("try", Self::KwTry),
        ("union", Self::KwUnion),
        ("unreachable", Self::KwUnreachable),
        ("var", Self::KwVar),
        ("volatile", Self::KwVolatile),
        ("where", Self::KwWhere),
        ("while", Self::KwWhile),
        ("with", Self::KwWith),
    ];

    pub fn as_lexeme(self) -> Option<&'static str> {
        match self {
            Tag::Invalid => None,
            Tag::EndOfFile => None,
            Tag::Ident => None,
            Tag::IdentCore => None,
            Tag::IdentBuiltin => None,
            Tag::LitChar => None,
            Tag::LitString => None,
            Tag::LitRawString => None,
            Tag::LitInt => None,
            Tag::LitFloat => None,
            Tag::InnerDocComment => None,
            Tag::OuterDocComment => None,
            Tag::SymAmpersand => Some("&"),
            Tag::SymAmpersandEqual => Some("&="),
            Tag::SymAsterisk => Some("*"),
            Tag::SymAsteriskEqual => Some("*="),
            Tag::SymAsteriskPercent => Some("*%"),
            Tag::SymAsteriskPercentEqual => Some("*%="),
            Tag::SymAsteriskPipe => Some("*|"),
            Tag::SymAsteriskPipeEqual => Some("*|="),
            Tag::SymCaret => Some("^"),
            Tag::SymCaretEqual => Some("^="),
            Tag::SymColon => Some(":"),
            Tag::SymColon2 => Some("::"),
            Tag::SymComma => Some(","),
            Tag::SymDot => Some("."),
            Tag::SymDot2 => Some(".."),
            Tag::SymDot2Equal => Some("..="),
            Tag::SymDot3 => Some("..."),
            Tag::SymDotAmpersand => Some(".&"),
            Tag::SymDotAsterisk => Some(".*"),
            Tag::SymDotExclamationmark => Some(".!"),
            Tag::SymDotQuestionmark => Some(".?"),
            Tag::SymEqual => Some("="),
            Tag::SymEqualEqual => Some("=="),
            Tag::SymEqualArrow => Some("=>"),
            Tag::SymExclamationmark => Some("!"),
            Tag::SymExclamationmarkEqual => Some("!="),
            Tag::SymLArrow => Some("<"),
            Tag::SymLArrow2 => Some("<<"),
            Tag::SymLArrow2Equal => Some("<<="),
            Tag::SymLArrow2Pipe => Some("<<|"),
            Tag::SymLArrow2PipeEqual => Some("<<|="),
            Tag::SymLArrowEqual => Some("<="),
            Tag::SymLArrowEqualRArrow => Some("<=>"),
            Tag::SymLBrace => Some("{"),
            Tag::SymLBracket => Some("["),
            Tag::SymLParen => Some("("),
            Tag::SymMinus => Some("-"),
            Tag::SymMinusEqual => Some("-="),
            Tag::SymMinusPercent => Some("-%"),
            Tag::SymMinusPercentEqual => Some("-%="),
            Tag::SymMinusPipe => Some("-|"),
            Tag::SymMinusPipeEqual => Some("-|="),
            Tag::SymMinusArrow => Some("->"),
            Tag::SymPercent => Some("%"),
            Tag::SymPercentEqual => Some("%="),
            Tag::SymPipe => Some("|"),
            Tag::SymPipeEqual => Some("|="),
            Tag::SymPlus => Some("+"),
            Tag::SymPlus2 => Some("++"),
            Tag::SymPlusEqual => Some("+="),
            Tag::SymPlusPercent => Some("+%"),
            Tag::SymPlusPercentEqual => Some("+%="),
            Tag::SymPlusPipe => Some("+|"),
            Tag::SymPlusPipeEqual => Some("+|="),
            Tag::SymPound => Some("#"),
            Tag::SymPoundExclamationmark => Some("#!"),
            Tag::SymQuestionmark => Some("?"),
            Tag::SymRArrow => Some(">"),
            Tag::SymRArrow2 => Some(">>"),
            Tag::SymRArrow2Equal => Some(">>="),
            Tag::SymRArrowEqual => Some(">="),
            Tag::SymRBrace => Some("}"),
            Tag::SymRBracket => Some("]"),
            Tag::SymRParen => Some(")"),
            Tag::SymSemicolon => Some(";"),
            Tag::SymSlash => Some("/"),
            Tag::SymSlashEqual => Some("/="),
            Tag::SymTilde => Some("~"),
            Tag::KwAlign => Some("align"),
            Tag::KwAnd => Some("and"),
            Tag::KwAsm => Some("asm"),
            Tag::KwBreak => Some("break"),
            Tag::KwCallconv => Some("callconv"),
            Tag::KwCatch => Some("catch"),
            Tag::KwConst => Some("const"),
            Tag::KwContext => Some("context"),
            Tag::KwContDefer => Some("cont_defer"),
            Tag::KwContinue => Some("continue"),
            Tag::KwDefer => Some("defer"),
            Tag::KwElse => Some("else"),
            Tag::KwEnum => Some("enum"),
            Tag::KwErrDefer => Some("err_defer"),
            Tag::KwFn => Some("fn"),
            Tag::KwFnptr => Some("fnptr"),
            Tag::KwFor => Some("for"),
            Tag::KwIf => Some("if"),
            Tag::KwImpl => Some("impl"),
            Tag::KwInline => Some("inline"),
            Tag::KwNamespace => Some("namespace"),
            Tag::KwNoAlias => Some("no_alias"),
            Tag::KwNoInline => Some("no_inline"),
            Tag::KwOpaque => Some("opaque"),
            Tag::KwOr => Some("or"),
            Tag::KwOrElse => Some("or_else"),
            Tag::KwPrimitive => Some("#primitive"),
            Tag::KwPub => Some("pub"),
            Tag::KwReturn => Some("return"),
            Tag::KwSelf => Some("Self"),
            Tag::KwSelfIdent => Some("self"),
            Tag::KwStruct => Some("struct"),
            Tag::KwSwitch => Some("switch"),
            Tag::KwThreadLocal => Some("thread_local"),
            Tag::KwThrow => Some("throw"),
            Tag::KwTry => Some("try"),
            Tag::KwUnion => Some("union"),
            Tag::KwUnreachable => Some("unreachable"),
            Tag::KwVar => Some("var"),
            Tag::KwVolatile => Some("volatile"),
            Tag::KwWhere => Some("where"),
            Tag::KwWhile => Some("while"),
            Tag::KwWith => Some("with"),
        }
    }

    fn as_keyword(ident: &str) -> Option<Self> {
        static KEYWORDS_CELL: OnceLock<BTreeMap<&'static str, Tag>> =
            OnceLock::<BTreeMap<&'static str, Tag>>::new();
        let keywords_map = KEYWORDS_CELL
            .get_or_init(|| Self::KEYWORDS.iter().copied().collect::<BTreeMap<_, _>>());

        if let Some(stripped_ident) = ident.strip_prefix(['@']) {
            keywords_map
                .get(ident)
                .or_else(|| keywords_map.get(stripped_ident))
                .copied()
        } else {
            keywords_map.get(ident).copied()
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Tag::Invalid => "#<invalid>",
            Tag::EndOfFile => "#<eof>",
            Tag::Ident => "#<identifier>",
            Tag::IdentCore => "#<core_identifier>",
            Tag::IdentBuiltin => "#<builtin_identifier>",
            Tag::LitChar => "#<char_literal>",
            Tag::LitString => "#<string_literal>",
            Tag::LitRawString => "#<raw_string_literal>",
            Tag::LitInt => "#<int_literal>",
            Tag::LitFloat => "#<float_literal>",
            Tag::InnerDocComment => "#<inner_doc_comment>",
            Tag::OuterDocComment => "#<outer_doc_comment>",
            _ => self.as_lexeme().unwrap(),
        };

        f.write_str(string)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub span: Span,
    pub tag: Tag,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.span)
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer {
    buffer: Arc<[u8]>,
    index: usize,
}

impl Tokenizer {
    pub fn new(buffer: Arc<[u8]>) -> Self {
        // Skip the UTF-8 BOM if present.
        let index = if buffer.starts_with(b"\xEF\xBB\xBF") {
            3
        } else {
            0
        };
        Self { buffer, index }
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.buffer.len() {
            return None;
        }

        let mut state = State::Start;
        enum State {
            Start,
            Identifier,
            IdentifierCoreStart,
            IdentifierCore,
            IdentifierBuiltin,
            Char,
            CharMb3Expect0x90_0xBF,
            CharMb3Expect0x80_0xBF,
            CharMb3Expect0x80_0x8F,
            CharMb2Expect0xA0_0xBF,
            CharMb2Expect0x80_0x9F,
            CharMb2Expect0x80_0xBF,
            CharMb1Expect0x80_0xBF,
            CharEscape,
            CharEscapeAscii,
            CharEscapeAscii2,
            CharEnd,
            CharInvalid,
            CharInvalidEscape,
            String,
            StringMb3Expect0x90_0xBF,
            StringMb3Expect0x80_0xBF,
            StringMb3Expect0x80_0x8F,
            StringMb2Expect0xA0_0xBF,
            StringMb2Expect0x80_0x9F,
            StringMb2Expect0x80_0xBF,
            StringMb1Expect0x80_0xBF,
            StringEscape,
            StringEscapeAscii,
            StringEscapeAscii2,
            StringInvalid,
            StringInvalidEscape,
            Pound,
            Ampersand,
            Asterisk,
            AsteriskPercent,
            AsteriskPipe,
            Caret,
            Colon,
            Dot,
            Dot2,
            Equal,
            Exclamationmark,
            LArrow,
            LArrow2,
            LArrow2Pipe,
            LArrowEqual,
            Minus,
            MinusPercent,
            MinusPipe,
            Percent,
            Pipe,
            Plus,
            PlusPercent,
            PlusPipe,
            RArrow,
            RArrow2,
            Slash,
            Slash2,
            LineComment,
            LineCommentExpectNewline,
            DocComment,
            BackSlash,
            RawString,
            IntPrefix,
            IntBinStart,
            IntBin,
            IntOctStart,
            IntOct,
            IntHexStart,
            IntHex,
            IntDec,
            FloatHexStart,
            FloatHex,
            FloatDecStart,
            FloatDec,
            FloatExponent,
            FloatExponentNumberStart,
            FloatExponentNumber,
            IntInvalid,
            Invalid,
        }

        let mut token = Token {
            tag: Tag::Invalid,
            span: Span {
                buffer: self.buffer.clone(),
                range: self.index..self.index,
            },
        };
        loop {
            match state {
                State::Start => match self.buffer[self.index] {
                    0 => {
                        if self.index == self.buffer.len() - 1 {
                            self.index += 1;
                            return Some(Token {
                                tag: Tag::EndOfFile,
                                span: Span {
                                    buffer: self.buffer.clone(),
                                    range: self.index - 1..self.index,
                                },
                            });
                        } else {
                            state = State::Invalid;
                            continue;
                        }
                    }
                    b' ' | b'\n' | b'\t' | b'\r' => {
                        self.index += 1;
                        token.span.range.start = self.index;
                        continue;
                    }
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                        token.tag = Tag::Ident;
                        state = State::Identifier;
                        continue;
                    }
                    b'\'' => {
                        token.tag = Tag::LitChar;
                        state = State::Char;
                        continue;
                    }
                    b'"' => {
                        token.tag = Tag::LitString;
                        state = State::String;
                        continue;
                    }
                    b'@' => {
                        token.tag = Tag::IdentCore;
                        state = State::IdentifierCoreStart;
                        continue;
                    }
                    b'#' => {
                        state = State::Pound;
                        continue;
                    }
                    b'&' => {
                        state = State::Ampersand;
                        continue;
                    }
                    b'*' => {
                        state = State::Asterisk;
                        continue;
                    }
                    b'^' => {
                        state = State::Caret;
                        continue;
                    }
                    b':' => {
                        state = State::Colon;
                        continue;
                    }
                    b',' => {
                        token.tag = Tag::SymComma;
                        self.index += 1;
                        break;
                    }
                    b'.' => {
                        state = State::Dot;
                        continue;
                    }
                    b'=' => {
                        state = State::Equal;
                        continue;
                    }
                    b'!' => {
                        state = State::Exclamationmark;
                        continue;
                    }
                    b'<' => {
                        state = State::LArrow;
                        continue;
                    }
                    b'{' => {
                        token.tag = Tag::SymLBrace;
                        self.index += 1;
                        break;
                    }
                    b'[' => {
                        token.tag = Tag::SymLBracket;
                        self.index += 1;
                        break;
                    }
                    b'(' => {
                        token.tag = Tag::SymLParen;
                        self.index += 1;
                        break;
                    }
                    b'-' => {
                        state = State::Minus;
                        continue;
                    }
                    b'%' => {
                        state = State::Percent;
                        continue;
                    }
                    b'|' => {
                        state = State::Pipe;
                        continue;
                    }
                    b'+' => {
                        state = State::Plus;
                        continue;
                    }
                    b'?' => {
                        token.tag = Tag::SymQuestionmark;
                        self.index += 1;
                        break;
                    }
                    b'>' => {
                        state = State::RArrow;
                        continue;
                    }
                    b'}' => {
                        token.tag = Tag::SymRBrace;
                        self.index += 1;
                        break;
                    }
                    b']' => {
                        token.tag = Tag::SymRBracket;
                        self.index += 1;
                        break;
                    }
                    b')' => {
                        token.tag = Tag::SymRParen;
                        self.index += 1;
                        break;
                    }
                    b';' => {
                        token.tag = Tag::SymSemicolon;
                        self.index += 1;
                        break;
                    }
                    b'/' => {
                        state = State::Slash;
                        continue;
                    }
                    b'\\' => {
                        state = State::BackSlash;
                        continue;
                    }
                    b'~' => {
                        token.tag = Tag::SymTilde;
                        self.index += 1;
                        break;
                    }
                    b'0' => {
                        token.tag = Tag::LitInt;
                        state = State::IntPrefix;
                        continue;
                    }
                    b'1'..=b'9' => {
                        token.tag = Tag::LitInt;
                        state = State::IntDec;
                        continue;
                    }
                    _ => {
                        state = State::Invalid;
                        continue;
                    }
                },
                State::Identifier => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => continue,
                        _ => {
                            let ident = &self.buffer[token.span.range.start..self.index];
                            let ident = str::from_utf8(ident).unwrap();
                            if let Some(tag) = Tag::as_keyword(ident) {
                                token.tag = tag;
                            }
                            break;
                        }
                    }
                }
                State::IdentifierCoreStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                            state = State::IdentifierCore;
                            continue;
                        }
                        _ => {
                            let ident = &self.buffer[token.span.range.start..self.index];
                            let ident = str::from_utf8(ident).unwrap();
                            if Tag::as_keyword(ident).is_some() {
                                token.tag = Tag::Invalid;
                            }
                            break;
                        }
                    }
                }
                State::IdentifierCore => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => continue,
                        _ => {
                            let ident = &self.buffer[token.span.range.start..self.index];
                            let ident = str::from_utf8(ident).unwrap();
                            if Tag::as_keyword(ident).is_some() {
                                token.tag = Tag::Invalid;
                            }
                            break;
                        }
                    }
                }
                State::IdentifierBuiltin => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'0'..=b'9' => continue,
                        _ => {
                            let ident = &self.buffer[token.span.range.start..self.index];
                            let ident = str::from_utf8(ident).unwrap();
                            if let Some(tag) = Tag::as_keyword(ident) {
                                token.tag = tag;
                            }
                            break;
                        }
                    }
                }
                State::Char => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 | b'\'' => {
                            state = State::CharInvalid;
                            continue;
                        }
                        b'\\' => {
                            state = State::CharEscape;
                            continue;
                        }
                        b'\x20'..=b'\x7E' => {
                            state = State::CharEnd;
                            continue;
                        }
                        b'\xC2'..=b'\xDF' => {
                            state = State::CharMb1Expect0x80_0xBF;
                            continue;
                        }
                        b'\xE0' => {
                            state = State::CharMb2Expect0xA0_0xBF;
                            continue;
                        }
                        b'\xED' => {
                            state = State::CharMb2Expect0x80_0x9F;
                            continue;
                        }
                        b'\xE1'..=b'\xEC' | b'\xEE'..=b'\xEF' => {
                            state = State::CharMb2Expect0x80_0xBF;
                            continue;
                        }
                        b'\xF0' => {
                            state = State::CharMb3Expect0x90_0xBF;
                            continue;
                        }
                        b'\xF1'..=b'\xF3' => {
                            state = State::CharMb3Expect0x80_0xBF;
                            continue;
                        }
                        b'\xF4' => {
                            state = State::CharMb3Expect0x80_0x8F;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb3Expect0x90_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x90'..=b'\xBF' => {
                            state = State::CharMb2Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb3Expect0x80_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\xBF' => {
                            state = State::CharMb2Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb3Expect0x80_0x8F => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\x8F' => {
                            state = State::CharMb2Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb2Expect0xA0_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\xA0'..=b'\xBF' => {
                            state = State::CharMb1Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb2Expect0x80_0x9F => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\x9F' => {
                            state = State::CharMb1Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb2Expect0x80_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\xBF' => {
                            state = State::CharMb1Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharMb1Expect0x80_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\xBF' => {
                            state = State::CharEnd;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharEscape => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'n' | b'r' | b'\\' | b't' | b'\'' | b'"' => {
                            state = State::CharEnd;
                            continue;
                        }
                        b'x' => {
                            state = State::CharEscapeAscii;
                            continue;
                        }
                        b'u' => {
                            todo!("implement utf8 char escape tokenization")
                        }
                        _ => {
                            state = State::CharInvalidEscape;
                            continue;
                        }
                    }
                }
                State::CharEscapeAscii => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                            state = State::CharEscapeAscii2;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharEscapeAscii2 => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                            state = State::CharEnd;
                            continue;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharEnd => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\'' => {
                            self.index += 1;
                            break;
                        }
                        _ => {
                            state = State::CharInvalid;
                            continue;
                        }
                    }
                }
                State::CharInvalid => match self.buffer[self.index] {
                    0 | b'\n' => {
                        token.tag = Tag::Invalid;
                        break;
                    }
                    b'\'' => {
                        token.tag = Tag::Invalid;
                        self.index += 1;
                        break;
                    }
                    b'\\' => {
                        state = State::CharInvalidEscape;
                        self.index += 1;
                        continue;
                    }
                    _ => {
                        self.index += 1;
                        continue;
                    }
                },
                State::CharInvalidEscape => match self.buffer[self.index] {
                    0 | b'\n' => {
                        token.tag = Tag::Invalid;
                        break;
                    }
                    _ => {
                        state = State::CharInvalid;
                        self.index += 1;
                        continue;
                    }
                },
                State::String => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 => {
                            state = State::StringInvalid;
                            continue;
                        }
                        b'\\' => {
                            state = State::StringEscape;
                            continue;
                        }
                        b'\"' => {
                            self.index += 1;
                            break;
                        }
                        b'\x20'..=b'\x7E' => continue,
                        b'\xC2'..=b'\xDF' => {
                            state = State::StringMb1Expect0x80_0xBF;
                            continue;
                        }
                        b'\xE0' => {
                            state = State::StringMb2Expect0xA0_0xBF;
                            continue;
                        }
                        b'\xED' => {
                            state = State::StringMb2Expect0x80_0x9F;
                            continue;
                        }
                        b'\xE1'..=b'\xEC' | b'\xEE'..=b'\xEF' => {
                            state = State::StringMb2Expect0x80_0xBF;
                            continue;
                        }
                        b'\xF0' => {
                            state = State::StringMb3Expect0x90_0xBF;
                            continue;
                        }
                        b'\xF1'..=b'\xF3' => {
                            state = State::StringMb3Expect0x80_0xBF;
                            continue;
                        }
                        b'\xF4' => {
                            state = State::StringMb3Expect0x80_0x8F;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb3Expect0x90_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x90'..=b'\xBF' => {
                            state = State::StringMb2Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb3Expect0x80_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\xBF' => {
                            state = State::StringMb2Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb3Expect0x80_0x8F => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\x8F' => {
                            state = State::StringMb2Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb2Expect0xA0_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\xA0'..=b'\xBF' => {
                            state = State::StringMb1Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb2Expect0x80_0x9F => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\x9F' => {
                            state = State::StringMb1Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb2Expect0x80_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\xBF' => {
                            state = State::StringMb1Expect0x80_0xBF;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringMb1Expect0x80_0xBF => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\x80'..=b'\xBF' => {
                            state = State::String;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringEscape => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'n' | b'r' | b'\\' | b't' | b'\'' | b'"' => {
                            state = State::String;
                            continue;
                        }
                        b'x' => {
                            state = State::StringEscapeAscii;
                            continue;
                        }
                        b'u' => {
                            todo!("implement utf8 string char escape tokenization")
                        }
                        _ => {
                            state = State::StringInvalidEscape;
                            continue;
                        }
                    }
                }
                State::StringEscapeAscii => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                            state = State::StringEscapeAscii2;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringEscapeAscii2 => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                            state = State::String;
                            continue;
                        }
                        _ => {
                            state = State::StringInvalid;
                            continue;
                        }
                    }
                }
                State::StringInvalid => match self.buffer[self.index] {
                    0 | b'\n' => {
                        token.tag = Tag::Invalid;
                        break;
                    }
                    b'\"' => {
                        token.tag = Tag::Invalid;
                        self.index += 1;
                        break;
                    }
                    b'\\' => {
                        state = State::StringInvalidEscape;
                        self.index += 1;
                        continue;
                    }
                    _ => {
                        self.index += 1;
                        continue;
                    }
                },
                State::StringInvalidEscape => match self.buffer[self.index] {
                    0 | b'\n' => {
                        token.tag = Tag::Invalid;
                        break;
                    }
                    _ => {
                        state = State::StringInvalid;
                        self.index += 1;
                        continue;
                    }
                },
                State::Ampersand => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymAmpersandEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymAmpersand,
                    }
                    break;
                }
                State::Asterisk => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymAsteriskEqual;
                            self.index += 1;
                        }
                        b'%' => {
                            state = State::AsteriskPercent;
                            continue;
                        }
                        b'|' => {
                            state = State::AsteriskPipe;
                            continue;
                        }
                        _ => token.tag = Tag::SymAsterisk,
                    }
                    break;
                }
                State::AsteriskPercent => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymAsteriskPercentEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymAsteriskPercent,
                    }
                    break;
                }
                State::AsteriskPipe => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymAsteriskPipeEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymAsteriskPipe,
                    }
                    break;
                }
                State::Caret => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymCaretEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymCaret,
                    }
                    break;
                }
                State::Colon => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b':' => {
                            token.tag = Tag::SymColon2;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymColon,
                    }
                    break;
                }
                State::Dot => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'.' => {
                            state = State::Dot2;
                            continue;
                        }
                        b'&' => {
                            token.tag = Tag::SymDotAmpersand;
                            self.index += 1;
                        }
                        b'*' => {
                            token.tag = Tag::SymDotAsterisk;
                            self.index += 1;
                        }
                        b'!' => {
                            token.tag = Tag::SymDotExclamationmark;
                            self.index += 1;
                        }
                        b'?' => {
                            token.tag = Tag::SymDotQuestionmark;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymDot,
                    }
                    break;
                }
                State::Dot2 => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymDot2Equal;
                            self.index += 1;
                        }
                        b'.' => {
                            token.tag = Tag::SymDot3;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymDot2,
                    }
                    break;
                }
                State::Equal => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymEqualEqual;
                            self.index += 1;
                        }
                        b'>' => {
                            token.tag = Tag::SymEqualArrow;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymEqual,
                    }
                    break;
                }
                State::Exclamationmark => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymExclamationmarkEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymExclamationmark,
                    }
                    break;
                }
                State::LArrow => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'<' => {
                            state = State::LArrow2;
                            continue;
                        }
                        b'=' => {
                            state = State::LArrowEqual;
                            continue;
                        }
                        _ => token.tag = Tag::SymLArrow,
                    }
                    break;
                }
                State::LArrow2 => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'|' => {
                            state = State::LArrow2Pipe;
                            continue;
                        }
                        b'=' => {
                            token.tag = Tag::SymLArrow2Equal;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymLArrow2,
                    }
                    break;
                }
                State::LArrow2Pipe => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymLArrow2PipeEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymLArrow2Pipe,
                    }
                    break;
                }
                State::LArrowEqual => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'>' => {
                            token.tag = Tag::SymLArrowEqualRArrow;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymLArrowEqual,
                    }
                    break;
                }
                State::Minus => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymMinusEqual;
                            self.index += 1;
                        }
                        b'%' => {
                            state = State::MinusPercent;
                            continue;
                        }
                        b'|' => {
                            state = State::MinusPipe;
                            continue;
                        }
                        b'>' => {
                            token.tag = Tag::SymMinusArrow;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymMinus,
                    }
                    break;
                }
                State::MinusPercent => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymMinusPercentEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymMinusPercent,
                    }
                    break;
                }
                State::MinusPipe => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymMinusPipeEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymMinusPipe,
                    }
                    break;
                }
                State::Percent => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymPercentEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymPercent,
                    }
                    break;
                }
                State::Pipe => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymPipeEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymPipe,
                    }
                    break;
                }
                State::Plus => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'+' => {
                            token.tag = Tag::SymPlus2;
                            self.index += 1;
                        }
                        b'=' => {
                            token.tag = Tag::SymPlusEqual;
                            self.index += 1;
                        }
                        b'%' => {
                            state = State::PlusPercent;
                            continue;
                        }
                        b'|' => {
                            state = State::PlusPipe;
                            continue;
                        }
                        _ => token.tag = Tag::SymPlus,
                    }
                    break;
                }
                State::PlusPercent => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymPlusPercentEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymPlusPercent,
                    }
                    break;
                }
                State::PlusPipe => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymPlusPipeEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymPlusPipe,
                    }
                    break;
                }
                State::Pound => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'!' => {
                            token.tag = Tag::SymPoundExclamationmark;
                            self.index += 1;
                        }
                        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                            token.tag = Tag::IdentBuiltin;
                            state = State::IdentifierBuiltin;
                            continue;
                        }
                        b'\"' => {
                            token.tag = Tag::Ident;
                            state = State::String;
                            continue;
                        }
                        _ => token.tag = Tag::SymPound,
                    }
                    break;
                }
                State::RArrow => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'>' => {
                            state = State::RArrow2;
                            continue;
                        }
                        b'=' => {
                            token.tag = Tag::SymRArrowEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymRArrow,
                    }
                    break;
                }
                State::RArrow2 => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'=' => {
                            token.tag = Tag::SymRArrow2Equal;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymRArrow2,
                    }
                    break;
                }
                State::Slash => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'/' => {
                            state = State::Slash2;
                            continue;
                        }
                        b'=' => {
                            token.tag = Tag::SymSlashEqual;
                            self.index += 1;
                        }
                        _ => token.tag = Tag::SymSlash,
                    }
                    break;
                }
                State::Slash2 => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 => {
                            if self.index != self.buffer.len() - 1 {
                                state = State::Invalid;
                                continue;
                            } else {
                                self.index += 1;
                                return Some(Token {
                                    tag: Tag::EndOfFile,
                                    span: Span {
                                        buffer: self.buffer.clone(),
                                        range: self.index - 1..self.index,
                                    },
                                });
                            }
                        }
                        b'!' => {
                            token.tag = Tag::InnerDocComment;
                            state = State::DocComment;
                            continue;
                        }
                        b'/' => {
                            token.tag = Tag::OuterDocComment;
                            state = State::DocComment;
                            continue;
                        }
                        b'\n' => {
                            self.index += 1;
                            token.span.range.start = self.index;
                            state = State::Start;
                            continue;
                        }
                        b'\r' => {
                            state = State::LineCommentExpectNewline;
                            continue;
                        }
                        b'\x00'..=b'\x1F' | b'\x7F' => {
                            state = State::Invalid;
                            continue;
                        }
                        _ => {
                            state = State::LineComment;
                            continue;
                        }
                    }
                }
                State::LineComment => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 => {
                            if self.index != self.buffer.len() - 1 {
                                state = State::Invalid;
                                continue;
                            } else {
                                self.index += 1;
                                return Some(Token {
                                    tag: Tag::EndOfFile,
                                    span: Span {
                                        buffer: self.buffer.clone(),
                                        range: self.index - 1..self.index,
                                    },
                                });
                            }
                        }
                        b'\n' => {
                            self.index += 1;
                            token.span.range.start = self.index;
                            state = State::Start;
                            continue;
                        }
                        b'\r' => {
                            state = State::LineCommentExpectNewline;
                            continue;
                        }
                        b'\x01'..=b'\x09' | b'\x0B'..=b'\x0C' | b'\x0E'..=b'\x1F' | b'\x7F' => {
                            state = State::Invalid;
                            continue;
                        }
                        _ => continue,
                    }
                }
                State::LineCommentExpectNewline => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 => {
                            if self.index == self.buffer.len() - 1 {
                                token.tag = Tag::Invalid;
                            } else {
                                state = State::Invalid;
                                continue;
                            }
                        }
                        b'\n' => {
                            self.index += 1;
                            token.span.range.start = self.index;
                            state = State::Start;
                            continue;
                        }
                        _ => {
                            state = State::Invalid;
                            continue;
                        }
                    }
                }
                State::DocComment => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 | b'\n' => break,
                        b'\r' if self.buffer[self.index + 1] != b'\n' => {
                            state = State::Invalid;
                            continue;
                        }
                        b'\x01'..=b'\x09' | b'\x0B'..=b'\x0C' | b'\x0E'..=b'\x1F' | b'\x7F' => {
                            state = State::Invalid;
                            continue;
                        }
                        _ => continue,
                    }
                }
                State::BackSlash => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'\n' => {
                            token.tag = Tag::Invalid;
                            break;
                        }
                        b'\\' => {
                            token.tag = Tag::LitRawString;
                            state = State::RawString;
                            continue;
                        }
                        _ => {
                            state = State::Invalid;
                            continue;
                        }
                    }
                }
                State::RawString => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        0 if self.index != self.buffer.len() - 1 => {
                            state = State::Invalid;
                            continue;
                        }
                        b'\n' => break,
                        b'\r' if self.buffer[self.index + 1] != b'\n' => {
                            state = State::Invalid;
                            continue;
                        }
                        b'\x01'..=b'\x09' | b'\x0B'..=b'\x0C' | b'\x0E'..=b'\x1F' | b'\x7F' => {
                            state = State::Invalid;
                            continue;
                        }
                        _ => continue,
                    }
                }
                State::IntPrefix => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'.' => {
                            token.tag = Tag::LitFloat;
                            state = State::FloatDecStart;
                            continue;
                        }
                        b'b' => {
                            state = State::IntBinStart;
                            continue;
                        }
                        b'e' | b'E' => {
                            token.tag = Tag::LitFloat;
                            state = State::FloatExponent;
                            continue;
                        }
                        b'o' => {
                            state = State::IntOctStart;
                            continue;
                        }
                        b'x' => {
                            state = State::IntHexStart;
                            continue;
                        }
                        b'_'
                        | b'a'
                        | b'c'..=b'd'
                        | b'f'..=b'n'
                        | b'p'..=b'w'
                        | b'z'
                        | b'A'
                        | b'C'..=b'D'
                        | b'F'..=b'N'
                        | b'P'..=b'W'
                        | b'Z'
                        | b'0'..=b'9' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => break,
                    }
                }
                State::IntBinStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0' | b'1' => {
                            state = State::IntBin;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::IntBin => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0' | b'1' | b'_' => {
                            state = State::IntBin;
                            continue;
                        }
                        b'a'..=b'z' | b'A'..=b'Z' | b'2'..=b'9' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::IntOctStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'7' => {
                            state = State::IntOct;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::IntOct => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'7' | b'_' => {
                            state = State::IntOct;
                            continue;
                        }
                        b'a'..=b'z' | b'A'..=b'Z' | b'8'..=b'9' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::IntHexStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9' => {
                            state = State::IntHex;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::IntHex => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9' | b'_' => {
                            state = State::IntHex;
                            continue;
                        }
                        b'.' => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }

                            token.tag = Tag::LitFloat;
                            state = State::FloatHexStart;
                            continue;
                        }
                        b'p' | b'P' => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }

                            token.tag = Tag::LitFloat;
                            state = State::FloatExponent;
                            continue;
                        }
                        b'g'..=b'o' | b'q'..=b'z' | b'G'..=b'O' | b'Q'..=b'Z' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::IntDec => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'_' => {
                            state = State::IntDec;
                            continue;
                        }
                        b'.' => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }

                            token.tag = Tag::LitFloat;
                            state = State::FloatDecStart;
                            continue;
                        }
                        b'e' | b'E' => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }

                            token.tag = Tag::LitFloat;
                            state = State::FloatExponent;
                            continue;
                        }
                        b'a'..=b'd' | b'f'..=b'z' | b'A'..=b'D' | b'F'..=b'Z' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::FloatHexStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'.' => {
                            token.tag = Tag::LitInt;
                            self.index -= 1;
                            break;
                        }
                        b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9' => {
                            state = State::FloatHex;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::FloatHex => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'a'..=b'f' | b'A'..=b'F' | b'0'..=b'9' | b'_' => {
                            state = State::FloatHex;
                            continue;
                        }
                        b'p' | b'P' => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }

                            state = State::FloatExponent;
                            continue;
                        }
                        b'g'..=b'o' | b'q'..=b'z' | b'G'..=b'O' | b'Q'..=b'Z' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::FloatDecStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'.' => {
                            token.tag = Tag::LitInt;
                            self.index -= 1;
                            break;
                        }
                        b'0'..=b'9' => {
                            state = State::FloatDec;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::FloatDec => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'_' => {
                            state = State::FloatDec;
                            continue;
                        }
                        b'e' | b'E' => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }

                            state = State::FloatExponent;
                            continue;
                        }
                        b'a'..=b'd' | b'f'..=b'z' | b'A'..=b'D' | b'F'..=b'Z' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::FloatExponent => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'-' | b'+' => {
                            state = State::FloatExponentNumberStart;
                            continue;
                        }
                        b'0'..=b'9' => {
                            state = State::FloatExponentNumber;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::FloatExponentNumberStart => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' => {
                            state = State::FloatExponentNumber;
                            continue;
                        }
                        _ => {
                            state = State::IntInvalid;
                            continue;
                        }
                    }
                }
                State::FloatExponentNumber => {
                    self.index += 1;
                    match self.buffer[self.index] {
                        b'0'..=b'9' | b'_' => {
                            state = State::FloatExponentNumber;
                            continue;
                        }
                        b'a'..=b'z' | b'A'..=b'Z' => {
                            state = State::IntInvalid;
                            continue;
                        }
                        _ => {
                            if self.buffer[self.index - 1] == b'_' {
                                state = State::IntInvalid;
                                continue;
                            }
                            break;
                        }
                    }
                }
                State::IntInvalid => match self.buffer[self.index] {
                    b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => {
                        self.index += 1;
                        continue;
                    }
                    b'\n' => {
                        token.tag = Tag::Invalid;
                        break;
                    }
                    _ => {
                        state = State::Invalid;
                        continue;
                    }
                },
                State::Invalid => {
                    self.index += 1;
                    if self.index < self.buffer.len() {
                        match self.buffer[self.index] {
                            0 if self.index == self.buffer.len() - 1 => {
                                token.tag = Tag::Invalid;
                            }
                            b'\n' => {
                                token.tag = Tag::Invalid;
                            }
                            _ => continue,
                        }
                    } else {
                        token.tag = Tag::Invalid;
                    }
                    break;
                }
            }
        }

        token.span.range.end = self.index;
        Some(token)
    }
}

#[cfg(test)]
fn tokenizer_test(buffer: &'_ CStr) -> Vec<Tag> {
    let buffer = buffer
        .to_bytes_with_nul()
        .iter()
        .copied()
        .collect::<Arc<_>>();
    let tokenizer = Tokenizer::new(buffer.clone());
    let mut tokens = tokenizer.collect::<Vec<_>>();

    let last_token = tokens.last().unwrap();
    assert_eq!(last_token.tag, Tag::EndOfFile);
    assert_eq!(last_token.span.range.start, buffer.len() - 1);
    assert_eq!(last_token.span.range.end, buffer.len());

    tokens.pop();
    tokens.into_iter().map(|t| t.tag).collect::<Vec<_>>()
}

#[test]
fn lexemes() {
    for &(kw, tag) in Tag::LEXEMES {
        let buffer = CString::new(kw).unwrap();
        assert_eq!(tokenizer_test(buffer.as_c_str()), vec![tag])
    }
}

#[test]
fn line_comment() {
    assert_eq!(
        tokenizer_test(
            cr"
            // line comment
            const {}
        "
        ),
        vec![Tag::KwConst, Tag::SymLBrace, Tag::SymRBrace]
    )
}

#[test]
fn char_literal_ascii_escape() {
    assert_eq!(
        tokenizer_test(
            cr"
            '\x1b'
        "
        ),
        vec![Tag::LitChar]
    );
    assert_eq!(
        tokenizer_test(
            cr"
            '\x1B'
        "
        ),
        vec![Tag::LitChar]
    )
}

#[test]
fn invalid_char_literal_ascii_escape() {
    assert_eq!(
        tokenizer_test(
            cr"
            '\x1'
        "
        ),
        vec![Tag::Invalid]
    );
    assert_eq!(
        tokenizer_test(
            cr"
            '\x1g'
        "
        ),
        vec![Tag::Invalid]
    )
}
