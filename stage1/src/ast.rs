use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    marker::PhantomData,
    num::NonZero,
    str,
};

use crate::packed_stream::{self, BitPacked, PackedStreamReader, PackedStreamWriter};
use crate::{lexer, packed_stream::Packable};

#[macro_export]
macro_rules! Token {
    (#<eof>) => {
        $crate::lexer::Tag::EndOfFile
    };
    (#<identifier>) => {
        $crate::lexer::Tag::Ident
    };
    (#<core_identifier>) => {
        $crate::lexer::Tag::IdentCore
    };
    (#<builtin_identifier>) => {
        $crate::lexer::Tag::IdentBuiltin
    };
    (#<char_literal>) => {
        $crate::lexer::Tag::LitChar
    };
    (#<string_literal>) => {
        $crate::lexer::Tag::LitString
    };
    (#<raw_string_literal>) => {
        $crate::lexer::Tag::LitRawString
    };
    (#<int_literal>) => {
        $crate::lexer::Tag::LitInt
    };
    (#<float_literal>) => {
        $crate::lexer::Tag::LitFloat
    };
    (#<inner_doc_comment>) => {
        $crate::lexer::Tag::InnerDocComment
    };
    (#<outer_doc_comment>) => {
        $crate::lexer::Tag::OuterDocComment
    };

    (&) => {
        $crate::lexer::Tag::SymAmpersand
    };
    (&=) => {
        $crate::lexer::Tag::SymAmpersandEqual
    };
    (*) => {
        $crate::lexer::Tag::SymAsterisk
    };
    (*=) => {
        $crate::lexer::Tag::SymAsteriskEqual
    };
    (*%) => {
        $crate::lexer::Tag::SymAsteriskPercent
    };
    (*%=) => {
        $crate::lexer::Tag::SymAsteriskPercentEqual
    };
    (*|) => {
        $crate::lexer::Tag::SymAsteriskPipe
    };
    (*|=) => {
        $crate::lexer::Tag::SymAsteriskPipeEqual
    };
    (^) => {
        $crate::lexer::Tag::SymCaret
    };
    (^=) => {
        $crate::lexer::Tag::SymCaretEqual
    };
    (:) => {
        $crate::lexer::Tag::SymColon
    };
    (::) => {
        $crate::lexer::Tag::SymColon2
    };
    (,) => {
        $crate::lexer::Tag::SymComma
    };
    (.) => {
        $crate::lexer::Tag::SymDot
    };
    (..) => {
        $crate::lexer::Tag::SymDot2
    };
    (..=) => {
        $crate::lexer::Tag::SymDot2Equal
    };
    (...) => {
        $crate::lexer::Tag::SymDot3
    };
    (.&) => {
        $crate::lexer::Tag::SymDotAmpersand
    };
    (.*) => {
        $crate::lexer::Tag::SymDotAsterisk
    };
    (.!) => {
        $crate::lexer::Tag::SymDotExclamationmark
    };
    (.?) => {
        $crate::lexer::Tag::SymDotQuestionmark
    };
    (=) => {
        $crate::lexer::Tag::SymEqual
    };
    (==) => {
        $crate::lexer::Tag::SymEqualEqual
    };
    (=>) => {
        $crate::lexer::Tag::SymEqualArrow
    };
    (!) => {
        $crate::lexer::Tag::SymExclamationmark
    };
    (!=) => {
        $crate::lexer::Tag::SymExclamationmarkEqual
    };
    (<) => {
        $crate::lexer::Tag::SymLArrow
    };
    (<<) => {
        $crate::lexer::Tag::SymLArrow2
    };
    (<<=) => {
        $crate::lexer::Tag::SymLArrow2Equal
    };
    (<<|) => {
        $crate::lexer::Tag::SymLArrow2Pipe
    };
    (<<|=) => {
        $crate::lexer::Tag::SymLArrow2PipeEqual
    };
    (<=) => {
        $crate::lexer::Tag::SymLArrowEqual
    };
    (<=>) => {
        $crate::lexer::Tag::SymLArrowEqualRArrow
    };
    ('{') => {
        $crate::lexer::Tag::SymLBrace
    };
    ('[') => {
        $crate::lexer::Tag::SymLBracket
    };
    ('(') => {
        $crate::lexer::Tag::SymLParen
    };
    (-) => {
        $crate::lexer::Tag::SymMinus
    };
    (-=) => {
        $crate::lexer::Tag::SymMinusEqual
    };
    (-%) => {
        $crate::lexer::Tag::SymMinusPercent
    };
    (-%=) => {
        $crate::lexer::Tag::SymMinusPercentEqual
    };
    (-|) => {
        $crate::lexer::Tag::SymMinusPipe
    };
    (-|=) => {
        $crate::lexer::Tag::SymMinusPipeEqual
    };
    (->) => {
        $crate::lexer::Tag::SymMinusArrow
    };
    (%) => {
        $crate::lexer::Tag::SymPercent
    };
    (%=) => {
        $crate::lexer::Tag::SymPercentEqual
    };
    (|) => {
        $crate::lexer::Tag::SymPipe
    };
    (|=) => {
        $crate::lexer::Tag::SymPipeEqual
    };
    (+) => {
        $crate::lexer::Tag::SymPlus
    };
    (++) => {
        $crate::lexer::Tag::SymPlus2
    };
    (+=) => {
        $crate::lexer::Tag::SymPlusEqual
    };
    (+%) => {
        $crate::lexer::Tag::SymPlusPercent
    };
    (+%=) => {
        $crate::lexer::Tag::SymPlusPercentEqual
    };
    (+|) => {
        $crate::lexer::Tag::SymPlusPipe
    };
    (+|=) => {
        $crate::lexer::Tag::SymPlusPipeEqual
    };
    (#) => {
        $crate::lexer::Tag::SymPound
    };
    (#!) => {
        $crate::lexer::Tag::SymPoundExclamationmark
    };
    (?) => {
        $crate::lexer::Tag::SymQuestionmark
    };
    (>) => {
        $crate::lexer::Tag::SymRArrow
    };
    (>>) => {
        $crate::lexer::Tag::SymRArrow2
    };
    (>>=) => {
        $crate::lexer::Tag::SymRArrow2Equal
    };
    (>=) => {
        $crate::lexer::Tag::SymRArrowEqual
    };
    ('}') => {
        $crate::lexer::Tag::SymRBrace
    };
    (']') => {
        $crate::lexer::Tag::SymRBracket
    };
    (')') => {
        $crate::lexer::Tag::SymRParen
    };
    (;) => {
        $crate::lexer::Tag::SymSemicolon
    };
    (/) => {
        $crate::lexer::Tag::SymSlash
    };
    (/=) => {
        $crate::lexer::Tag::SymSlashEqual
    };
    (~) => {
        $crate::lexer::Tag::SymTilde
    };

    (align) => {
        $crate::lexer::Tag::KwAlign
    };
    (and) => {
        $crate::lexer::Tag::KwAnd
    };
    (asm) => {
        $crate::lexer::Tag::KwAsm
    };
    (break) => {
        $crate::lexer::Tag::KwBreak
    };
    (callconv) => {
        $crate::lexer::Tag::KwCallconv
    };
    (catch) => {
        $crate::lexer::Tag::KwCatch
    };
    (const) => {
        $crate::lexer::Tag::KwConst
    };
    (context) => {
        $crate::lexer::Tag::KwContext
    };
    (cont_defer) => {
        $crate::lexer::Tag::KwContDefer
    };
    (continue) => {
        $crate::lexer::Tag::KwContinue
    };
    (defer) => {
        $crate::lexer::Tag::KwDefer
    };
    (else) => {
        $crate::lexer::Tag::KwElse
    };
    (enum) => {
        $crate::lexer::Tag::KwEnum
    };
    (err_defer) => {
        $crate::lexer::Tag::KwErrDefer
    };
    (fn) => {
        $crate::lexer::Tag::KwFn
    };
    (fnptr) => {
        $crate::lexer::Tag::KwFnptr
    };
    (for) => {
        $crate::lexer::Tag::KwFor
    };
    (if) => {
        $crate::lexer::Tag::KwIf
    };
    (inline) => {
        $crate::lexer::Tag::KwInline
    };
    (namespace) => {
        $crate::lexer::Tag::KwNamespace
    };
    (no_alias) => {
        $crate::lexer::Tag::KwNoAlias
    };
    (no_inline) => {
        $crate::lexer::Tag::KwNoInline
    };
    (opaque) => {
        $crate::lexer::Tag::KwOpaque
    };
    (or) => {
        $crate::lexer::Tag::KwOr
    };
    (or_else) => {
        $crate::lexer::Tag::KwOrElse
    };
    (#primitive) => {
        $crate::lexer::Tag::KwPrimitive
    };
    (pub) => {
        $crate::lexer::Tag::KwPub
    };
    (#run) => {
        $crate::lexer::Tag::KwRun
    };
    (return) => {
        $crate::lexer::Tag::KwReturn
    };
    (Self) => {
        $crate::lexer::Tag::KwSelf
    };
    (self) => {
        $crate::lexer::Tag::KwSelfIdent
    };
    (static) => {
        $crate::lexer::Tag::KwStatic
    };
    (struct) => {
        $crate::lexer::Tag::KwStruct
    };
    (switch) => {
        $crate::lexer::Tag::KwSwitch
    };
    (thread_local) => {
        $crate::lexer::Tag::KwThreadLocal
    };
    (throw) => {
        $crate::lexer::Tag::KwThrow
    };
    (try) => {
        $crate::lexer::Tag::KwTry
    };
    (union) => {
        $crate::lexer::Tag::KwUnion
    };
    (unreachable) => {
        $crate::lexer::Tag::KwUnreachable
    };
    (var) => {
        $crate::lexer::Tag::KwVar
    };
    (volatile) => {
        $crate::lexer::Tag::KwVolatile
    };
    (where) => {
        $crate::lexer::Tag::KwWhere
    };
    (while) => {
        $crate::lexer::Tag::KwWhile
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    pub tag: lexer::Tag,
    pub start: u32,
    pub length: u32,
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

    pub const fn prev(self) -> Self {
        assert!(self.get() != 0);
        Self(unsafe { NonZero::new_unchecked(self.0.get() - 1) })
    }

    pub const fn next(self) -> Self {
        assert!(self.get() < u32::MAX);
        Self(unsafe { NonZero::new_unchecked(self.0.get() + 1) })
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
    pub source: Box<[u8]>,
    pub tokens: Box<[Token]>,
    pub nodes: Box<[Node]>,
    pub extra_data: Box<[u32]>,
    pub errors: Box<[Error]>,
}

impl Ast {
    pub fn parse(source: Box<[u8]>) -> Self {
        let tokenizer = lexer::Tokenizer::new(&source);
        let tokens = tokenizer
            .map(|t| Token {
                tag: t.tag,
                start: t.span.start as u32,
                length: (t.span.end - t.span.start) as u32,
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

    pub fn get_source(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.source) }
    }

    pub fn get_node(&self, idx: NodeIndex) -> &Node {
        let idx = idx.get();
        &self.nodes[idx]
    }

    pub fn get_token(&self, idx: TokenIndex) -> Token {
        let idx = idx.get() as usize;
        self.tokens[idx]
    }

    pub fn get_ident(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<identifier>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_core_ident(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<core_identifier>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_builtin_ident(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<builtin_identifier>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_char_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<char_literal>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_float_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<float_literal>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_int_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<int_literal>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_string_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<string_literal>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_raw_string_lit(&self, idx: TokenIndex) -> &str {
        let idx = idx.get() as usize;
        let token = self.tokens[idx];
        assert_eq!(token.tag, Token!(#<raw_string_literal>));
        let start = token.start as usize;
        let end = start + token.length as usize;
        let bytes = &self.source[start..end];
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    pub fn get_packed<T: Packable>(&self, idx: ExtraIndex<T>) -> T {
        let start = idx.get();
        let mut stream = PackedStreamReader::new(&self.extra_data[start..]);
        stream.read()
    }

    pub fn get_node_leftmost_token(&self, mut idx: NodeIndex) -> TokenIndex {
        loop {
            let node = self.get_node(idx);
            return match node.data {
                NodeData::Root(_) => node.main_token,
                NodeData::RootStructAttribute(_, _) => node.main_token,
                NodeData::RootStructConstAttribute(_, _) => node.main_token,
                NodeData::RootStructLayoutAttribute(_, _) => node.main_token,
                NodeData::RootStructLayoutConstAttribute(_, _) => node.main_token,
                NodeData::OuterAnnotation(_, _) => node.main_token,
                NodeData::InnerAnnotation(_, _) => node.main_token,
                NodeData::Const(_) => node.main_token,
                NodeData::Run(_) => node.main_token,
                NodeData::Defer(_) => node.main_token,
                NodeData::Decl(decl_list, _) => {
                    let decl_list = self.get_packed(decl_list);
                    let first = self.get_packed(decl_list.prototypes.start());
                    let mut token = first.ident;
                    if first.is_pub {
                        token = token.prev();
                    }
                    if first.is_var {
                        token = token.prev();
                    }
                    if first.is_inline {
                        token = token.prev();
                    }
                    if !matches!(decl_list.decl_type, DeclType::Normal | DeclType::Const) {
                        token = token.prev();
                    }

                    token
                }
                NodeData::BlockTwo(_, _) => node.main_token,
                NodeData::BlockTwoSemicolon(_, _) => node.main_token,
                NodeData::Block(_) => node.main_token,
                NodeData::BlockSemicolon(_) => node.main_token,
                NodeData::ExprSemicolon(expr) => {
                    idx = expr;
                    continue;
                }
                NodeData::Range(lhs, _) => {
                    if let Some(lhs) = lhs {
                        idx = lhs;
                        continue;
                    } else {
                        node.main_token
                    }
                }
                NodeData::Binary(lhs, _) => {
                    idx = lhs;
                    continue;
                }
                NodeData::Unary(_) => node.main_token,
                NodeData::AsmSimple(_, _) => node.main_token,
                NodeData::AsmVolatileSimple(_, _) => node.main_token,
                NodeData::Asm(_, _) => node.main_token,
                NodeData::AsmVolatile(_, _) => node.main_token,
                NodeData::Jump(_, _) => node.main_token,
                NodeData::Label(_) => node.main_token,
                NodeData::TryBlock(_) => node.main_token,
                NodeData::Fn(_, _) => node.main_token,
                NodeData::Payload(is_ptr, _) => {
                    if is_ptr {
                        node.main_token.prev()
                    } else {
                        node.main_token
                    }
                }
                NodeData::ForSimple(_, _) => node.main_token,
                NodeData::ForSimpleInline(_, _) => node.main_token,
                NodeData::For(_, _) => node.main_token,
                NodeData::ForInline(_, _) => node.main_token,
                NodeData::ExprInit(expr, _) => {
                    idx = expr;
                    continue;
                }
                NodeData::TokenInit(_) => node.main_token,
                NodeData::ExprInitListTwo(_, _) => node.main_token,
                NodeData::ExprInitListTwoComma(_, _) => node.main_token,
                NodeData::ExprInitList(_) => node.main_token,
                NodeData::ExprInitListComma(_) => node.main_token,
                NodeData::FieldInitListTwo(_, _) => node.main_token,
                NodeData::FieldInitListTwoComma(_, _) => node.main_token,
                NodeData::FieldInitList(_) => node.main_token,
                NodeData::FieldInitListComma(_) => node.main_token,
                NodeData::ErrorUnionType(lhs, _) => {
                    idx = lhs;
                    continue;
                }
                NodeData::Pack(ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::OptionType(_) => node.main_token,
                NodeData::Slice(_, _) => node.main_token,
                NodeData::SinglePointerSimple(_, _) => node.main_token,
                NodeData::SinglePointer(_, _) => node.main_token,
                NodeData::MultiPointerSimple(_, _) => node.main_token,
                NodeData::MultiPointer(_, _) => node.main_token,
                NodeData::Vector(_, _) => node.main_token,
                NodeData::MatrixSimple(_, _) => node.main_token,
                NodeData::Matrix(_, _) => node.main_token,
                NodeData::ArraySimple(_, _) => node.main_token,
                NodeData::Array(_, _) => node.main_token,
                NodeData::EnumLiteral(_) => node.main_token,
                NodeData::Ident => node.main_token,
                NodeData::CoreIdent => node.main_token,
                NodeData::BuiltinIdent => node.main_token,
                NodeData::WhereExpr(_) => node.main_token,
                NodeData::SelfType => node.main_token,
                NodeData::SelfIdent => node.main_token,
                NodeData::Unreachable => node.main_token,
                NodeData::CharLiteral => node.main_token,
                NodeData::FloatLiteral => node.main_token,
                NodeData::IntLiteral => node.main_token,
                NodeData::StringLiteral => node.main_token,
                NodeData::RawStringLiteral(_, _) => node.main_token,
                NodeData::Container(_, _) => node.main_token,
                NodeData::ContainerConst(_, _) => node.main_token,
                NodeData::Primitive(_, _) => node.main_token,
                NodeData::Index(ty_expr, _) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::Call1(ty_expr, _) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::Call1Comma(ty_expr, _) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::Call(ty_expr, _) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::CallComma(ty_expr, _) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::TypeBinarySuffix(ty_expr, _) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::TypeUnarySuffix(ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
            };
        }
    }

    pub fn get_node_rightmost_token(&self, mut idx: NodeIndex) -> TokenIndex {
        loop {
            let node = self.get_node(idx);
            return match node.data {
                NodeData::Root(members) => match members {
                    Some(members) => {
                        idx = self.get_packed(members.end());
                        continue;
                    }
                    None => node.main_token,
                },
                NodeData::RootStructAttribute(_, tok) => tok,
                NodeData::RootStructConstAttribute(_, tok) => tok,
                NodeData::RootStructLayoutAttribute(_, tok) => tok,
                NodeData::RootStructLayoutConstAttribute(_, tok) => tok,
                NodeData::OuterAnnotation(_, tok) => tok,
                NodeData::InnerAnnotation(_, tok) => tok,
                NodeData::Const(node_index) => {
                    idx = node_index;
                    continue;
                }
                NodeData::Run(node_index) => {
                    idx = node_index;
                    continue;
                }
                NodeData::Defer(node_index) => {
                    idx = node_index;
                    continue;
                }
                NodeData::Decl(_, _) => node.main_token,
                NodeData::BlockTwo(stmt_1, stmt_2) => {
                    let mut last_token = if let Some(idx) = stmt_2 {
                        self.get_node_rightmost_token(idx).next()
                    } else if let Some(idx) = stmt_1 {
                        self.get_node_rightmost_token(idx).next()
                    } else {
                        node.main_token.next()
                    };
                    let mut token = self.get_token(last_token);
                    while token.tag != Token!('}') {
                        last_token = last_token.next();
                        token = self.get_token(last_token);
                    }
                    last_token
                }
                NodeData::BlockTwoSemicolon(stmt_1, stmt_2) => {
                    let mut last_token = if let Some(idx) = stmt_2 {
                        self.get_node_rightmost_token(idx).next()
                    } else if let Some(idx) = stmt_1 {
                        self.get_node_rightmost_token(idx).next()
                    } else {
                        node.main_token.next()
                    };
                    let mut token = self.get_token(last_token);
                    while token.tag != Token!('}') {
                        last_token = last_token.next();
                        token = self.get_token(last_token);
                    }
                    last_token.next()
                }
                NodeData::Block(stmts) => {
                    let last_stmt = self.get_packed(stmts.end());
                    let mut last_token = self.get_node_rightmost_token(last_stmt).next();
                    let mut token = self.get_token(last_token);
                    while token.tag != Token!('}') {
                        last_token = last_token.next();
                        token = self.get_token(last_token);
                    }
                    last_token
                }
                NodeData::BlockSemicolon(stmts) => {
                    let last_stmt = self.get_packed(stmts.end());
                    let mut last_token = self.get_node_rightmost_token(last_stmt).next();
                    let mut token = self.get_token(last_token);
                    while token.tag != Token!('}') {
                        last_token = last_token.next();
                        token = self.get_token(last_token);
                    }
                    last_token.next()
                }
                NodeData::ExprSemicolon(_) => node.main_token,
                NodeData::Range(_, rhs) => {
                    if let Some(rhs) = rhs {
                        idx = rhs;
                        continue;
                    } else {
                        node.main_token
                    }
                }
                NodeData::Binary(_, rhs) => {
                    idx = rhs;
                    continue;
                }
                NodeData::Unary(expr) => {
                    idx = expr;
                    continue;
                }
                NodeData::AsmSimple(node_index, node_index1) => todo!(),
                NodeData::AsmVolatileSimple(node_index, node_index1) => todo!(),
                NodeData::Asm(extra_index, node_index) => todo!(),
                NodeData::AsmVolatile(extra_index, node_index) => todo!(),
                NodeData::Jump(_, expr) => {
                    if let Some(expr) = expr {
                        idx = expr;
                        continue;
                    } else {
                        node.main_token
                    }
                }
                NodeData::Label(expr) => {
                    idx = expr;
                    continue;
                }
                NodeData::TryBlock(block) => {
                    idx = block;
                    continue;
                }
                NodeData::Fn(_, block) => {
                    idx = block;
                    continue;
                }
                NodeData::Payload(_, trailing_comma) => {
                    if trailing_comma {
                        node.main_token.next()
                    } else {
                        node.main_token
                    }
                }
                NodeData::ForSimple(extra_index, node_index) => todo!(),
                NodeData::ForSimpleInline(extra_index, node_index) => todo!(),
                NodeData::For(extra_index, node_index) => todo!(),
                NodeData::ForInline(extra_index, node_index) => todo!(),
                NodeData::ExprInit(_, block) => {
                    idx = block;
                    continue;
                }
                NodeData::TokenInit(block) => {
                    idx = block;
                    continue;
                }
                NodeData::ExprInitListTwo(node_index, node_index1) => todo!(),
                NodeData::ExprInitListTwoComma(node_index, node_index1) => todo!(),
                NodeData::ExprInitList(extra_index_range) => todo!(),
                NodeData::ExprInitListComma(extra_index_range) => todo!(),
                NodeData::FieldInitListTwo(extra_index, extra_index1) => todo!(),
                NodeData::FieldInitListTwoComma(extra_index, extra_index1) => todo!(),
                NodeData::FieldInitList(extra_index_range) => todo!(),
                NodeData::FieldInitListComma(extra_index_range) => todo!(),
                NodeData::ErrorUnionType(_, rhs) => {
                    idx = rhs;
                    continue;
                }
                NodeData::Pack(_) => node.main_token,
                NodeData::OptionType(expr) => {
                    idx = expr;
                    continue;
                }
                NodeData::Slice(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::SinglePointerSimple(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::SinglePointer(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::MultiPointerSimple(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::MultiPointer(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::Vector(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::MatrixSimple(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::Matrix(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::ArraySimple(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::Array(_, ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::EnumLiteral(tok) => tok,
                NodeData::Ident => node.main_token,
                NodeData::CoreIdent => node.main_token,
                NodeData::BuiltinIdent => node.main_token,
                NodeData::WhereExpr(ty_expr) => {
                    idx = ty_expr;
                    continue;
                }
                NodeData::SelfType => node.main_token,
                NodeData::SelfIdent => node.main_token,
                NodeData::Unreachable => node.main_token,
                NodeData::CharLiteral => node.main_token,
                NodeData::FloatLiteral => node.main_token,
                NodeData::IntLiteral => node.main_token,
                NodeData::StringLiteral => node.main_token,
                NodeData::RawStringLiteral(_, tok) => tok,
                NodeData::Container(_, block) => {
                    idx = block;
                    continue;
                }
                NodeData::ContainerConst(_, block) => {
                    idx = block;
                    continue;
                }
                NodeData::Primitive(token_index, node_index) => todo!(),
                NodeData::Index(_, _) => node.main_token,
                NodeData::Call1(_, _) => node.main_token,
                NodeData::Call1Comma(_, _) => node.main_token,
                NodeData::Call(_, _) => node.main_token,
                NodeData::CallComma(_, _) => node.main_token,
                NodeData::TypeBinarySuffix(_, ident) => ident,
                NodeData::TypeUnarySuffix(_) => node.main_token,
            };
        }
    }

    pub fn get_node_token_span(&self, idx: NodeIndex) -> (TokenIndex, TokenIndex) {
        let left = self.get_node_leftmost_token(idx);
        let right = self.get_node_rightmost_token(idx);
        (left, right)
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
                NodeData::Run(expr) => writeln!(f, "{idx} := #run {expr}")?,
                NodeData::Defer(expr) => {
                    let defer_op = self.get_token(main_token);
                    let defer_op = defer_op.tag.as_lexeme().unwrap();
                    writeln!(f, "{idx} := {defer_op} {expr}")?
                }
                NodeData::Decl(decl_list, init_exprs) => {
                    let DeclList {
                        decl_type: decl_ty,
                        prototypes,
                        type_expr,
                    } = self.get_packed(*decl_list);
                    match decl_ty {
                        DeclType::Normal => write!(f, "{idx} := decl(protos := [")?,
                        DeclType::Const => write!(f, "{idx} := const_decl(protos := [")?,
                        DeclType::ThreadLocal => {
                            write!(f, "{idx} := thread_local_decl(protos := [")?
                        }
                        DeclType::Static => write!(f, "{idx} := static_decl(protos := [")?,
                    }

                    for (i, prototype) in prototypes.enumerate() {
                        let DeclProto {
                            is_pub,
                            is_var,
                            is_inline,
                            ident,
                            align_expr,
                            annotations,
                        } = self.get_packed(prototype);
                        let ident = self.get_ident(ident);

                        if i != 0 {
                            write!(f, ",")?;
                        }
                        write!(f, "\n\t{{ name := {ident}")?;
                        if is_pub {
                            write!(f, ", pub")?;
                        }
                        if is_var {
                            write!(f, ", var")?;
                        }
                        if is_inline {
                            write!(f, ", inline")?;
                        }
                        if let Some(align_expr) = align_expr {
                            write!(f, ", align := {align_expr}")?;
                        }
                        if let Some(annotations) = annotations {
                            write!(f, ", annotations := [")?;
                            for (j, expr) in annotations.enumerate() {
                                let expr = self.get_packed(expr);
                                if j != 0 {
                                    write!(f, ", ")?;
                                }
                                write!(f, "{expr}")?;
                            }
                            write!(f, "]")?;
                        } else {
                            write!(f, ", annotations := []")?;
                        }
                        write!(f, "}}")?;
                    }
                    write!(f, "],\n\t")?;
                    if let Some(type_expr) = type_expr {
                        write!(f, "type := {type_expr}, ")?
                    }

                    let init_exprs = self.get_packed(*init_exprs);
                    write!(f, "init := [")?;
                    for (i, init_expr) in init_exprs.enumerate() {
                        let init_expr = self.get_packed(init_expr);

                        if i != 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{init_expr}")?;
                    }
                    writeln!(f, "])")?
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
                NodeData::ExprSemicolon(expr) => writeln!(f, "{idx} := expr({expr});")?,
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
                NodeData::Label(expr) => {
                    let ident = self.get_ident(main_token);
                    writeln!(f, "{idx} := label({ident}, expr := {expr})")?
                }
                NodeData::TryBlock(expr) => writeln!(f, "{idx} := try(expr := {expr})")?,
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
                NodeData::Payload(is_ptr, has_comma) => {
                    let ident = self.get_ident(main_token);
                    writeln!(
                        f,
                        "{idx} := payload(ident := {ident}, is_ptr := {is_ptr}, has_comma := {has_comma})"
                    )?
                }
                NodeData::ForSimple(main_expr, else_expr) => {
                    let ForSimple {
                        iter_expr1,
                        iter_expr2,
                        payload1,
                        payload2,
                        expr,
                    } = self.get_packed(*main_expr);
                    write!(f, "{idx} := for(iterators := [")?;
                    if let Some(iter_expr) = iter_expr1 {
                        write!(f, "{iter_expr}")?;
                    }
                    if let Some(iter_expr) = iter_expr2 {
                        write!(f, ", {iter_expr}")?;
                    }
                    write!(f, "], payloads := [")?;
                    if let Some(payload) = payload1 {
                        write!(f, "{payload}")?;
                    }
                    if let Some(payload) = payload2 {
                        write!(f, ", {payload}")?;
                    }
                    write!(f, "], expr := {expr}")?;
                    if let Some(else_expr) = else_expr {
                        write!(f, ", else_expr := {else_expr}")?;
                    }
                    writeln!(f, ")")?;
                }
                NodeData::ForSimpleInline(main_expr, else_expr) => {
                    let ForSimple {
                        iter_expr1,
                        iter_expr2,
                        payload1,
                        payload2,
                        expr,
                    } = self.get_packed(*main_expr);
                    write!(f, "{idx} := for(iterators := [")?;
                    if let Some(iter_expr) = iter_expr1 {
                        write!(f, "{iter_expr}")?;
                    }
                    if let Some(iter_expr) = iter_expr2 {
                        write!(f, ", {iter_expr}")?;
                    }
                    write!(f, "], payloads := [")?;
                    if let Some(payload) = payload1 {
                        write!(f, "{payload}")?;
                    }
                    if let Some(payload) = payload2 {
                        write!(f, ", {payload}")?;
                    }
                    write!(f, "], expr := {expr}")?;
                    if let Some(else_expr) = else_expr {
                        write!(f, ", else_expr := {else_expr}")?;
                    }
                    writeln!(f, ", inline := true)")?;
                }
                NodeData::For(main_expr, else_expr) => {
                    let For {
                        iter_exprs,
                        payloads,
                        expr,
                    } = self.get_packed(*main_expr);
                    write!(f, "{idx} := for(iterators := [")?;
                    if let Some(iter_exprs) = iter_exprs {
                        for (i, iter) in iter_exprs.enumerate() {
                            let iter = self.get_packed(iter);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{iter}")?;
                        }
                    }
                    write!(f, "], payloads := [")?;
                    if let Some(payloads) = payloads {
                        for (i, payload) in payloads.enumerate() {
                            let payload = self.get_packed(payload);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{payload}")?;
                        }
                    }
                    write!(f, "], expr := {expr}")?;
                    if let Some(else_expr) = else_expr {
                        write!(f, ", else_expr := {else_expr}")?;
                    }
                    writeln!(f, ")")?;
                }
                NodeData::ForInline(main_expr, else_expr) => {
                    let For {
                        iter_exprs,
                        payloads,
                        expr,
                    } = self.get_packed(*main_expr);
                    write!(f, "{idx} := for(iterators := [")?;
                    if let Some(iter_exprs) = iter_exprs {
                        for (i, iter) in iter_exprs.enumerate() {
                            let iter = self.get_packed(iter);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{iter}")?;
                        }
                    }
                    write!(f, "], payloads := [")?;
                    if let Some(payloads) = payloads {
                        for (i, payload) in payloads.enumerate() {
                            let payload = self.get_packed(payload);
                            if i != 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{payload}")?;
                        }
                    }
                    write!(f, "], expr := {expr}")?;
                    if let Some(else_expr) = else_expr {
                        write!(f, ", else_expr := {else_expr}")?;
                    }
                    writeln!(f, ", inline := true)")?;
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
                NodeData::WhereExpr(type_expr) => {
                    writeln!(f, "{idx} := where(type := {type_expr})")?
                }
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
                NodeData::Primitive(id, block) => {
                    let id = self.get_string_lit(*id);
                    writeln!(f, "{idx} := #primitive({id}, block := {block})")?
                }
                NodeData::Index(expr, index_expr) => {
                    writeln!(f, "{idx} := index(expr := {expr}, index := {index_expr})")?
                }
                NodeData::Call1(type_expr, arg_expr) => {
                    if let Some(arg_expr) = arg_expr {
                        writeln!(
                            f,
                            "{idx} := call(expr := {type_expr}, args := [{arg_expr}])"
                        )?
                    } else {
                        writeln!(f, "{idx} := call(expr := {type_expr}, args := [])")?
                    }
                }
                NodeData::Call1Comma(type_expr, arg_expr) => {
                    let arg_expr = arg_expr.unwrap();
                    writeln!(
                        f,
                        "{idx} := call(expr := {type_expr}, args := [{arg_expr},])"
                    )?
                }
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
    pub is_warn: bool,
    pub token: TokenIndex,
    pub data: ErrorData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorData {
    DocCommentAtContainerEnd,
    DocCommentOnStatement,
    DocCommentOnExpression,
    ExpectedBlock,
    ExpectedBlockStatement,
    ExpectedVarInlineOrIdent,
    ExpectedInitExpression,
    ExpectedExpr,
    ExpectedPrimaryExpr,
    ExpectedTypeExpr,
    ExpectedTypeExprOrInit,
    ExpectedPrefixExpr,
    DoublePubToken,
    DoubleVarToken,
    DoubleInlineToken,
    DoubleVolatileToken,
    DoubleAlignment,
    PubTokenOutOfOrder,
    PubTokenAfterVarToken,
    StructFieldWithoutTypeOrInitExpr,
    UnexpectedSemicolonAfterBlock,
    ExpectedToken(lexer::Tag),
}

impl ErrorData {
    pub fn description(&self) -> Cow<'static, str> {
        match self {
            Self::DocCommentAtContainerEnd => "found doc-comment at the end of a container".into(),
            Self::DocCommentOnStatement => "found doc-comment on a statement".into(),
            Self::DocCommentOnExpression => "found doc-comment on an expression".into(),
            Self::ExpectedBlock => "expected a block".into(),
            Self::ExpectedBlockStatement => "expected a block statement".into(),
            Self::ExpectedVarInlineOrIdent => {
                "expected a `var`, `inline` or identifier token".into()
            }
            Self::ExpectedInitExpression => "expected an init expression".into(),
            Self::ExpectedExpr => "expected an expression".into(),
            Self::ExpectedPrimaryExpr => "expected a primary expression".into(),
            Self::ExpectedTypeExpr => "expected a type expression".into(),
            Self::ExpectedTypeExprOrInit => todo!(),
            Self::ExpectedPrefixExpr => "expected a prefix expression".into(),
            Self::DoublePubToken => "found a duplicate `pub` token".into(),
            Self::DoubleVarToken => "found a duplicate `var` token".into(),
            Self::DoubleInlineToken => "found a duplicate `inline` token".into(),
            Self::DoubleVolatileToken => "found a duplicate `volatile` token".into(),
            Self::DoubleAlignment => "found a duplicate alignment specifier".into(),
            Self::PubTokenOutOfOrder => {
                "found a `pub` token after a `var` or `inline` token".into()
            }
            Self::PubTokenAfterVarToken => todo!(),
            Self::StructFieldWithoutTypeOrInitExpr => todo!(),
            Self::UnexpectedSemicolonAfterBlock => {
                "found an unexpected semicolon after a block".into()
            }
            Self::ExpectedToken(tag) => format!("expected token `{tag}`").into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

impl<T: Packable> Debug for ExtraIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ExtraIndex").field(&self.0).finish()
    }
}

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
        self.1
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn len(&self) -> usize {
        let step = const { T::LEN };
        let end = self.end().get() + 1;
        let start = self.start().get();
        (end - start) / step
    }
}

impl<T: Packable> Packable for ExtraIndexRange<T> {
    const LEN: usize = <(ExtraIndex<T>, ExtraIndex<T>)>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.0);
        buffer.write(self.1);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self(buffer.read(), buffer.read())
    }
}

impl<T: Packable> Packable for Option<ExtraIndexRange<T>> {
    const LEN: usize = <(Option<ExtraIndex<T>>, Option<ExtraIndex<T>>)>::LEN;

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
    const LEN: usize = <(T, U)>::LEN;

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

    pub const fn next(self) -> Self {
        assert!(self.get_u32() < u32::MAX);
        Self(unsafe { NonZero::new_unchecked(self.0.get() + 1) })
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
    /// 1. List of attributes/members/fields
    ///
    /// `main_token` is the first token for the source file.
    Root(Option<ExtraIndexRange<NodeIndex>>),
    /// `#![struct]`
    /// 1. Index to the `#!` token.
    /// 2. Index to the `]` token.
    ///
    /// `main_token` is the `#!` token.
    RootStructAttribute(TokenIndex, TokenIndex),
    /// `#![struct const]`
    /// 1. Index to the `#!` token.
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
    /// `#run expr`
    /// 1. Index to the sub-expression.
    ///
    /// `main_token` is the `#run` token.
    Run(NodeIndex),
    /// `defer_op expr`
    /// 1. Index to the sub-expression.
    ///
    /// `main_token` is the defer_op token.
    Defer(NodeIndex),
    /// `decl_type pub a, ..., z : type_expr = expr_0, ..., expr_n;`
    /// 1. Declaration list.
    /// 2. Init expressions.
    ///
    /// `main_token` is the `;` token.
    Decl(ExtraIndex<DeclList>, ExtraIndex<ExtraIndexRange<NodeIndex>>),
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
    /// `expr;`
    /// 1. Expression.
    ///
    /// `main_token` is the `;` token.
    ExprSemicolon(NodeIndex),
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
    /// 1. Clobbers expression.
    /// 2. Instructions expression.
    ///
    /// `main_token` is the `asm` token.
    AsmSimple(NodeIndex, NodeIndex),
    /// `asm(expr) volatile { expr }`
    /// 1. Clobbers expression.
    /// 2. Instructions expression.
    ///
    /// `main_token` is the `asm` token.
    AsmVolatileSimple(NodeIndex, NodeIndex),
    /// `asm[...](..., expr) -> (...) { expr }`
    /// 1. Asm prototype.
    /// 2. Instructions expression.
    ///
    /// `main_token` is the `asm` token.
    Asm(ExtraIndex<AsmProto>, NodeIndex),
    /// `asm[...](..., expr) volatile -> (...) { expr }`
    /// 1. Asm prototype.
    /// 2. Instructions expression.
    ///
    /// `main_token` is the `asm` token.
    AsmVolatile(ExtraIndex<AsmProto>, NodeIndex),
    /// `jump_op label expr`
    /// 1. Jump label.
    /// 2. Jump value.
    ///
    /// `main_token` is the jump_op token.
    Jump(Option<TokenIndex>, Option<NodeIndex>),
    /// `label: expr`
    /// 1. Sub-expression.
    ///
    /// `main_token` is the label token.
    Label(NodeIndex),
    /// `try {...}`
    /// 1. Block.
    ///
    /// `main_token` is the `try` token.
    TryBlock(NodeIndex),
    /// `fn[...](...) modifier -> ret_type where(...) { ... }`
    /// 1. Funtion prototype
    /// 2. Block.
    ///
    /// `main_token` is the `fn` token.
    Fn(ExtraIndex<FnProto>, NodeIndex),
    /// `|... *ident, ...|`
    /// 1. Is pointer payload.
    /// 2. Has trailing comma.
    ///
    /// `main_token` is the ident token.
    Payload(bool, bool),
    // TODO: If
    /// `for (a, b) |c, d| expr else expr`
    /// 1. For expression without else expression.
    /// 2. Optional else expression
    ///
    /// `main_token` is the `for` token.
    ForSimple(ExtraIndex<ForSimple>, Option<NodeIndex>),
    /// `for (a, b) |c, d| inline expr else expr`
    /// 1. For expression without else expression.
    /// 2. Optional else expression
    ///
    /// `main_token` is the `for` token.
    ForSimpleInline(ExtraIndex<ForSimple>, Option<NodeIndex>),
    /// `for (a, b, c, d) |e, f, g, h| expr else expr`
    /// 1. For expression without else expression.
    /// 2. Optional else expression
    ///
    /// `main_token` is the `for` token.
    For(ExtraIndex<For>, Option<NodeIndex>),
    /// `for (a, b, c, d) |e, f, g, h| inline expr else expr`
    /// 1. For expression without else expression.
    /// 2. Optional else expression
    ///
    /// `main_token` is the `for` token.
    ForInline(ExtraIndex<For>, Option<NodeIndex>),
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
    /// `where type_expr`
    /// 1. Type expression
    ///
    /// `main_token` is the `where` token.
    WhereExpr(NodeIndex),
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
    /// `#primitive("...", {...})`
    /// 1. String token
    /// 2. Block
    ///
    /// `main_token` is the `#primitive` token.
    Primitive(TokenIndex, NodeIndex),
    /// `type_expr[expr]`
    /// 1. Type expression
    /// 2. Index expression
    ///
    /// `main_token` is the `]` token.
    Index(NodeIndex, NodeIndex),
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
    Call1Comma(NodeIndex, Option<NodeIndex>),
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
pub struct DeclProto {
    pub is_pub: bool,
    pub is_var: bool,
    pub is_inline: bool,
    pub ident: TokenIndex,
    pub align_expr: Option<NodeIndex>,
    pub annotations: Option<ExtraIndexRange<NodeIndex>>,
}

impl Packable for DeclProto {
    const LEN: usize = <(
        BitPacked<(bool, bool, bool)>,
        TokenIndex,
        Option<NodeIndex>,
        Option<ExtraIndexRange<NodeIndex>>,
    )>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits((
            self.is_pub,
            self.is_var,
            self.is_inline,
        )));
        buffer.write(self.ident);
        buffer.write(self.align_expr);
        buffer.write(self.annotations);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let (is_pub, is_var, is_inline) = buffer.read::<BitPacked<(bool, bool, bool)>>().unpack();
        let ident = buffer.read();
        let align_expr = buffer.read();
        let annotations = buffer.read();
        Self {
            is_pub,
            is_var,
            is_inline,
            ident,
            align_expr,
            annotations,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeclType {
    Normal = 0,
    Const = 1,
    ThreadLocal = 2,
    Static = 3,
}

impl packed_stream::DefaultPackable for DeclType {}

impl packed_stream::BitPackable for DeclType {
    const BITS: usize = 2;

    fn pack(self) -> u32 {
        self as u32
    }

    fn unpack(value: u32) -> Self {
        debug_assert!(value <= Self::Static as u32);
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclList {
    pub decl_type: DeclType,
    pub prototypes: ExtraIndexRange<DeclProto>,
    pub type_expr: Option<NodeIndex>,
}

impl Packable for DeclList {
    const LEN: usize = <(
        BitPacked<DeclType>,
        ExtraIndexRange<DeclProto>,
        Option<NodeIndex>,
    )>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits(self.decl_type));
        buffer.write(self.prototypes);
        buffer.write(self.type_expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let decl_type = buffer.read::<BitPacked<DeclType>>().unpack();
        let prototypes = buffer.read();
        let type_expr = buffer.read();
        Self {
            decl_type,
            prototypes,
            type_expr,
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
    const LEN: usize = <(TokenIndex, Option<NodeIndex>, Option<NodeIndex>)>::LEN;

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
    const LEN: usize = <(TokenIndex, TokenIndex)>::LEN;

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
    const LEN: usize = <(TokenIndex, NodeIndex, TokenIndex)>::LEN;

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
    )>::LEN;

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
    const LEN: usize = <(bool, TokenIndex, Option<NodeIndex>)>::LEN;

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
    const LEN: usize = <(TokenIndex, Option<NodeIndex>, Option<NodeIndex>)>::LEN;

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
    )>::LEN;

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
    )>::LEN;

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
    )>::LEN;

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
        debug_assert!(value <= Self::NoAlias as u32);
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
        debug_assert!(value <= Self::NoInline as u32);
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ForSimple {
    iter_expr1: Option<NodeIndex>,
    iter_expr2: Option<NodeIndex>,
    payload1: Option<NodeIndex>,
    payload2: Option<NodeIndex>,
    expr: NodeIndex,
}

impl Packable for ForSimple {
    const LEN: usize = <(
        Option<NodeIndex>,
        Option<NodeIndex>,
        Option<NodeIndex>,
        Option<NodeIndex>,
        NodeIndex,
    )>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.iter_expr1);
        buffer.write(self.iter_expr2);
        buffer.write(self.payload1);
        buffer.write(self.payload2);
        buffer.write(self.expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            iter_expr1: buffer.read(),
            iter_expr2: buffer.read(),
            payload1: buffer.read(),
            payload2: buffer.read(),
            expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct For {
    iter_exprs: Option<ExtraIndexRange<NodeIndex>>,
    payloads: Option<ExtraIndexRange<NodeIndex>>,
    expr: NodeIndex,
}

impl Packable for For {
    const LEN: usize = <(
        Option<ExtraIndexRange<NodeIndex>>,
        Option<ExtraIndexRange<NodeIndex>>,
        NodeIndex,
    )>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.iter_exprs);
        buffer.write(self.payloads);
        buffer.write(self.expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            iter_exprs: buffer.read(),
            payloads: buffer.read(),
            expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct While {
    condition_expr: NodeIndex,
    payload: Option<NodeIndex>,
    expr: NodeIndex,
}

impl Packable for While {
    const LEN: usize = <(NodeIndex, Option<NodeIndex>, NodeIndex)>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.condition_expr);
        buffer.write(self.payload);
        buffer.write(self.expr);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            condition_expr: buffer.read(),
            payload: buffer.read(),
            expr: buffer.read(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WhilePayloadElse {
    condition_expr: NodeIndex,
    payload: NodeIndex,
}

impl Packable for WhilePayloadElse {
    const LEN: usize = <(NodeIndex, NodeIndex)>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(self.condition_expr);
        buffer.write(self.payload);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        Self {
            condition_expr: buffer.read(),
            payload: buffer.read(),
        }
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
        debug_assert!(value <= Self::OptVarOptVolatile as u32);
        unsafe { std::mem::transmute(value as u8) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldInit {
    ident: TokenIndex,
    init_expr: NodeIndex,
}

impl Packable for FieldInit {
    const LEN: usize = <(TokenIndex, NodeIndex)>::LEN;

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
    const LEN: usize = <(PointerType, NodeIndex)>::LEN;

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
    const LEN: usize = <(PointerType, Option<NodeIndex>, Option<NodeIndex>)>::LEN;

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
    const LEN: usize = <(NodeIndex, NodeIndex, NodeIndex)>::LEN;

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

struct Parser<'a> {
    _source: &'a [u8],
    tokens: &'a [Token],
    index: TokenIndex,
    errors: Vec<Error>,
    nodes: Vec<Node>,
    extra_data: Vec<u32>,
}

#[derive(Debug, Clone, Copy)]
struct Snapshot {
    index: TokenIndex,
    errors_idx: usize,
    nodes_idx: usize,
    extra_data_idx: usize,
}

#[allow(dead_code)]
impl Parser<'_> {
    fn next(&mut self) -> TokenIndex {
        let idx = self.index;
        self.index = TokenIndex::new(self.index.get() + 1);
        idx
    }

    fn snapshot(&self) -> Snapshot {
        Snapshot {
            index: self.index,
            errors_idx: self.errors.len(),
            nodes_idx: self.nodes.len(),
            extra_data_idx: self.extra_data.len(),
        }
    }

    fn rollback(&mut self, snapshot: Snapshot) {
        let Snapshot {
            index,
            errors_idx,
            nodes_idx,
            extra_data_idx,
        } = snapshot;
        self.index = index;
        self.errors.drain(errors_idx..);
        self.nodes.drain(nodes_idx..);
        self.extra_data.drain(extra_data_idx..);
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
            is_warn: true,
            token: self.index,
            data,
        });
    }

    fn warn_expected(&mut self, tag: lexer::Tag) {
        self.warn_msg(Error {
            is_warn: true,
            token: self.index,
            data: ErrorData::ExpectedToken(tag),
        })
    }

    fn warn_msg(&mut self, msg: Error) {
        self.errors.push(msg);
    }

    fn fail(&mut self, data: ErrorData) -> Result<std::convert::Infallible, ()> {
        self.fail_msg(Error {
            is_warn: false,
            token: self.index,
            data,
        })
    }

    fn fail_expected(&mut self, tag: lexer::Tag) -> Result<std::convert::Infallible, ()> {
        self.fail_msg(Error {
            is_warn: false,
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
            }
            None => break,
        };
    }

    let members = parse_block_members(p, true)?;
    root_nodes.extend(members);

    p.nodes[0].data = NodeData::Root(p.push_packed_list(&root_nodes));
    Ok(())
}

// Blocks

/// Block <- LBRACE BlockContents RBRACE
fn expect_block(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let start = match p.take_token(Token!('{')) {
        Some(token) => token,
        None => {
            p.fail(ErrorData::ExpectedBlock)?;
            unreachable!();
        }
    };

    let members = parse_block_members(p, false)?;
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

/// BlockContents <- InnerAttribute* Statement*
fn parse_block_members(p: &mut Parser<'_>, is_root: bool) -> Result<Vec<NodeIndex>, ()> {
    let mut nodes = vec![];
    while let Some(attr) = parse_inner_attribute(p)? {
        nodes.push(attr);
    }

    loop {
        let has_doc_comment = parse_outer_doc_commens(p);
        match p.tag() {
            Token!(#<eof>) => {
                if has_doc_comment {
                    p.warn(ErrorData::DocCommentAtContainerEnd);
                }

                assert!(is_root);
                break;
            }
            Token!('}') => {
                if has_doc_comment {
                    p.warn(ErrorData::DocCommentAtContainerEnd);
                }

                if is_root {
                    p.fail_expected(Token!(#<eof>))?;
                }
                break;
            }
            _ => {
                nodes.push(expect_statement(p)?);
            }
        }
    }

    Ok(nodes)
}

// Statement

/// Statement
///    <- KEYWORD_const BlockExpr
///     / KEYWORD_run BlockExpr
///     / KEYWORD_defer BlockExprStatement
///     / KEYWORD_cont_defer BlockExprStatement
///     / KEYWORD_err_defer BlockExprStatement
///     / IfStatement
///     / LabeledStatement
///     / DeclStatement
///     / ExprStatement
fn expect_statement(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let has_doc_comment = parse_outer_doc_commens(p);
    match p.tag() {
        Token!(const) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_const(p)
        }
        Token!(#run) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_run(p)
        }
        Token!(defer) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_defer(p)
        }
        Token!(cont_defer) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_cont_defer(p)
        }
        Token!(err_defer) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_err_defer(p)
        }
        Token!(if) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_if(p)
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!('{')) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            let label = p.next();
            p.next();
            let block = expect_block(p)?;
            Ok(p.push_node(label, NodeData::Label(block)))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(for)) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            let label = p.next();
            p.next();
            let stmt = expect_statement_for(p)?;
            Ok(p.push_node(label, NodeData::Label(stmt)))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(while)) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            let label = p.next();
            p.next();
            let stmt = expect_statement_while(p)?;
            Ok(p.push_node(label, NodeData::Label(stmt)))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(switch)) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            todo!()
        }
        Token!('{') => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_block(p)
        }
        Token!(for) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_for(p)
        }
        Token!(while) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            expect_statement_while(p)
        }
        Token!(switch) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnStatement);
            }
            todo!()
        }
        Token!(thread_local) => expect_statement_global_decl_thread_local(p),
        Token!(static) => expect_statement_global_decl_static(p),
        Token!(pub) | Token!(var) | Token!(inline) => expect_statement_const_or_global_decl(p),
        Token!(#<identifier>) => expect_statement_decl_or_expr(p),
        Token!(!)
        | Token!(-)
        | Token!(~)
        | Token!(-%)
        | Token!(...)
        | Token!(asm)
        | Token!(break)
        | Token!(continue)
        | Token!(return)
        | Token!(throw)
        | Token!(fn)
        | Token!(?)
        | Token!('[')
        | Token!(*)
        | Token!(.)
        | Token!(#)
        | Token!('(')
        | Token!(#<core_identifier>)
        | Token!(#<builtin_identifier>)
        | Token!(where)
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
        | Token!(#primitive)
        | Token!(struct)
        | Token!(union) => {
            if has_doc_comment {
                p.warn(ErrorData::DocCommentOnExpression);
            }
            expect_statement_expr(p)
        }
        _ => {
            p.fail(ErrorData::ExpectedBlockStatement)?;
            unreachable!()
        }
    }
}

/// KEYWORD_const BlockExpr
fn expect_statement_const(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let const_token = p.expect_token(Token!(const))?;
    let block_expr = expect_block_expr(p)?;
    Ok(p.push_node(const_token, NodeData::Const(block_expr)))
}

/// KEYWORD_run BlockExpr
fn expect_statement_run(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let run_token = p.expect_token(Token!(#run))?;
    let block_expr = expect_block_expr(p)?;
    Ok(p.push_node(run_token, NodeData::Run(block_expr)))
}

/// BlockExpr <- BlockLabel? Block
fn expect_block_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    if let Some(label) = p.take_token(Token!(#<identifier>)) {
        p.expect_token(Token!(:))?;
        let block = expect_block(p)?;
        Ok(p.push_node(label, NodeData::Label(block)))
    } else {
        expect_block(p)
    }
}

/// KEYWORD_defer BlockExprStatement
fn expect_statement_defer(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let defer_token = p.expect_token(Token!(defer))?;
    let block_expr_statement = expect_block_expr_statement(p)?;
    Ok(p.push_node(defer_token, NodeData::Defer(block_expr_statement)))
}

/// KEYWORD_cont_defer BlockExprStatement
fn expect_statement_cont_defer(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let defer_token = p.expect_token(Token!(cont_defer))?;
    let block_expr_statement = expect_block_expr_statement(p)?;
    Ok(p.push_node(defer_token, NodeData::Defer(block_expr_statement)))
}

/// KEYWORD_err_defer BlockExprStatement
fn expect_statement_err_defer(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let defer_token = p.expect_token(Token!(err_defer))?;
    let block_expr_statement = expect_block_expr_statement(p)?;
    Ok(p.push_node(defer_token, NodeData::Defer(block_expr_statement)))
}

/// BlockExprStatement
///    <- BlockExpr
///     / AssignExpr SEMICOLON
fn expect_block_expr_statement(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    match p.tag() {
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!('{')) => {
            let label = p.next();
            p.next();
            let block = expect_block(p)?;
            Ok(p.push_node(label, NodeData::Label(block)))
        }
        Token!('{') => expect_block(p),
        _ => expect_statement_expr(p),
    }
}

fn expect_statement_if(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!()
}

fn expect_statement_for(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!()
}

fn expect_statement_while(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!()
}

/// GlobalDeclStatementThreadLocal <- KEYWORD_thread_local LocalDeclStatement
/// LocalDeclStatement <- DeclProto (COMMA DeclProto)* COLON TypeExpr? EQUAL Expr (COMMA Expr)*
/// DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAttribute*
fn expect_statement_global_decl_thread_local(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    _ = p.expect_token(Token!(thread_local))?;
    let mut prototypes = vec![expect_decl_proto(p)?];
    while p.eat_token(Token!(,)) {
        prototypes.push(expect_decl_proto(p)?);
    }
    _ = p.expect_token(Token!(:))?;
    let type_expr = if !p.peek(Token!(=)) {
        Some(expect_type_expr(p)?)
    } else {
        None
    };
    _ = p.expect_token(Token!(=))?;

    let mut init_exprs = vec![expect_expr(p)?];
    while p.eat_token(Token!(,)) {
        init_exprs.push(expect_expr(p)?);
    }

    let semicolon_token = p.expect_token(Token!(;))?;

    let prototypes = p.push_packed_list(&prototypes).unwrap();
    let init_exprs = p.push_packed_list(&init_exprs).unwrap();
    let init_exprs = p.push_packed(init_exprs);
    let decl_list = p.push_packed(DeclList {
        decl_type: DeclType::ThreadLocal,
        prototypes,
        type_expr,
    });
    Ok(p.push_node(semicolon_token, NodeData::Decl(decl_list, init_exprs)))
}

/// GlobalDeclStatementStatic <- KEYWORD_static LocalDeclStatement
/// LocalDeclStatement <- DeclProto (COMMA DeclProto)* COLON TypeExpr? EQUAL Expr (COMMA Expr)*
/// DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAttribute*
fn expect_statement_global_decl_static(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    _ = p.expect_token(Token!(static))?;
    let mut prototypes = vec![expect_decl_proto(p)?];
    while p.eat_token(Token!(,)) {
        prototypes.push(expect_decl_proto(p)?);
    }
    _ = p.expect_token(Token!(:))?;
    let type_expr = if !p.peek(Token!(=)) {
        Some(expect_type_expr(p)?)
    } else {
        None
    };
    _ = p.expect_token(Token!(=))?;

    let mut init_exprs = vec![expect_expr(p)?];
    while p.eat_token(Token!(,)) {
        init_exprs.push(expect_expr(p)?);
    }

    let semicolon_token = p.expect_token(Token!(;))?;

    let prototypes = p.push_packed_list(&prototypes).unwrap();
    let init_exprs = p.push_packed_list(&init_exprs).unwrap();
    let init_exprs = p.push_packed(init_exprs);
    let decl_list = p.push_packed(DeclList {
        decl_type: DeclType::Static,
        prototypes,
        type_expr,
    });
    Ok(p.push_node(semicolon_token, NodeData::Decl(decl_list, init_exprs)))
}

/// ConstDeclStatement <- DeclProto (COMMA DeclProto)* (COLON TypeExpr? COLON / COLON2) Expr (COMMA Expr)*
/// LocalDeclStatement <- DeclProto (COMMA DeclProto)* COLON TypeExpr? EQUAL Expr (COMMA Expr)*
/// DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAttribute*
fn parse_ambiguous_statement_const_or_global_decl(
    p: &mut Parser<'_>,
) -> Result<Option<NodeIndex>, ()> {
    let mut prototypes = vec![];
    match expect_decl_proto(p) {
        Ok(proto) => prototypes.push(proto),
        Err(_) => return Ok(None),
    }
    while p.eat_token(Token!(,)) {
        match expect_decl_proto(p) {
            Ok(proto) => prototypes.push(proto),
            Err(_) => return Ok(None),
        }
    }

    // If we see `:{` we know that we must be parsing an expression and not a declaration.
    match p.tag() {
        Token!(:) if !p.peek2(Token!('{')) => {
            p.next();
            let type_expr = if !p.peek_any([Token!(:), Token!(=)]) {
                Some(expect_type_expr(p)?)
            } else {
                None
            };

            let decl_type = match p.tag() {
                Token!(:) => {
                    p.next();
                    DeclType::Const
                }
                Token!(=) => {
                    p.next();
                    DeclType::Normal
                }
                _ => {
                    p.fail(ErrorData::ExpectedInitExpression)?;
                    unreachable!()
                }
            };

            let mut init_exprs = vec![expect_expr(p)?];
            while p.eat_token(Token!(,)) {
                init_exprs.push(expect_expr(p)?);
            }
            let semicolon_token = p.expect_token(Token!(;))?;

            let prototypes = p.push_packed_list(&prototypes).unwrap();
            let init_exprs = p.push_packed_list(&init_exprs).unwrap();
            let init_exprs = p.push_packed(init_exprs);
            let decl_list = p.push_packed(DeclList {
                decl_type,
                prototypes,
                type_expr,
            });
            Ok(Some(p.push_node(
                semicolon_token,
                NodeData::Decl(decl_list, init_exprs),
            )))
        }
        Token!(::) => {
            p.next();
            let mut init_exprs = vec![expect_expr(p)?];
            while p.eat_token(Token!(,)) {
                init_exprs.push(expect_expr(p)?);
            }
            let semicolon_token = p.expect_token(Token!(;))?;

            let prototypes = p.push_packed_list(&prototypes).unwrap();
            let init_exprs = p.push_packed_list(&init_exprs).unwrap();
            let init_exprs = p.push_packed(init_exprs);
            let decl_list = p.push_packed(DeclList {
                decl_type: DeclType::Const,
                prototypes,
                type_expr: None,
            });
            Ok(Some(p.push_node(
                semicolon_token,
                NodeData::Decl(decl_list, init_exprs),
            )))
        }
        _ => Ok(None),
    }
}

/// ConstDeclStatement <- DeclProto (COMMA DeclProto)* (COLON TypeExpr? COLON / COLON2) Expr (COMMA Expr)*
/// LocalDeclStatement <- DeclProto (COMMA DeclProto)* COLON TypeExpr? EQUAL Expr (COMMA Expr)*
/// DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAttribute*
fn expect_statement_const_or_global_decl(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let mut prototypes = vec![expect_decl_proto(p)?];
    while p.eat_token(Token!(,)) {
        prototypes.push(expect_decl_proto(p)?);
    }

    match p.tag() {
        Token!(:) => {
            p.next();
            let type_expr = if !p.peek_any([Token!(:), Token!(=)]) {
                Some(expect_type_expr(p)?)
            } else {
                None
            };

            let decl_type = match p.tag() {
                Token!(:) => {
                    p.next();
                    DeclType::Const
                }
                Token!(=) => {
                    p.next();
                    DeclType::Normal
                }
                _ => {
                    p.fail(ErrorData::ExpectedInitExpression)?;
                    unreachable!()
                }
            };

            let mut init_exprs = vec![expect_expr(p)?];
            while p.eat_token(Token!(,)) {
                init_exprs.push(expect_expr(p)?);
            }
            let semicolon_token = p.expect_token(Token!(;))?;

            let prototypes = p.push_packed_list(&prototypes).unwrap();
            let init_exprs = p.push_packed_list(&init_exprs).unwrap();
            let init_exprs = p.push_packed(init_exprs);
            let decl_list = p.push_packed(DeclList {
                decl_type,
                prototypes,
                type_expr,
            });
            Ok(p.push_node(semicolon_token, NodeData::Decl(decl_list, init_exprs)))
        }
        Token!(::) => {
            p.next();
            let mut init_exprs = vec![expect_expr(p)?];
            while p.eat_token(Token!(,)) {
                init_exprs.push(expect_expr(p)?);
            }
            let semicolon_token = p.expect_token(Token!(;))?;

            let prototypes = p.push_packed_list(&prototypes).unwrap();
            let init_exprs = p.push_packed_list(&init_exprs).unwrap();
            let init_exprs = p.push_packed(init_exprs);
            let decl_list = p.push_packed(DeclList {
                decl_type: DeclType::Const,
                prototypes,
                type_expr: None,
            });
            Ok(p.push_node(semicolon_token, NodeData::Decl(decl_list, init_exprs)))
        }
        _ => {
            p.fail(ErrorData::ExpectedInitExpression)?;
            unreachable!()
        }
    }
}

/// ConstDeclStatement <- DeclProto (COMMA DeclProto)* (COLON TypeExpr? COLON / COLON2) Expr (COMMA Expr)*
/// LocalDeclStatement <- DeclProto (COMMA DeclProto)* COLON TypeExpr? EQUAL Expr (COMMA Expr)*
/// ExprStatement <- AssignExpr SEMICOLON
/// DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAttribute*
/// AssignExpr <- Expr (AssignOp Expr / (COMMA Expr)+ EQUAL Expr (COMMA Expr)*)?
fn expect_statement_decl_or_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    debug_assert!(
        p.peek(Token!(#<identifier>)),
        "expected a declaration or expression statement"
    );

    let snapshot = p.snapshot();
    if let Some(idx) = parse_ambiguous_statement_const_or_global_decl(p)? {
        return Ok(idx);
    }

    p.rollback(snapshot);
    expect_statement_expr(p)
}

/// ExprStatement <- AssignExpr SEMICOLON
fn expect_statement_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = expect_assign_expr(p)?;
    let semicolon_token = p.expect_token(Token!(;))?;
    Ok(p.push_node(semicolon_token, NodeData::ExprSemicolon(expr)))
}

/// DeclProto <- KEYWORD_pub? KEYWORD_var? KEYWORD_inline? IDENTIFIER ByteAlign? OuterAttribute*
fn expect_decl_proto(p: &mut Parser<'_>) -> Result<DeclProto, ()> {
    let is_pub = p.eat_token(Token!(pub));
    let mut is_var = false;
    let mut is_inline = false;
    while !p.peek(Token!(#<identifier>)) {
        match p.tag() {
            Token!(pub) => {
                p.fail(ErrorData::PubTokenOutOfOrder)?;
            }
            Token!(var) => {
                if is_var {
                    p.fail(ErrorData::DoubleVarToken)?;
                }
                is_var = true;
                p.next();
            }
            Token!(inline) => {
                if is_inline {
                    p.fail(ErrorData::DoubleInlineToken)?;
                }
                is_inline = true;
                p.next();
            }
            _ => _ = p.fail(ErrorData::ExpectedVarInlineOrIdent)?,
        }
    }

    let ident = p.next();
    let align_expr = if p.eat_token(Token!(align)) {
        p.expect_token(Token!('('))?;
        let expr = expect_expr(p)?;
        p.expect_token(Token!(')'))?;
        Some(expr)
    } else {
        None
    };

    let mut annotations = vec![];
    while p.peek(Token!(#)) {
        annotations.push(expect_outer_annotation(p)?);
    }
    let annotations = p.push_packed_list(&annotations);

    Ok(DeclProto {
        is_pub,
        is_var,
        is_inline,
        ident,
        align_expr,
        annotations,
    })
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

/// OuterDocComment
fn parse_outer_doc_commens(p: &mut Parser<'_>) -> bool {
    let mut has_doc_comments = false;
    while p.eat_token(Token!(#<outer_doc_comment>)) {
        has_doc_comments = true;
    }
    has_doc_comments
}

/// OuterAnnotation <- #[=Expr]
fn expect_outer_annotation(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let pound_token = p.expect_token(Token!(#))?;
    _ = p.expect_token(Token!('['))?;
    _ = p.expect_token(Token!(=))?;
    let expr = expect_expr(p)?;
    let rbracket_token = p.expect_token(Token!(']'))?;

    Ok(p.push_node(pound_token, NodeData::OuterAnnotation(expr, rbracket_token)))
}

// Expressions

/// AssignExpr <- Expr (AssignOp Expr / (COMMA Expr)+ EQUAL Expr (COMMA Expr)*)?
fn expect_assign_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let expr = expect_expr(p)?;
    match p.tag() {
        Token!(*=)
        | Token!(*|=)
        | Token!(/=)
        | Token!(%=)
        | Token!(+=)
        | Token!(+|=)
        | Token!(-=)
        | Token!(-|=)
        | Token!(<<=)
        | Token!(<<|=)
        | Token!(>>=)
        | Token!(&=)
        | Token!(^=)
        | Token!(|=)
        | Token!(*%=)
        | Token!(+%=)
        | Token!(-%=)
        | Token!(=) => {
            let assign_token = p.next();
            let rhs = expect_expr(p)?;
            Ok(p.push_node(assign_token, NodeData::Binary(expr, rhs)))
        }
        Token!(,) => {
            let mut lhs = vec![expr];
            while p.eat_token(Token!(,)) {
                lhs.push(expect_expr(p)?);
            }

            let eq_token = p.expect_token(Token!(=))?;
            let mut rhs = vec![expect_expr(p)?];
            while p.eat_token(Token!(,)) {
                rhs.push(expect_expr(p)?);
            }
            todo!()
        }
        _ => Ok(expr),
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
                | Token!(where)
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
                | Token!(#primitive) => {
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
///     / Block
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
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(for)) => {
            let tok = p.next();
            p.next();
            let while_expr = expect_for_expr(p)?;
            Ok(p.push_node(tok, NodeData::Label(while_expr)))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(while)) => {
            let tok = p.next();
            p.next();
            let while_expr = expect_while_expr(p)?;
            Ok(p.push_node(tok, NodeData::Label(while_expr)))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(try)) => {
            let tok = p.next();
            p.next();
            let try_expr = expect_try_expr(p)?;
            Ok(p.push_node(tok, NodeData::Label(try_expr)))
        }
        Token!(for) => expect_for_expr(p),
        Token!(while) => expect_while_expr(p),
        Token!(try) => expect_try_expr(p),
        Token!(fn) if p.peek2(Token!('{')) => {
            let tok = p.next();
            let init_list = expect_init_list(p)?;
            Ok(p.push_node(tok, NodeData::TokenInit(init_list)))
        }
        Token!(fn) => expect_fn_expr(p),
        Token!('{') => expect_block(p),
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
        | Token!(where)
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
        | Token!(#primitive) => expect_curly_suffix_expr(p),
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

/// KEYWORD_break BreakLabel? Expr?
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
        | Token!(where)
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
        | Token!(#primitive) => Some(expect_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(break_token, NodeData::Jump(label_token, value_expr)))
}

/// KEYWORD_const Expr
fn expect_const_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let const_token = p.expect_token(Token!(const))?;
    let expr = expect_expr(p)?;
    Ok(p.push_node(const_token, NodeData::Const(expr)))
}

/// KEYWORD_continue BreakLabel? Expr?
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
        | Token!(where)
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
        | Token!(#primitive) => Some(expect_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(continue_token, NodeData::Jump(label_token, value_expr)))
}

/// KEYWORD_return Expr?
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
        | Token!(where)
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
        | Token!(#primitive) => Some(expect_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(return_token, NodeData::Jump(None, value_expr)))
}

/// KEYWORD_throw Expr?
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
        | Token!(where)
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
        | Token!(#primitive) => Some(expect_expr(p)?),
        _ => None,
    };

    Ok(p.push_node(throw_token, NodeData::Jump(None, value_expr)))
}

/// ForExpr <- ForPrefix Expr (KEYWORD_else Expr)?
/// ForPrefix <- KEYWORD_for LPAREN ExprList RPAREN PtrListPayload KEYWORD_inline?
fn expect_for_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let for_token = p.expect_token(Token!(for))?;
    p.expect_token(Token!('('))?;

    let mut iter_exprs = vec![];
    while !p.eat_token(Token!(')')) {
        iter_exprs.push(expect_expr(p)?);
        if !p.eat_token(Token!(,)) {
            p.expect_token(Token!(')'))?;
            break;
        }
    }

    let mut payloads = vec![];
    p.expect_token(Token!(|))?;
    while !p.eat_token(Token!(|)) {
        let is_ptr = p.eat_token(Token!(*));
        let ident = p.expect_token(Token!(#<identifier>))?;
        let has_comma = p.eat_token(Token!(,));
        payloads.push(p.push_node(ident, NodeData::Payload(is_ptr, has_comma)));
        if !has_comma {
            p.expect_token(Token!(|))?;
            break;
        }
    }

    let is_inline = p.eat_token(Token!(inline));
    let expr = expect_expr(p)?;
    let else_expr = if p.eat_token(Token!(else)) {
        Some(expect_expr(p)?)
    } else {
        None
    };

    if iter_exprs.len() <= 2 && payloads.len() <= 2 {
        let iter_expr1 = iter_exprs.first().copied();
        let iter_expr2 = iter_exprs.get(1).copied();
        let payload1 = payloads.first().copied();
        let payload2 = payloads.get(1).copied();
        let main_expr = p.push_packed(ForSimple {
            iter_expr1,
            iter_expr2,
            payload1,
            payload2,
            expr,
        });
        if is_inline {
            Ok(p.push_node(for_token, NodeData::ForSimpleInline(main_expr, else_expr)))
        } else {
            Ok(p.push_node(for_token, NodeData::ForSimple(main_expr, else_expr)))
        }
    } else {
        let iter_exprs = p.push_packed_list(&iter_exprs);
        let payloads = p.push_packed_list(&payloads);
        let main_expr = p.push_packed(For {
            iter_exprs,
            payloads,
            expr,
        });
        if is_inline {
            Ok(p.push_node(for_token, NodeData::ForInline(main_expr, else_expr)))
        } else {
            Ok(p.push_node(for_token, NodeData::For(main_expr, else_expr)))
        }
    }
}

/// WhileExpr <- WhilePrefix Expr (KEYWORD_else Payload? Expr)?
/// WhilePrefix <- KEYWORD_while LPAREN Expr RPAREN PtrPayload? KEYWORD_inline?
fn expect_while_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    todo!("while")
}

/// TryExpr <- KEYWORD_try Block
fn expect_try_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let try_token = p.expect_token(Token!(try))?;
    let block = expect_block(p)?;
    Ok(p.push_node(try_token, NodeData::TryBlock(block)))
}

/// FnExpr <- KEYWORD_fn FnCaptures? FnArgs FnModifier? FnCallConv? FnReturn? FnWhere? Block
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
                Some(expect_expr(p)?)
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

    let block = expect_block(p)?;

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
    Ok(p.push_node(fn_token, NodeData::Fn(proto, block)))
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
///     / SliceTypeStart
///     / PtrTypeStart (ByteAlign / QUESTIONMARK? KEYWORD_var / QUESTIONMARK? KEYWORD_volatile)*
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
///     / KEYWORD_where TypeExpr
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
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!('{')) => {
            let tok = p.next();
            p.next();
            let block = expect_block(p)?;
            p.push_node(tok, NodeData::Label(block))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(for)) => {
            let tok = p.next();
            p.next();
            let for_expr = expect_for_type_expr(p)?;
            p.push_node(tok, NodeData::Label(for_expr))
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(while)) => {
            todo!("LabeledTypeExpr")
        }
        Token!(#<identifier>) if p.peek2(Token!(:)) && p.peek3(Token!(switch)) => {
            todo!("LabeledTypeExpr")
        }
        Token!(for) => expect_for_type_expr(p)?,
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
        Token!(where) => {
            let tok = p.next();
            let expr = expect_type_expr(p)?;
            p.push_node(tok, NodeData::WhereExpr(expr))
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
/// FnCallArguments <- LPAREN ExprList RPAREN
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
                let mut arg_exprs = vec![];
                let mut has_trailing_comma = false;
                while !p.peek(Token!(')')) {
                    arg_exprs.push(expect_expr(p)?);
                    has_trailing_comma = p.eat_token(Token!(,));
                    if !has_trailing_comma {
                        break;
                    }
                }
                let end = p.expect_token(Token!(')'))?;

                if arg_exprs.len() <= 1 {
                    assert!(arg_exprs.len() == 1 || !has_trailing_comma);
                    let arg_expr = arg_exprs.first().copied();
                    if has_trailing_comma {
                        p.push_node(end, NodeData::Call1Comma(expr, arg_expr))
                    } else {
                        p.push_node(end, NodeData::Call1(expr, arg_expr))
                    }
                } else {
                    let arg_exprs = p.push_packed_list(&arg_exprs).unwrap();
                    let arg_exprs = p.push_packed(arg_exprs);
                    if has_trailing_comma {
                        p.push_node(end, NodeData::CallComma(expr, arg_exprs))
                    } else {
                        p.push_node(end, NodeData::Call(expr, arg_exprs))
                    }
                }
            }
            _ => break,
        };
    }

    Ok(expr)
}

/// ASTERISK (ByteAlign / QUESTIONMARK? KEYWORD_var / QUESTIONMARK? KEYWORD_volatile)*
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

/// LBRACKET ASTERISK (COLON Expr)? RBRACKET (ByteAlign / QUESTIONMARK? KEYWORD_var / QUESTIONMARK? KEYWORD_volatile)*
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

/// (ByteAlign / QUESTIONMARK? KEYWORD_var / QUESTIONMARK? KEYWORD_volatile)*
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

/// ForTypeExpr <- ForPrefix TypeExpr (KEYWORD_else TypeExpr)?
/// ForPrefix <- KEYWORD_for LPAREN ExprList RPAREN PtrListPayload KEYWORD_inline?
fn expect_for_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let for_token = p.expect_token(Token!(for))?;
    p.expect_token(Token!('('))?;

    let mut iter_exprs = vec![];
    while !p.eat_token(Token!(')')) {
        iter_exprs.push(expect_expr(p)?);
        if !p.eat_token(Token!(,)) {
            p.expect_token(Token!(')'))?;
            break;
        }
    }

    let mut payloads = vec![];
    p.expect_token(Token!(|))?;
    while !p.eat_token(Token!(|)) {
        let is_ptr = p.eat_token(Token!(*));
        let ident = p.expect_token(Token!(#<identifier>))?;
        let has_comma = p.eat_token(Token!(,));
        payloads.push(p.push_node(ident, NodeData::Payload(is_ptr, has_comma)));
        if !has_comma {
            p.expect_token(Token!(|))?;
            break;
        }
    }

    let is_inline = p.eat_token(Token!(inline));
    let expr = expect_type_expr(p)?;
    let else_expr = if p.eat_token(Token!(else)) {
        Some(expect_type_expr(p)?)
    } else {
        None
    };

    if iter_exprs.len() <= 2 && payloads.len() <= 2 {
        let iter_expr1 = iter_exprs.first().copied();
        let iter_expr2 = iter_exprs.get(1).copied();
        let payload1 = payloads.first().copied();
        let payload2 = payloads.get(1).copied();
        let main_expr = p.push_packed(ForSimple {
            iter_expr1,
            iter_expr2,
            payload1,
            payload2,
            expr,
        });
        if is_inline {
            Ok(p.push_node(for_token, NodeData::ForSimpleInline(main_expr, else_expr)))
        } else {
            Ok(p.push_node(for_token, NodeData::ForSimple(main_expr, else_expr)))
        }
    } else {
        let iter_exprs = p.push_packed_list(&iter_exprs);
        let payloads = p.push_packed_list(&payloads);
        let main_expr = p.push_packed(For {
            iter_exprs,
            payloads,
            expr,
        });
        if is_inline {
            Ok(p.push_node(for_token, NodeData::ForInline(main_expr, else_expr)))
        } else {
            Ok(p.push_node(for_token, NodeData::For(main_expr, else_expr)))
        }
    }
}

/// NamespaceTypeExpr <- KEYWORD_namespace Block
fn expect_namespace_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let ns_token = p.expect_token(Token!(namespace))?;
    let block = expect_block(p)?;

    Ok(p.push_node(ns_token, NodeData::Container(None, block)))
}

/// OpaqueTypeExpr <- KEYWORD_opaque (LPAREN Expr RPAREN)? KEYWORD_const? Block
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
    let block = expect_block(p)?;

    if is_const {
        Ok(p.push_node(
            container_token,
            NodeData::ContainerConst(layout_expr, block),
        ))
    } else {
        Ok(p.push_node(container_token, NodeData::Container(layout_expr, block)))
    }
}

/// StructTypeExpr <- KEYWORD_struct (LPAREN Expr RPAREN)? KEYWORD_const? Block
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
    let block = expect_block(p)?;

    if is_const {
        Ok(p.push_node(
            container_token,
            NodeData::ContainerConst(layout_expr, block),
        ))
    } else {
        Ok(p.push_node(container_token, NodeData::Container(layout_expr, block)))
    }
}

/// KEYWORD_primitive LPAREN STRING_LITERAL COMMA Block RPAREN
fn expect_primitive_type_expr(p: &mut Parser<'_>) -> Result<NodeIndex, ()> {
    let primitive_token = p.expect_token(Token!(#primitive))?;
    p.expect_token(Token!('('))?;
    let id_token = p.expect_token(Token!(#<string_literal>))?;
    p.expect_token(Token!(,))?;
    let block = expect_block(p)?;
    p.expect_token(Token!(')'))?;
    Ok(p.push_node(primitive_token, NodeData::Primitive(id_token, block)))
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
                let init_expr = expect_expr(p)?;
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
                    let init_expr = expect_expr(p)?;
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
                exprs.push(expect_expr(p)?);
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
