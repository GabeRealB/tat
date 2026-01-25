use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    num::NonZero,
    str,
    sync::Arc,
};

use crate::packed_stream::{self, BitPacked, PackedStreamReader, PackedStreamWriter};
use crate::{lexer, packed_stream::Packable};

macro_rules! Token {
    (#<eof>) => {
        lexer::Tag::EndOfFile
    };
    (#<identifier>) => {
        lexer::Tag::Ident
    };
    (#<core_identifier>) => {
        lexer::Tag::IdentCore
    };
    (#<builtin_identifier>) => {
        lexer::Tag::IdentBuiltin
    };
    (#<char_literal>) => {
        lexer::Tag::LitChar
    };
    (#<string_literal>) => {
        lexer::Tag::LitString
    };
    (#<raw_string_literal>) => {
        lexer::Tag::LitRawString
    };
    (#<int_literal>) => {
        lexer::Tag::LitInt
    };
    (#<float_literal>) => {
        lexer::Tag::LitFloat
    };
    (#<inner_doc_comment>) => {
        lexer::Tag::InnerDocComment
    };
    (#<outer_doc_comment>) => {
        lexer::Tag::OuterDocComment
    };

    (&) => {
        lexer::Tag::SymAmpersand
    };
    (&=) => {
        lexer::Tag::SymAmpersandEqual
    };
    (*) => {
        lexer::Tag::SymAsterisk
    };
    (*=) => {
        lexer::Tag::SymAsteriskEqual
    };
    (*%) => {
        lexer::Tag::SymAsteriskPercent
    };
    (*%=) => {
        lexer::Tag::SymAsteriskPercentEqual
    };
    (*|) => {
        lexer::Tag::SymAsteriskPipe
    };
    (*|=) => {
        lexer::Tag::SymAsteriskPipeEqual
    };
    (^) => {
        lexer::Tag::SymCaret
    };
    (^=) => {
        lexer::Tag::SymCaretEqual
    };
    (:) => {
        lexer::Tag::SymColon
    };
    (::) => {
        lexer::Tag::SymColon2
    };
    (,) => {
        lexer::Tag::SymComma
    };
    (.) => {
        lexer::Tag::SymDot
    };
    (..) => {
        lexer::Tag::SymDot2
    };
    (..=) => {
        lexer::Tag::SymDot2Equal
    };
    (...) => {
        lexer::Tag::SymDot3
    };
    (.&) => {
        lexer::Tag::SymDotAmpersand
    };
    (.*) => {
        lexer::Tag::SymDotAsterisk
    };
    (.!) => {
        lexer::Tag::SymDotExclamationmark
    };
    (.?) => {
        lexer::Tag::SymDotQuestionmark
    };
    (=) => {
        lexer::Tag::SymEqual
    };
    (==) => {
        lexer::Tag::SymEqualEqual
    };
    (=>) => {
        lexer::Tag::SymEqualArrow
    };
    (!) => {
        lexer::Tag::SymExclamationmark
    };
    (!=) => {
        lexer::Tag::SymExclamationmarkEqual
    };
    (<) => {
        lexer::Tag::SymLArrow
    };
    (<<) => {
        lexer::Tag::SymLArrow2
    };
    (<<=) => {
        lexer::Tag::SymLArrow2Equal
    };
    (<<|) => {
        lexer::Tag::SymLArrow2Pipe
    };
    (<<|=) => {
        lexer::Tag::SymLArrow2PipeEqual
    };
    (<=) => {
        lexer::Tag::SymLArrowEqual
    };
    (<=>) => {
        lexer::Tag::SymLArrowEqualRArrow
    };
    ('{') => {
        lexer::Tag::SymLBrace
    };
    ('[') => {
        lexer::Tag::SymLBracket
    };
    ('(') => {
        lexer::Tag::SymLParen
    };
    (-) => {
        lexer::Tag::SymMinus
    };
    (-=) => {
        lexer::Tag::SymMinusEqual
    };
    (-%) => {
        lexer::Tag::SymMinusPercent
    };
    (-%=) => {
        lexer::Tag::SymMinusPercentEqual
    };
    (-|) => {
        lexer::Tag::SymMinusPipe
    };
    (-|=) => {
        lexer::Tag::SymMinusPipeEqual
    };
    (->) => {
        lexer::Tag::SymMinusArrow
    };
    (%) => {
        lexer::Tag::SymPercent
    };
    (%=) => {
        lexer::Tag::SymPercentEqual
    };
    (|) => {
        lexer::Tag::SymPipe
    };
    (|=) => {
        lexer::Tag::SymPipeEqual
    };
    (+) => {
        lexer::Tag::SymPlus
    };
    (++) => {
        lexer::Tag::SymPlus2
    };
    (+=) => {
        lexer::Tag::SymPlusEqual
    };
    (+%) => {
        lexer::Tag::SymPlusPercent
    };
    (+%=) => {
        lexer::Tag::SymPlusPercentEqual
    };
    (+|) => {
        lexer::Tag::SymPlusPipe
    };
    (+|=) => {
        lexer::Tag::SymPlusPipeEqual
    };
    (#) => {
        lexer::Tag::SymPound
    };
    (#!) => {
        lexer::Tag::SymPoundExclamationmark
    };
    (?) => {
        lexer::Tag::SymQuestionmark
    };
    (>) => {
        lexer::Tag::SymRArrow
    };
    (>>) => {
        lexer::Tag::SymRArrow2
    };
    (>>=) => {
        lexer::Tag::SymRArrow2Equal
    };
    (>=) => {
        lexer::Tag::SymRArrowEqual
    };
    ('}') => {
        lexer::Tag::SymRBrace
    };
    (']') => {
        lexer::Tag::SymRBracket
    };
    (')') => {
        lexer::Tag::SymRParen
    };
    (;) => {
        lexer::Tag::SymSemicolon
    };
    (/) => {
        lexer::Tag::SymSlash
    };
    (/=) => {
        lexer::Tag::SymSlashEqual
    };
    (~) => {
        lexer::Tag::SymTilde
    };

    (align) => {
        lexer::Tag::KwAlign
    };
    (and) => {
        lexer::Tag::KwAnd
    };
    (asm) => {
        lexer::Tag::KwAsm
    };
    (break) => {
        lexer::Tag::KwBreak
    };
    (callconv) => {
        lexer::Tag::KwCallconv
    };
    (catch) => {
        lexer::Tag::KwCatch
    };
    (const) => {
        lexer::Tag::KwConst
    };
    (context) => {
        lexer::Tag::KwContext
    };
    (cont_defer) => {
        lexer::Tag::KwContDefer
    };
    (continue) => {
        lexer::Tag::KwContinue
    };
    (defer) => {
        lexer::Tag::KwDefer
    };
    (else) => {
        lexer::Tag::KwElse
    };
    (enum) => {
        lexer::Tag::KwEnum
    };
    (err_defer) => {
        lexer::Tag::KwErrDefer
    };
    (fn) => {
        lexer::Tag::KwFn
    };
    (fnptr) => {
        lexer::Tag::KwFnptr
    };
    (for) => {
        lexer::Tag::KwFor
    };
    (if) => {
        lexer::Tag::KwIf
    };
    (impl) => {
        lexer::Tag::KwImpl
    };
    (inline) => {
        lexer::Tag::KwInline
    };
    (namespace) => {
        lexer::Tag::KwNamespace
    };
    (no_alias) => {
        lexer::Tag::KwNoAlias
    };
    (no_inline) => {
        lexer::Tag::KwNoInline
    };
    (opaque) => {
        lexer::Tag::KwOpaque
    };
    (or) => {
        lexer::Tag::KwOr
    };
    (or_else) => {
        lexer::Tag::KwOrElse
    };
    (#primitive) => {
        lexer::Tag::KwPrimitive
    };
    (pub) => {
        lexer::Tag::KwPub
    };
    (return) => {
        lexer::Tag::KwReturn
    };
    (Self) => {
        lexer::Tag::KwSelf
    };
    (self) => {
        lexer::Tag::KwSelfIdent
    };
    (struct) => {
        lexer::Tag::KwStruct
    };
    (switch) => {
        lexer::Tag::KwSwitch
    };
    (thread_local) => {
        lexer::Tag::KwThreadLocal
    };
    (throw) => {
        lexer::Tag::KwThrow
    };
    (try) => {
        lexer::Tag::KwTry
    };
    (union) => {
        lexer::Tag::KwUnion
    };
    (unreachable) => {
        lexer::Tag::KwUnreachable
    };
    (var) => {
        lexer::Tag::KwVar
    };
    (volatile) => {
        lexer::Tag::KwVolatile
    };
    (where) => {
        lexer::Tag::KwWhere
    };
    (while) => {
        lexer::Tag::KwWhile
    };
    (with) => {
        lexer::Tag::KwWith
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    pub tag: lexer::Tag,
    pub start: ByteOffset,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenIndex(NonZero<u32>);

impl TokenIndex {
    pub const fn new(index: u32) -> Self {
        assert!(index < u32::MAX);
        Self(unsafe { NonZero::new_unchecked(index + 1) })
    }

    pub const fn get(self) -> u32 {
        self.0.get() - 1
    }
}

impl packed_stream::DefaultPackable for TokenIndex {}

impl Debug for TokenIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TokenIndex").field(&self.get()).finish()
    }
}

impl Display for TokenIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.get())
    }
}

#[derive(Debug, Clone)]
pub struct Ast {
    pub source: Arc<[u8]>,
    pub tokens: Box<[Token]>,
    pub nodes: Box<[Node]>,
    pub extra_data: Box<[u32]>,
    pub errors: Box<[Error]>,
}

impl Ast {
    pub fn parse(source: Arc<[u8]>) -> Self {
        let tokenizer = lexer::Tokenizer::new(source.clone());
        let tokens = tokenizer
            .map(|t| Token {
                tag: t.tag,
                start: ByteOffset(t.span.byte_range().start as u32),
            })
            .collect::<Box<_>>();

        let mut parser = Parser {
            _source: &source,
            tokens: &tokens,
            index: TokenIndex::new(0),
            errors: vec![],
            nodes: vec![],
            extra_data: vec![],
        };

        match parse_root(&mut parser) {
            Ok(_) => {}
            Err(_) => assert!(!parser.errors.is_empty()),
        }

        let nodes = parser.nodes.into_boxed_slice();
        let extra_data = parser.extra_data.into_boxed_slice();
        let errors = parser.errors.into_boxed_slice();

        Self {
            source,
            tokens,
            nodes,
            extra_data,
            errors,
        }
    }

    pub fn get_token(&self, idx: TokenIndex) -> Token {
        let idx = idx.get() as usize;
        self.tokens[idx]
    }

    pub fn get_ident(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<identifier>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        if bytes[0] == b'#' {
            // Skip the first two `#"` bytes
            let mut offset = 2;
            loop {
                offset += bytes[offset..]
                    .iter()
                    .position(|byte| *byte == b'"')
                    .expect("checked by the lexer")
                    + 1;
                if bytes[offset - 2] != b'\\' {
                    break;
                }
            }
            unsafe { str::from_utf8_unchecked(&bytes[..offset]) }
        } else {
            let end = bytes
                .iter()
                .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
                .expect("source has zero terminator");
            unsafe { str::from_utf8_unchecked(&bytes[..end]) }
        }
    }

    pub fn get_core_ident(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<core_identifier>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        let end = bytes[1..]
            .iter()
            .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            .expect("source has zero terminator")
            + 1;
        unsafe { str::from_utf8_unchecked(&bytes[..end]) }
    }

    pub fn get_builtin_ident(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<builtin_identifier>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        let end = bytes[1..]
            .iter()
            .position(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            .expect("source has zero terminator")
            + 1;
        unsafe { str::from_utf8_unchecked(&bytes[..end]) }
    }

    pub fn get_char_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<char_literal>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        // Skip the first `'` byte
        let mut offset = 1;
        loop {
            offset += bytes[offset..]
                .iter()
                .position(|byte| *byte == b'\'')
                .expect("checked by the lexer")
                + 1;
            if bytes[offset - 2] != b'\\' {
                break;
            }
        }
        unsafe { str::from_utf8_unchecked(&bytes[..offset]) }
    }

    pub fn get_float_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<float_literal>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        let end = if bytes.len() > 2 && bytes[0] == b'0' && bytes[1] == b'x' {
            let prefix_end = bytes[2..]
                .iter()
                .position(|&b| !b.is_ascii_hexdigit() && b != b'_')
                .expect("checked by the lexer")
                + 2;
            let exponent_start = if bytes[prefix_end] == b'.' {
                bytes[prefix_end..]
                    .iter()
                    .position(|&b| !b.is_ascii_hexdigit() && b != b'_')
                    .expect("checked by the lexer")
                    + prefix_end
            } else {
                prefix_end
            };

            if !matches!(bytes[exponent_start], b'p' | b'P') {
                exponent_start
            } else {
                let exp_number_start = if matches!(bytes[exponent_start + 1], b'-' | b'+') {
                    exponent_start + 2
                } else {
                    exponent_start + 1
                };
                bytes[exp_number_start..]
                    .iter()
                    .position(|&b| !b.is_ascii_digit() && b != b'_')
                    .expect("checked by the lexer")
                    + exp_number_start
            }
        } else {
            let prefix_end = bytes
                .iter()
                .position(|&b| !b.is_ascii_digit() && b != b'_')
                .expect("checked by the lexer");
            let exponent_start = if bytes[prefix_end] == b'.' {
                bytes[prefix_end..]
                    .iter()
                    .position(|&b| !b.is_ascii_digit() && b != b'_')
                    .expect("checked by the lexer")
                    + prefix_end
            } else {
                prefix_end
            };

            if !matches!(bytes[exponent_start], b'e' | b'E') {
                exponent_start
            } else {
                let exp_number_start = if matches!(bytes[exponent_start + 1], b'-' | b'+') {
                    exponent_start + 2
                } else {
                    exponent_start + 1
                };
                bytes[exp_number_start..]
                    .iter()
                    .position(|&b| !b.is_ascii_digit() && b != b'_')
                    .expect("checked by the lexer")
                    + exp_number_start
            }
        };
        unsafe { str::from_utf8_unchecked(&bytes[..end]) }
    }

    pub fn get_int_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<int_literal>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        let end = if bytes.len() > 2 && bytes[0] == b'0' {
            match bytes[1] {
                b'b' => {
                    bytes[2..]
                        .iter()
                        .position(|&b| b != b'0' && b != b'1' && b != b'_')
                        .expect("checked by the lexer")
                        + 2
                }
                b'o' => {
                    bytes[2..]
                        .iter()
                        .position(|&b| !matches!(b, b'0'..=b'7') && b != b'_')
                        .expect("checked by the lexer")
                        + 2
                }
                b'x' => {
                    bytes[2..]
                        .iter()
                        .position(|&b| !b.is_ascii_hexdigit() && b != b'_')
                        .expect("checked by the lexer")
                        + 2
                }
                _ => 1,
            }
        } else {
            bytes
                .iter()
                .position(|&b| !b.is_ascii_digit() && b != b'_')
                .expect("checked by the lexer")
        };
        unsafe { str::from_utf8_unchecked(&bytes[..end]) }
    }

    pub fn get_string_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<string_literal>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        // Skip the first `"` byte
        let mut offset = 1;
        loop {
            offset += bytes[offset..]
                .iter()
                .position(|byte| *byte == b'\"')
                .expect("checked by the lexer")
                + 1;
            if bytes[offset - 2] != b'\\' {
                break;
            }
        }
        unsafe { str::from_utf8_unchecked(&bytes[..offset]) }
    }

    pub fn get_raw_string_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<raw_string_literal>));
        let start = token.start.0 as usize;

        let bytes = if let Some(next) = self.tokens.get(idx + 1) {
            let end = next.start.0 as usize;
            &self.source[start..=end]
        } else {
            &self.source[start..]
        };

        // Skip the first two `\\` bytes
        let end = bytes
            .iter()
            .position(|byte| *byte == b'\n')
            .expect("checked by the lexer")
            + 1;
        unsafe { str::from_utf8_unchecked(&bytes[..end]) }
    }

    pub fn get_packed<T: Packable>(&self, idx: ExtraIndex<T>) -> T {
        let start = idx.get();
        let mut stream = PackedStreamReader::new(&self.extra_data[start..]);
        stream.read()
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, node) in self.nodes.iter().enumerate() {
            let idx = NodeIndex::new(idx);
            let main_token = node.main_token;
            match &node.data {
                NodeData::Root(members) => {
                    write!(f, "{idx} := root(")?;
                    if let Some(members) = members {
                        for (i, member) in members.enumerate() {
                            let idx = self.get_packed(member);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                    }
                    writeln!(f, ")")?
                }
                NodeData::RootStructAttribute(_, _) => writeln!(f, "{idx} := #![struct]")?,
                NodeData::RootStructConstAttribute(_, _) => {
                    writeln!(f, "{idx} := #![struct const]")?
                }
                NodeData::RootStructLayoutAttribute(expr, _) => {
                    writeln!(f, "{idx} := #![struct({expr})]")?
                }
                NodeData::RootStructLayoutConstAttribute(expr, _) => {
                    writeln!(f, "{idx} := #![struct({expr}) const]")?
                }
                NodeData::OuterAnnotation(expr, _) => writeln!(f, "{idx} := #[={expr}]")?,
                NodeData::InnerAnnotation(expr, _) => writeln!(f, "{idx} := #![={expr}]")?,
                NodeData::Const(expr) => writeln!(f, "{idx} := const {expr}")?,
                NodeData::ThreadLocalDeclAlign(align_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := thread_local(name := {ident}, align := {align_expr}, init := {init_expr});"
                    )?
                }
                NodeData::ThreadLocalDeclTypeInit(type_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    write!(f, "{idx} := thread_local(name := {ident}")?;
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr}")?;
                    }
                    writeln!(f, ", init := {init_expr})")?
                }
                NodeData::ThreadLocalDecl(proto, type_init_expr) => {
                    let DeclProtoPub {
                        attrs,
                        is_pub,
                        is_var,
                        ident,
                        align_expr,
                    } = self.get_packed(*proto);
                    let PackedPair(type_expr, init_expr) = self.get_packed(*type_init_expr);

                    write!(f, "{idx} := thread_local(name := {ident}")?;
                    if let Some(attrs) = attrs {
                        write!(f, ", attrs := [")?;
                        for (i, extra) in attrs.enumerate() {
                            let idx = self.get_packed(extra);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                        write!(f, "]")?;
                    }
                    if is_pub {
                        write!(f, ", pub")?;
                    }
                    if is_var {
                        write!(f, ", var")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr})")?
                    }
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr})")?
                    }
                    writeln!(f, ", init := {init_expr})")?
                }
                NodeData::ThreadLocalDeclDestructure(protos, type_init_expr) => {
                    let protos = self.get_packed(*protos);
                    let PackedPair(type_expr, init_expr) = self.get_packed(*type_init_expr);

                    write!(f, "{idx} := thread_local_destructure(")?;
                    for (i, proto) in protos.enumerate() {
                        let DeclProtoPub {
                            attrs,
                            is_pub,
                            is_var,
                            ident,
                            align_expr,
                        } = self.get_packed(proto);
                        let ident = self.get_ident(ident);

                        if i == 0 {
                            write!(f, "\n\t{i} := .{{ name := {ident}")?;
                        } else {
                            write!(f, ",\n\t{i} := .{{ name := {ident}")?;
                        }
                        if let Some(attrs) = attrs {
                            write!(f, ", attrs := [")?;
                            for (i, extra) in attrs.enumerate() {
                                let idx = self.get_packed(extra);
                                if i == 0 {
                                    write!(f, "{idx}")?;
                                } else {
                                    write!(f, ", {idx}")?;
                                }
                            }
                            write!(f, "]")?;
                        }
                        if is_pub {
                            write!(f, ", pub")?;
                        }
                        if is_var {
                            write!(f, ", var")?;
                        }
                        if let Some(align_expr) = align_expr {
                            write!(f, ", align := {align_expr}")?;
                        }
                        write!(f, " }}")?;
                    }
                    write!(f, ",\n\t")?;
                    if let Some(type_expr) = type_expr {
                        write!(f, "type := {type_expr}), ")?
                    }
                    writeln!(f, "init := {init_expr})")?
                }
                NodeData::GlobalDeclAlign(align_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := global(name := {ident}, align := {align_expr}, init := {init_expr});"
                    )?
                }
                NodeData::GlobalDeclTypeInit(type_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    write!(f, "{idx} := global(name := {ident}")?;
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr}")?;
                    }
                    writeln!(f, ", init := {init_expr})")?
                }
                NodeData::GlobalDecl(proto, type_init_expr) => {
                    let DeclProtoPub {
                        attrs,
                        is_pub,
                        is_var,
                        ident,
                        align_expr,
                    } = self.get_packed(*proto);
                    let PackedPair(type_expr, init_expr) = self.get_packed(*type_init_expr);

                    write!(f, "{idx} := global(name := {ident}")?;
                    if let Some(attrs) = attrs {
                        write!(f, ", attrs := [")?;
                        for (i, extra) in attrs.enumerate() {
                            let idx = self.get_packed(extra);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                        write!(f, "]")?;
                    }
                    if is_pub {
                        write!(f, ", pub")?;
                    }
                    if is_var {
                        write!(f, ", var")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr})")?
                    }
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr})")?
                    }
                    writeln!(f, ", init := {init_expr})")?
                }
                NodeData::GlobalDeclDestructure(protos, type_init_expr) => {
                    let protos = self.get_packed(*protos);
                    let PackedPair(type_expr, init_expr) = self.get_packed(*type_init_expr);

                    write!(f, "{idx} := global_destructure(")?;
                    for (i, proto) in protos.enumerate() {
                        let DeclProtoPub {
                            attrs,
                            is_pub,
                            is_var,
                            ident,
                            align_expr,
                        } = self.get_packed(proto);
                        let ident = self.get_ident(ident);

                        if i == 0 {
                            write!(f, "\n\t{i} := .{{ name := {ident}")?;
                        } else {
                            write!(f, ",\n\t{i} := .{{ name := {ident}")?;
                        }
                        if let Some(attrs) = attrs {
                            write!(f, ", attrs := [")?;
                            for (i, extra) in attrs.enumerate() {
                                let idx = self.get_packed(extra);
                                if i == 0 {
                                    write!(f, "{idx}")?;
                                } else {
                                    write!(f, ", {idx}")?;
                                }
                            }
                            write!(f, "]")?;
                        }
                        if is_pub {
                            write!(f, ", pub")?;
                        }
                        if is_var {
                            write!(f, ", var")?;
                        }
                        if let Some(align_expr) = align_expr {
                            write!(f, ", align := {align_expr}")?;
                        }
                        write!(f, " }}")?;
                    }
                    write!(f, ",\n\t")?;
                    if let Some(type_expr) = type_expr {
                        write!(f, "type := {type_expr}), ")?
                    }
                    writeln!(f, "init := {init_expr})")?
                }
                NodeData::ContainerFieldType(align, type_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, align := {align}, type := {type_expr})"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, type := {type_expr})"
                        )?
                    }
                }
                NodeData::ContainerFieldTypeComma(align, type_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, align := {align}, type := {type_expr}),"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, type := {type_expr}),"
                        )?
                    }
                }
                NodeData::ContainerFieldInit(align, init_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, align := {align}, init := {init_expr})"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, init := {init_expr})"
                        )?
                    }
                }
                NodeData::ContainerFieldInitComma(align, init_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, align := {align}, init := {init_expr}),"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field(name := {ident}, init := {init_expr}),"
                        )?
                    }
                }
                NodeData::ContainerFieldTypeInit(type_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := container_field(name := {ident}, type := {type_expr}, init := {init_expr})"
                    )?
                }
                NodeData::ContainerFieldTypeInitComma(type_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := container_field(name := {ident}, type := {type_expr}, init := {init_expr}),"
                    )?
                }
                NodeData::ContainerField(extra) => {
                    let ident = self.get_ident(main_token);
                    let StructFieldProto {
                        attrs,
                        align_expr,
                        type_expr,
                        init_expr,
                    } = self.get_packed(*extra);

                    write!(f, "{idx} := container_field(name := {ident}")?;
                    if let Some(attrs) = attrs {
                        write!(f, ", attrs := [")?;
                        for (i, attr) in attrs.enumerate() {
                            let idx = self.get_packed(attr);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                        write!(f, "]")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr}")?;
                    }
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr}")?;
                    }
                    if let Some(init_expr) = init_expr {
                        write!(f, ", init := {init_expr}")?;
                    }
                    writeln!(f, ")")?
                }
                NodeData::ContainerFieldComma(extra) => {
                    let ident = self.get_ident(main_token);
                    let StructFieldProto {
                        attrs,
                        align_expr,
                        type_expr,
                        init_expr,
                    } = self.get_packed(*extra);

                    write!(f, "{idx} := container_field(name := {ident}")?;
                    if let Some(attrs) = attrs {
                        write!(f, ", attrs := [")?;
                        for (i, attrs) in attrs.enumerate() {
                            let idx = self.get_packed(attrs);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                        write!(f, "]")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr}")?;
                    }
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr}")?;
                    }
                    if let Some(init_expr) = init_expr {
                        write!(f, ", init := {init_expr}")?;
                    }
                    writeln!(f, "),")?
                }
                NodeData::ContainerFieldInlineType(align, type_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, align := {align}, type := {type_expr})"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, type := {type_expr})"
                        )?
                    }
                }
                NodeData::ContainerFieldInlineTypeComma(align, type_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, align := {align}, type := {type_expr}),"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, type := {type_expr}),"
                        )?
                    }
                }
                NodeData::ContainerFieldInlineInit(align, init_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, align := {align}, init := {init_expr})"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, init := {init_expr})"
                        )?
                    }
                }
                NodeData::ContainerFieldInlineInitComma(align, init_expr) => {
                    let ident = self.get_ident(main_token);
                    if let Some(align) = align {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, align := {align}, init := {init_expr}),"
                        )?
                    } else {
                        writeln!(
                            f,
                            "{idx} := container_field_inlined(name := {ident}, init := {init_expr}),"
                        )?
                    }
                }
                NodeData::ContainerFieldInlineTypeInit(type_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := container_field_inlined(name := {ident}, type := {type_expr}, init := {init_expr})"
                    )?
                }
                NodeData::ContainerFieldInlineTypeInitComma(type_expr, init_expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := container_field_inlined(name := {ident}, type := {type_expr}, init := {init_expr}),"
                    )?
                }
                NodeData::ContainerFieldInline(extra) => {
                    let ident = self.get_ident(main_token);
                    let StructFieldProto {
                        attrs,
                        align_expr,
                        type_expr,
                        init_expr,
                    } = self.get_packed(*extra);

                    write!(f, "{idx} := container_field_inlined(name := {ident}")?;
                    if let Some(attrs) = attrs {
                        write!(f, ", attrs := [")?;
                        for (i, attr) in attrs.enumerate() {
                            let idx = self.get_packed(attr);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                        write!(f, "]")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr}")?;
                    }
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr}")?;
                    }
                    if let Some(init_expr) = init_expr {
                        write!(f, ", init := {init_expr}")?;
                    }
                    writeln!(f, ")")?
                }
                NodeData::ContainerFieldInlineComma(extra) => {
                    let ident = self.get_ident(main_token);
                    let StructFieldProto {
                        attrs,
                        align_expr,
                        type_expr,
                        init_expr,
                    } = self.get_packed(*extra);

                    write!(f, "{idx} := container_field_inlined(name := {ident}")?;
                    if let Some(attrs) = attrs {
                        write!(f, ", attrs := [")?;
                        for (i, attr) in attrs.enumerate() {
                            let idx = self.get_packed(attr);
                            if i == 0 {
                                write!(f, "{idx}")?;
                            } else {
                                write!(f, ", {idx}")?;
                            }
                        }
                        write!(f, "]")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr}")?;
                    }
                    if let Some(type_expr) = type_expr {
                        write!(f, ", type := {type_expr}")?;
                    }
                    if let Some(init_expr) = init_expr {
                        write!(f, ", init := {init_expr}")?;
                    }
                    writeln!(f, "),")?
                }
                NodeData::ImplBlock(cond_expr, decl_block) => writeln!(
                    f,
                    "{idx} := impl_block(condition := {cond_expr}, block := {decl_block})"
                )?,
                NodeData::ImplBlockAttrs(impl_block) => {
                    let ImplBlock {
                        attrs,
                        cond_expr,
                        decl_block,
                    } = self.get_packed(*impl_block);
                    write!(f, "{idx} := impl_block(attrs := [")?;
                    for (i, attr) in attrs.enumerate() {
                        let attr = self.get_packed(attr);
                        if i == 0 {
                            write!(f, "{attr}")?;
                        } else {
                            write!(f, ", {attr}")?;
                        }
                    }
                    writeln!(f, "], condition := {cond_expr}, block := {decl_block})")?;
                }
                NodeData::BlockTwo(expr1, expr2) => {
                    if let &Some(expr2) = expr2 {
                        let expr1 = expr1.unwrap();
                        writeln!(f, "{idx} := block({expr1}, {expr2})")?
                    } else if let &Some(expr1) = expr1 {
                        writeln!(f, "{idx} := block({expr1})")?
                    } else {
                        writeln!(f, "{idx} := block()")?
                    }
                }
                NodeData::BlockTwoSemicolon(expr1, expr2) => {
                    if let &Some(expr2) = expr2 {
                        let expr1 = expr1.unwrap();
                        writeln!(f, "{idx} := block({expr1}, {expr2});")?
                    } else if let &Some(expr1) = expr1 {
                        writeln!(f, "{idx} := block({expr1});")?
                    } else {
                        writeln!(f, "{idx} := block();")?
                    }
                }
                NodeData::Block(members) => {
                    write!(f, "{idx} := block(")?;
                    for (i, member) in members.enumerate() {
                        let idx = self.get_packed(member);
                        if i == 0 {
                            write!(f, "{idx}")?;
                        } else {
                            write!(f, ", {idx}")?;
                        }
                    }
                    writeln!(f, ")")?;
                }
                NodeData::BlockSemicolon(members) => {
                    write!(f, "{idx} := block(")?;
                    for (i, member) in members.enumerate() {
                        let idx = self.get_packed(member);
                        if i == 0 {
                            write!(f, "{idx}")?;
                        } else {
                            write!(f, ", {idx}")?;
                        }
                    }
                    writeln!(f, ");")?;
                }
                NodeData::TemplateExprSimple(where_expr, expr) => {
                    if let Some(where_expr) = where_expr {
                        writeln!(
                            f,
                            "{idx} := with(args := [], where := {where_expr}, expr := {expr}"
                        )?;
                    } else {
                        writeln!(f, "{idx} := with(args := [], expr := {expr}")?;
                    }
                }
                NodeData::TemplateExpr(proto, expr) => {
                    let TemplateExprProto { args, where_expr } = self.get_packed(*proto);
                    write!(f, "{idx} := with(args := [")?;
                    for (i, arg) in args.enumerate() {
                        let TemplateExprArg {
                            ident,
                            type_expr,
                            init_expr,
                        } = self.get_packed(arg);
                        let ident = self.get_ident(ident);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{ident}: {type_expr}")?;
                        if let Some(init_expr) = init_expr {
                            write!(f, " = {init_expr}")?;
                        }
                    }
                    if let Some(where_expr) = where_expr {
                        writeln!(f, "], where := {where_expr}, expr := {expr}")?;
                    } else {
                        writeln!(f, "], expr := {expr}")?;
                    }
                }
                NodeData::TemplateExprComma(proto, expr) => {
                    let TemplateExprProto { args, where_expr } = self.get_packed(*proto);
                    write!(f, "{idx} := with(args := [")?;
                    for (i, arg) in args.enumerate() {
                        let TemplateExprArg {
                            ident,
                            type_expr,
                            init_expr,
                        } = self.get_packed(arg);
                        let ident = self.get_ident(ident);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{ident}: {type_expr}")?;
                        if let Some(init_expr) = init_expr {
                            write!(f, " = {init_expr}")?;
                        }
                    }
                    if let Some(where_expr) = where_expr {
                        writeln!(f, ",], where := {where_expr}, expr := {expr}")?;
                    } else {
                        writeln!(f, ",], expr := {expr}")?;
                    }
                }
                NodeData::Range(lhs, rhs) => {
                    let op_token = self.get_token(main_token);
                    let op = op_token.tag.as_lexeme().unwrap();
                    write!(f, "{idx} := range(op := {op})")?;
                    if let Some(lhs) = lhs {
                        write!(f, ", lhs := {lhs}")?;
                    }
                    if let Some(rhs) = rhs {
                        write!(f, ", rhs := {rhs}")?;
                    }
                    writeln!(f, ")")?
                }
                NodeData::Binary(lhs, rhs) => {
                    let op_token = self.get_token(main_token);
                    let op = op_token.tag.as_lexeme().unwrap();
                    writeln!(
                        f,
                        "{idx} := binary_op(op := {op}, lhs := {lhs}, rhs := {rhs})"
                    )?
                }
                NodeData::Unary(expr) => {
                    let op_token = self.get_token(main_token);
                    let op = op_token.tag.as_lexeme().unwrap();
                    writeln!(f, "{idx} := unary_op(op := {op}, expr := {expr})")?
                }
                NodeData::AsmSimple(clobbers_expr, instr_expr) => writeln!(
                    f,
                    "{idx} := asm(captures := [], args := [], clobbers := {clobbers_expr}, outputs := [], instr := {instr_expr})"
                )?,
                NodeData::AsmVolatileSimple(clobbers_expr, instr_expr) => writeln!(
                    f,
                    "{idx} := asm(volatile, captures := [], args := [], clobbers := {clobbers_expr}, outputs := [], instr := {instr_expr})"
                )?,
                NodeData::Asm(proto, instr_expr) => {
                    let AsmProto {
                        has_trailing_capture_comma,
                        has_trailing_input_comma,
                        clobbers_expr,
                        captures,
                        inputs,
                        outputs,
                    } = self.get_packed(*proto);
                    write!(f, "{idx} := asm(captures := [")?;
                    if let Some(captures) = captures {
                        for (i, capture) in captures.enumerate() {
                            let AsmCapture {
                                ident,
                                type_expr,
                                init_expr,
                            } = self.get_packed(capture);
                            let ident = self.get_ident(ident);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            if let Some((type_expr, init_expr)) = type_expr.zip(init_expr) {
                                write!(f, "{ident}: {type_expr} = {init_expr}")?;
                            } else if let Some(init_expr) = init_expr {
                                write!(f, "{ident} := {init_expr}")?;
                            } else {
                                write!(f, "{ident}")?;
                            }
                        }
                    }
                    if has_trailing_capture_comma {
                        write!(f, ",], inputs := [")?;
                    } else {
                        write!(f, "], inputs := [")?;
                    }

                    if let Some(inputs) = inputs {
                        for (i, input) in inputs.enumerate() {
                            let AsmInput { ident, constraint } = self.get_packed(input);
                            let ident = self.get_ident(ident);
                            let constraint = self.get_string_lit(constraint);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{ident}: {constraint}")?;
                        }
                    }
                    if has_trailing_input_comma {
                        write!(f, ",], clobbers := {clobbers_expr}, outputs := [")?;
                    } else {
                        write!(f, "], clobbers := {clobbers_expr}, outputs := [")?;
                    }

                    if let Some(outputs) = outputs {
                        for (i, output) in outputs.enumerate() {
                            let AsmOutput {
                                ident,
                                type_expr,
                                constraint,
                            } = self.get_packed(output);
                            let ident = self.get_ident(ident);
                            let constraint = self.get_string_lit(constraint);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{ident} : {type_expr} {constraint}")?;
                        }
                    }
                    writeln!(f, "], instr := {instr_expr})")?
                }
                NodeData::AsmVolatile(proto, instr_expr) => {
                    let AsmProto {
                        has_trailing_capture_comma,
                        has_trailing_input_comma,
                        clobbers_expr,
                        captures,
                        inputs,
                        outputs,
                    } = self.get_packed(*proto);
                    write!(f, "{idx} := asm(volatile, captures := [")?;
                    if let Some(captures) = captures {
                        for (i, capture) in captures.enumerate() {
                            let AsmCapture {
                                ident,
                                type_expr,
                                init_expr,
                            } = self.get_packed(capture);
                            let ident = self.get_ident(ident);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            if let Some((type_expr, init_expr)) = type_expr.zip(init_expr) {
                                write!(f, "{ident}: {type_expr} = {init_expr}")?;
                            } else if let Some(init_expr) = init_expr {
                                write!(f, "{ident} := {init_expr}")?;
                            } else {
                                write!(f, "{ident}")?;
                            }
                        }
                    }
                    if has_trailing_capture_comma {
                        write!(f, ",], inputs := [")?;
                    } else {
                        write!(f, "], inputs := [")?;
                    }

                    if let Some(inputs) = inputs {
                        for (i, input) in inputs.enumerate() {
                            let AsmInput { ident, constraint } = self.get_packed(input);
                            let ident = self.get_ident(ident);
                            let constraint = self.get_string_lit(constraint);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{ident}: {constraint}")?;
                        }
                    }
                    if has_trailing_input_comma {
                        write!(f, ",], clobbers := {clobbers_expr}, outputs := [")?;
                    } else {
                        write!(f, "], clobbers := {clobbers_expr}, outputs := [")?;
                    }

                    if let Some(outputs) = outputs {
                        for (i, output) in outputs.enumerate() {
                            let AsmOutput {
                                ident,
                                type_expr,
                                constraint,
                            } = self.get_packed(output);
                            let ident = self.get_ident(ident);
                            let constraint = self.get_string_lit(constraint);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{ident} : {type_expr} {constraint}")?;
                        }
                    }

                    writeln!(f, "], instr := {instr_expr})")?
                }
                NodeData::Jump(label, value_expr) => {
                    let jump_op = self.get_token(main_token);
                    let jump_op = jump_op.tag.as_lexeme().unwrap();
                    write!(f, "{idx} := {jump_op}(")?;
                    if let Some(label) = label {
                        let label = self.get_ident(*label);
                        write!(f, "label := :{label}")?;
                    }
                    if let Some(value_expr) = value_expr {
                        if label.is_some() {
                            write!(f, ", ")?;
                        }
                        write!(f, "value := {value_expr}")?;
                    }
                    writeln!(f, ")")?;
                }
                NodeData::Fn(proto, expr_block) => {
                    let FnProto {
                        has_trailing_capture_comma,
                        has_trailing_arg_comma,
                        modifier,
                        context,
                        captures,
                        receiver,
                        args,
                        call_conv_expr,
                        return_type_expr,
                        where_expr,
                    } = self.get_packed(*proto);
                    write!(f, "{idx} := fn(")?;
                    if let Some(context) = context {
                        let FnContext {
                            is_optional,
                            ctx_token: _,
                            type_expr,
                        } = self.get_packed(context);
                        if let Some(type_expr) = type_expr {
                            write!(f, "context := {type_expr}, ")?;
                        } else if is_optional {
                            write!(f, "context := opt_any, ")?;
                        } else {
                            write!(f, "context := any, ")?;
                        }
                    }

                    write!(f, "captures := [")?;
                    if let Some(captures) = captures {
                        for (i, capture) in captures.enumerate() {
                            let FnCapture {
                                ident,
                                type_expr,
                                init_expr,
                            } = self.get_packed(capture);
                            let ident = self.get_ident(ident);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            if let Some((type_expr, init_expr)) = type_expr.zip(init_expr) {
                                write!(f, "{ident}: {type_expr} = {init_expr}")?;
                            } else if let Some(init_expr) = init_expr {
                                write!(f, "{ident} := {init_expr}")?;
                            } else {
                                write!(f, "{ident}")?;
                            }
                        }
                    }
                    if has_trailing_capture_comma {
                        write!(f, ",")?;
                    }
                    write!(f, "], ")?;

                    write!(f, "args := [")?;
                    if let Some(receiver) = receiver {
                        let FnReceiver {
                            modifier,
                            is_var,
                            pointer_type,
                            self_token: _,
                        } = self.get_packed(receiver);
                        match modifier {
                            FnArgModifier::None => {}
                            FnArgModifier::Const => write!(f, "const ")?,
                            FnArgModifier::NoAlias => write!(f, "no_alias ")?,
                        }
                        if is_var {
                            write!(f, "var ")?
                        }
                        match pointer_type {
                            None => write!(f, "self")?,
                            Some(PointerType::Default) => write!(f, "*self")?,
                            Some(PointerType::Var) => write!(f, "*var self")?,
                            Some(PointerType::OptVar) => write!(f, "*?var self")?,
                            Some(PointerType::Volatile) => write!(f, "*volatile self")?,
                            Some(PointerType::OptVolatile) => write!(f, "*?volatile self")?,
                            Some(PointerType::VarVolatile) => write!(f, "*var volatile self")?,
                            Some(PointerType::OptVarVolatile) => write!(f, "*?var volatile self")?,
                            Some(PointerType::VarOptVolatile) => write!(f, "*var ?volatile self")?,
                            Some(PointerType::OptVarOptVolatile) => {
                                write!(f, "*?var ?volatile self")?
                            }
                        }
                    }
                    if let Some(args) = args {
                        for (i, arg) in args.enumerate() {
                            let FnArg {
                                modifier,
                                is_var,
                                ident,
                                type_expr,
                                default_expr,
                            } = self.get_packed(arg);
                            let ident = self.get_ident(ident);
                            if i != 0 || receiver.is_some() {
                                write!(f, ", ")?;
                            }
                            match modifier {
                                FnArgModifier::None => {}
                                FnArgModifier::Const => write!(f, "const ")?,
                                FnArgModifier::NoAlias => write!(f, "no_alias ")?,
                            }
                            if is_var {
                                write!(f, "var ")?
                            }
                            if let Some(default_expr) = default_expr {
                                write!(f, "{ident}: {type_expr} = {default_expr}")?
                            } else {
                                write!(f, "{ident}: {type_expr}")?
                            }
                        }
                    }
                    if has_trailing_arg_comma {
                        write!(f, ",")?;
                    }
                    write!(f, "], ")?;

                    match modifier {
                        FnModifier::None => {}
                        FnModifier::Const => write!(f, "modifier := const, ")?,
                        FnModifier::Inline => write!(f, "modifier := inline, ")?,
                        FnModifier::NoInline => write!(f, "modifier := no_inline, ")?,
                    }
                    if let Some(call_conv_expr) = call_conv_expr {
                        write!(f, "call_conv := {call_conv_expr}, ")?;
                    }
                    if let Some(return_type_expr) = return_type_expr {
                        write!(f, "return_type := {return_type_expr}, ")?;
                    }
                    if let Some(where_expr) = where_expr {
                        write!(f, "where := {where_expr}, ")?;
                    }

                    writeln!(f, "block := {expr_block})")?
                }
                NodeData::ExprInit(expr, init_list) => {
                    writeln!(f, "{idx} := init(expr := {expr}, init_list := {init_list})")?
                }
                NodeData::TokenInit(init_list) => {
                    let token = self.get_token(main_token);
                    let token = token.tag.as_lexeme().unwrap();
                    writeln!(
                        f,
                        "{idx} := init(token := {token}, init_list := {init_list})"
                    )?
                }
                NodeData::ExprInitListTwo(lhs, rhs) => {
                    write!(f, "{idx} := init_list(exprs := [")?;
                    if let Some(lhs) = lhs {
                        write!(f, "{lhs}")?;
                    }
                    if let Some(rhs) = rhs {
                        write!(f, ", {rhs}")?;
                    }
                    writeln!(f, "])")?;
                }
                NodeData::ExprInitListTwoComma(lhs, rhs) => {
                    write!(f, "{idx} := init_list(exprs := [")?;
                    if let Some(lhs) = lhs {
                        write!(f, "{lhs}")?;
                    }
                    if let Some(rhs) = rhs {
                        write!(f, ", {rhs}")?;
                    }
                    writeln!(f, ",])")?;
                }
                NodeData::ExprInitList(exprs) => {
                    write!(f, "{idx} := init_list(exprs := [")?;
                    for (i, expr) in exprs.enumerate() {
                        let expr = self.get_packed(expr);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{expr}")?;
                    }
                    writeln!(f, "])")?;
                }
                NodeData::ExprInitListComma(exprs) => {
                    write!(f, "{idx} := init_list(exprs := [")?;
                    for (i, expr) in exprs.enumerate() {
                        let expr = self.get_packed(expr);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{expr}")?;
                    }
                    writeln!(f, ",])")?;
                }
                NodeData::FieldInitListTwo(first, second) => {
                    write!(f, "{idx} := init_list(fields := [")?;
                    let FieldInit { ident, init_expr } = self.get_packed(*first);
                    write!(f, "{ident} := {init_expr}")?;
                    if let Some(second) = second {
                        let FieldInit { ident, init_expr } = self.get_packed(*second);
                        write!(f, ", {ident} := {init_expr}")?;
                    }
                    writeln!(f, "])")?;
                }
                NodeData::FieldInitListTwoComma(first, second) => {
                    write!(f, "{idx} := init_list(fields := [")?;
                    let FieldInit { ident, init_expr } = self.get_packed(*first);
                    write!(f, "{ident} := {init_expr}")?;
                    if let Some(second) = second {
                        let FieldInit { ident, init_expr } = self.get_packed(*second);
                        write!(f, ", {ident} := {init_expr}")?;
                    }
                    writeln!(f, ",])")?;
                }
                NodeData::FieldInitList(fields) => {
                    write!(f, "{idx} := init_list(fields := [")?;
                    for (i, field) in fields.enumerate() {
                        let FieldInit { ident, init_expr } = self.get_packed(field);
                        let ident = self.get_ident(ident);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{ident} := {init_expr}")?;
                    }
                    writeln!(f, "])")?;
                }
                NodeData::FieldInitListComma(fields) => {
                    write!(f, "{idx} := init_list(fields := [")?;
                    for (i, field) in fields.enumerate() {
                        let FieldInit { ident, init_expr } = self.get_packed(field);
                        let ident = self.get_ident(ident);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{ident} := {init_expr}")?;
                    }
                    writeln!(f, ",])")?;
                }
                NodeData::ErrorUnionType(lhs, rhs) => {
                    writeln!(f, "{idx} := error_union_type(lhs := {lhs}, rhs := {rhs})")?
                }
                NodeData::Pack(type_expr) => writeln!(f, "{idx} := pack(type := {type_expr})")?,
                NodeData::OptionType(expr) => writeln!(f, "{idx} := option_type(child := {expr})")?,
                NodeData::Slice(sentinel_expr, type_expr) => {
                    if let Some(sentinel_expr) = sentinel_expr {
                        writeln!(
                            f,
                            "{idx} := slice(sentinel := {sentinel_expr}, type := {type_expr})"
                        )?
                    } else {
                        writeln!(f, "{idx} := slice(type := {type_expr})")?
                    }
                }
                NodeData::SinglePointerSimple(pointer_type, type_expr) => {
                    write!(f, "{idx} := single_pointer(type := ")?;
                    match pointer_type {
                        PointerType::Default => writeln!(f, "{type_expr})")?,
                        PointerType::Var => writeln!(f, "var {type_expr})")?,
                        PointerType::OptVar => writeln!(f, "?var {type_expr})")?,
                        PointerType::Volatile => writeln!(f, "volatile {type_expr})")?,
                        PointerType::OptVolatile => writeln!(f, "?volatile {type_expr})")?,
                        PointerType::VarVolatile => writeln!(f, "var volatile {type_expr})")?,
                        PointerType::OptVarVolatile => writeln!(f, "?var volatile {type_expr})")?,
                        PointerType::VarOptVolatile => writeln!(f, "var ?volatile {type_expr})")?,
                        PointerType::OptVarOptVolatile => {
                            writeln!(f, "?var ?volatile {type_expr})")?
                        }
                    }
                }
                NodeData::SinglePointer(prefix, type_expr) => {
                    let SinglePointerPrefix {
                        pointer_type,
                        align_expr,
                    } = self.get_packed(*prefix);
                    write!(f, "{idx} := single_pointer(type := ")?;
                    match pointer_type {
                        PointerType::Default => write!(f, "{type_expr}")?,
                        PointerType::Var => write!(f, "var {type_expr}")?,
                        PointerType::OptVar => write!(f, "?var {type_expr}")?,
                        PointerType::Volatile => write!(f, "volatile {type_expr}")?,
                        PointerType::OptVolatile => write!(f, "?volatile {type_expr}")?,
                        PointerType::VarVolatile => write!(f, "var volatile {type_expr}")?,
                        PointerType::OptVarVolatile => write!(f, "?var volatile {type_expr}")?,
                        PointerType::VarOptVolatile => write!(f, "var ?volatile {type_expr}")?,
                        PointerType::OptVarOptVolatile => write!(f, "?var ?volatile {type_expr}")?,
                    }
                    writeln!(f, ", align := {align_expr})")?
                }
                NodeData::MultiPointerSimple(pointer_type, type_expr) => {
                    write!(f, "{idx} := multi_pointer(type := ")?;
                    match pointer_type {
                        PointerType::Default => writeln!(f, "{type_expr})")?,
                        PointerType::Var => writeln!(f, "var {type_expr})")?,
                        PointerType::OptVar => writeln!(f, "?var {type_expr})")?,
                        PointerType::Volatile => writeln!(f, "volatile {type_expr})")?,
                        PointerType::OptVolatile => writeln!(f, "?volatile {type_expr})")?,
                        PointerType::VarVolatile => writeln!(f, "var volatile {type_expr})")?,
                        PointerType::OptVarVolatile => writeln!(f, "?var volatile {type_expr})")?,
                        PointerType::VarOptVolatile => writeln!(f, "var ?volatile {type_expr})")?,
                        PointerType::OptVarOptVolatile => {
                            writeln!(f, "?var ?volatile {type_expr})")?
                        }
                    }
                }
                NodeData::MultiPointer(prefix, type_expr) => {
                    let MultiPointerPrefix {
                        pointer_type,
                        sentinel_expr,
                        align_expr,
                    } = self.get_packed(*prefix);
                    write!(f, "{idx} := multi_pointer(type := ")?;
                    match pointer_type {
                        PointerType::Default => write!(f, "{type_expr}")?,
                        PointerType::Var => write!(f, "var {type_expr}")?,
                        PointerType::OptVar => write!(f, "?var {type_expr}")?,
                        PointerType::Volatile => write!(f, "volatile {type_expr}")?,
                        PointerType::OptVolatile => write!(f, "?volatile {type_expr}")?,
                        PointerType::VarVolatile => write!(f, "var volatile {type_expr}")?,
                        PointerType::OptVarVolatile => write!(f, "?var volatile {type_expr}")?,
                        PointerType::VarOptVolatile => write!(f, "var ?volatile {type_expr}")?,
                        PointerType::OptVarOptVolatile => write!(f, "?var ?volatile {type_expr}")?,
                    }
                    if let Some(sentinel_expr) = sentinel_expr {
                        write!(f, ", sentinel := {sentinel_expr}")?;
                    }
                    if let Some(align_expr) = align_expr {
                        write!(f, ", align := {align_expr}")?;
                    }
                    writeln!(f, ")")?;
                }
                NodeData::Vector(len_expr, type_expr) => {
                    writeln!(f, "{idx} := vector(len := {len_expr}, type := {type_expr})")?
                }
                NodeData::MatrixSimple(rows_cols, type_expr) => {
                    let PackedPair(rows, cols) = self.get_packed(*rows_cols);
                    writeln!(
                        f,
                        "{idx} := matrix(rows := {rows}, columns := {cols}, type := {type_expr})"
                    )?
                }
                NodeData::Matrix(info, type_expr) => {
                    let Matrix { rows, cols, layout } = self.get_packed(*info);
                    writeln!(
                        f,
                        "{idx} := matrix(rows := {rows}, columns := {cols}, layout := {layout}, type := {type_expr})"
                    )?
                }
                NodeData::ArraySimple(len_expr, type_expr) => {
                    writeln!(f, "{idx} := array(len := {len_expr}, type := {type_expr})")?
                }
                NodeData::Array(len_sentinel, type_expr) => {
                    let PackedPair(len_expr, sentinel_expr) = self.get_packed(*len_sentinel);
                    writeln!(
                        f,
                        "{idx} := array(len := {len_expr}, sentinel := {sentinel_expr}, type := {type_expr})"
                    )?
                }
                NodeData::EnumLiteral(literal) => {
                    let ident = self.get_ident(*literal);
                    writeln!(f, "{idx} := enum_literal({ident})")?
                }
                NodeData::Ident => {
                    let ident = self.get_ident(main_token);
                    writeln!(f, "{idx} := ident(name := {ident})")?
                }
                NodeData::CoreIdent => {
                    let ident = self.get_core_ident(main_token);
                    writeln!(f, "{idx} := core_ident(name := {ident})")?
                }
                NodeData::BuiltinIdent => {
                    let ident = self.get_builtin_ident(main_token);
                    writeln!(f, "{idx} := builtin_ident(name := {ident})")?
                }
                NodeData::ImplExpr(type_expr) => writeln!(f, "{idx} := impl(type := {type_expr})")?,
                NodeData::SelfType => writeln!(f, "{idx} := Self")?,
                NodeData::SelfIdent => writeln!(f, "{idx} := self")?,
                NodeData::Unreachable => writeln!(f, "{idx} := unreachable")?,
                NodeData::CharLiteral => {
                    let lit = self.get_char_lit(main_token);
                    writeln!(f, "{idx} := char({lit})")?
                }
                NodeData::FloatLiteral => {
                    let lit = self.get_float_lit(main_token);
                    writeln!(f, "{idx} := float({lit})")?
                }
                NodeData::IntLiteral => {
                    let lit = self.get_int_lit(main_token);
                    writeln!(f, "{idx} := int({lit})")?
                }
                NodeData::StringLiteral => {
                    let lit = self.get_string_lit(main_token);
                    writeln!(f, "{idx} := string({lit})")?
                }
                NodeData::RawStringLiteral(first, last) => {
                    if first == last {
                        let lit = self.get_raw_string_lit(*first);
                        let lit = lit.trim_suffix("\n").trim_suffix("\r");
                        if lit.len() > 80 {
                            let lit = &lit[..80];
                            writeln!(f, "{idx} := raw_string({lit}...)")?
                        } else {
                            writeln!(f, "{idx} := raw_string({lit})")?
                        }
                    } else {
                        let mut curr = *first;
                        writeln!(f, "{idx} := raw_string(")?;
                        while curr.get() <= last.get() {
                            let lit = self.get_raw_string_lit(curr);
                            let lit = lit.trim_suffix("\n").trim_suffix("\r");
                            if lit.len() > 80 {
                                let lit = &lit[..80];
                                write!(f, "\t{lit}...{}", if curr == *last { ")" } else { "\n" })?;
                            } else {
                                write!(f, "\t{lit}{}", if curr == *last { ")" } else { "\n" })?;
                            }
                            curr = TokenIndex::new(curr.get() + 1);
                        }
                    }
                }
                NodeData::Container(layout_expr, block) => {
                    let container_tok = self.get_token(main_token);
                    let container = container_tok.tag.as_lexeme().unwrap();
                    write!(f, "{idx} := {container}(")?;
                    if let Some(layout_expr) = layout_expr {
                        write!(f, "layout := {layout_expr}, ")?;
                    }
                    writeln!(f, "block := {block})")?;
                }
                NodeData::ContainerConst(layout_expr, block) => {
                    let container_tok = self.get_token(main_token);
                    let container = container_tok.tag.as_lexeme().unwrap();
                    write!(f, "{idx} := {container}(")?;
                    if let Some(layout_expr) = layout_expr {
                        write!(f, "layout := {layout_expr}, ")?;
                    }
                    writeln!(f, "const := true, block := {block})")?;
                }
                NodeData::Namespace(block) => writeln!(f, "{idx} := namespace(block := {block})")?,
                NodeData::Primitive(id, block) => {
                    let id = self.get_string_lit(*id);
                    writeln!(f, "{idx} := #primitive({id}, block := {block})")?
                }
                NodeData::TemplateTypeExprOne(arg_type_expr, type_expr) => {
                    if let Some(arg_type_expr) = arg_type_expr {
                        writeln!(
                            f,
                            "{idx} := with_type(args := [{arg_type_expr}], type := {type_expr}])"
                        )?
                    } else {
                        writeln!(f, "{idx} := with_type(args := [], type := {type_expr})")?
                    }
                }
                NodeData::TemplateTypeExprOneComma(arg_type_expr, type_expr) => writeln!(
                    f,
                    "{idx} := with_type(args := [{arg_type_expr},], type := {type_expr}])"
                )?,
                NodeData::TemplateTypeExprOneDots(arg_type_expr, type_expr) => {
                    if let Some(arg_type_expr) = arg_type_expr {
                        writeln!(
                            f,
                            "{idx} := with_type(args := [{arg_type_expr}, ...], type := {type_expr}])"
                        )?
                    } else {
                        writeln!(f, "{idx} := with_type(args := [...], type := {type_expr})")?
                    }
                }
                NodeData::TemplateTypeExprOneDotsComma(arg_type_expr, type_expr) => {
                    if let Some(arg_type_expr) = arg_type_expr {
                        writeln!(
                            f,
                            "{idx} := with_type(args := [{arg_type_expr}, ...,], type := {type_expr}])"
                        )?
                    } else {
                        writeln!(f, "{idx} := with_type(args := [...,], type := {type_expr})")?
                    }
                }
                NodeData::TemplateTypeExpr(args, type_expr) => {
                    write!(f, "{idx} := with_type(args := [")?;
                    let args = self.get_packed(*args);
                    for (i, arg) in args.enumerate() {
                        let TemplateTypeExprArg {
                            type_expr,
                            has_init,
                        } = self.get_packed(arg);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        if has_init {
                            write!(f, "{type_expr} = ...")?;
                        } else {
                            write!(f, "{type_expr}")?;
                        }
                    }
                    writeln!(f, "], type := {type_expr})")?
                }
                NodeData::TemplateTypeExprComma(args, type_expr) => {
                    write!(f, "{idx} := with_type(args := [")?;
                    let args = self.get_packed(*args);
                    for (i, arg) in args.enumerate() {
                        let TemplateTypeExprArg {
                            type_expr,
                            has_init,
                        } = self.get_packed(arg);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        if has_init {
                            write!(f, "{type_expr} = ...")?;
                        } else {
                            write!(f, "{type_expr}")?;
                        }
                    }
                    writeln!(f, ",], type := {type_expr})")?
                }
                NodeData::TemplateTypeExprDots(args, type_expr) => {
                    write!(f, "{idx} := with_type(args := [")?;
                    let args = self.get_packed(*args);
                    for (i, arg) in args.enumerate() {
                        let TemplateTypeExprArg {
                            type_expr,
                            has_init,
                        } = self.get_packed(arg);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        if has_init {
                            write!(f, "{type_expr} = ...")?;
                        } else {
                            write!(f, "{type_expr}")?;
                        }
                    }
                    writeln!(f, ", ...], type := {type_expr})")?
                }
                NodeData::TemplateTypeExprDotsComma(args, type_expr) => {
                    write!(f, "{idx} := with_type(args := [")?;
                    let args = self.get_packed(*args);
                    for (i, arg) in args.enumerate() {
                        let TemplateTypeExprArg {
                            type_expr,
                            has_init,
                        } = self.get_packed(arg);
                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        if has_init {
                            write!(f, "{type_expr} = ...")?;
                        } else {
                            write!(f, "{type_expr}")?;
                        }
                    }
                    writeln!(f, ", ...,], type := {type_expr})")?
                }
                NodeData::Index(expr, index_expr) => {
                    writeln!(f, "{idx} := index(expr := {expr}, index := {index_expr})")?
                }
                NodeData::Alias(type_expr) => writeln!(f, "{idx} := alias(expr := {type_expr})")?,
                NodeData::Bind1(type_expr, bind_expr) => {
                    if let Some(bind_expr) = bind_expr {
                        writeln!(
                            f,
                            "{idx} := bind(expr := {type_expr}, args := [{bind_expr}])"
                        )?
                    } else {
                        writeln!(f, "{idx} := bind(expr := {type_expr}, args := [])")?
                    }
                }
                NodeData::Bind1Comma(type_expr, bind_expr) => writeln!(
                    f,
                    "{idx} := bind(expr := {type_expr}, args := [{bind_expr},])"
                )?,
                NodeData::Bind(type_expr, exprs) => {
                    write!(f, "{idx} := bind(expr := {type_expr}, args := [")?;
                    let exprs = self.get_packed(*exprs);
                    for (i, expr) in exprs.enumerate() {
                        let expr = self.get_packed(expr);
                        if i == 0 {
                            write!(f, "{expr}")?;
                        } else {
                            write!(f, ", {expr}")?;
                        }
                    }
                    writeln!(f, "])")?;
                }
                NodeData::BindComma(type_expr, exprs) => {
                    write!(f, "{idx} := bind(expr := {type_expr}, args := [")?;
                    let exprs = self.get_packed(*exprs);
                    for (i, expr) in exprs.enumerate() {
                        let expr = self.get_packed(expr);
                        if i == 0 {
                            write!(f, "{expr}")?;
                        } else {
                            write!(f, ", {expr}")?;
                        }
                    }
                    writeln!(f, ",])")?;
                }
                NodeData::Call1(type_expr, arg_expr) => {
                    if let Some(bind_expr) = arg_expr {
                        writeln!(
                            f,
                            "{idx} := call(expr := {type_expr}, args := [{bind_expr}])"
                        )?
                    } else {
                        writeln!(f, "{idx} := call(expr := {type_expr}, args := [])")?
                    }
                }
                NodeData::Call1Comma(type_expr, arg_expr) => writeln!(
                    f,
                    "{idx} := call(expr := {type_expr}, args := [{arg_expr},])"
                )?,
                NodeData::Call(type_expr, exprs) => {
                    write!(f, "{idx} := call(expr := {type_expr}, args := [")?;
                    let exprs = self.get_packed(*exprs);
                    for (i, expr) in exprs.enumerate() {
                        let expr = self.get_packed(expr);
                        if i == 0 {
                            write!(f, "{expr}")?;
                        } else {
                            write!(f, ", {expr}")?;
                        }
                    }
                    writeln!(f, "])")?;
                }
                NodeData::CallComma(type_expr, exprs) => {
                    write!(f, "{idx} := call(expr := {type_expr}, args := [")?;
                    let exprs = self.get_packed(*exprs);
                    for (i, expr) in exprs.enumerate() {
                        let expr = self.get_packed(expr);
                        if i == 0 {
                            write!(f, "{expr}")?;
                        } else {
                            write!(f, ", {expr}")?;
                        }
                    }
                    writeln!(f, ",])")?;
                }

                NodeData::TypeBinarySuffix(expr, ident_token) => {
                    let op_token = self.get_token(main_token);
                    let op = op_token.tag.as_lexeme().unwrap();
                    let ident = self.get_ident(*ident_token);
                    writeln!(
                        f,
                        "{idx} := binary_type_suffix_op(op := {op}, expr := {expr}, ident := {ident})"
                    )?
                }
                NodeData::TypeUnarySuffix(expr) => {
                    let op_token = self.get_token(main_token);
                    let op = op_token.tag.as_lexeme().unwrap();
                    writeln!(
                        f,
                        "{idx} := unary_type_suffix_op(op := {op}, expr := {expr})"
                    )?
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error {
    is_note: bool,
    token: TokenIndex,
    data: ErrorData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorData {
    DocCommentAtContainerEnd,
    DocCommentOnStatement,
    DocCommentBetweenDeclProtos,
    AttributeAtContainerEnd,
    AttributeOnStatement,
    AttributeOnExpression,
    AttributeBeforeThreadLocal,
    ExpectedDeclBlock,
    ExpectedDeclBlockStatement,
    ExpectedStructBlock,
    ExpectedStructBlockStatement,
    ExpectedThreadLocalOrGlobalDeclStatement,
    ExpectedTemplateExpr,
    ExpectedExpr,
    ExpectedPrimaryExpr,
    ExpectedTypeExpr,
    ExpectedTypeExprOrInit,
    ExpectedPrefixExpr,
    DoublePubToken,
    DoubleVarToken,
    DoubleVolatileToken,
    DoubleAlignment,
    PubTokenAfterVarToken,
    StructFieldWithoutTypeOrInitExpr,
    UnexpectedSemicolonAfterBlock,
    ExpectedToken(lexer::Tag),
    InvalidByte(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtraIndex<T: Packable>(NonZero<u32>, PhantomData<fn() -> T>);

impl<T: Packable> ExtraIndex<T> {
    pub const fn new(index: usize) -> Self {
        assert!(index < u32::MAX as usize);
        Self(
            unsafe { NonZero::new_unchecked((index + 1) as u32) },
            PhantomData,
        )
    }

    pub const fn new_u32(index: u32) -> Self {
        assert!(index < u32::MAX);
        Self(unsafe { NonZero::new_unchecked(index + 1) }, PhantomData)
    }

    pub const fn new_optional(index: u32) -> Option<Self> {
        if index == u32::MAX {
            None
        } else {
            Some(Self(
                unsafe { NonZero::new_unchecked(index + 1) },
                PhantomData,
            ))
        }
    }

    pub const fn unwrap_optional(value: Option<Self>) -> u32 {
        match value {
            Some(v) => v.get_u32(),
            None => u32::MAX,
        }
    }

    pub const fn get(self) -> usize {
        (self.0.get() - 1) as usize
    }

    pub const fn get_u32(self) -> u32 {
        self.0.get() - 1
    }
}

impl<T: Packable> packed_stream::DefaultPackable for ExtraIndex<T> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtraIndexRange<T: Packable>(pub ExtraIndex<T>, pub ExtraIndex<T>);

impl<T: Packable> ExtraIndexRange<T> {
    pub const fn new(start: ExtraIndex<T>, end: ExtraIndex<T>) -> Self {
        Self(start, end)
    }

    pub const fn start(&self) -> ExtraIndex<T> {
        self.0
    }

    pub const fn end(&self) -> ExtraIndex<T> {
        self.0
    }
}

impl<T: Packable> Packable for ExtraIndexRange<T> {
    const LEN: usize = <(ExtraIndex<T>, ExtraIndex<T>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.0);
        buffer.write(self.1);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self(buffer.read(), buffer.read())
    }
}

impl<T: Packable> Packable for Option<ExtraIndexRange<T>> {
    const LEN: usize = <(Option<ExtraIndex<T>>, Option<ExtraIndex<T>>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        let (start, end) = match self {
            Some(x) => (Some(x.0), Some(x.1)),
            None => (None, None),
        };
        buffer.write(start);
        buffer.write(end);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let start = buffer.read::<Option<ExtraIndex<T>>>();
        let end = buffer.read::<Option<ExtraIndex<T>>>();
        if let Some((start, end)) = start.zip(end) {
            Some(ExtraIndexRange(start, end))
        } else {
            None
        }
    }
}

impl<T: Packable> Iterator for ExtraIndexRange<T> {
    type Item = ExtraIndex<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.get() > self.1.get() {
            None
        } else {
            let step = const { T::LEN as u32 };
            let index = self.0;
            self.0 = ExtraIndex::new_u32(self.0.get_u32() + step);
            Some(index)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedPair<T: Packable, U: Packable>(pub T, pub U);

impl<T: Packable, U: Packable> Packable for PackedPair<T, U> {
    const LEN: usize = <(T, U) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.0);
        buffer.write(self.1);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self(buffer.read(), buffer.read())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex(NonZero<u32>);

impl NodeIndex {
    pub const ROOT: Self = Self::new(0);

    pub const fn new(index: usize) -> Self {
        assert!(index < u32::MAX as usize);
        Self(unsafe { NonZero::new_unchecked((index + 1) as u32) })
    }

    pub const fn new_u32(index: u32) -> Self {
        assert!(index < u32::MAX);
        Self(unsafe { NonZero::new_unchecked(index + 1) })
    }

    pub const fn new_optional(index: u32) -> Option<Self> {
        if index == u32::MAX {
            None
        } else {
            Some(Self(unsafe { NonZero::new_unchecked(index + 1) }))
        }
    }

    pub const fn unwrap_optional(value: Option<Self>) -> u32 {
        match value {
            Some(v) => v.get_u32(),
            None => u32::MAX,
        }
    }

    pub const fn get(self) -> usize {
        (self.0.get() - 1) as usize
    }

    pub const fn get_u32(self) -> u32 {
        self.0.get() - 1
    }
}

impl packed_stream::DefaultPackable for NodeIndex {}

impl Debug for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NodeIndex").field(&self.get()).finish()
    }
}

impl Display for NodeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.get())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node {
    pub main_token: TokenIndex,
    pub data: NodeData,
}

const _: () = const {
    assert!(size_of::<Node>() == 4 * size_of::<u32>());
    assert!(size_of::<NodeData>() == 3 * size_of::<u32>());
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeData {
    /// The root node which is guaranteed to be at [`NodeIndex.ROOT`].
    /// 1. First attribute/member/field
    /// 2. Last attribute/member/field
    ///
    /// `main_token` is the first token for the source file.
    Root(Option<ExtraIndexRange<NodeIndex>>),
    /// `#![struct]`
    /// 1. Index to the `#` token.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#!` token.
    RootStructAttribute(TokenIndex, TokenIndex),
    /// `#![struct const]`
    /// 1. Index to the `#` token.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#!` token.
    RootStructConstAttribute(TokenIndex, TokenIndex),
    /// `#![struct(expr)]`
    /// 1. Index to the sub-expression.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#!` token.
    RootStructLayoutAttribute(NodeIndex, TokenIndex),
    /// `#![struct(expr) const]`
    /// 1. Index to the sub-expression.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#!` token.
    RootStructLayoutConstAttribute(NodeIndex, TokenIndex),
    /// `#[=expr]`
    /// 1. Index to the sub-expression.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#` token.
    OuterAnnotation(NodeIndex, TokenIndex),
    /// `#![=expr]`
    /// 1. Index to the sub-expression.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#!` token.
    InnerAnnotation(NodeIndex, TokenIndex),
    /// `const expr`
    /// 1. Index to the sub-expression.
    ///
    /// `main_token` is the `const` token.
    Const(NodeIndex),
    /// `thread_local a align(expr) :: generic_expr;`
    /// 1. Align expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ThreadLocalDeclAlign(NodeIndex, NodeIndex),
    /// `thread_local a : type_expr : generic_expr;`
    /// 1. Optional type expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ThreadLocalDeclTypeInit(Option<NodeIndex>, NodeIndex),
    /// `thread_local #[...] a align(expr) : type_expr : generic_expr;`
    /// 1. Declaration prototype
    /// 2. Pair of type expr and init expr
    ///
    /// `main_token` is the identifier token.
    ThreadLocalDecl(
        ExtraIndex<DeclProtoPub>,
        ExtraIndex<PackedPair<Option<NodeIndex>, NodeIndex>>,
    ),
    /// `thread_local #[...] a align(expr_1), ..., #[...] n align(expr_n) : type_expr : generic_expr;`
    /// 1. Declaration prototype list.
    /// 2. Type and init expression.
    ///
    /// `main_token` is the `thread_local` token.
    ThreadLocalDeclDestructure(
        ExtraIndex<ExtraIndexRange<DeclProtoPub>>,
        ExtraIndex<PackedPair<Option<NodeIndex>, NodeIndex>>,
    ),
    /// `a align(expr) :: generic_expr;`
    /// 1. Align expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    GlobalDeclAlign(NodeIndex, NodeIndex),
    /// `a : type_expr : generic_expr;`
    /// 1. Optional type expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    GlobalDeclTypeInit(Option<NodeIndex>, NodeIndex),
    /// `#[...] a align(expr) : type_expr : generic_expr;`
    /// 1. Declaration prototype
    /// 2. Pair of type expr and init expr
    ///
    /// `main_token` is the identifier token.
    GlobalDecl(
        ExtraIndex<DeclProtoPub>,
        ExtraIndex<PackedPair<Option<NodeIndex>, NodeIndex>>,
    ),
    /// `#[...] a align(expr_1), ..., #[...] n align(expr_n) : type_expr : generic_expr;`
    /// 1. Declaration prototype list.
    /// 2. Type and init expression.
    ///
    /// `main_token` is the `;` token.
    GlobalDeclDestructure(
        ExtraIndex<ExtraIndexRange<DeclProtoPub>>,
        ExtraIndex<PackedPair<Option<NodeIndex>, NodeIndex>>,
    ),
    /// `a align(expr) : type_expr`
    /// 1. Optional alignment expression.
    /// 2. Type expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldType(Option<NodeIndex>, NodeIndex),
    /// `a align(expr) : type_expr,`
    /// 1. Optional alignment expression.
    /// 2. Type expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldTypeComma(Option<NodeIndex>, NodeIndex),
    /// `a align(expr) = generic_expr`
    /// 1. Optional alignment expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInit(Option<NodeIndex>, NodeIndex),
    /// `a align(expr) = generic_expr,`
    /// 1. Optional alignment expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInitComma(Option<NodeIndex>, NodeIndex),
    /// `a : type_expr = generic_expr`
    /// 1. Type expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldTypeInit(NodeIndex, NodeIndex),
    /// `a : type_expr = generic_expr,`
    /// 1. Type expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldTypeInitComma(NodeIndex, NodeIndex),
    /// `a align(expr) : type_expr = generic_expr`
    /// 1. Field prototype.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerField(ExtraIndex<StructFieldProto>),
    /// `a align(expr) : type_expr = generic_expr,`
    /// 1. Field prototype.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldComma(ExtraIndex<StructFieldProto>),
    /// `inline a align(expr) : type_expr`
    /// 1. Optional alignment expression.
    /// 2. Type expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineType(Option<NodeIndex>, NodeIndex),
    /// `inline a align(expr) : type_expr,`
    /// 1. Optional alignment expression.
    /// 2. Type expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineTypeComma(Option<NodeIndex>, NodeIndex),
    /// `inline a align(expr) = generic_expr`
    /// 1. Optional alignment expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineInit(Option<NodeIndex>, NodeIndex),
    /// `inline a align(expr) = generic_expr,`
    /// 1. Optional alignment expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineInitComma(Option<NodeIndex>, NodeIndex),
    /// `inline a : type_expr = generic_expr`
    /// 1. Type expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineTypeInit(NodeIndex, NodeIndex),
    /// `inline a : type_expr = generic_expr,`
    /// 1. Type expression.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineTypeInitComma(NodeIndex, NodeIndex),
    /// `inline a align(expr) : type_expr = generic_expr`
    /// 1. Field prototype.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInline(ExtraIndex<StructFieldProto>),
    /// `inline a align(expr) : type_expr = generic_expr,`
    /// 1. Field prototype.
    /// 2. Init expression.
    ///
    /// `main_token` is the identifier token.
    ContainerFieldInlineComma(ExtraIndex<StructFieldProto>),
    /// `impl condition {}`
    /// 1. Condition expression
    /// 2. Declaration block.
    ///
    /// `main_token` is the `impl` token.
    ImplBlock(NodeIndex, NodeIndex),
    /// `#[...] impl condition {}`
    /// 1. Impl block
    ///
    /// `main_token` is the `impl` token.
    ImplBlockAttrs(ExtraIndex<ImplBlock>),
    /// `{lhs rhs}`
    /// 1. First statement.
    /// 2. Second statement.
    ///
    /// `main_token` is the `{` token.
    BlockTwo(Option<NodeIndex>, Option<NodeIndex>),
    /// `{lhs rhs};`
    /// 1. First statement.
    /// 2. Second statement.
    ///
    /// `main_token` is the `{` token.
    BlockTwoSemicolon(Option<NodeIndex>, Option<NodeIndex>),
    /// `{a b}`
    /// 1. Index to the first statement.
    /// 2. Index to the last statement.
    ///
    /// `main_token` is the `{` token.
    Block(ExtraIndexRange<NodeIndex>),
    /// `{a b};`
    /// 1. Index to the first statement.
    /// 2. Index to the last statement.
    ///
    /// `main_token` is the `{` token.
    BlockSemicolon(ExtraIndexRange<NodeIndex>),
    /// `with[] where(expr) expr`
    /// 1. Optional where expression.
    /// 2. Template expression.
    ///
    /// `main_token` is the `with` token.
    TemplateExprSimple(Option<NodeIndex>, NodeIndex),
    /// `with[x: t = expr] where(expr) expr`
    /// 1. Template prototype.
    /// 2. Template expression.
    ///
    /// `main_token` is the `with` token.
    TemplateExpr(ExtraIndex<TemplateExprProto>, NodeIndex),
    /// `with[x: t = expr,] where(expr) expr`
    /// 1. Template prototype.
    /// 2. Template expression.
    ///
    /// `main_token` is the `with` token.
    TemplateExprComma(ExtraIndex<TemplateExprProto>, NodeIndex),
    /// `expr range_op expr`
    /// 1. First optional sub-expression.
    /// 2. Second optional sub-expression.
    ///
    /// `main_token` is the range_op token.
    Range(Option<NodeIndex>, Option<NodeIndex>),
    /// `expr op expr`
    /// 1. First sub-expression.
    /// 2. Second sub-expression.
    ///
    /// `main_token` is the op token.
    Binary(NodeIndex, NodeIndex),
    /// `op expr`
    /// 1. Sub-expression.
    ///
    /// `main_token` is the op token.
    Unary(NodeIndex),
    /// `asm(expr) { expr }`
    /// 1. Clobbers expression
    /// 2. Instructions expression
    ///
    /// `main_token` is the `asm` token.
    AsmSimple(NodeIndex, NodeIndex),
    /// `asm(expr) volatile { expr }`
    /// 1. Clobbers expression
    /// 2. Instructions expression
    ///
    /// `main_token` is the `asm` token.
    AsmVolatileSimple(NodeIndex, NodeIndex),
    /// `asm[...](..., expr) -> (...) { expr }`
    /// 1. Asm prototype
    /// 2. Instructions expression
    ///
    /// `main_token` is the `asm` token.
    Asm(ExtraIndex<AsmProto>, NodeIndex),
    /// `asm[...](..., expr) volatile -> (...) { expr }`
    /// 1. Asm prototype
    /// 2. Instructions expression
    ///
    /// `main_token` is the `asm` token.
    AsmVolatile(ExtraIndex<AsmProto>, NodeIndex),
    /// `jump_op label expr`
    /// 1. Jump label
    /// 2. Jump value
    ///
    /// `main_token` is the jump_op token.
    Jump(Option<TokenIndex>, Option<NodeIndex>),
    /// `fn[...](...) modifier -> ret_type where(...) { ... }`
    /// 1. Funtion prototype
    /// 2. Expression block.
    ///
    /// `main_token` is the `fn` token.
    Fn(ExtraIndex<FnProto>, NodeIndex),
    // TODO: If
    // TODO: For
    // TODO: While
    /// `expr {...}`
    /// 1. Expression.
    /// 2. Init list
    ///
    /// `main_token` is unused.
    ExprInit(NodeIndex, NodeIndex),
    /// `token {...}`
    /// 1. Init list
    ///
    /// `main_token` is the token token.
    TokenInit(NodeIndex),
    /// `{lhs rhs}`
    /// 1. First expression.
    /// 2. Second expression.
    ///
    /// `main_token` is the `{` token.
    ExprInitListTwo(Option<NodeIndex>, Option<NodeIndex>),
    /// `{lhs rhs,}`
    /// 1. First expression.
    /// 2. Second expression.
    ///
    /// `main_token` is the `{` token.
    ExprInitListTwoComma(Option<NodeIndex>, Option<NodeIndex>),
    /// `{lhs rhs}`
    /// 1. Expression list
    ///
    /// `main_token` is the `{` token.
    ExprInitList(ExtraIndexRange<NodeIndex>),
    /// `{lhs rhs,}`
    /// 1. Expression list
    ///
    /// `main_token` is the `{` token.
    ExprInitListComma(ExtraIndexRange<NodeIndex>),
    /// `{.a=expr, .b=expr}`
    /// 1. First field.
    /// 2. Second field.
    ///
    /// `main_token` is the `{` token.
    FieldInitListTwo(ExtraIndex<FieldInit>, Option<ExtraIndex<FieldInit>>),
    /// `{.a=expr, .b=expr,}`
    /// 1. First field.
    /// 2. Second field.
    ///
    /// `main_token` is the `{` token.
    FieldInitListTwoComma(ExtraIndex<FieldInit>, Option<ExtraIndex<FieldInit>>),
    /// `{.a=expr, ..., .z=expr}`
    /// 1. Field list.
    ///
    /// `main_token` is the `{` token.
    FieldInitList(ExtraIndexRange<FieldInit>),
    /// `{.a=expr, ..., .z=expr,}`
    /// 1. Field list.
    ///
    /// `main_token` is the `{` token.
    FieldInitListComma(ExtraIndexRange<FieldInit>),
    /// `lhs!rhs`
    /// 1. Left sub-expression.
    /// 2. Right sub-expression.
    ///
    /// `main_token` is the `!` token.
    ErrorUnionType(NodeIndex, NodeIndex),
    /// `type_expr...`
    /// 1. Type expression
    ///
    /// `main_token` is the `...` token.
    Pack(NodeIndex),
    /// `?expr`
    /// 1. Sub-expression.
    ///
    /// `main_token` is the `?` token.
    OptionType(NodeIndex),
    /// `[:sentinel]type_expr`
    /// 1. Optional sentinel.
    /// 2. Type expression.
    ///
    /// `main_token` is the `[` token.
    Slice(Option<NodeIndex>, NodeIndex),
    /// `*type_expr`
    /// 1. Pointer type.
    /// 2. Type expression.
    ///
    /// `main_token` is the `*` token.
    SinglePointerSimple(PointerType, NodeIndex),
    /// `* align(...) type_expr`
    /// 1. Prefix.
    /// 2. Type expression.
    ///
    /// `main_token` is the `*` token.
    SinglePointer(ExtraIndex<SinglePointerPrefix>, NodeIndex),
    /// `[*]type_expr`
    /// 1. Pointer type.
    /// 2. Type expression.
    ///
    /// `main_token` is the `[` token.
    MultiPointerSimple(PointerType, NodeIndex),
    /// `[*:sentinel] align(...) type_expr`
    /// 1. Prefix.
    /// 2. Type expression.
    ///
    /// `main_token` is the `[` token.
    MultiPointer(ExtraIndex<MultiPointerPrefix>, NodeIndex),
    /// `[[len]]type_expr`
    /// 1. Length expression.
    /// 2. Type expression.
    ///
    /// `main_token` is the first `[` token.
    Vector(NodeIndex, NodeIndex),
    /// `[[rows, columns]]type_expr`
    /// 1. Row-Column-length expression pair.
    /// 2. Type expression.
    ///
    /// `main_token` is the first `[` token.
    MatrixSimple(ExtraIndex<PackedPair<NodeIndex, NodeIndex>>, NodeIndex),
    /// `[[rows, columns]:layout]type_expr`
    /// 1. Matrix info.
    /// 2. Type expression.
    ///
    /// `main_token` is the first `[` token.
    Matrix(ExtraIndex<Matrix>, NodeIndex),
    /// `[len]type_expr`
    /// 1. Length expression.
    /// 2. Type expression.
    ///
    /// `main_token` is the `[` token.
    ArraySimple(NodeIndex, NodeIndex),
    /// `[len: sentinel]type_expr`
    /// 1. Length and sentinel expression pair.
    /// 2. Type expression.
    ///
    /// `main_token` is the `[` token.
    Array(ExtraIndex<PackedPair<NodeIndex, NodeIndex>>, NodeIndex),
    /// `.enum_literal`
    /// 1. Literal token
    ///
    /// `main_token` is the `.` token.
    EnumLiteral(TokenIndex),
    // TODO: LabeledTypeExpr
    // TODO: Tuple
    // TODO: Group
    /// `a`
    ///
    /// `main_token` is the identifier token.
    Ident,
    /// `@a`
    ///
    /// `main_token` is the identifier token.
    CoreIdent,
    /// `#a`
    ///
    /// `main_token` is the identifier token.
    BuiltinIdent,
    /// `impl type_expr`
    /// 1. Type expression
    ///
    /// `main_token` is the `impl` token.
    ImplExpr(NodeIndex),
    /// `Self`
    ///
    /// `main_token` is the `Self` token.
    SelfType,
    /// `self`
    ///
    /// `main_token` is the `self` token.
    SelfIdent,
    /// `unreachable`
    ///
    /// `main_token` is the `unreachable` token.
    Unreachable,
    /// `'c'`
    ///
    /// `main_token` is the literal token.
    CharLiteral,
    /// `0.1234`
    ///
    /// `main_token` is the literal token.
    FloatLiteral,
    /// `1234`
    ///
    /// `main_token` is the literal token.
    IntLiteral,
    /// `"string"`
    ///
    /// `main_token` is the literal token.
    StringLiteral,
    /// `\\string`
    /// 1. First token
    /// 2. Last token
    ///
    /// `main_token` is the first token.
    RawStringLiteral(TokenIndex, TokenIndex),
    // TODO: Fnptr
    /// `container(expr) {...}`
    /// 1. Optional layout expr
    /// 2. Block
    ///
    /// `main_token` is the container token.
    Container(Option<NodeIndex>, NodeIndex),
    /// `container(expr) const {...}`
    /// 1. Optional layout expr
    /// 2. Block
    ///
    /// `main_token` is the container token.
    ContainerConst(Option<NodeIndex>, NodeIndex),
    /// `namespace {...}`
    /// 1. Declaration block
    ///
    /// `main_token` is the `namespace` token.
    Namespace(NodeIndex),
    /// `#primitive("...", {...})`
    /// 1. String token
    /// 2. Declaration block
    ///
    /// `main_token` is the `#primitive` token.
    Primitive(TokenIndex, NodeIndex),
    /// `with[type_expr] -> expr`
    /// 1. Optional type expression.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprOne(Option<NodeIndex>, NodeIndex),
    /// `with[type_expr,] -> expr`
    /// 1. Type expression.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprOneComma(NodeIndex, NodeIndex),
    /// `with[type_expr, ...] -> expr`
    /// 1. Optional type expression.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprOneDots(Option<NodeIndex>, NodeIndex),
    /// `with[type_expr, ...,] -> expr`
    /// 1. Optional type expression.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprOneDotsComma(Option<NodeIndex>, NodeIndex),
    /// `with[type_expr = ..., ..., type_expr = ...] -> expr`
    /// 1. List of args.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExpr(ExtraIndex<ExtraIndexRange<TemplateTypeExprArg>>, NodeIndex),
    /// `with[type_expr = ..., ..., type_expr = ...] -> expr`
    /// 1. List of args.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprComma(ExtraIndex<ExtraIndexRange<TemplateTypeExprArg>>, NodeIndex),
    /// `with[type_expr = ..., ..., type_expr = ..., ...] -> expr`
    /// 1. List of args.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprDots(ExtraIndex<ExtraIndexRange<TemplateTypeExprArg>>, NodeIndex),
    /// `with[type_expr = ..., ..., type_expr = ..., ...] -> expr`
    /// 1. List of args.
    /// 2. Template type expression.
    ///
    /// `main_token` is the `with` token.
    TemplateTypeExprDotsComma(ExtraIndex<ExtraIndexRange<TemplateTypeExprArg>>, NodeIndex),
    /// `type_expr[expr]`
    /// 1. Type expression
    /// 2. Index expression
    ///
    /// `main_token` is the `]` token.
    Index(NodeIndex, NodeIndex),
    /// `type_expr::[...]`
    /// 1. Type expression.
    ///
    /// `main_token` is the `]` token.
    Alias(NodeIndex),
    /// `type_expr::[expr]`
    /// 1. Type expression.
    /// 2. Optional expression.
    ///
    /// `main_token` is the `]` token.
    Bind1(NodeIndex, Option<NodeIndex>),
    /// `type_expr::[expr,]`
    /// 1. Type expression
    /// 2. Expression
    ///
    /// `main_token` is the `]` token.
    Bind1Comma(NodeIndex, NodeIndex),
    /// `type_expr::[expr, ..., expr]`
    /// 1. Type expression
    /// 2. Expression list
    ///
    /// `main_token` is the `]` token.
    Bind(NodeIndex, ExtraIndex<ExtraIndexRange<NodeIndex>>),
    /// `type_expr::[expr, ..., expr,]`
    /// 1. Type expression
    /// 2. Expression list
    ///
    /// `main_token` is the `]` token.
    BindComma(NodeIndex, ExtraIndex<ExtraIndexRange<NodeIndex>>),
    /// `type_expr(expr)`
    /// 1. Type expression
    /// 2. Optional expression
    ///
    /// `main_token` is the `)` token.
    Call1(NodeIndex, Option<NodeIndex>),
    /// `type_expr(expr)`
    /// 1. Type expression
    /// 2. Expression
    ///
    /// `main_token` is the `)` token.
    Call1Comma(NodeIndex, NodeIndex),
    /// `type_expr(expr, ..., expr)`
    /// 1. Type expression
    /// 2. Expression list
    ///
    /// `main_token` is the `)` token.
    Call(NodeIndex, ExtraIndex<ExtraIndexRange<NodeIndex>>),
    /// `type_expr(expr, ..., expr,)`
    /// 1. Type expression
    /// 2. Expression list
    ///
    /// `main_token` is the `)` token.
    CallComma(NodeIndex, ExtraIndex<ExtraIndexRange<NodeIndex>>),
    /// `type_expr op ident`
    /// 1. Type expression
    /// 2. Identifier
    ///
    /// `main_token` is the op token.
    TypeBinarySuffix(NodeIndex, TokenIndex),
    /// `type_expr op`
    /// 1. Type expression
    ///
    /// `main_token` is the op token.
    TypeUnarySuffix(NodeIndex),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclProtoPub {
    attrs: Option<ExtraIndexRange<NodeIndex>>,
    is_pub: bool,
    is_var: bool,
    ident: TokenIndex,
    align_expr: Option<NodeIndex>,
}

impl Packable for DeclProtoPub {
    const LEN: usize = <(BitPacked<(bool, bool)>, TokenIndex, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.attrs);
        buffer.write(BitPacked::pack_bits((self.is_pub, self.is_var)));
        buffer.write(self.ident);
        buffer.write(self.align_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let attrs = buffer.read();
        let (is_pub, is_var) = buffer.read::<BitPacked<(bool, bool)>>().unpack();
        let ident = buffer.read();
        let align_expr = buffer.read();
        Self {
            attrs,
            is_pub,
            is_var,
            ident,
            align_expr,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructFieldProto {
    attrs: Option<ExtraIndexRange<NodeIndex>>,
    align_expr: Option<NodeIndex>,
    type_expr: Option<NodeIndex>,
    init_expr: Option<NodeIndex>,
}

impl Packable for StructFieldProto {
    const LEN: usize = <(
        Option<ExtraIndexRange<NodeIndex>>,
        Option<NodeIndex>,
        Option<NodeIndex>,
        Option<NodeIndex>,
    ) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.attrs);
        buffer.write(self.align_expr);
        buffer.write(self.type_expr);
        buffer.write(self.init_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            attrs: buffer.read(),
            align_expr: buffer.read(),
            type_expr: buffer.read(),
            init_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImplBlock {
    attrs: ExtraIndexRange<NodeIndex>,
    cond_expr: NodeIndex,
    decl_block: NodeIndex,
}

impl Packable for ImplBlock {
    const LEN: usize = <(ExtraIndexRange<NodeIndex>, NodeIndex, NodeIndex) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.attrs);
        buffer.write(self.cond_expr);
        buffer.write(self.decl_block);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            attrs: buffer.read(),
            cond_expr: buffer.read(),
            decl_block: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateExprArg {
    ident: TokenIndex,
    type_expr: NodeIndex,
    init_expr: Option<NodeIndex>,
}

impl Packable for TemplateExprArg {
    const LEN: usize = <(TokenIndex, NodeIndex, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.ident);
        buffer.write(self.type_expr);
        buffer.write(self.init_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            ident: buffer.read(),
            type_expr: buffer.read(),
            init_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateExprProto {
    args: ExtraIndexRange<TemplateExprArg>,
    where_expr: Option<NodeIndex>,
}

impl Packable for TemplateExprProto {
    const LEN: usize = <(ExtraIndexRange<TemplateExprArg>, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.args);
        buffer.write(self.where_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            args: buffer.read(),
            where_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsmCapture {
    ident: TokenIndex,
    type_expr: Option<NodeIndex>,
    init_expr: Option<NodeIndex>,
}

impl Packable for AsmCapture {
    const LEN: usize = <(TokenIndex, Option<NodeIndex>, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.ident);
        buffer.write(self.type_expr);
        buffer.write(self.init_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            ident: buffer.read(),
            type_expr: buffer.read(),
            init_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsmInput {
    ident: TokenIndex,
    constraint: TokenIndex,
}

impl Packable for AsmInput {
    const LEN: usize = <(TokenIndex, TokenIndex) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.ident);
        buffer.write(self.constraint);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            ident: buffer.read(),
            constraint: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsmOutput {
    ident: TokenIndex,
    type_expr: NodeIndex,
    constraint: TokenIndex,
}

impl Packable for AsmOutput {
    const LEN: usize = <(TokenIndex, NodeIndex, TokenIndex) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.ident);
        buffer.write(self.type_expr);
        buffer.write(self.constraint);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            ident: buffer.read(),
            type_expr: buffer.read(),
            constraint: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AsmProto {
    has_trailing_capture_comma: bool,
    has_trailing_input_comma: bool,
    clobbers_expr: NodeIndex,
    captures: Option<ExtraIndexRange<AsmCapture>>,
    inputs: Option<ExtraIndexRange<AsmInput>>,
    outputs: Option<ExtraIndexRange<AsmOutput>>,
}

impl Packable for AsmProto {
    const LEN: usize = <(
        BitPacked<(bool, bool)>,
        Option<ExtraIndexRange<AsmCapture>>,
        Option<ExtraIndexRange<AsmInput>>,
        Option<ExtraIndexRange<AsmOutput>>,
    ) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits((
            self.has_trailing_capture_comma,
            self.has_trailing_input_comma,
        )));
        buffer.write(self.clobbers_expr);
        buffer.write(self.captures);
        buffer.write(self.inputs);
        buffer.write(self.outputs);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let (has_trailing_capture_comma, has_trailing_input_comma) =
            buffer.read::<BitPacked<(bool, bool)>>().unpack();
        Self {
            has_trailing_capture_comma,
            has_trailing_input_comma,
            clobbers_expr: buffer.read(),
            captures: buffer.read(),
            inputs: buffer.read(),
            outputs: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnContext {
    is_optional: bool,
    ctx_token: TokenIndex,
    type_expr: Option<NodeIndex>,
}

impl Packable for FnContext {
    const LEN: usize = <(bool, TokenIndex, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.is_optional);
        buffer.write(self.ctx_token);
        buffer.write(self.type_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            is_optional: buffer.read(),
            ctx_token: buffer.read(),
            type_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnCapture {
    ident: TokenIndex,
    type_expr: Option<NodeIndex>,
    init_expr: Option<NodeIndex>,
}

impl Packable for FnCapture {
    const LEN: usize = <(TokenIndex, Option<NodeIndex>, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.ident);
        buffer.write(self.type_expr);
        buffer.write(self.init_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            ident: buffer.read(),
            type_expr: buffer.read(),
            init_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnReceiver {
    modifier: FnArgModifier,
    is_var: bool,
    pointer_type: Option<PointerType>,
    self_token: TokenIndex,
}

impl Packable for FnReceiver {
    const LEN: usize = <(
        BitPacked<(FnArgModifier, bool, Option<PointerType>)>,
        TokenIndex,
    ) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits((
            self.modifier,
            self.is_var,
            self.pointer_type,
        )));
        buffer.write(self.self_token);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let (modifier, is_var, pointer_type) = buffer
            .read::<BitPacked<(FnArgModifier, bool, Option<PointerType>)>>()
            .unpack();
        Self {
            modifier,
            is_var,
            pointer_type,
            self_token: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnArg {
    modifier: FnArgModifier,
    is_var: bool,
    ident: TokenIndex,
    type_expr: NodeIndex,
    default_expr: Option<NodeIndex>,
}

impl Packable for FnArg {
    const LEN: usize = <(
        BitPacked<(FnArgModifier, bool)>,
        TokenIndex,
        NodeIndex,
        Option<NodeIndex>,
    ) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits((self.modifier, self.is_var)));
        buffer.write(self.ident);
        buffer.write(self.type_expr);
        buffer.write(self.default_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let (modifier, is_var) = buffer.read::<BitPacked<(FnArgModifier, bool)>>().unpack();
        Self {
            modifier,
            is_var,
            ident: buffer.read(),
            type_expr: buffer.read(),
            default_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FnProto {
    has_trailing_capture_comma: bool,
    has_trailing_arg_comma: bool,
    modifier: FnModifier,
    context: Option<ExtraIndex<FnContext>>,
    captures: Option<ExtraIndexRange<FnCapture>>,
    receiver: Option<ExtraIndex<FnReceiver>>,
    args: Option<ExtraIndexRange<FnArg>>,
    call_conv_expr: Option<NodeIndex>,
    return_type_expr: Option<NodeIndex>,
    where_expr: Option<NodeIndex>,
}

impl Packable for FnProto {
    const LEN: usize = <(
        BitPacked<(bool, bool, FnModifier)>,
        Option<ExtraIndex<FnContext>>,
        Option<ExtraIndexRange<FnCapture>>,
        Option<ExtraIndex<FnReceiver>>,
        Option<ExtraIndexRange<FnArg>>,
        Option<NodeIndex>,
        Option<NodeIndex>,
        Option<NodeIndex>,
    ) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits((
            self.has_trailing_capture_comma,
            self.has_trailing_arg_comma,
            self.modifier,
        )));
        buffer.write(self.context);
        buffer.write(self.captures);
        buffer.write(self.receiver);
        buffer.write(self.args);
        buffer.write(self.call_conv_expr);
        buffer.write(self.return_type_expr);
        buffer.write(self.where_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let (has_trailing_capture_comma, has_trailing_arg_comma, modifier) = buffer
            .read::<BitPacked<(bool, bool, FnModifier)>>()
            .unpack();
        Self {
            has_trailing_capture_comma,
            has_trailing_arg_comma,
            modifier,
            context: buffer.read(),
            captures: buffer.read(),
            receiver: buffer.read(),
            args: buffer.read(),
            call_conv_expr: buffer.read(),
            return_type_expr: buffer.read(),
            where_expr: buffer.read(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FnArgModifier {
    None = 0,
    /// const
    Const = 1,
    /// no_alias
    NoAlias = 2,
}

impl packed_stream::DefaultPackable for FnArgModifier {}

impl packed_stream::BitPackable for FnArgModifier {
    const BITS: usize = 2;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        debug_assert!(value < Self::NoAlias as u32);
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FnModifier {
    None = 0,
    /// const
    Const = 1,
    /// inline
    Inline = 2,
    /// no_inline
    NoInline = 3,
}

impl packed_stream::DefaultPackable for FnModifier {}

impl packed_stream::BitPackable for FnModifier {
    const BITS: usize = 2;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        debug_assert!(value < Self::NoInline as u32);
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerType {
    /// *
    Default = 0,
    /// *var
    Var = 1,
    /// *?var
    OptVar = 2,
    /// *volatile
    Volatile = 3,
    /// *?volatile
    OptVolatile = 4,
    /// *var volatile
    VarVolatile = 5,
    /// *?var volatile
    OptVarVolatile = 6,
    /// *var ?volatile
    VarOptVolatile = 7,
    /// *?var ?volatile
    OptVarOptVolatile = 8,
}

impl packed_stream::DefaultPackable for PointerType {}

impl packed_stream::BitPackable for PointerType {
    const BITS: usize = 4;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        debug_assert!(value < Self::OptVarOptVolatile as u32);
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldInit {
    ident: TokenIndex,
    init_expr: NodeIndex,
}

impl Packable for FieldInit {
    const LEN: usize = <(TokenIndex, NodeIndex) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.ident);
        buffer.write(self.init_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            ident: buffer.read(),
            init_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SinglePointerPrefix {
    pointer_type: PointerType,
    align_expr: NodeIndex,
}

impl Packable for SinglePointerPrefix {
    const LEN: usize = <(PointerType, NodeIndex) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.pointer_type);
        buffer.write(self.align_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            pointer_type: buffer.read(),
            align_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MultiPointerPrefix {
    pointer_type: PointerType,
    sentinel_expr: Option<NodeIndex>,
    align_expr: Option<NodeIndex>,
}

impl Packable for MultiPointerPrefix {
    const LEN: usize = <(PointerType, Option<NodeIndex>, Option<NodeIndex>) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.pointer_type);
        buffer.write(self.sentinel_expr);
        buffer.write(self.align_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            pointer_type: buffer.read(),
            sentinel_expr: buffer.read(),
            align_expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Matrix {
    rows: NodeIndex,
    cols: NodeIndex,
    layout: NodeIndex,
}

impl Packable for Matrix {
    const LEN: usize = <(NodeIndex, NodeIndex, NodeIndex) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.rows);
        buffer.write(self.cols);
        buffer.write(self.layout);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            rows: buffer.read(),
            cols: buffer.read(),
            layout: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateTypeExprArg {
    type_expr: NodeIndex,
    has_init: bool,
}

impl Packable for TemplateTypeExprArg {
    const LEN: usize = <(NodeIndex, bool) as Packable>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.type_expr);
        buffer.write(self.has_init);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            type_expr: buffer.read(),
            has_init: buffer.read(),
        }
    }
}

struct Parser<'a> {
    _source: &'a [u8],
    tokens: &'a [Token],
    index: TokenIndex,
    errors: Vec<Error>,
    nodes: Vec<Node>,
    extra_data: Vec<u32>,
}

#[allow(dead_code)]
impl Parser<'_> {
    fn next(&mut self) -> TokenIndex {
        let idx = self.index;
        self.index = TokenIndex::new(self.index.get() + 1);
        idx
    }

    fn rollback(&mut self, positions: u32) {
        self.index = TokenIndex::new(self.index.get() - positions);
    }

    fn eat_token(&mut self, tag: lexer::Tag) -> bool {
        if self.peek(tag) {
            self.next();
            true
        } else {
            false
        }
    }

    fn take_token(&mut self, tag: lexer::Tag) -> Option<TokenIndex> {
        if self.peek(tag) {
            Some(self.next())
        } else {
            None
        }
    }

    fn expect_token(&mut self, tag: lexer::Tag) -> Result<TokenIndex, ()> {
        let index = self.index.get() as usize;
        match self.tokens.get(index) {
            Some(t) if t.tag == tag => Ok(self.next()),
            Some(_) => {
                self.fail_expected(tag)?;
                unreachable!()
            }
            None => panic!("out of bounds"),
        }
    }

    fn tag(&self) -> lexer::Tag {
        let index = self.index.get() as usize;
        self.tokens[index].tag
    }

    fn tag_offset(&self, offset: usize) -> lexer::Tag {
        let index = self.index.get() as usize;
        self.tokens[index + offset].tag
    }

    fn peek(&self, tag: lexer::Tag) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index) {
            Some(t) => t.tag == tag,
            None => false,
        }
    }

    fn peek_any<const N: usize>(&self, tag: [lexer::Tag; N]) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index) {
            Some(t) => tag.into_iter().any(|tag| t.tag == tag),
            None => false,
        }
    }

    fn peek2(&self, tag: lexer::Tag) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index + 1) {
            Some(t) => t.tag == tag,
            None => false,
        }
    }

    fn peek2_any<const N: usize>(&self, tag: [lexer::Tag; N]) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index + 1) {
            Some(t) => tag.into_iter().any(|tag| t.tag == tag),
            None => false,
        }
    }

    fn peek3(&self, tag: lexer::Tag) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index + 2) {
            Some(t) => t.tag == tag,
            None => false,
        }
    }

    fn peek3_any<const N: usize>(&self, tag: [lexer::Tag; N]) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index + 2) {
            Some(t) => tag.into_iter().any(|tag| t.tag == tag),
            None => false,
        }
    }

    fn peek_offset(&self, tag: lexer::Tag, offset: usize) -> bool {
        let index = self.index.get() as usize;
        match self.tokens.get(index + offset) {
            Some(t) => t.tag == tag,
            None => false,
        }
    }

    fn warn(&mut self, data: ErrorData) {
        self.warn_msg(Error {
            is_note: false,
            token: self.index,
            data,
        });
    }

    fn warn_expected(&mut self, tag: lexer::Tag) {
        self.warn_msg(Error {
            is_note: false,
            token: self.index,
            data: ErrorData::ExpectedToken(tag),
        })
    }

    fn warn_msg(&mut self, msg: Error) {
        self.errors.push(msg);
    }

    fn fail(&mut self, data: ErrorData) -> Result<std::convert::Infallible, ()> {
        self.fail_msg(Error {
            is_note: false,
            token: self.index,
            data,
        })
    }

    fn fail_expected(&mut self, tag: lexer::Tag) -> Result<std::convert::Infallible, ()> {
        self.fail_msg(Error {
            is_note: false,
            token: self.index,
            data: ErrorData::ExpectedToken(tag),
        })
    }

    fn fail_msg(&mut self, msg: Error) -> Result<std::convert::Infallible, ()> {
        self.warn_msg(msg);
        Err(())
    }

    fn push_node(&mut self, main_token: TokenIndex, node: NodeData) -> NodeIndex {
        let idx = NodeIndex::new(self.nodes.len());
        self.nodes.push(Node {
            main_token,
            data: node,
        });
        idx
    }

    fn push_packed<T: Packable>(&mut self, value: T) -> ExtraIndex<T> {
        let idx = ExtraIndex::new(self.extra_data.len());
        let mut stream = PackedStreamWriter::new(&mut self.extra_data);
        stream.write(value);
        idx
    }

    fn push_optional_packed<T: Packable>(&mut self, value: Option<T>) -> Option<ExtraIndex<T>> {
        value.map(|v| self.push_packed(v))
    }

    fn push_packed_list<T: Packable>(&mut self, values: &[T]) -> Option<ExtraIndexRange<T>> {
        if values.is_empty() {
            return None;
        }

        let start = ExtraIndex::new(self.extra_data.len());
        let mut stream = PackedStreamWriter::new(&mut self.extra_data);
        stream.write_slice(values);
        let end = ExtraIndex::new(self.extra_data.len() - 1);
        Some(ExtraIndexRange(start, end))
    }
}

fn parse_root(p: &mut Parser<'_>) -> Result<(), ()> {
    p.push_node(TokenIndex::new(0), NodeData::Root(None));
    let mut root_nodes = vec![];

    // Parse all annotations and attributes.
    let mut parse_as_struct = false;
    loop {
        if p.eat_token(Token!(#<inner_doc_comment>)) {
            continue;
        }
        if let Some(annotation) = parse_inner_annotation(p)? {
            root_nodes.push(annotation);
            continue;
        }
        match parse_root_attribute(p)? {
            Some(attribute) => {
                root_nodes.push(attribute);
                parse_as_struct = true
            }
            None => break,
        };
    }

    let members = if parse_as_struct {
        parse_struct_block_members(p, true)?
    } else {
        parse_decl_block_members(p, true)?
    };
    root_nodes.extend(members);

    p.nodes[0].data = NodeData::Root(p.push_packed_list(&root_nodes));
    Ok(())
}

// Blocks

/// DeclBlock <- { DeclBlockContents }
fn expect_decl_block(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let start = match p.take_token(Token!('{')) {
        Some(token) => token,
        None => {
            p.fail(ErrorData::ExpectedDeclBlock)?;
            unreachable!();
        }
    };

    let members = parse_decl_block_members(p, false)?;
    p.expect_token(Token!('}'))?;

    if members.len() < 2 {
        let first = members.first().copied();
        let second = members.get(1).copied();
        Ok(p.push_node(start, NodeData::BlockTwo(first, second)))
    } else {
        let members = p.push_packed_list(&members).unwrap();
        Ok(p.push_node(start, NodeData::Block(members)))
    }
}

/// DeclBlockContents <- InnerAttribute* DeclBlockStatement*
/// DeclBlockStatement
///     <- KEYWORD_const ExprBlock
///      / ContainerDeclStatement
fn parse_decl_block_members(p: &mut Parser<'_>, is_root: bool) -> Result<Vec<NodeIndex>, ()> {
    let mut nodes = vec![];
    while let Some(attr) = parse_inner_attribute(p)? {
        nodes.push(attr);
    }

    loop {
        match p.tag() {
            Token!(#<eof>) => {
                assert!(is_root);
                break;
            }
            Token!('}') => {
                if is_root {
                    p.fail_expected(Token!(#<eof>))?;
                }
                break;
            }
            Token!(const) => nodes.push(expect_const_block(p)?),
            Token!(thread_local) => nodes.push(expect_thread_local_decl_statement(p)?),
            Token!(pub) | Token!(var) | Token!(#<identifier>) => {
                nodes.push(expect_global_decl_statement(p, vec![])?)
            }
            Token!(#) | Token!(#<outer_doc_comment>) => nodes.push(expect_decl_statement(p)?),
            _ => {
                p.fail(ErrorData::ExpectedDeclBlockStatement)?;
                unreachable!()
            }
        }
    }

    Ok(nodes)
}

/// ExprBlock <- LBRACE ExprBlockContents RBRACE
/// ExprBlockContents <- ExprBlockStatement*
/// ExprBlockStatement
///    <- KEYWORD_const ExprBlock
///     / DeclStatement
///     / KEYWORD_defer BlockExprStatement
///     / KEYWORD_cont_defer BlockExprStatement
///     / KEYWORD_err_defer BlockExprStatement
///     / IfStatement
///     / LabeledStatement
///     / AssignExpr
fn expect_expr_block(p: &mut Parser<'_>, warn_on_semicolon: bool) -> Result<NodeIndex, ()> {
    let start = p.expect_token(Token!('{'))?;

    let mut scratch: Vec<NodeIndex> = vec![];
    loop {
        if p.eat_token(Token!('}')) {
            break;
        } else if p.peek(Token!(const)) && p.peek2(Token!('{')) {
            scratch.push(expect_const_block(p)?);
        } else {
            todo!()
        }
    }

    let node_idx = NodeIndex::new(p.nodes.len());
    if p.peek(Token!(;)) {
        if warn_on_semicolon {
            p.warn(ErrorData::UnexpectedSemicolonAfterBlock);
            _ = p.next();
        }

        if scratch.len() <= 2 {
            let first = scratch.first().copied();
            let second = scratch.get(1).copied();
            p.nodes.push(Node {
                main_token: start,
                data: NodeData::BlockTwoSemicolon(first, second),
            });
        } else {
            let members = p.push_packed_list(&scratch).unwrap();
            p.nodes.push(Node {
                main_token: start,
                data: NodeData::BlockSemicolon(members),
            })
        }
    } else {
        if scratch.len() <= 2 {
            let first = scratch.first().copied();
            let second = scratch.get(1).copied();
            p.nodes.push(Node {
                main_token: start,
                data: NodeData::BlockTwo(first, second),
            });
        } else {
            let members = p.push_packed_list(&scratch).unwrap();
            p.nodes.push(Node {
                main_token: start,
                data: NodeData::Block(members),
            });
        }
    }

    Ok(node_idx)
}

/// StructBlock <- LBRACE StructBlockContents RBRACE
fn expect_struct_block(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let start = match p.take_token(Token!('{')) {
        Some(token) => token,
        None => {
            p.fail(ErrorData::ExpectedStructBlock)?;
            unreachable!();
        }
    };

    let members = parse_struct_block_members(p, false)?;
    p.expect_token(Token!('}'))?;

    if members.len() < 2 {
        let first = members.first().copied();
        let second = members.get(1).copied();
        Ok(p.push_node(start, NodeData::BlockTwo(first, second)))
    } else {
        let members = p.push_packed_list(&members).unwrap();
        Ok(p.push_node(start, NodeData::Block(members)))
    }
}

/// StructBlockContents <- InnerAttribute* StructBlockStatement*
/// StructBlockStatement
///     <- KEYWORD_const ExprBlock
///      / ImplStatement
///      / StructField
///      / ContainerDeclStatement
fn parse_struct_block_members(p: &mut Parser<'_>, is_root: bool) -> Result<Vec<NodeIndex>, ()> {
    let mut nodes = vec![];
    while let Some(attr) = parse_inner_attribute(p)? {
        nodes.push(attr);
    }

    loop {
        let mut has_doc_comment = false;
        let mut attrs = vec![];
        loop {
            let (idx, skipped) = parse_outer_attribute(p)?;
            has_doc_comment |= skipped;
            if let Some(idx) = idx {
                attrs.push(idx);
            } else {
                break;
            }
        }

        match p.tag() {
            Token!(#<eof>) => {
                if has_doc_comment {
                    p.warn(ErrorData::DocCommentAtContainerEnd);
                } else if !attrs.is_empty() {
                    p.warn(ErrorData::AttributeAtContainerEnd);
                }
                assert!(is_root);
                break;
            }
            Token!('}') => {
                if has_doc_comment {
                    p.warn(ErrorData::DocCommentAtContainerEnd);
                } else if !attrs.is_empty() {
                    p.warn(ErrorData::AttributeAtContainerEnd);
                }
                if is_root {
                    p.fail_expected(Token!(#<eof>))?;
                }
                break;
            }
            // const {...}
            Token!(const) => {
                if has_doc_comment {
                    p.warn(ErrorData::DocCommentOnStatement);
                } else if !attrs.is_empty() {
                    p.warn(ErrorData::AttributeOnStatement);
                }
                nodes.push(expect_const_block(p)?);
            }
            // impl condition {...}
            Token!(impl) => {
                if has_doc_comment {
                    p.warn(ErrorData::DocCommentOnStatement);
                }
                nodes.push(expect_impl_block_statement(p, Some(attrs))?);
            }
            // thread_local pub var a ...
            Token!(thread_local) => {
                if !attrs.is_empty() {
                    p.warn(ErrorData::AttributeBeforeThreadLocal);
                }
                nodes.push(expect_thread_local_decl_statement(p)?);
            }
            // pub var a ...
            Token!(pub) | Token!(var) => {
                nodes.push(expect_global_decl_statement(p, attrs)?);
            }
            // inline a ...
            Token!(inline) => nodes.push(expect_inlined_struct_field(p, attrs)?),
            // a ..
            Token!(#<identifier>) => {
                nodes.push(expect_struct_field_or_global_container_decl_statement(
                    p, attrs,
                )?);
            }
            _ => {
                p.fail(ErrorData::ExpectedStructBlockStatement)?;
                unreachable!()
            }
        }
    }

    Ok(nodes)
}

// Statement

/// ConstBlock <- const ExprBlock
fn expect_const_block(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let const_token = p.expect_token(Token!(const))?;
    let expr_block = expect_expr_block(p, true)?;
    if p.peek(Token!(;)) {
        p.warn(ErrorData::UnexpectedSemicolonAfterBlock);
        _ = p.next();
    }

    Ok(p.push_node(const_token, NodeData::Const(expr_block)))
}

/// ContainerDeclStatement <- KEYWORD_thread_local? DeclProtoPub (COMMA DeclProtoPub)* (COLON TypeExpr? COLON / COLON2) TemplateExpr SEMICOLON
/// DeclProtoPub <- OuterAttribute* KEYWORD_pub? KEYWORD_var? IDENTIFIER ByteAlign?
fn expect_decl_statement(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut attrs = vec![];
    loop {
        let (idx, _) = parse_outer_attribute(p)?;
        if let Some(idx) = idx {
            attrs.push(idx);
        } else {
            break;
        }
    }

    match p.tag() {
        Token!(thread_local) => {
            if !attrs.is_empty() {
                p.warn(ErrorData::AttributeBeforeThreadLocal);
            }
            expect_thread_local_decl_statement(p)
        }
        Token!(pub) | Token!(var) | Token!(#<identifier>) => expect_global_decl_statement(p, attrs),
        _ => {
            p.fail(ErrorData::ExpectedThreadLocalOrGlobalDeclStatement)?;
            unreachable!()
        }
    }
}

/// ContainerDeclStatement <- KEYWORD_thread_local? DeclProtoPub (COMMA DeclProtoPub)* (COLON TypeExpr? COLON / COLON2) TemplateExpr SEMICOLON
/// DeclProtoPub <- OuterAttribute* KEYWORD_pub? KEYWORD_var? IDENTIFIER ByteAlign?
fn expect_thread_local_decl_statement(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let thread_local_token = p.expect_token(Token!(thread_local))?;
    let mut decls = vec![];
    decls.push(expect_decl_proto_pub(p, None, true)?);
    while p.peek(Token!(,)) {
        p.next();
        decls.push(expect_decl_proto_pub(p, None, false)?);
    }

    let (type_expr, init_expr) = if p.eat_token(Token!(:)) {
        if p.eat_token(Token!(:)) {
            let expr = expect_template_expr(p)?;
            p.expect_token(Token!(;))?;
            (None, expr)
        } else {
            let type_expr = expect_type_expr(p)?;
            p.expect_token(Token!(:))?;
            let expr = expect_template_expr(p)?;
            p.expect_token(Token!(;))?;
            (Some(type_expr), expr)
        }
    } else {
        p.expect_token(Token!(::))?;
        let expr = expect_template_expr(p)?;
        p.expect_token(Token!(;))?;
        (None, expr)
    };

    if decls.len() == 1 {
        let decl @ DeclProtoPub {
            attrs,
            is_pub,
            is_var,
            ident,
            align_expr,
        } = decls[0];
        if attrs.is_some() || is_pub || is_var || (align_expr.is_some() && type_expr.is_some()) {
            let decl = p.push_packed(decl);
            let type_init_expr = p.push_packed(PackedPair(type_expr, init_expr));
            Ok(p.push_node(ident, NodeData::ThreadLocalDecl(decl, type_init_expr)))
        } else if let Some(align_expr) = align_expr {
            Ok(p.push_node(ident, NodeData::ThreadLocalDeclAlign(align_expr, init_expr)))
        } else {
            Ok(p.push_node(
                ident,
                NodeData::ThreadLocalDeclTypeInit(type_expr, init_expr),
            ))
        }
    } else {
        let protos = p.push_packed_list(&decls).unwrap();
        let protos = p.push_packed(protos);
        let type_init_expr = p.push_packed(PackedPair(type_expr, init_expr));
        Ok(p.push_node(
            thread_local_token,
            NodeData::ThreadLocalDeclDestructure(protos, type_init_expr),
        ))
    }
}

/// ContainerDeclStatement <- KEYWORD_thread_local? DeclProtoPub (COMMA DeclProtoPub)* (COLON TypeExpr? COLON / COLON2) TemplateExpr SEMICOLON
/// DeclProtoPub <- OuterAttribute* KEYWORD_pub? KEYWORD_var? IDENTIFIER ByteAlign?
fn expect_global_decl_statement(
    p: &mut Parser<'_>,
    attrs: Vec<NodeIndex>,
) -> Result<NodeIndex, ()> {
    let proto = expect_decl_proto_pub(p, Some(attrs), true)?;
    if p.peek(Token!(,)) {
        p.next();
        return expect_global_decl_statement2(p, proto);
    }

    let (type_expr, init_expr) = if p.eat_token(Token!(:)) {
        if p.eat_token(Token!(:)) {
            let expr = expect_template_expr(p)?;
            p.expect_token(Token!(;))?;
            (None, expr)
        } else {
            let type_expr = expect_type_expr(p)?;
            p.expect_token(Token!(:))?;
            let expr = expect_template_expr(p)?;
            p.expect_token(Token!(;))?;
            (Some(type_expr), expr)
        }
    } else {
        p.expect_token(Token!(::))?;
        let expr = expect_template_expr(p)?;
        p.expect_token(Token!(;))?;
        (None, expr)
    };

    Ok(emit_global_decl(p, proto, type_expr, init_expr))
}

/// ContainerDeclStatement <- KEYWORD_thread_local? DeclProtoPub (COMMA DeclProtoPub)* (COLON TypeExpr? COLON / COLON2) TemplateExpr SEMICOLON
/// DeclProtoPub <- OuterAttribute* KEYWORD_pub? KEYWORD_var? IDENTIFIER ByteAlign?
fn expect_global_decl_statement2(p: &mut Parser<'_>, proto: DeclProtoPub) -> Result<NodeIndex, ()> {
    let mut protos = vec![proto];
    protos.push(expect_decl_proto_pub(p, None, false)?);
    while p.peek(Token!(,)) {
        p.next();
        protos.push(expect_decl_proto_pub(p, None, false)?);
    }

    let (type_expr, init_expr, semicolon_token) = if p.eat_token(Token!(:)) {
        if p.eat_token(Token!(:)) {
            let expr = expect_template_expr(p)?;
            let semicolon_token = p.expect_token(Token!(;))?;
            (None, expr, semicolon_token)
        } else {
            let type_expr = expect_type_expr(p)?;
            p.expect_token(Token!(:))?;
            let expr = expect_template_expr(p)?;
            let semicolon_token = p.expect_token(Token!(;))?;
            (Some(type_expr), expr, semicolon_token)
        }
    } else {
        p.expect_token(Token!(::))?;
        let expr = expect_template_expr(p)?;
        let semicolon_token = p.expect_token(Token!(;))?;
        (None, expr, semicolon_token)
    };

    let protos = p.push_packed_list(&protos).unwrap();
    let protos = p.push_packed(protos);
    let type_init_expr = p.push_packed(PackedPair(type_expr, init_expr));
    Ok(p.push_node(
        semicolon_token,
        NodeData::GlobalDeclDestructure(protos, type_init_expr),
    ))
}

/// StructField <- OuterAttribute* KEYWORD_inline? IDENTIFIER ByteAlign? COLON (TypeExpr / TypeExpr? EQUAL TemplateExpr) COMMA?
fn expect_inlined_struct_field(p: &mut Parser<'_>, attrs: Vec<NodeIndex>) -> Result<NodeIndex, ()> {
    p.expect_token(Token!(inline))?;
    let ident_token = p.expect_token(Token!(#<identifier>))?;
    let align_expr: Option<NodeIndex> = if p.peek(Token!(align)) {
        p.next();
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };
    p.expect_token(Token!(:))?;

    let type_expr = if !p.peek(Token!(=)) {
        Some(expect_type_expr(p)?)
    } else {
        None
    };
    let init_expr = if p.eat_token(Token!(=)) {
        Some(expect_template_expr(p)?)
    } else {
        None
    };
    let has_comma = p.eat_token(Token!(,));

    Ok(emit_container_field_inline(
        p,
        ident_token,
        attrs,
        align_expr,
        type_expr,
        init_expr,
        has_comma,
    ))
}

/// StructField / ContainerDeclStatement
/// StructField <- OuterAttribute* KEYWORD_inline? IDENTIFIER ByteAlign? COLON (TypeExpr / TypeExpr? EQUAL TemplateExpr) COMMA?
/// ContainerDeclStatement <- KEYWORD_thread_local? DeclProtoPub (COMMA DeclProtoPub)* (COLON TypeExpr? COLON / COLON2) TemplateExpr SEMICOLON
/// DeclProtoPub <- OuterAttribute* KEYWORD_pub? KEYWORD_var? IDENTIFIER ByteAlign?
fn expect_struct_field_or_global_container_decl_statement(
    p: &mut Parser<'_>,
    attrs: Vec<NodeIndex>,
) -> Result<NodeIndex, ()> {
    let ident_token = p.expect_token(Token!(#<identifier>))?;
    let align_expr: Option<NodeIndex> = if p.peek(Token!(align)) {
        p.next();
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };

    match p.tag() {
        // a align(expr) : ...
        Token!(:) => {
            p.next();
            match p.tag() {
                // a align(expr) := generic_expr
                Token!(=) => {
                    p.next();
                    let init_expr = expect_template_expr(p)?;
                    let has_comma = match p.tag() {
                        Token!('}') | Token!(#<eof>) => false,
                        _ => {
                            p.expect_token(Token!(,))?;
                            true
                        }
                    };
                    Ok(emit_container_field(
                        p,
                        ident_token,
                        attrs,
                        align_expr,
                        None,
                        Some(init_expr),
                        has_comma,
                    ))
                }
                // a align(expr) : : generic_expr;
                Token!(:) => {
                    p.next();
                    let init_expr = expect_template_expr(p)?;
                    p.expect_token(Token!(;))?;

                    let attrs = p.push_packed_list(&attrs);
                    let proto = DeclProtoPub {
                        attrs,
                        is_pub: false,
                        is_var: false,
                        ident: ident_token,
                        align_expr,
                    };
                    Ok(emit_global_decl(p, proto, None, init_expr))
                }
                // a align(expr) : type_expr ...
                _ => {
                    let type_expr = expect_type_expr(p)?;
                    match p.tag() {
                        // a align(expr) : type_expr
                        Token!('}') | Token!(#<eof>) => Ok(emit_container_field(
                            p,
                            ident_token,
                            attrs,
                            align_expr,
                            Some(type_expr),
                            None,
                            false,
                        )),
                        // a align(expr) : type_expr,
                        Token!(,) => {
                            p.next();
                            Ok(emit_container_field(
                                p,
                                ident_token,
                                attrs,
                                align_expr,
                                Some(type_expr),
                                None,
                                true,
                            ))
                        }
                        // a align(expr) : type_expr = generic_expr
                        Token!(=) => {
                            p.next();
                            let expr_idx = expect_template_expr(p)?;

                            let has_comma = match p.tag() {
                                Token!('}') | Token!(#<eof>) => false,
                                _ => {
                                    p.expect_token(Token!(,))?;
                                    true
                                }
                            };
                            Ok(emit_container_field(
                                p,
                                ident_token,
                                attrs,
                                align_expr,
                                Some(type_expr),
                                Some(expr_idx),
                                has_comma,
                            ))
                        }
                        _ => {
                            p.expect_token(Token!(:))?;
                            let init_expr = expect_template_expr(p)?;
                            p.expect_token(Token!(;))?;

                            let attrs = p.push_packed_list(&attrs);
                            let proto = DeclProtoPub {
                                attrs,
                                is_pub: false,
                                is_var: false,
                                ident: ident_token,
                                align_expr,
                            };
                            Ok(emit_global_decl(p, proto, Some(type_expr), init_expr))
                        }
                    }
                }
            }
        }
        // a align(expr) :: generic_expr;
        Token!(::) => {
            p.next();
            let init_expr = expect_template_expr(p)?;
            p.expect_token(Token!(;))?;

            let attrs = p.push_packed_list(&attrs);
            let proto = DeclProtoPub {
                attrs,
                is_pub: false,
                is_var: false,
                ident: ident_token,
                align_expr,
            };
            Ok(emit_global_decl(p, proto, None, init_expr))
        }
        // a align(expr), ...
        Token!(,) => {
            p.next();
            if !p.peek_any([Token!(#), Token!(pub), Token!(var), Token!(#<identifier>)]) {
                p.fail(ErrorData::StructFieldWithoutTypeOrInitExpr)?;
                unreachable!()
            }

            let attrs = p.push_packed_list(&attrs);
            let proto = DeclProtoPub {
                attrs,
                is_pub: false,
                is_var: false,
                ident: ident_token,
                align_expr,
            };
            expect_global_decl_statement2(p, proto)
        }
        _ => {
            p.fail(ErrorData::ExpectedTypeExprOrInit)?;
            unreachable!()
        }
    }
}

fn emit_global_decl(
    p: &mut Parser<'_>,
    proto: DeclProtoPub,
    type_expr: Option<NodeIndex>,
    init_expr: NodeIndex,
) -> NodeIndex {
    let proto @ DeclProtoPub {
        attrs,
        is_pub,
        is_var,
        ident,
        align_expr,
    } = proto;
    if attrs.is_some() || is_pub || is_var || (align_expr.is_some() && type_expr.is_some()) {
        let proto = p.push_packed(proto);
        let type_init_expr = p.push_packed(PackedPair(type_expr, init_expr));
        p.push_node(ident, NodeData::GlobalDecl(proto, type_init_expr))
    } else if let Some(align_expr) = align_expr {
        p.push_node(ident, NodeData::GlobalDeclAlign(align_expr, init_expr))
    } else {
        p.push_node(ident, NodeData::GlobalDeclTypeInit(type_expr, init_expr))
    }
}

fn emit_container_field(
    p: &mut Parser<'_>,
    ident: TokenIndex,
    attrs: Vec<NodeIndex>,
    align_expr: Option<NodeIndex>,
    type_expr: Option<NodeIndex>,
    init_expr: Option<NodeIndex>,
    has_comma: bool,
) -> NodeIndex {
    let attrs = p.push_packed_list(&attrs);

    if let Some(attrs) = attrs {
        let extra_idx = p.push_packed(StructFieldProto {
            attrs: Some(attrs),
            align_expr,
            type_expr,
            init_expr,
        });
        if has_comma {
            p.push_node(ident, NodeData::ContainerFieldComma(extra_idx))
        } else {
            p.push_node(ident, NodeData::ContainerField(extra_idx))
        }
    } else if init_expr.is_none() {
        let type_expr = type_expr.unwrap();
        if has_comma {
            p.push_node(
                ident,
                NodeData::ContainerFieldTypeComma(align_expr, type_expr),
            )
        } else {
            p.push_node(ident, NodeData::ContainerFieldType(align_expr, type_expr))
        }
    } else if type_expr.is_none() {
        #[allow(clippy::unnecessary_unwrap)]
        let init_expr = init_expr.unwrap();
        if has_comma {
            p.push_node(
                ident,
                NodeData::ContainerFieldInitComma(align_expr, init_expr),
            )
        } else {
            p.push_node(ident, NodeData::ContainerFieldInit(align_expr, init_expr))
        }
    } else if align_expr.is_none() {
        let type_expr = type_expr.unwrap();
        #[allow(clippy::unnecessary_unwrap)]
        let init_expr = init_expr.unwrap();
        if has_comma {
            p.push_node(
                ident,
                NodeData::ContainerFieldTypeInitComma(type_expr, init_expr),
            )
        } else {
            p.push_node(
                ident,
                NodeData::ContainerFieldTypeInit(type_expr, init_expr),
            )
        }
    } else {
        let extra_idx = p.push_packed(StructFieldProto {
            attrs: None,
            align_expr,
            type_expr,
            init_expr,
        });
        if has_comma {
            p.push_node(ident, NodeData::ContainerFieldComma(extra_idx))
        } else {
            p.push_node(ident, NodeData::ContainerField(extra_idx))
        }
    }
}

fn emit_container_field_inline(
    p: &mut Parser<'_>,
    ident: TokenIndex,
    attrs: Vec<NodeIndex>,
    align_expr: Option<NodeIndex>,
    type_expr: Option<NodeIndex>,
    init_expr: Option<NodeIndex>,
    has_comma: bool,
) -> NodeIndex {
    let attrs = p.push_packed_list(&attrs);
    if let Some(attrs) = attrs {
        let extra_idx = p.push_packed(StructFieldProto {
            attrs: Some(attrs),
            align_expr,
            type_expr,
            init_expr,
        });
        if has_comma {
            p.push_node(ident, NodeData::ContainerFieldInlineComma(extra_idx))
        } else {
            p.push_node(ident, NodeData::ContainerFieldInline(extra_idx))
        }
    } else if init_expr.is_none() {
        let type_expr = type_expr.unwrap();
        if has_comma {
            p.push_node(
                ident,
                NodeData::ContainerFieldInlineTypeComma(align_expr, type_expr),
            )
        } else {
            p.push_node(
                ident,
                NodeData::ContainerFieldInlineType(align_expr, type_expr),
            )
        }
    } else if type_expr.is_none() {
        #[allow(clippy::unnecessary_unwrap)]
        let init_expr = init_expr.unwrap();
        if has_comma {
            p.push_node(
                ident,
                NodeData::ContainerFieldInlineInitComma(align_expr, init_expr),
            )
        } else {
            p.push_node(
                ident,
                NodeData::ContainerFieldInlineInit(align_expr, init_expr),
            )
        }
    } else if align_expr.is_none() {
        let type_expr = type_expr.unwrap();
        #[allow(clippy::unnecessary_unwrap)]
        let init_expr = init_expr.unwrap();
        if has_comma {
            p.push_node(
                ident,
                NodeData::ContainerFieldInlineTypeInitComma(type_expr, init_expr),
            )
        } else {
            p.push_node(
                ident,
                NodeData::ContainerFieldInlineTypeInit(type_expr, init_expr),
            )
        }
    } else {
        let extra_idx = p.push_packed(StructFieldProto {
            attrs: None,
            align_expr,
            type_expr,
            init_expr,
        });
        if has_comma {
            p.push_node(ident, NodeData::ContainerFieldInlineComma(extra_idx))
        } else {
            p.push_node(ident, NodeData::ContainerFieldInline(extra_idx))
        }
    }
}

/// DeclProtoPub <- OuterAttribute* KEYWORD_pub? KEYWORD_var? IDENTIFIER ByteAlign?
fn expect_decl_proto_pub(
    p: &mut Parser<'_>,
    attrs: Option<Vec<NodeIndex>>,
    is_first: bool,
) -> Result<DeclProtoPub, ()> {
    let mut attrs = attrs.unwrap_or_default();
    loop {
        let (attr, has_doc_comment) = parse_outer_attribute(p)?;
        if has_doc_comment && !is_first {
            p.fail(ErrorData::DocCommentBetweenDeclProtos)?;
            unreachable!();
        }
        if let Some(attr) = attr {
            attrs.push(attr);
        } else {
            break;
        }
    }

    let mut pub_token = p.take_token(Token!(pub));
    let var_token = p.take_token(Token!(var));
    if p.peek(Token!(pub)) {
        if pub_token.is_some() {
            p.fail(ErrorData::DoublePubToken)?;
            unreachable!();
        } else {
            p.warn(ErrorData::PubTokenAfterVarToken);
            pub_token = Some(p.next());
        }
    } else if p.peek(Token!(var)) {
        assert!(var_token.is_some());
        p.fail(ErrorData::DoubleVarToken)?;
        unreachable!();
    }
    let ident_token = p.expect_token(Token!(#<identifier>))?;
    let align_expr = if p.eat_token(Token!(align)) {
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };
    let attrs = p.push_packed_list(&attrs);

    Ok(DeclProtoPub {
        attrs,
        is_pub: pub_token.is_some(),
        is_var: var_token.is_some(),
        ident: ident_token,
        align_expr,
    })
}

/// ImplBlockStatement <- OuterAttribute* KEYWORD_impl Expr DeclBlock
fn expect_impl_block_statement(
    p: &mut Parser<'_>,
    attrs: Option<Vec<NodeIndex>>,
) -> Result<NodeIndex, ()> {
    let mut warned = false;
    let mut attrs = attrs.unwrap_or_default();
    loop {
        let (attr, has_doc_comment) = parse_outer_attribute(p)?;
        if has_doc_comment && !warned {
            warned = true;
            p.warn(ErrorData::DocCommentOnStatement);
        }
        if let Some(attr) = attr {
            attrs.push(attr);
        } else {
            break;
        }
    }

    let impl_token = p.expect_token(Token!(impl))?;
    let cond_expr = expect_expr(p)?;
    let decl_block = expect_decl_block(p)?;

    if let Some(attrs) = p.push_packed_list(&attrs) {
        let impl_block = ImplBlock {
            attrs,
            cond_expr,
            decl_block,
        };
        let impl_block = p.push_packed(impl_block);
        Ok(p.push_node(impl_token, NodeData::ImplBlockAttrs(impl_block)))
    } else {
        Ok(p.push_node(impl_token, NodeData::ImplBlock(cond_expr, decl_block)))
    }
}

// Attributes

/// InnerAttribute <- InnerDocComment / InnerAnnotation
fn parse_inner_attribute(p: &mut Parser<'_>) -> Result<Option<NodeIndex>, ()> {
    while p.eat_token(Token!(#<inner_doc_comment>)) {}
    parse_inner_annotation(p)
}

/// RootAttribute <- #![struct] / #![struct const] / #![struct(expr)] / #![struct(expr) const]
fn parse_root_attribute(p: &mut Parser<'_>) -> Result<Option<NodeIndex>, ()> {
    if !(p.peek(Token!(#!)) && p.peek2(Token!('['))) {
        return Ok(None);
    }

    let node_idx = NodeIndex::new(p.nodes.len());

    let start = p.next();
    p.next();
    p.expect_token(Token!(struct))?;
    if let Some(end) = p.take_token(Token!(']')) {
        p.nodes.push(Node {
            main_token: start,
            data: NodeData::RootStructAttribute(start, end),
        });
    } else if p.eat_token(Token!(const)) {
        let end = p.expect_token(Token!(']'))?;
        p.nodes.push(Node {
            main_token: start,
            data: NodeData::RootStructConstAttribute(start, end),
        });
    } else {
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        if p.eat_token(Token!(const)) {
            let end = p.expect_token(Token!(']'))?;
            p.nodes.push(Node {
                main_token: start,
                data: NodeData::RootStructLayoutAttribute(expr, end),
            });
        } else {
            let end = p.expect_token(Token!(']'))?;
            p.nodes.push(Node {
                main_token: start,
                data: NodeData::RootStructLayoutConstAttribute(expr, end),
            });
        }
    }

    Ok(Some(node_idx))
}

/// InnerAnnotation <- #![=Expr]
fn parse_inner_annotation(p: &mut Parser<'_>) -> Result<Option<NodeIndex>, ()> {
    if !(p.peek(Token!(#!)) && p.peek2(Token!('[')) && p.peek3(Token!(=))) {
        return Ok(None);
    }

    let start = p.next();
    p.next();
    p.next();
    let expr = expect_expr(p)?;
    let end = p.expect_token(Token!(']'))?;

    Ok(Some(
        p.push_node(start, NodeData::InnerAnnotation(expr, end)),
    ))
}

/// OuterAttribute <- OuterDocComment \ OuterAnnotation
fn parse_outer_attribute(p: &mut Parser<'_>) -> Result<(Option<NodeIndex>, bool), ()> {
    let mut skipped = false;
    while p.eat_token(Token!(#<outer_doc_comment>)) {
        skipped = true;
    }
    parse_outer_annotation(p).map(|idx| (idx, skipped))
}

/// OuterAnnotation <- #[=Expr]
fn parse_outer_annotation(p: &mut Parser<'_>) -> Result<Option<NodeIndex>, ()> {
    if !(p.peek(Token!(#)) && p.peek2(Token!('[')) && p.peek3(Token!(=))) {
        return Ok(None);
    }

    let start = p.next();
    p.next();
    p.next();
    let expr = expect_expr(p)?;
    let end = p.expect_token(Token!(']'))?;

    Ok(Some(
        p.push_node(start, NodeData::OuterAnnotation(expr, end)),
    ))
}

// Expressions

/// TemplateExpr <- TemplateExprPrefix? TemplateExprConstraint? Expr
fn expect_template_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    if let Some(with_token) = p.take_token(Token!(with)) {
        p.expect_token(Token!('['))?;
        match p.tag() {
            // Template expressions
            // with[] where expr expr
            Token!(']') if !p.peek2(Token!(->)) => {
                p.next();
                let where_expr = if p.eat_token(Token!(where)) {
                    p.expect_token(Token!('('))?;
                    let expr = expect_expr(p)?;
                    p.expect_token(Token!(')'))?;
                    Some(expr)
                } else {
                    None
                };
                let expr = expect_expr(p)?;
                Ok(p.push_node(with_token, NodeData::TemplateExprSimple(where_expr, expr)))
            }
            // with[a: t = v] where expr expr
            Token!(#<identifier>) if p.peek2(Token!(:)) && !p.peek3(Token!('{')) => {
                let mut args = vec![];
                let mut trailing_comma = false;
                while !p.eat_token(Token!(']')) {
                    let ident = p.expect_token(Token!(#<identifier>))?;
                    p.expect_token(Token!(:))?;
                    let type_expr = expect_type_expr(p)?;
                    let init_expr = if p.eat_token(Token!(=)) {
                        Some(expect_expr(p)?)
                    } else {
                        None
                    };
                    args.push(TemplateExprArg {
                        ident,
                        type_expr,
                        init_expr,
                    });

                    trailing_comma = p.eat_token(Token!(,));
                    if !trailing_comma {
                        p.expect_token(Token!(']'))?;
                        break;
                    }
                }
                let where_expr = if p.eat_token(Token!(where)) {
                    p.expect_token(Token!('('))?;
                    let expr = expect_expr(p)?;
                    p.expect_token(Token!(')'))?;
                    Some(expr)
                } else {
                    None
                };
                let expr = expect_expr(p)?;

                let args = p.push_packed_list(&args).unwrap();
                let proto = p.push_packed(TemplateExprProto { args, where_expr });
                if trailing_comma {
                    Ok(p.push_node(with_token, NodeData::TemplateExprComma(proto, expr)))
                } else {
                    Ok(p.push_node(with_token, NodeData::TemplateExpr(proto, expr)))
                }
            }
            // Template type expressions
            _ => {
                p.rollback(2);
                expect_template_type_expr(p)
            }
        }
    } else {
        expect_expr(p)
    }
}

/// Expr <- RangeExpr
fn expect_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    parse_range_expr(p)
}

/// RangeExpr <- BoolOrExpr (DOT2 BoolOrExpr? / DOT2EQUAL BoolOrExpr) / (DOT2 / DOT2EQUAL) BoolOrExpr
fn parse_range_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = match p.tag() {
        Token!(..) | Token!(..=) => {
            let range_token = p.next();
            let expr = parse_bool_or_expr(p)?;
            return Ok(p.push_node(range_token, NodeData::Range(None, Some(expr))));
        }
        _ => parse_bool_or_expr(p)?,
    };

    match p.tag() {
        Token!(..) => {
            let range_token = p.next();
            match p.tag() {
                Token!(!)
                | Token!(-)
                | Token!(~)
                | Token!(-%)
                | Token!(asm)
                | Token!(if)
                | Token!(break)
                | Token!(const)
                | Token!(continue)
                | Token!(return)
                | Token!(throw)
                | Token!(#<identifier>)
                | Token!(for)
                | Token!(while)
                | Token!(try)
                | Token!(fn)
                | Token!('{')
                | Token!(#)
                | Token!(?)
                | Token!('[')
                | Token!(*)
                | Token!(.)
                | Token!(switch)
                | Token!('(')
                | Token!(#<core_identifier>)
                | Token!(#<builtin_identifier>)
                | Token!(impl)
                | Token!(Self)
                | Token!(self)
                | Token!(unreachable)
                | Token!(#<char_literal>)
                | Token!(#<float_literal>)
                | Token!(#<int_literal>)
                | Token!(#<string_literal>)
                | Token!(#<raw_string_literal>)
                | Token!(enum)
                | Token!(fnptr)
                | Token!(namespace)
                | Token!(opaque)
                | Token!(struct)
                | Token!(union)
                | Token!(#primitive)
                | Token!(with) => {
                    let right = parse_bool_or_expr(p)?;
                    Ok(p.push_node(range_token, NodeData::Range(Some(expr), Some(right))))
                }
                _ => Ok(p.push_node(range_token, NodeData::Range(Some(expr), None))),
            }
        }
        Token!(..=) => {
            let range_token = p.next();
            let right = parse_bool_or_expr(p)?;
            Ok(p.push_node(range_token, NodeData::Range(Some(expr), Some(right))))
        }
        _ => Ok(expr),
    }
}

/// BoolOrExpr <- BoolAndExpr (KEYWORD_or BoolAndExpr)*
fn parse_bool_or_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = parse_bool_and_expr(p)?;
    while let Some(op) = p.take_token(Token!(or)) {
        let right = parse_bool_and_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// BoolAndExpr <- CompareExpr (KEYWORD_and CompareExpr)*
fn parse_bool_and_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = parse_compare_expr(p)?;
    while let Some(op) = p.take_token(Token!(and)) {
        let right = parse_compare_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// CompareExpr <- BitwiseExpr (CompareOp BitwiseExpr)?
fn parse_compare_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = parse_bitwise_expr(p)?;
    while let Token!(==)
    | Token!(!=)
    | Token!(<)
    | Token!(>)
    | Token!(<=)
    | Token!(>=)
    | Token!(<=>) = p.tag()
    {
        let op = p.next();
        let right = parse_bitwise_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// BitwiseExpr <- BitShiftExpr (BitwiseOp BitShiftExpr)*
fn parse_bitwise_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = parse_bit_shift_expr(p)?;
    while let Token!(&) | Token!(^) | Token!(|) | Token!(or_else) | Token!(catch) = p.tag() {
        let op = p.next();
        let right = parse_bit_shift_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// BitShiftExpr <- AdditionExpr (BitShiftOp AdditionExpr)*
fn parse_bit_shift_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = parse_addition_expr(p)?;
    while let Token!(<<) | Token!(>>) | Token!(<<|) = p.tag() {
        let op = p.next();
        let right = parse_addition_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// AdditionExpr <- MultiplyExpr (AdditionOp MultiplyExpr)*
fn parse_addition_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = parse_multiply_expr(p)?;
    while let Token!(+)
    | Token!(-)
    | Token!(++)
    | Token!(+%)
    | Token!(-%)
    | Token!(+|)
    | Token!(-|) = p.tag()
    {
        let op = p.next();
        let right = parse_multiply_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// MultiplyExpr <- PrefixExpr (MultiplyOp PrefixExpr)*
fn parse_multiply_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut expr = expect_prefix_expr(p)?;
    while let Token!(*) | Token!(/) | Token!(%) | Token!(*%) | Token!(*|) = p.tag() {
        let op = p.next();
        let right = expect_prefix_expr(p)?;
        expr = p.push_node(op, NodeData::Binary(expr, right));
    }
    Ok(expr)
}

/// PrefixExpr <- PrefixOp* PrimaryExpr
/// PrefixOp
///    <- EXCLAMATIONMARK
///     / MINUS
///     / TILDE
///     / MINUSPERCENT
/// PrimaryExpr
///    <- AsmExpr
///     / IfExpr
///     / KEYWORD_break BreakLabel? Expr?
///     / KEYWORD_const Expr
///     / KEYWORD_continue BreakLabel? Expr?
///     / KEYWORD_return Expr?
///     / KEYWORD_throw Expr?
///     / BlockLabel? LoopExpr / FnExpr
///     / FnExpr
///     / ExprBlock
///     / CurlySuffixExpr
fn expect_prefix_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    match p.tag() {
        Token!(!) | Token!(-) | Token!(~) | Token!(-%) | Token!(...) => {
            let op = p.next();
            let expr = expect_prefix_expr(p)?;
            Ok(p.push_node(op, NodeData::Unary(expr)))
        }
        Token!(asm) => expect_asm_expr(p),
        Token!(if) => expect_if_expr(p),
        Token!(break) => expect_break_expr(p),
        Token!(const) => expect_const_expr(p),
        Token!(continue) => expect_continue_expr(p),
        Token!(return) => expect_return_expr(p),
        Token!(throw) => expect_throw_expr(p),
        Token!(#<identifier>) if p.peek2(Token!(:)) => todo!("BlockLabel? LoopExpr / TryExpr"),
        Token!(for) => expect_for_expr(p),
        Token!(while) => expect_while_expr(p),
        Token!(try) => expect_try_expr(p),
        Token!(fn) if p.peek2(Token!('{')) => {
            let tok = p.next();
            let init_list = expect_init_list(p)?;
            Ok(p.push_node(tok, NodeData::TokenInit(init_list)))
        }
        Token!(fn) => expect_fn_expr(p),
        Token!('{') => expect_expr_block(p, false),
        Token!(#) if !p.peek2(Token!('(')) => {
            p.fail(ErrorData::AttributeOnExpression)?;
            unreachable!();
        }
        Token!(?)
        | Token!('[')
        | Token!(*)
        | Token!(.)
        | Token!(switch)
        | Token!(#)
        | Token!('(')
        | Token!(#<identifier>)
        | Token!(#<core_identifier>)
        | Token!(#<builtin_identifier>)
        | Token!(impl)
        | Token!(Self)
        | Token!(self)
        | Token!(unreachable)
        | Token!(#<char_literal>)
        | Token!(#<float_literal>)
        | Token!(#<int_literal>)
        | Token!(#<string_literal>)
        | Token!(#<raw_string_literal>)
        | Token!(enum)
        | Token!(fnptr)
        | Token!(namespace)
        | Token!(opaque)
        | Token!(struct)
        | Token!(union)
        | Token!(#primitive)
        | Token!(with) => expect_curly_suffix_expr(p),
        _ => {
            p.fail(ErrorData::ExpectedPrefixExpr)?;
            unreachable!()
        }
    }
}

fn expect_asm_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let asm_token = p.expect_token(Token!(asm))?;

    let mut captures = vec![];
    let mut has_trailing_capture_comma = false;
    if p.eat_token(Token!('[')) {
        while !p.eat_token(Token!(']')) {
            let ident = p.expect_token(Token!(#<identifier>))?;
            if p.eat_token(Token!(:)) {
                let type_expr = if !p.peek(Token!(=)) {
                    Some(expect_type_expr(p)?)
                } else {
                    None
                };
                p.expect_token(Token!(=))?;
                let init_expr = expect_expr(p)?;
                has_trailing_capture_comma = p.eat_token(Token!(,));
                captures.push(AsmCapture {
                    ident,
                    type_expr,
                    init_expr: Some(init_expr),
                });
            } else {
                captures.push(AsmCapture {
                    ident,
                    type_expr: None,
                    init_expr: None,
                });
                has_trailing_capture_comma = p.eat_token(Token!(,));
            }
        }
    }

    let mut inputs = vec![];
    let has_trailing_input_comma;
    p.expect_token(Token!('('))?;
    let clobbers_expr = loop {
        if p.peek(Token!(#<identifier>)) && p.peek2(Token!(:)) {
            let ident = p.next();
            p.next();
            let constraint = p.expect_token(Token!(#<string_literal>))?;
            p.expect_token(Token!(,))?;
            inputs.push(AsmInput { ident, constraint });
        } else {
            let expr = expect_expr(p)?;
            has_trailing_input_comma = p.eat_token(Token!(,));
            p.expect_token(Token!(')'))?;
            break expr;
        }
    };

    let volatile_token = p.take_token(Token!(volatile));
    let mut outputs = vec![];
    if p.eat_token(Token!(->)) {
        p.expect_token(Token!('('))?;
        while !p.eat_token(Token!(')')) {
            let ident = p.expect_token(Token!(#<identifier>))?;
            p.expect_token(Token!(:))?;
            let type_expr = expect_type_expr(p)?;
            let constraint = p.expect_token(Token!(#<string_literal>))?;
            outputs.push(AsmOutput {
                ident,
                type_expr,
                constraint,
            });
            if !p.eat_token(Token!(,)) {
                p.expect_token(Token!(')'))?;
                break;
            }
        }
    }

    p.expect_token(Token!('{'))?;
    let instr_expr = expect_expr(p)?;
    p.expect_token(Token!('}'))?;

    if captures.is_empty() && inputs.is_empty() && outputs.is_empty() && !has_trailing_input_comma {
        if volatile_token.is_none() {
            Ok(p.push_node(asm_token, NodeData::AsmSimple(clobbers_expr, instr_expr)))
        } else {
            Ok(p.push_node(
                asm_token,
                NodeData::AsmVolatileSimple(clobbers_expr, instr_expr),
            ))
        }
    } else {
        let captures = p.push_packed_list(&captures);
        let inputs = p.push_packed_list(&inputs);
        let outputs = p.push_packed_list(&outputs);
        let proto = p.push_packed(AsmProto {
            has_trailing_capture_comma,
            has_trailing_input_comma,
            clobbers_expr,
            captures,
            inputs,
            outputs,
        });

        if volatile_token.is_none() {
            Ok(p.push_node(asm_token, NodeData::Asm(proto, instr_expr)))
        } else {
            Ok(p.push_node(asm_token, NodeData::AsmVolatile(proto, instr_expr)))
        }
    }
}

fn expect_if_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!("if")
}

/// KEYWORD_break BreakLabel? TemplateExpr?
fn expect_break_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let break_token = p.expect_token(Token!(break))?;
    let label_token = if p.peek(Token!(:)) && p.peek2(Token!(#<identifier>)) {
        p.next();
        Some(p.next())
    } else {
        None
    };
    let value_expr = match p.tag() {
        Token!(!)
        | Token!(-)
        | Token!(~)
        | Token!(-%)
        | Token!(asm)
        | Token!(if)
        | Token!(break)
        | Token!(const)
        | Token!(continue)
        | Token!(return)
        | Token!(throw)
        | Token!(#<identifier>)
        | Token!(for)
        | Token!(while)
        | Token!(try)
        | Token!(fn)
        | Token!('{')
        | Token!(#)
        | Token!(?)
        | Token!('[')
        | Token!(*)
        | Token!(.)
        | Token!(switch)
        | Token!('(')
        | Token!(#<core_identifier>)
        | Token!(#<builtin_identifier>)
        | Token!(impl)
        | Token!(Self)
        | Token!(self)
        | Token!(unreachable)
        | Token!(#<char_literal>)
        | Token!(#<float_literal>)
        | Token!(#<int_literal>)
        | Token!(#<string_literal>)
        | Token!(#<raw_string_literal>)
        | Token!(enum)
        | Token!(fnptr)
        | Token!(namespace)
        | Token!(opaque)
        | Token!(struct)
        | Token!(union)
        | Token!(#primitive)
        | Token!(with) => Some(expect_template_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(break_token, NodeData::Jump(label_token, value_expr)))
}

/// KEYWORD_const Expr
fn expect_const_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let const_token = p.expect_token(Token!(const))?;
    let expr = expect_template_expr(p)?;
    Ok(p.push_node(const_token, NodeData::Const(expr)))
}

/// KEYWORD_continue BreakLabel? TemplateExpr?
fn expect_continue_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let continue_token = p.expect_token(Token!(continue))?;
    let label_token = if p.peek(Token!(:)) && p.peek2(Token!(#<identifier>)) {
        p.next();
        Some(p.next())
    } else {
        None
    };
    let value_expr = match p.tag() {
        Token!(!)
        | Token!(-)
        | Token!(~)
        | Token!(-%)
        | Token!(asm)
        | Token!(if)
        | Token!(break)
        | Token!(const)
        | Token!(continue)
        | Token!(return)
        | Token!(throw)
        | Token!(#<identifier>)
        | Token!(for)
        | Token!(while)
        | Token!(try)
        | Token!(fn)
        | Token!('{')
        | Token!(#)
        | Token!(?)
        | Token!('[')
        | Token!(*)
        | Token!(.)
        | Token!(switch)
        | Token!('(')
        | Token!(#<core_identifier>)
        | Token!(#<builtin_identifier>)
        | Token!(impl)
        | Token!(Self)
        | Token!(self)
        | Token!(unreachable)
        | Token!(#<char_literal>)
        | Token!(#<float_literal>)
        | Token!(#<int_literal>)
        | Token!(#<string_literal>)
        | Token!(#<raw_string_literal>)
        | Token!(enum)
        | Token!(fnptr)
        | Token!(namespace)
        | Token!(opaque)
        | Token!(struct)
        | Token!(union)
        | Token!(#primitive)
        | Token!(with) => Some(expect_template_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(continue_token, NodeData::Jump(label_token, value_expr)))
}

/// KEYWORD_return TemplateExpr?
fn expect_return_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let return_token = p.expect_token(Token!(return))?;
    let value_expr = match p.tag() {
        Token!(!)
        | Token!(-)
        | Token!(~)
        | Token!(-%)
        | Token!(asm)
        | Token!(if)
        | Token!(break)
        | Token!(const)
        | Token!(continue)
        | Token!(return)
        | Token!(throw)
        | Token!(#<identifier>)
        | Token!(for)
        | Token!(while)
        | Token!(try)
        | Token!(fn)
        | Token!('{')
        | Token!(#)
        | Token!(?)
        | Token!('[')
        | Token!(*)
        | Token!(.)
        | Token!(switch)
        | Token!('(')
        | Token!(#<core_identifier>)
        | Token!(#<builtin_identifier>)
        | Token!(impl)
        | Token!(Self)
        | Token!(self)
        | Token!(unreachable)
        | Token!(#<char_literal>)
        | Token!(#<float_literal>)
        | Token!(#<int_literal>)
        | Token!(#<string_literal>)
        | Token!(#<raw_string_literal>)
        | Token!(enum)
        | Token!(fnptr)
        | Token!(namespace)
        | Token!(opaque)
        | Token!(struct)
        | Token!(union)
        | Token!(#primitive)
        | Token!(with) => Some(expect_template_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(return_token, NodeData::Jump(None, value_expr)))
}

/// KEYWORD_throw TemplateExpr?
fn expect_throw_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let throw_token = p.expect_token(Token!(throw))?;
    let value_expr = match p.tag() {
        Token!(!)
        | Token!(-)
        | Token!(~)
        | Token!(-%)
        | Token!(asm)
        | Token!(if)
        | Token!(break)
        | Token!(const)
        | Token!(continue)
        | Token!(return)
        | Token!(throw)
        | Token!(#<identifier>)
        | Token!(for)
        | Token!(while)
        | Token!(try)
        | Token!(fn)
        | Token!('{')
        | Token!(#)
        | Token!(?)
        | Token!('[')
        | Token!(*)
        | Token!(.)
        | Token!(switch)
        | Token!('(')
        | Token!(#<core_identifier>)
        | Token!(#<builtin_identifier>)
        | Token!(impl)
        | Token!(Self)
        | Token!(self)
        | Token!(unreachable)
        | Token!(#<char_literal>)
        | Token!(#<float_literal>)
        | Token!(#<int_literal>)
        | Token!(#<string_literal>)
        | Token!(#<raw_string_literal>)
        | Token!(enum)
        | Token!(fnptr)
        | Token!(namespace)
        | Token!(opaque)
        | Token!(struct)
        | Token!(union)
        | Token!(#primitive)
        | Token!(with) => Some(expect_template_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(throw_token, NodeData::Jump(None, value_expr)))
}

fn expect_for_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!("for")
}

fn expect_while_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!("while")
}

fn expect_try_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!("try")
}

fn expect_fn_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let fn_token = p.expect_token(Token!(fn))?;

    let mut captures = vec![];
    let mut has_trailing_capture_comma = false;
    let context = if p.eat_token(Token!('[')) {
        let (context, stop) = match p.tag() {
            Token!(context) => {
                let context_token = p.next();
                let type_expr = if p.eat_token(Token!(:)) {
                    Some(expect_type_expr(p)?)
                } else {
                    None
                };
                let context = FnContext {
                    is_optional: false,
                    ctx_token: context_token,
                    type_expr,
                };
                (Some(context), !p.eat_token(Token!(,)))
            }
            Token!(?) if p.peek2(Token!(context)) => {
                p.next();
                let context_token = p.next();
                let context = FnContext {
                    is_optional: true,
                    ctx_token: context_token,
                    type_expr: None,
                };
                (Some(context), !p.eat_token(Token!(,)))
            }
            _ => (None, false),
        };

        if !stop {
            while !p.eat_token(Token!(']')) {
                let ident = p.expect_token(Token!(#<identifier>))?;
                if p.eat_token(Token!(:)) {
                    let type_expr = if !p.peek(Token!(=)) {
                        Some(expect_type_expr(p)?)
                    } else {
                        None
                    };
                    p.expect_token(Token!(=))?;
                    let init_expr = expect_expr(p)?;
                    has_trailing_capture_comma = p.eat_token(Token!(,));
                    captures.push(FnCapture {
                        ident,
                        type_expr,
                        init_expr: Some(init_expr),
                    });
                } else {
                    captures.push(FnCapture {
                        ident,
                        type_expr: None,
                        init_expr: None,
                    });
                    has_trailing_capture_comma = p.eat_token(Token!(,));
                }
            }
        } else {
            p.expect_token(Token!(']'))?;
        }

        context
    } else {
        None
    };

    let mut args = vec![];
    let mut has_trailing_arg_comma = false;
    p.expect_token(Token!('('))?;
    let mut modifier = match p.tag() {
        Token!(const) => {
            p.next();
            FnArgModifier::Const
        }
        Token!(no_alias) => {
            p.next();
            FnArgModifier::NoAlias
        }
        _ => FnArgModifier::None,
    };
    let mut is_var = p.eat_token(Token!(var));

    let receiver = match p.tag() {
        Token!(self) => Some(FnReceiver {
            modifier,
            is_var,
            pointer_type: None,
            self_token: p.next(),
        }),
        // *...
        Token!(*) => {
            p.next();
            match p.tag() {
                // *var ...
                Token!(var) => {
                    p.next();
                    match p.tag() {
                        // *var var
                        Token!(var) => {
                            p.fail(ErrorData::DoubleVarToken)?;
                            unreachable!();
                        }
                        // *var volatile self
                        Token!(volatile) => {
                            p.next();
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::VarVolatile),
                                self_token,
                            })
                        }
                        // *var ?volatile self
                        Token!(?) => {
                            p.next();
                            if p.eat_token(Token!(var)) {
                                p.fail(ErrorData::DoubleVarToken)?;
                                unreachable!();
                            }
                            p.expect_token(Token!(volatile))?;
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::VarOptVolatile),
                                self_token,
                            })
                        }
                        // *var self
                        _ => {
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::Var),
                                self_token,
                            })
                        }
                    }
                }
                // *volatile ...
                Token!(volatile) => {
                    p.next();
                    match p.tag() {
                        // *volatile var self
                        Token!(var) => {
                            p.next();
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::VarVolatile),
                                self_token,
                            })
                        }
                        // *volatile volatile
                        Token!(volatile) => {
                            p.fail(ErrorData::DoubleVolatileToken)?;
                            unreachable!();
                        }
                        // *volatile ?var self
                        Token!(?) => {
                            p.next();
                            if p.eat_token(Token!(volatile)) {
                                p.fail(ErrorData::DoubleVolatileToken)?;
                                unreachable!();
                            }
                            p.expect_token(Token!(var))?;
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::OptVarVolatile),
                                self_token,
                            })
                        }
                        // *volatile self
                        _ => {
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::Volatile),
                                self_token,
                            })
                        }
                    }
                }
                // *?var ...
                Token!(?) if p.peek2(Token!(var)) => {
                    p.next();
                    p.next();
                    match p.tag() {
                        // *?var var
                        Token!(var) => {
                            p.fail(ErrorData::DoubleVarToken)?;
                            unreachable!();
                        }
                        // *?var volatile self
                        Token!(volatile) => {
                            p.next();
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::OptVarVolatile),
                                self_token,
                            })
                        }
                        // *?var ?volatile self
                        Token!(?) => {
                            p.next();
                            if p.eat_token(Token!(var)) {
                                p.fail(ErrorData::DoubleVarToken)?;
                                unreachable!();
                            }
                            p.expect_token(Token!(volatile))?;
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::OptVarOptVolatile),
                                self_token,
                            })
                        }
                        // *?var self
                        _ => {
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::OptVar),
                                self_token,
                            })
                        }
                    }
                }
                // *?volatile
                Token!(?) if p.peek2(Token!(volatile)) => {
                    p.next();
                    p.next();
                    match p.tag() {
                        // *?volatile var self
                        Token!(var) => {
                            p.next();
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::VarOptVolatile),
                                self_token,
                            })
                        }
                        // *?volatile volatile
                        Token!(volatile) => {
                            p.fail(ErrorData::DoubleVolatileToken)?;
                            unreachable!();
                        }
                        // *?volatile ?var self
                        Token!(?) => {
                            p.next();
                            if p.eat_token(Token!(volatile)) {
                                p.fail(ErrorData::DoubleVolatileToken)?;
                                unreachable!();
                            }
                            p.expect_token(Token!(var))?;
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::OptVarOptVolatile),
                                self_token,
                            })
                        }
                        // *?volatile self
                        _ => {
                            let self_token = p.expect_token(Token!(self))?;
                            Some(FnReceiver {
                                modifier,
                                is_var,
                                pointer_type: Some(PointerType::OptVolatile),
                                self_token,
                            })
                        }
                    }
                }
                _ => {
                    let self_token = p.expect_token(Token!(self))?;
                    Some(FnReceiver {
                        modifier,
                        is_var,
                        pointer_type: None,
                        self_token,
                    })
                }
            }
        }
        _ => None,
    };
    if receiver.is_some() {
        has_trailing_arg_comma = p.eat_token(Token!(,));
        if has_trailing_capture_comma {
            modifier = match p.tag() {
                Token!(const) => {
                    p.next();
                    FnArgModifier::Const
                }
                Token!(no_alias) => {
                    p.next();
                    FnArgModifier::NoAlias
                }
                _ => FnArgModifier::None,
            };
            is_var = p.eat_token(Token!(var));
        }
    }

    if !p.eat_token(Token!(')')) {
        loop {
            let ident = p.expect_token(Token!(#<identifier>))?;
            p.expect_token(Token!(:))?;
            let type_expr = expect_type_expr(p)?;
            let default_expr = if p.eat_token(Token!(=)) {
                Some(expect_template_expr(p)?)
            } else {
                None
            };
            args.push(FnArg {
                modifier,
                is_var,
                ident,
                type_expr,
                default_expr,
            });

            has_trailing_arg_comma = p.eat_token(Token!(,));
            if !has_trailing_arg_comma {
                break;
            }
            modifier = match p.tag() {
                Token!(')') => break,
                Token!(const) => {
                    p.next();
                    FnArgModifier::Const
                }
                Token!(no_alias) => {
                    p.next();
                    FnArgModifier::NoAlias
                }
                _ => FnArgModifier::None,
            };
            is_var = p.eat_token(Token!(var));
        }
        p.expect_token(Token!(')'))?;
    }

    let modifier = match p.tag() {
        Token!(const) => {
            p.next();
            FnModifier::Const
        }
        Token!(inline) => {
            p.next();
            FnModifier::Inline
        }
        Token!(no_inline) => {
            p.next();
            FnModifier::NoInline
        }
        _ => FnModifier::None,
    };
    let call_conv_expr = if p.eat_token(Token!(callconv)) {
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };
    let return_type_expr = if p.eat_token(Token!(->)) {
        Some(expect_type_expr(p)?)
    } else {
        None
    };
    let where_expr = if p.eat_token(Token!(where)) {
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };

    let expr_block = expect_expr_block(p, false)?;

    let context = p.push_optional_packed(context);
    let captures = p.push_packed_list(&captures);
    let receiver = p.push_optional_packed(receiver);
    let args = p.push_packed_list(&args);
    let proto = p.push_packed(FnProto {
        has_trailing_capture_comma,
        has_trailing_arg_comma,
        modifier,
        context,
        captures,
        receiver,
        args,
        call_conv_expr,
        return_type_expr,
        where_expr,
    });
    Ok(p.push_node(fn_token, NodeData::Fn(proto, expr_block)))
}

fn expect_curly_suffix_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = expect_type_expr(p)?;
    if p.peek(Token!('{')) {
        let init_list = expect_init_list(p)?;
        Ok(p.push_node(
            TokenIndex::new(u32::MAX - 1),
            NodeData::ExprInit(expr, init_list),
        ))
    } else {
        Ok(expr)
    }
}

// Type Expressions

/// TypeExpr <- ErrorUnionExpr
fn expect_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    expect_error_union_type_expr(p)
}

/// ErrorUnionExpr <- PackTypeExpr (EXCLAMATIONMARK PackTypeExpr)?
fn expect_error_union_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = expect_pack_type_expr(p)?;
    if let Some(tok) = p.take_token(Token!(!)) {
        let rhs = expect_pack_type_expr(p)?;
        Ok(p.push_node(tok, NodeData::ErrorUnionType(expr, rhs)))
    } else {
        Ok(expr)
    }
}

/// PackTypeExpr <- SingleTypeExpr DOT3?
fn expect_pack_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = expect_single_type_expr(p)?;
    if let Some(pack_token) = p.take_token(Token!(...)) {
        Ok(p.push_node(pack_token, NodeData::Pack(expr)))
    } else {
        Ok(expr)
    }
}

/// SingleTypeExpr <- PrefixTypeOp* PrimaryTypeExpr (SuffixOp / MacroCallArguments / FnCallArguments)*
/// PrefixTypeOp
///    <- QUESTIONMARK
///     / SliceTypeStart (ByteAlign / COMMA? KEYWORD_var / COMMA? KEYWORD_volatile)*
///     / PtrTypeStart (ByteAlign / COMMA? KEYWORD_var / COMMA? KEYWORD_volatile)*
///     / VectorTypeStart
///     / MatrixTypeStart
///     / ArrayTypeStart
/// PrimaryTypeExpr
///    <- DOT IDENTIFIER
///     / DOT InitList
///     / IfTypeExpr
///     / LabeledTypeExpr
///     / GroupedTypeExpr
///     / IDENTIFIER
///     / CORE_IDENTIFIER
///     / BUILTIN_IDENTIFIER
///     / KEYWORD_const TypeExpr
///     / KEYWORD_impl TypeExpr
///     / KEYWORD_Self
///     / KEYWORD_self
///     / KEYWORD_unreachable
///     / CHAR_LITERAL
///     / FLOAT_LITERAL
///     / INT_LITERAL
///     / STRING_LITERAL
///     / RAW_STRING_LITERAL
///     / EnumTypeExpr
///     / FnptrTypeExpr
///     / NamespaceTypeExpr
///     / OpaqueTypeExpr
///     / PrimitiveTypeExpr
///     / StructTypeExpr
///     / UnionTypeExpr
///     / TemplateTypeExpr
fn expect_single_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = match p.tag() {
        Token!(?) => {
            let tok = p.next();
            let expr = expect_type_expr(p)?;
            return Ok(p.push_node(tok, NodeData::OptionType(expr)));
        }
        Token!('[') if p.peek2_any([Token!(:), Token!(']')]) => {
            let tok = p.next();
            let sentinel_expr = if p.eat_token(Token!(:)) {
                Some(expect_expr(p)?)
            } else {
                None
            };
            p.expect_token(Token!(']'))?;
            let type_expr = expect_type_expr(p)?;
            return Ok(p.push_node(tok, NodeData::Slice(sentinel_expr, type_expr)));
        }
        Token!(*) => return expect_single_pointer_type_expr(p),
        Token!('[') if p.peek2(Token!(*)) => return expect_multi_pointer_type_expr(p),
        Token!('[') if p.peek2(Token!('[')) => {
            let tok = p.next();
            p.next();
            let len_expr = expect_expr(p)?;
            if p.eat_token(Token!(']')) {
                p.expect_token(Token!(']'))?;
                let type_expr = expect_type_expr(p)?;
                return Ok(p.push_node(tok, NodeData::Vector(len_expr, type_expr)));
            } else {
                p.expect_token(Token!(,))?;
                let cols_expr = expect_expr(p)?;
                p.expect_token(Token!(']'))?;
                let layout = if p.eat_token(Token!(:)) {
                    Some(expect_expr(p)?)
                } else {
                    None
                };
                p.expect_token(Token!(']'))?;
                let type_expr = expect_type_expr(p)?;
                if let Some(layout) = layout {
                    let matrix = p.push_packed(Matrix {
                        rows: len_expr,
                        cols: cols_expr,
                        layout,
                    });
                    return Ok(p.push_node(tok, NodeData::Matrix(matrix, type_expr)));
                } else {
                    let rows = len_expr;
                    let cols = cols_expr;
                    let rows_cols = p.push_packed(PackedPair(rows, cols));
                    return Ok(p.push_node(tok, NodeData::MatrixSimple(rows_cols, type_expr)));
                }
            }
        }
        Token!('[') => {
            let tok = p.next();
            let len_expr = expect_expr(p)?;
            let sentinel_expr = if p.eat_token(Token!(:)) {
                Some(expect_expr(p)?)
            } else {
                None
            };
            p.expect_token(Token!(']'))?;
            let type_expr = expect_type_expr(p)?;

            if let Some(sentinel_expr) = sentinel_expr {
                let len_sentinel = p.push_packed(PackedPair(len_expr, sentinel_expr));
                return Ok(p.push_node(tok, NodeData::Array(len_sentinel, type_expr)));
            } else {
                return Ok(p.push_node(tok, NodeData::ArraySimple(len_expr, type_expr)));
            }
        }
        Token!(.) if p.peek2(Token!(#<identifier>)) => {
            let tok = p.next();
            let ident = p.next();
            p.push_node(tok, NodeData::EnumLiteral(ident))
        }
        Token!(.) if p.peek2(Token!('{')) => {
            let tok = p.next();
            let init_list = expect_init_list(p)?;
            p.push_node(tok, NodeData::TokenInit(init_list))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) => {
            todo!("LabeledTypeExpr")
        }
        Token!(for) => {
            todo!("for")
        }
        Token!(while) => {
            todo!("while")
        }
        Token!(switch) => {
            todo!("switch")
        }
        Token!(#) if p.peek2(Token!('(')) => {
            todo!("TupleTypeExpr")
        }
        Token!('(') => {
            todo!("GroupedTypeExpr")
        }
        Token!(#<identifier>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::Ident)
        }
        Token!(#<core_identifier>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::CoreIdent)
        }
        Token!(#<builtin_identifier>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::BuiltinIdent)
        }
        Token!(const) => {
            let const_token = p.next();
            let expr = expect_type_expr(p)?;
            p.push_node(const_token, NodeData::Const(expr))
        }
        Token!(impl) => {
            let tok = p.next();
            let expr = expect_type_expr(p)?;
            p.push_node(tok, NodeData::ImplExpr(expr))
        }
        Token!(Self) => {
            let tok = p.next();
            p.push_node(tok, NodeData::SelfType)
        }
        Token!(self) => {
            let tok = p.next();
            p.push_node(tok, NodeData::SelfIdent)
        }
        Token!(unreachable) => {
            let tok = p.next();
            p.push_node(tok, NodeData::Unreachable)
        }
        Token!(#<char_literal>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::CharLiteral)
        }
        Token!(#<float_literal>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::FloatLiteral)
        }
        Token!(#<int_literal>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::IntLiteral)
        }
        Token!(#<string_literal>) => {
            let tok = p.next();
            p.push_node(tok, NodeData::StringLiteral)
        }
        Token!(#<raw_string_literal>) => {
            let first_tok = p.next();
            let mut last_tok = first_tok;
            while let Some(tok) = p.take_token(Token!(#<raw_string_literal>)) {
                last_tok = tok;
            }
            p.push_node(first_tok, NodeData::RawStringLiteral(first_tok, last_tok))
        }
        Token!(enum) => todo!("enum"),
        Token!(fnptr) => {
            todo!("fnptr")
        }
        Token!(namespace) => expect_namespace_type_expr(p)?,
        Token!(opaque) => expect_opaque_type_expr(p)?,
        Token!(struct) => expect_struct_type_expr(p)?,
        Token!(union) => todo!("union"),
        Token!(#primitive) => expect_primitive_type_expr(p)?,
        Token!(with) => expect_template_type_expr(p)?,
        _ => {
            p.fail(ErrorData::ExpectedTypeExpr)?;
            unreachable!()
        }
    };
    parse_single_type_expr_suffixes(p, expr)
}

/// SingleTypeExpr <- PrefixTypeOp* PrimaryTypeExpr (SuffixOp / MacroCallArguments / FnCallArguments)*
/// SuffixOp
///    <- LBRACKET Expr RBRACKET
///     / DOT2 LBRACKET (TemplateExprList / DOT3) RBRACKET
///     / DOT IDENTIFIER
///     / MINUSARROW IDENTIFIER
///     / DOTAMPERSAND
///     / DOTASTERISK
///     / DOTEXCLAMATIONMARK
///     / DOTQUESTIONMARK
/// MacroCallArguments
///    <- EXCLAMATIONMARK LPAREN TokenSequence RPAREN
///     / EXCLAMATIONMARK LBRACKET TokenSequence RBRACKET
///     / EXCLAMATIONMARK LBRACE TokenSequence RBRACE
/// FnCallArguments <- LPAREN TemplateExprList RPAREN
fn parse_single_type_expr_suffixes(
    p: &mut Parser<'_>,
    mut expr: NodeIndex,
) -> Result<NodeIndex, ()> {
    loop {
        expr = match p.tag() {
            Token!('[') => {
                p.next();
                let index_expr = expect_expr(p)?;
                let end_tok = p.expect_token(Token!(']'))?;
                p.push_node(end_tok, NodeData::Index(expr, index_expr))
            }
            Token!(::) => {
                p.next();
                p.expect_token(Token!('['))?;
                if p.peek(Token!(...)) && p.peek2(Token!(']')) {
                    _ = p.next();
                    let end = p.next();
                    p.push_node(end, NodeData::Alias(expr))
                } else {
                    let mut bind_exprs = vec![];
                    let mut has_trailing_comma = false;
                    while !p.peek(Token!(']')) {
                        bind_exprs.push(expect_template_expr(p)?);
                        has_trailing_comma = p.eat_token(Token!(,));
                        if !has_trailing_comma {
                            break;
                        }
                    }
                    let end = p.expect_token(Token!(']'))?;

                    if bind_exprs.len() <= 1 {
                        assert!(bind_exprs.len() == 1 || !has_trailing_comma);
                        let bind_expr = bind_exprs.first().copied();
                        if has_trailing_comma {
                            p.push_node(end, NodeData::Bind1Comma(expr, bind_expr.unwrap()))
                        } else {
                            p.push_node(end, NodeData::Bind1(expr, bind_expr))
                        }
                    } else {
                        let bind_exprs = p.push_packed_list(&bind_exprs).unwrap();
                        let bind_exprs = p.push_packed(bind_exprs);
                        if has_trailing_comma {
                            p.push_node(end, NodeData::BindComma(expr, bind_exprs))
                        } else {
                            p.push_node(end, NodeData::Bind(expr, bind_exprs))
                        }
                    }
                }
            }
            Token!(.) | Token!(->) => {
                let op = p.next();
                let ident = p.expect_token(Token!(#<identifier>))?;
                p.push_node(op, NodeData::TypeBinarySuffix(expr, ident))
            }
            Token!(.&) | Token!(.*) | Token!(.!) | Token!(.?) => {
                let op = p.next();
                p.push_node(op, NodeData::TypeUnarySuffix(expr))
            }
            Token!(!) if p.peek2(Token!('(')) => todo!("macro!()"),
            Token!(!) if p.peek2(Token!('[')) => todo!("macro![]"),
            Token!(!) if p.peek2(Token!('{')) => todo!("macro!{{}}"),
            Token!('(') => {
                p.next();
                let mut args_exprs = vec![];
                let mut has_trailing_comma = false;
                while !p.peek(Token!(')')) {
                    args_exprs.push(expect_template_expr(p)?);
                    has_trailing_comma = p.eat_token(Token!(,));
                    if !has_trailing_comma {
                        break;
                    }
                }
                let end = p.expect_token(Token!(')'))?;

                if args_exprs.len() <= 1 {
                    assert!(args_exprs.len() == 1 || !has_trailing_comma);
                    let bind_expr = args_exprs.first().copied();
                    if has_trailing_comma {
                        p.push_node(end, NodeData::Call1Comma(expr, bind_expr.unwrap()))
                    } else {
                        p.push_node(end, NodeData::Call1(expr, bind_expr))
                    }
                } else {
                    let bind_exprs = p.push_packed_list(&args_exprs).unwrap();
                    let bind_exprs = p.push_packed(bind_exprs);
                    if has_trailing_comma {
                        p.push_node(end, NodeData::CallComma(expr, bind_exprs))
                    } else {
                        p.push_node(end, NodeData::Call(expr, bind_exprs))
                    }
                }
            }
            _ => break,
        };
    }

    Ok(expr)
}

/// ASTERISK (ByteAlign / COMMA? KEYWORD_var / COMMA? KEYWORD_volatile)*
fn expect_single_pointer_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let tok = p.expect_token(Token!(*))?;
    let (align_expr, pointer_type) = parse_pointer_prefix(p)?;
    let type_expr = expect_type_expr(p)?;

    if let Some(align_expr) = align_expr {
        let prefix = p.push_packed(SinglePointerPrefix {
            pointer_type,
            align_expr,
        });
        Ok(p.push_node(tok, NodeData::SinglePointer(prefix, type_expr)))
    } else {
        Ok(p.push_node(tok, NodeData::SinglePointerSimple(pointer_type, type_expr)))
    }
}

/// LBRACKET ASTERISK (COLON Expr)? RBRACKET (ByteAlign / COMMA? KEYWORD_var / COMMA? KEYWORD_volatile)*
fn expect_multi_pointer_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let tok = p.expect_token(Token!('['))?;
    p.expect_token(Token!(*))?;
    let sentinel_expr = if p.eat_token(Token!(:)) {
        Some(expect_expr(p)?)
    } else {
        None
    };
    p.expect_token(Token!(']'))?;
    let (align_expr, pointer_type) = parse_pointer_prefix(p)?;
    let type_expr = expect_type_expr(p)?;

    if sentinel_expr.is_some() || align_expr.is_some() {
        let prefix = p.push_packed(MultiPointerPrefix {
            pointer_type,
            sentinel_expr,
            align_expr,
        });
        Ok(p.push_node(tok, NodeData::MultiPointer(prefix, type_expr)))
    } else {
        Ok(p.push_node(tok, NodeData::MultiPointerSimple(pointer_type, type_expr)))
    }
}

/// (ByteAlign / COMMA? KEYWORD_var / COMMA? KEYWORD_volatile)*
fn parse_pointer_prefix(p: &mut Parser<'_>) -> Result<(Option<NodeIndex>, PointerType), ()> {
    let mut align_expr: Option<NodeIndex> = None;
    let mut pointer_type = PointerType::Default;
    loop {
        match p.tag() {
            Token!(align) => {
                if align_expr.is_some() {
                    p.fail(ErrorData::DoubleAlignment)?;
                    unreachable!();
                }
                p.next();
                p.expect_token(Token!('('))?;
                align_expr = Some(expect_expr(p)?);
                p.expect_token(Token!(')'))?;
            }
            Token!(var) => {
                pointer_type = match pointer_type {
                    PointerType::Default => PointerType::Var,
                    PointerType::Var
                    | PointerType::OptVar
                    | PointerType::VarVolatile
                    | PointerType::OptVarVolatile
                    | PointerType::VarOptVolatile
                    | PointerType::OptVarOptVolatile => {
                        p.fail(ErrorData::DoubleVarToken)?;
                        unreachable!()
                    }
                    PointerType::Volatile => PointerType::VarVolatile,
                    PointerType::OptVolatile => PointerType::VarOptVolatile,
                }
            }
            Token!(?) if p.peek2(Token!(var)) => {
                pointer_type = match pointer_type {
                    PointerType::Default => PointerType::OptVar,
                    PointerType::Var
                    | PointerType::OptVar
                    | PointerType::VarVolatile
                    | PointerType::OptVarVolatile
                    | PointerType::VarOptVolatile
                    | PointerType::OptVarOptVolatile => {
                        p.fail(ErrorData::DoubleVarToken)?;
                        unreachable!()
                    }
                    PointerType::Volatile => PointerType::OptVarVolatile,
                    PointerType::OptVolatile => PointerType::OptVarOptVolatile,
                }
            }
            Token!(volatile) => {
                pointer_type = match pointer_type {
                    PointerType::Default => PointerType::Volatile,
                    PointerType::Var => PointerType::VarVolatile,
                    PointerType::OptVar => PointerType::OptVarVolatile,
                    PointerType::Volatile
                    | PointerType::OptVolatile
                    | PointerType::VarVolatile
                    | PointerType::OptVarVolatile
                    | PointerType::VarOptVolatile
                    | PointerType::OptVarOptVolatile => {
                        p.fail(ErrorData::DoubleVolatileToken)?;
                        unreachable!()
                    }
                }
            }
            Token!(?) if p.peek2(Token!(volatile)) => {
                pointer_type = match pointer_type {
                    PointerType::Default => PointerType::OptVolatile,
                    PointerType::Var => PointerType::OptVarVolatile,
                    PointerType::OptVar => PointerType::OptVarOptVolatile,
                    PointerType::Volatile
                    | PointerType::OptVolatile
                    | PointerType::VarVolatile
                    | PointerType::OptVarVolatile
                    | PointerType::VarOptVolatile
                    | PointerType::OptVarOptVolatile => {
                        p.fail(ErrorData::DoubleVolatileToken)?;
                        unreachable!()
                    }
                }
            }
            _ => break,
        }
    }

    Ok((align_expr, pointer_type))
}

/// NamespaceTypeExpr <- KEYWORD_namespace DeclBlock
fn expect_namespace_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let ns_token = p.expect_token(Token!(namespace))?;
    let decl_block = expect_decl_block(p)?;

    Ok(p.push_node(ns_token, NodeData::Namespace(decl_block)))
}

/// OpaqueTypeExpr <- KEYWORD_opaque (LPAREN Expr RPAREN)? KEYWORD_const? DeclBlock
fn expect_opaque_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let container_token = p.expect_token(Token!(opaque))?;
    let layout_expr = if p.eat_token(Token!('(')) {
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };
    let is_const = p.eat_token(Token!(const));
    let decl_block = expect_decl_block(p)?;

    if is_const {
        Ok(p.push_node(
            container_token,
            NodeData::ContainerConst(layout_expr, decl_block),
        ))
    } else {
        Ok(p.push_node(
            container_token,
            NodeData::Container(layout_expr, decl_block),
        ))
    }
}

/// StructTypeExpr <- KEYWORD_struct (LPAREN Expr RPAREN)? KEYWORD_const? StructBlock
fn expect_struct_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let container_token = p.expect_token(Token!(struct))?;
    let layout_expr = if p.eat_token(Token!('(')) {
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };
    let is_const = p.eat_token(Token!(const));
    let struct_block = expect_struct_block(p)?;

    if is_const {
        Ok(p.push_node(
            container_token,
            NodeData::ContainerConst(layout_expr, struct_block),
        ))
    } else {
        Ok(p.push_node(
            container_token,
            NodeData::Container(layout_expr, struct_block),
        ))
    }
}

/// KEYWORD_primitive LPAREN STRING_LITERAL COMMA DeclBlock RPAREN
fn expect_primitive_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let primitive_token = p.expect_token(Token!(#primitive))?;
    p.expect_token(Token!('('))?;
    let id_token = p.expect_token(Token!(#<string_literal>))?;
    p.expect_token(Token!(,))?;
    let decl_block = expect_decl_block(p)?;
    p.expect_token(Token!(')'))?;
    Ok(p.push_node(primitive_token, NodeData::Primitive(id_token, decl_block)))
}

/// InitList
///     <- LBRACE FieldInit (COMMA FieldInit)* COMMA? RBRACE
///      / LBRACE Expr (COMMA Expr)* COMMA? RBRACE
///      / LBRACE RBRACE
fn expect_init_list(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let lbrace_token = p.expect_token(Token!('{'))?;
    match p.tag() {
        Token!('}') => {
            p.next();
            Ok(p.push_node(lbrace_token, NodeData::ExprInitListTwo(None, None)))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(=)) => {
            let field = {
                let ident = p.next();
                p.next();
                p.next();
                let init_expr = expect_template_expr(p)?;
                FieldInit { ident, init_expr }
            };
            if p.eat_token(Token!(,)) {
                let mut fields = vec![field];
                let comma = loop {
                    if p.eat_token(Token!('}')) {
                        break true;
                    }

                    let ident = p.expect_token(Token!(#<identifier>))?;
                    p.expect_token(Token!(:))?;
                    p.expect_token(Token!(=))?;
                    let init_expr = expect_template_expr(p)?;
                    fields.push(FieldInit { ident, init_expr });

                    let comma = p.eat_token(Token!(,));
                    if !comma {
                        p.expect_token(Token!('}'))?;
                        break false;
                    }
                };

                if fields.len() <= 2 {
                    let first = p.push_packed(fields[0]);
                    let second = p.push_optional_packed(fields.get(1).copied());
                    if comma {
                        Ok(p.push_node(
                            lbrace_token,
                            NodeData::FieldInitListTwoComma(first, second),
                        ))
                    } else {
                        Ok(p.push_node(lbrace_token, NodeData::FieldInitListTwo(first, second)))
                    }
                } else {
                    let fields = p.push_packed_list(&fields).unwrap();
                    if comma {
                        Ok(p.push_node(lbrace_token, NodeData::FieldInitListComma(fields)))
                    } else {
                        Ok(p.push_node(lbrace_token, NodeData::FieldInitList(fields)))
                    }
                }
            } else {
                p.expect_token(Token!('}'))?;
                let field = p.push_packed(field);
                Ok(p.push_node(lbrace_token, NodeData::FieldInitListTwo(field, None)))
            }
        }
        _ => {
            let mut exprs = vec![];
            let comma = loop {
                exprs.push(expect_template_expr(p)?);
                let comma = p.eat_token(Token!(,));
                if !comma {
                    p.expect_token(Token!('}'))?;
                    break false;
                }
                if p.eat_token(Token!('}')) {
                    break comma;
                }
            };

            if exprs.len() <= 2 {
                let first = exprs.first().copied();
                let second = exprs.get(1).copied();
                if comma {
                    Ok(p.push_node(lbrace_token, NodeData::ExprInitListTwoComma(first, second)))
                } else {
                    Ok(p.push_node(lbrace_token, NodeData::ExprInitListTwo(first, second)))
                }
            } else {
                let exprs = p.push_packed_list(&exprs).unwrap();
                if comma {
                    Ok(p.push_node(lbrace_token, NodeData::ExprInitListComma(exprs)))
                } else {
                    Ok(p.push_node(lbrace_token, NodeData::ExprInitList(exprs)))
                }
            }
        }
    }
}

fn expect_template_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let with_token = p.expect_token(Token!(with))?;
    p.expect_token(Token!('['))?;
    match p.tag() {
        // with[] -> type_expr
        Token!(']') => {
            p.next();
            p.next();
            let expr = expect_type_expr(p)?;
            Ok(p.push_node(with_token, NodeData::TemplateTypeExprOne(None, expr)))
        }
        // with[...] -> type_expr
        Token!(...) if p.peek2(Token!(']')) => {
            p.next();
            p.next();
            p.expect_token(Token!(->))?;
            let expr = expect_type_expr(p)?;
            Ok(p.push_node(with_token, NodeData::TemplateTypeExprOneDots(None, expr)))
        }
        // with[...,] -> type_expr
        Token!(...) if p.peek2(Token!(,)) => {
            p.next();
            p.next();
            p.expect_token(Token!(']'))?;
            p.expect_token(Token!(->))?;
            let expr = expect_type_expr(p)?;
            Ok(p.push_node(
                with_token,
                NodeData::TemplateTypeExprOneDotsComma(None, expr),
            ))
        }
        // with[x = ...] -> type_expr
        _ => {
            let type_expr = expect_type_expr(p)?;
            let has_init = if p.eat_token(Token!(=)) {
                p.expect_token(Token!(...))?;
                true
            } else {
                false
            };
            let comma = p.eat_token(Token!(,));
            if !comma && !has_init {
                p.expect_token(Token!(']'))?;
                p.expect_token(Token!(->))?;
                let expr = expect_type_expr(p)?;
                return Ok(p.push_node(
                    with_token,
                    NodeData::TemplateTypeExprOne(Some(type_expr), expr),
                ));
            }

            if !has_init {
                match p.tag() {
                    Token!(']') => {
                        p.next();
                        p.expect_token(Token!(->))?;
                        let expr = expect_type_expr(p)?;
                        return Ok(p.push_node(
                            with_token,
                            NodeData::TemplateTypeExprOneComma(type_expr, expr),
                        ));
                    }
                    Token!(...) if p.peek2(Token!(']')) => {
                        p.next();
                        p.next();
                        p.expect_token(Token!(->))?;
                        let expr = expect_type_expr(p)?;
                        return Ok(p.push_node(
                            with_token,
                            NodeData::TemplateTypeExprOneDots(Some(type_expr), expr),
                        ));
                    }
                    Token!(...) if p.peek2(Token!(,)) => {
                        p.next();
                        p.next();
                        p.expect_token(Token!(']'))?;
                        p.expect_token(Token!(->))?;
                        let expr = expect_type_expr(p)?;
                        return Ok(p.push_node(
                            with_token,
                            NodeData::TemplateTypeExprOneDotsComma(Some(type_expr), expr),
                        ));
                    }
                    _ => {}
                }
            }

            let mut comma = true;
            let mut args = vec![TemplateTypeExprArg {
                type_expr,
                has_init,
            }];
            while !p.eat_token(Token!(']')) {
                match p.tag() {
                    Token!(...) if p.peek2(Token!(']')) => {
                        p.next();
                        p.next();
                        p.expect_token(Token!(->))?;
                        let expr = expect_type_expr(p)?;
                        let args = p.push_packed_list(&args).unwrap();
                        let args = p.push_packed(args);
                        return Ok(
                            p.push_node(with_token, NodeData::TemplateTypeExprDots(args, expr))
                        );
                    }
                    Token!(...) if p.peek2(Token!(,)) => {
                        p.next();
                        p.next();
                        p.expect_token(Token!(']'))?;
                        p.expect_token(Token!(->))?;
                        let expr = expect_type_expr(p)?;
                        let args = p.push_packed_list(&args).unwrap();
                        let args = p.push_packed(args);
                        return Ok(p.push_node(
                            with_token,
                            NodeData::TemplateTypeExprDotsComma(args, expr),
                        ));
                    }
                    _ => {}
                }

                let type_expr = expect_type_expr(p)?;
                let has_init = if p.eat_token(Token!(=)) {
                    p.expect_token(Token!(...))?;
                    true
                } else {
                    false
                };
                comma = p.eat_token(Token!(,));
                args.push(TemplateTypeExprArg {
                    type_expr,
                    has_init,
                });
            }

            p.expect_token(Token!(->))?;
            let expr = expect_type_expr(p)?;
            let args = p.push_packed_list(&args).unwrap();
            let args = p.push_packed(args);
            if comma {
                Ok(p.push_node(with_token, NodeData::TemplateTypeExprComma(args, expr)))
            } else {
                Ok(p.push_node(with_token, NodeData::TemplateTypeExpr(args, expr)))
            }
        }
    }
}
