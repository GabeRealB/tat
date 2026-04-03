use std::{
    fmt::Display,
    ops::{Add, Range, Sub},
    sync::Arc,
};

use annotate_snippets::{AnnotationKind, Level, Snippet};

use crate::{
    Token,
    ast::{Ast, DeclType, ExtraIndex, ExtraIndexRange, NodeData, NodeIndex, TokenIndex},
    compilation_unit::{Cu, ReportKind},
    intern_pool::{AstInfo, Index, InternPool, TypedIndex},
    packed_stream::{BitPacked, DefaultPackable, Packable, PackedStreamReader, PackedStreamWriter},
    util::{NonMaxU32, indent_fmt},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirIdx(NonMaxU32);

impl HirIdx {
    /// Index of a [`HirChunk`] root instruction.
    ///
    /// Is guaranteed to be a [`Hir::InlineBlock`].
    pub const ROOT: Self = Self::new(0);

    pub const fn new(index: u32) -> Self {
        let inner = NonMaxU32::new(index).expect("index must be less than u32::MAX");
        Self(inner)
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl Add<u32> for HirIdx {
    type Output = Self;

    fn add(self, rhs: u32) -> Self::Output {
        let inner = self.0.checked_add(rhs).expect("overflow");
        Self(inner)
    }
}

impl Sub<u32> for HirIdx {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        let inner = self.0.checked_sub(rhs).expect("overflow");
        Self(inner)
    }
}

impl Display for HirIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl DefaultPackable for HirIdx {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SrcLocation {
    pub byte_start: u32,
    pub byte_len: u32,
}

impl SrcLocation {
    pub const fn into_range(self) -> Range<usize> {
        let start = self.byte_start as usize;
        let len = self.byte_len as usize;
        let end = start + len;
        start..end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclProto {
    pub is_pub: bool,
    pub is_var: bool,
    pub is_inline: bool,
    pub ident: TokenIndex,
}

impl Packable for DeclProto {
    const LEN: usize = <(BitPacked<(bool, bool, bool)>, TokenIndex)>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits((
            self.is_pub,
            self.is_var,
            self.is_inline,
        )));
        buffer.write(self.ident);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let (is_pub, is_var, is_inline) = buffer.read::<BitPacked<(bool, bool, bool)>>().unpack();
        let ident = buffer.read();
        Self {
            is_pub,
            is_var,
            is_inline,
            ident,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclInfo {
    pub decl_type: DeclType,
    pub prototype: DeclProto,
}

impl Packable for DeclInfo {
    const LEN: usize = <(BitPacked<DeclType>, DeclProto)>::LEN;

    fn write_packed(self, buffer: &mut PackedStreamWriter<'_>) {
        buffer.write(BitPacked::pack_bits(self.decl_type));
        buffer.write(self.prototype);
    }

    fn read_packed(buffer: &mut PackedStreamReader) -> Self {
        let decl_type = buffer.read::<BitPacked<DeclType>>().unpack();
        let prototype = buffer.read();
        Self {
            decl_type,
            prototype,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameStrategy {
    Parent,
    Decl,
    Anon,
}

impl Display for NameStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameStrategy::Parent => write!(f, ".parent"),
            NameStrategy::Decl => write!(f, ".decl"),
            NameStrategy::Anon => write!(f, ".anon"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltInFunction {
    ConstPrint,
    ExportSymbol,
    Import,
    ImportSymbol,
    InConst,
}

impl BuiltInFunction {
    pub fn from_string(value: &str) -> Option<Self> {
        match value {
            "#constPrint" => Some(Self::ConstPrint),
            "#exportSymbol" => Some(Self::ExportSymbol),
            "#import" => Some(Self::Import),
            "#importSymbol" => Some(Self::ImportSymbol),
            _ => None,
        }
    }

    pub fn as_instr_string(self) -> &'static str {
        match self {
            BuiltInFunction::ConstPrint => "builtin.const_print",
            BuiltInFunction::ExportSymbol => "builtin.export_symbol",
            BuiltInFunction::Import => "builtin.import",
            BuiltInFunction::ImportSymbol => "builtin.import_symbol",
            BuiltInFunction::InConst => "builtin.in_const",
        }
    }
}

fn is_primitive(ident: &str) -> bool {
    match ident {
        "any_int" | "any_float" | "any_opaque" | "any_type" | "bool" | "char" | "int" | "uint"
        | "intptr" | "uintptr" | "no_return" | "rawptr" | "string" | "cstring" | "void"
        | "type" | "type_id" | "null" | "undefined" | "true" | "false" => true,
        _ if ident.starts_with("b")
            && let Ok(_) = ident[1..].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("i")
            && ident.ends_with("le")
            && let Ok(_) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("i")
            && ident.ends_with("be")
            && let Ok(_) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("i")
            && let Ok(_) = ident[1..].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("u")
            && ident.ends_with("le")
            && let Ok(_) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("u")
            && ident.ends_with("be")
            && let Ok(_) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("u")
            && let Ok(_) = ident[1..].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("f")
            && ident.ends_with("le")
            && let Ok(_) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("f")
            && ident.ends_with("be")
            && let Ok(_) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("f")
            && let Ok(_) = ident[1..].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("complex")
            && let Ok(_) = ident[7..].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("quat")
            && let Ok(_) = ident[4..].parse::<u16>() =>
        {
            true
        }
        _ if ident.starts_with("dquat")
            && let Ok(_) = ident[5..].parse::<u16>() =>
        {
            true
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hir {
    Noop,
    AstInfo(TypedIndex<AstInfo>),
    DbgLocation(SrcLocation),

    Namespace(NameStrategy, ExtraIndex<Option<ExtraIndexRange<HirIdx>>>),
    Declarations(Option<ExtraIndexRange<HirIdx>>),

    PushDeclType(HirIdx),
    PushDeclAlign(u32, HirIdx),
    PushDeclAnnotation(u32, HirIdx),
    InitDecl(HirIdx),
    EnsureDeclsInit,
    Declaration(ExtraIndex<DeclInfo>),

    InlineBlock(Option<ExtraIndexRange<HirIdx>>),
    LazyBlock(Option<ExtraIndexRange<HirIdx>>),
    ScopingBlock(Option<ExtraIndexRange<HirIdx>>),
    ConstBlock(Option<ExtraIndexRange<HirIdx>>),
    RunBlock(Option<ExtraIndexRange<HirIdx>>),
    DeferBlock(Option<ExtraIndexRange<HirIdx>>),
    ErrDeferBlock(Option<ExtraIndexRange<HirIdx>>),
    ContDeferBlock(Option<ExtraIndexRange<HirIdx>>),

    BreakInline(HirIdx),
    BreakLazy(HirIdx),

    MulAssign(HirIdx, HirIdx),
    MulSatAssign(HirIdx, HirIdx),
    MulWrapAssign(HirIdx, HirIdx),
    DivAssign(HirIdx, HirIdx),
    RemAssign(HirIdx, HirIdx),
    AddAssign(HirIdx, HirIdx),
    AddSatAssign(HirIdx, HirIdx),
    AddWrapAssign(HirIdx, HirIdx),
    SubAssign(HirIdx, HirIdx),
    SubSatAssign(HirIdx, HirIdx),
    SubWrapAssign(HirIdx, HirIdx),
    ShlAssign(HirIdx, HirIdx),
    ShlSatAssign(HirIdx, HirIdx),
    ShrAssign(HirIdx, HirIdx),
    BitAndAssign(HirIdx, HirIdx),
    BitXorAssign(HirIdx, HirIdx),
    BitOrAssign(HirIdx, HirIdx),
    Assign(HirIdx, HirIdx),

    LogicalOr(HirIdx, HirIdx),
    LogicalAnd(HirIdx, HirIdx),

    Eq(HirIdx, HirIdx),
    NotEq(HirIdx, HirIdx),
    Lt(HirIdx, HirIdx),
    LtEq(HirIdx, HirIdx),
    Gt(HirIdx, HirIdx),
    GtEq(HirIdx, HirIdx),
    Cmp(HirIdx, HirIdx),

    BitAnd(HirIdx, HirIdx),
    BitXor(HirIdx, HirIdx),
    BitOr(HirIdx, HirIdx),
    OrElse(HirIdx, HirIdx),
    Catch(HirIdx, HirIdx),

    Shl(HirIdx, HirIdx),
    ShlSat(HirIdx, HirIdx),
    Shr(HirIdx, HirIdx),

    Add(HirIdx, HirIdx),
    AddSat(HirIdx, HirIdx),
    AddWrap(HirIdx, HirIdx),
    Sub(HirIdx, HirIdx),
    SubSat(HirIdx, HirIdx),
    SubWrap(HirIdx, HirIdx),
    Concat(HirIdx, HirIdx),

    Mul(HirIdx, HirIdx),
    MulSat(HirIdx, HirIdx),
    MulWrap(HirIdx, HirIdx),
    Div(HirIdx, HirIdx),
    Rem(HirIdx, HirIdx),

    Not(HirIdx),
    Neg(HirIdx),
    BitNot(HirIdx),
    NegWrap(HirIdx),
    Destructure(HirIdx),

    AnyOpaque,
    Bool {
        width: u16,
    },
    Int {
        width: u16,
        is_signed: bool,
        is_little_endian: Option<bool>,
    },
    Float {
        width: u16,
        is_little_endian: Option<bool>,
    },
    Complex {
        width: u16,
    },
    Quat {
        width: u16,
    },
    DQuat {
        width: u16,
    },

    Immediate(Index),
    IntLiteral(TokenIndex),
    StringLiteral(TokenIndex),

    LoadIdent(TokenIndex),
    LoadIdentPtr(TokenIndex),
    LoadCoreIdent(TokenIndex),
    LoadCoreIdentPtr(TokenIndex),

    InjectStmt(TokenIndex),
    InjectExpr(TokenIndex),

    As(HirIdx, HirIdx),
    AsImmLhs(Index, HirIdx),
    AsPtrChild(HirIdx, HirIdx),

    BuiltInCall(BuiltInFunction, Option<ExtraIndex<ExtraIndexRange<HirIdx>>>),
    BuiltInArgTypeOf(BuiltInFunction, Option<ExtraIndex<ExtraIndexRange<HirIdx>>>),
    BuiltInArgDefault(BuiltInFunction, Option<ExtraIndex<ExtraIndexRange<HirIdx>>>),

    /// `#constPrint(...)`
    BuiltInConstPrint(Option<ExtraIndexRange<HirIdx>>),
    /// `#exportSymbol(...)`
    BuiltInExportSymbol(Option<ExtraIndexRange<HirIdx>>),
    /// `#import(...)`
    BuiltInImport(Option<ExtraIndexRange<HirIdx>>),
}

const _: () = const { assert!(size_of::<Hir>() == size_of::<u32>() * 3) };

#[derive(Debug, Clone, Copy)]
pub struct HirChunkView<'a> {
    pub nodes: &'a [Hir],
    pub extra: &'a [u32],
}

impl HirChunkView<'_> {
    pub fn root_range_iter(&self) -> HirRangeIterator {
        let idx = HirIdx::ROOT;
        let node = self.get_node(idx);
        match node {
            Hir::InlineBlock(range) => range.into(),
            _ => unreachable!(),
        }
    }

    pub fn block_range_iter(&self, idx: HirIdx) -> HirRangeIterator {
        let node = self.get_node(idx);
        match node {
            Hir::Namespace(_, range) => self.get_packed(range).into(),
            Hir::Declarations(range) => range.into(),
            Hir::InlineBlock(range) => range.into(),
            Hir::LazyBlock(range) => range.into(),
            Hir::ScopingBlock(range) => range.into(),
            Hir::ConstBlock(range) => range.into(),
            Hir::RunBlock(range) => range.into(),
            Hir::DeferBlock(range) => range.into(),
            Hir::ErrDeferBlock(range) => range.into(),
            Hir::ContDeferBlock(range) => range.into(),
            _ => unreachable!(),
        }
    }

    pub fn get_node(&self, ir_idx: HirIdx) -> Hir {
        self.nodes[ir_idx.get() as usize]
    }

    pub fn get_packed<T: Packable>(&self, idx: ExtraIndex<T>) -> T {
        let start = idx.get();
        let mut stream = PackedStreamReader::new(&self.extra[start..]);
        stream.read()
    }

    fn fmt_node(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        pool: &InternPool,
        node_idx: HirIdx,
        helper: &mut HirFmtHelper,
        nesting: usize,
    ) -> std::fmt::Result {
        indent_fmt(f, nesting)?;
        write!(f, "{node_idx} := ")?;

        let node = self.get_node(node_idx);
        match node {
            Hir::Noop => writeln!(f, "noop")?,
            Hir::AstInfo(idx) => {
                helper.set_ast(idx);
                let ast_info = idx.get_from_pool(pool);
                let file = ast_info.file.get_from_pool(pool);
                let file_path = unsafe { file.file_path.as_str() };
                let module = file.module.get_from_pool(pool);
                let module_name = unsafe { module.name.as_str() };
                writeln!(f, "ast_info({module_name:?}, {file_path:?})")?;
            }
            Hir::DbgLocation(SrcLocation {
                byte_start,
                byte_len,
            }) => {
                let byte_end = byte_start + byte_len;
                writeln!(f, "dbg_location({byte_start}..{byte_end})")?;
            }

            Hir::Namespace(name_strategy, members) => {
                let members = self.get_packed(members);
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "namespace({name_strategy}) {{}}")?
                } else {
                    writeln!(f, "namespace({name_strategy}) {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }
            Hir::Declarations(members) => {
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "declarations {{}}")?
                } else {
                    writeln!(f, "declarations {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }

            Hir::PushDeclType(idx) => writeln!(f, "push_decl_ty({idx})")?,
            Hir::PushDeclAlign(decl, idx) => writeln!(f, "push_decl_align({decl}, {idx})")?,
            Hir::PushDeclAnnotation(decl, idx) => {
                writeln!(f, "push_decl_annotation({decl}, {idx})")?
            }
            Hir::InitDecl(idx) => writeln!(f, "init_decl({idx})")?,
            Hir::EnsureDeclsInit => writeln!(f, "ensure_decls_init()")?,
            Hir::Declaration(decl) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let DeclInfo {
                    decl_type,
                    prototype:
                        DeclProto {
                            is_pub,
                            is_var,
                            is_inline,
                            ident,
                        },
                } = self.get_packed(decl);
                match decl_type {
                    DeclType::Normal => write!(f, "decl(")?,
                    DeclType::Const => write!(f, "const_decl(")?,
                    DeclType::ThreadLocal => write!(f, "thread_local_decl(")?,
                    DeclType::Static => write!(f, "static_decl(")?,
                }
                let ident = ast.get_ident(ident);
                let mut is_first = true;
                let prefix_strings = ["pub", "var", "inline", ident];
                let prefixes = [is_pub, is_var, is_inline, true];
                for (&has_prefix, prefix) in prefixes.iter().zip(prefix_strings) {
                    if !has_prefix {
                        continue;
                    }
                    if !is_first {
                        write!(f, " ")?
                    }
                    write!(f, "{prefix}")?;
                    is_first = false;
                }
                writeln!(f, ")")?
            }

            Hir::InlineBlock(members) => {
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "inline_block {{}}")?
                } else {
                    writeln!(f, "inline_block {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }
            Hir::LazyBlock(members) => {
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "lazy_block {{}}")?
                } else {
                    writeln!(f, "lazy_block {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }
            Hir::ScopingBlock(members) => {
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "scoping_block {{}}")?
                } else {
                    writeln!(f, "scoping_block {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }
            Hir::ConstBlock(members) => {
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "const_block {{}}")?
                } else {
                    writeln!(f, "const_block {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }
            Hir::RunBlock(members) => {
                let mut iter = HirRangeIterator::Many(members);
                if iter.is_empty() {
                    writeln!(f, "run_block {{}}")?
                } else {
                    writeln!(f, "run_block {{")?;
                    while let Some(member) = iter.next(*self) {
                        self.fmt_node(f, pool, member, helper, nesting + 1)?;
                    }
                    indent_fmt(f, nesting)?;
                    writeln!(f, "}}")?;
                }
            }
            Hir::DeferBlock(_) => todo!(),
            Hir::ErrDeferBlock(_) => todo!(),
            Hir::ContDeferBlock(_) => todo!(),

            Hir::BreakInline(idx) => writeln!(f, "break_inline({idx})")?,
            Hir::BreakLazy(idx) => writeln!(f, "break_lazy({idx})")?,

            Hir::MulAssign(lhs, rhs) => writeln!(f, "mul_assign({lhs}, {rhs})")?,
            Hir::MulSatAssign(lhs, rhs) => writeln!(f, "mul_sat_assign({lhs}, {rhs})")?,
            Hir::MulWrapAssign(lhs, rhs) => writeln!(f, "mul_wrap_assign({lhs}, {rhs})")?,
            Hir::DivAssign(lhs, rhs) => writeln!(f, "div_assign({lhs}, {rhs})")?,
            Hir::RemAssign(lhs, rhs) => writeln!(f, "rem_assign({lhs}, {rhs})")?,
            Hir::AddAssign(lhs, rhs) => writeln!(f, "add_assign({lhs}, {rhs})")?,
            Hir::AddSatAssign(lhs, rhs) => writeln!(f, "add_sat_assign({lhs}, {rhs})")?,
            Hir::AddWrapAssign(lhs, rhs) => writeln!(f, "add_wrap_assign({lhs}, {rhs})")?,
            Hir::SubAssign(lhs, rhs) => writeln!(f, "sub_assign({lhs}, {rhs})")?,
            Hir::SubSatAssign(lhs, rhs) => writeln!(f, "sub_sat_assign({lhs}, {rhs})")?,
            Hir::SubWrapAssign(lhs, rhs) => writeln!(f, "sub_wrap_assign({lhs}, {rhs})")?,
            Hir::ShlAssign(lhs, rhs) => writeln!(f, "shift_left_assign({lhs}, {rhs})")?,
            Hir::ShlSatAssign(lhs, rhs) => writeln!(f, "shift_left_sat_assign({lhs}, {rhs})")?,
            Hir::ShrAssign(lhs, rhs) => writeln!(f, "shift_right_assign({lhs}, {rhs})")?,
            Hir::BitAndAssign(lhs, rhs) => writeln!(f, "bit_and_assign({lhs}, {rhs})")?,
            Hir::BitXorAssign(lhs, rhs) => writeln!(f, "bit_xor_assign({lhs}, {rhs})")?,
            Hir::BitOrAssign(lhs, rhs) => writeln!(f, "bit_or_assign({lhs}, {rhs})")?,
            Hir::Assign(lhs, rhs) => writeln!(f, "assign({lhs}, {rhs})")?,
            Hir::LogicalOr(lhs, rhs) => writeln!(f, "logic_or({lhs}, {rhs})")?,
            Hir::LogicalAnd(lhs, rhs) => writeln!(f, "logic_and({lhs}, {rhs})")?,
            Hir::Eq(lhs, rhs) => writeln!(f, "eq({lhs}, {rhs})")?,
            Hir::NotEq(lhs, rhs) => writeln!(f, "not_eq({lhs}, {rhs})")?,
            Hir::Lt(lhs, rhs) => writeln!(f, "lt({lhs}, {rhs})")?,
            Hir::LtEq(lhs, rhs) => writeln!(f, "lt_eq({lhs}, {rhs})")?,
            Hir::Gt(lhs, rhs) => writeln!(f, "gt({lhs}, {rhs})")?,
            Hir::GtEq(lhs, rhs) => writeln!(f, "gt_eq({lhs}, {rhs})")?,
            Hir::Cmp(lhs, rhs) => writeln!(f, "cmp({lhs}, {rhs})")?,
            Hir::BitAnd(lhs, rhs) => writeln!(f, "and({lhs}, {rhs})")?,
            Hir::BitXor(lhs, rhs) => writeln!(f, "xor({lhs}, {rhs})")?,
            Hir::BitOr(lhs, rhs) => writeln!(f, "or({lhs}, {rhs})")?,
            Hir::OrElse(lhs, rhs) => writeln!(f, "or_else({lhs}, {rhs})")?,
            Hir::Catch(lhs, rhs) => writeln!(f, "catch({lhs}, {rhs})")?,
            Hir::Shl(lhs, rhs) => writeln!(f, "shift_left({lhs}, {rhs})")?,
            Hir::ShlSat(lhs, rhs) => writeln!(f, "shift_left_sat({lhs}, {rhs})")?,
            Hir::Shr(lhs, rhs) => writeln!(f, "shift_right({lhs}, {rhs})")?,
            Hir::Add(lhs, rhs) => writeln!(f, "add({lhs}, {rhs})")?,
            Hir::AddSat(lhs, rhs) => writeln!(f, "add_sat({lhs}, {rhs})")?,
            Hir::AddWrap(lhs, rhs) => writeln!(f, "add_wrap({lhs}, {rhs})")?,
            Hir::Sub(lhs, rhs) => writeln!(f, "sub({lhs}, {rhs})")?,
            Hir::SubSat(lhs, rhs) => writeln!(f, "sub_sat({lhs}, {rhs})")?,
            Hir::SubWrap(lhs, rhs) => writeln!(f, "sub_wrap({lhs}, {rhs})")?,
            Hir::Concat(lhs, rhs) => writeln!(f, "concat({lhs}, {rhs})")?,
            Hir::Mul(lhs, rhs) => writeln!(f, "mul({lhs}, {rhs})")?,
            Hir::MulSat(lhs, rhs) => writeln!(f, "mul_sat({lhs}, {rhs})")?,
            Hir::MulWrap(lhs, rhs) => writeln!(f, "mul_wrap({lhs}, {rhs})")?,
            Hir::Div(lhs, rhs) => writeln!(f, "div({lhs}, {rhs})")?,
            Hir::Rem(lhs, rhs) => writeln!(f, "rem({lhs}, {rhs})")?,
            Hir::Not(ir_index) => writeln!(f, "not({ir_index})")?,
            Hir::Neg(ir_index) => writeln!(f, "neg({ir_index})")?,
            Hir::BitNot(ir_index) => writeln!(f, "bit_not({ir_index})")?,
            Hir::NegWrap(ir_index) => writeln!(f, "neg_wrap({ir_index})")?,
            Hir::Destructure(ir_index) => writeln!(f, "destructure({ir_index})")?,

            Hir::AnyOpaque => writeln!(f, "type(any_opaque)")?,
            Hir::Bool { width } => match width {
                1 => writeln!(f, "type(bool)")?,
                _ => writeln!(f, "type(b{width})")?,
            },
            Hir::Int {
                width,
                is_signed,
                is_little_endian,
            } => match is_signed {
                true => match is_little_endian {
                    Some(true) => writeln!(f, "type(i{width}le)")?,
                    Some(false) => writeln!(f, "type(i{width}be)")?,
                    None => writeln!(f, "type(i{width})")?,
                },
                false => match is_little_endian {
                    Some(true) => writeln!(f, "type(u{width}le)")?,
                    Some(false) => writeln!(f, "type(u{width}be)")?,
                    None => writeln!(f, "type(u{width})")?,
                },
            },
            Hir::Float {
                width,
                is_little_endian,
            } => match is_little_endian {
                Some(true) => writeln!(f, "type(f{width}le)")?,
                Some(false) => writeln!(f, "type(f{width}be)")?,
                None => writeln!(f, "type(f{width})")?,
            },
            Hir::Complex { width } => writeln!(f, "type(complex{width})")?,
            Hir::Quat { width } => writeln!(f, "type(quat{width})")?,
            Hir::DQuat { width } => writeln!(f, "type(dquat{width})")?,

            Hir::Immediate(idx) => writeln!(f, "{}", pool.display_index(idx))?,
            Hir::IntLiteral(token_idx) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let literal = ast.get_int_lit(token_idx);
                writeln!(f, "int_literal({literal})")?
            }
            Hir::StringLiteral(token_idx) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let literal = ast.get_string_lit(token_idx);
                writeln!(f, "string_literal({literal})")?
            }

            Hir::LoadIdent(token_idx) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let literal = ast.get_ident(token_idx);
                writeln!(f, "load_ident({literal})")?
            }
            Hir::LoadIdentPtr(token_idx) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let literal = ast.get_ident(token_idx);
                writeln!(f, "load_ident_ptr({literal})")?
            }
            Hir::LoadCoreIdent(token_idx) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let literal = ast.get_core_ident(token_idx);
                writeln!(f, "load_core_ident({literal})")?
            }
            Hir::LoadCoreIdentPtr(token_idx) => {
                let ast_idx = helper.get_ast();
                let ast_info = ast_idx.get_from_pool(pool);
                let ast = pool.get_ast(ast_info.id);
                let literal = ast.get_core_ident(token_idx);
                writeln!(f, "load_core_ident_ptr({literal})")?
            }

            Hir::InjectStmt(ir_idx) => writeln!(f, "inject_stmt({ir_idx})")?,
            Hir::InjectExpr(ir_idx) => writeln!(f, "inject_expr({ir_idx})")?,

            Hir::As(lhs, rhs) => writeln!(f, "as({lhs}, {rhs})")?,
            Hir::AsImmLhs(lhs, rhs) => writeln!(f, "as({}, {rhs})", pool.display_index(lhs))?,
            Hir::AsPtrChild(lhs, rhs) => writeln!(f, "as_ptr_child({lhs}, {rhs})")?,

            Hir::BuiltInCall(builtin, args) => match args {
                Some(args) => {
                    write!(f, "{}(", builtin.as_instr_string())?;
                    let mut is_first = true;
                    let args = self.get_packed(args);
                    for arg in args {
                        let arg = self.get_packed(arg);
                        if !is_first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                        is_first = false;
                    }
                    writeln!(f, ")")?
                }
                None => writeln!(f, "{}()", builtin.as_instr_string())?,
            },
            Hir::BuiltInArgDefault(builtin, args) => match args {
                Some(args) => {
                    write!(f, "builtin_arg_default({}", builtin.as_instr_string())?;
                    let args = self.get_packed(args);
                    for arg in args {
                        let arg = self.get_packed(arg);
                        write!(f, ", {arg}")?;
                    }
                    writeln!(f, ")")?
                }
                None => writeln!(f, "builtin_arg_default({})", builtin.as_instr_string())?,
            },
            Hir::BuiltInArgTypeOf(builtin, args) => match args {
                Some(args) => {
                    write!(f, "builtin_arg_type_of({}", builtin.as_instr_string())?;
                    let args = self.get_packed(args);
                    for arg in args {
                        let arg = self.get_packed(arg);
                        write!(f, ", {arg}")?;
                    }
                    writeln!(f, ")")?
                }
                None => writeln!(f, "builtin_arg_type_of({})", builtin.as_instr_string())?,
            },

            Hir::BuiltInConstPrint(args) => match args {
                Some(args) => {
                    write!(f, "builtin.const_print(")?;
                    let mut is_first = true;
                    for arg in args {
                        let arg = self.get_packed(arg);
                        if !is_first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                        is_first = false;
                    }
                    writeln!(f, ")")?
                }
                None => writeln!(f, "builtin.const_print()")?,
            },
            Hir::BuiltInExportSymbol(args) => match args {
                Some(args) => {
                    write!(f, "builtin.export_symbol(")?;
                    let mut is_first = true;
                    for arg in args {
                        let arg = self.get_packed(arg);
                        if !is_first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                        is_first = false;
                    }
                    writeln!(f, ")")?
                }
                None => writeln!(f, "builtin.export_symbol()")?,
            },
            Hir::BuiltInImport(args) => match args {
                Some(args) => {
                    write!(f, "builtin.import(")?;
                    let mut is_first = true;
                    for arg in args {
                        let arg = self.get_packed(arg);
                        if !is_first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                        is_first = false;
                    }
                    writeln!(f, ")")?
                }
                None => writeln!(f, "builtin.import()")?,
            },
        }

        Ok(())
    }

    pub fn display(self, pool: &InternPool) -> impl Display {
        struct Displayable<'a, 'b> {
            chunk: HirChunkView<'a>,
            pool: &'b InternPool,
        }

        impl Display for Displayable<'_, '_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.chunk
                    .fmt_node(f, self.pool, HirIdx::new(0), &mut Default::default(), 0)
            }
        }

        Displayable { chunk: self, pool }
    }
}

#[derive(Debug)]
pub struct HirChunk {
    pub nodes: Box<[Hir]>,
    pub extra: Box<[u32]>,
}

#[derive(Debug, Default)]
struct HirFmtHelper {
    ast: Option<TypedIndex<AstInfo>>,
}

impl HirFmtHelper {
    fn set_ast(&mut self, ast: TypedIndex<AstInfo>) {
        self.ast = Some(ast)
    }

    fn get_ast(&self) -> TypedIndex<AstInfo> {
        self.ast.unwrap()
    }
}

impl HirChunk {
    pub fn view(&self) -> HirChunkView<'_> {
        HirChunkView {
            nodes: &self.nodes,
            extra: &self.extra,
        }
    }

    pub fn display(&self, pool: &InternPool) -> impl Display {
        self.view().display(pool)
    }
}

struct Builder<'a> {
    nodes: Vec<Hir>,
    extra: Vec<u32>,
    cu: Arc<Cu>,
    ast: &'a Ast,
    loc: Option<SrcLocation>,
}

impl<'a> Builder<'a> {
    fn new(cu: Arc<Cu>, ast: &'a Ast) -> Self {
        Self {
            nodes: Default::default(),
            extra: Default::default(),
            cu,
            ast,
            loc: Default::default(),
        }
    }

    fn get_loc(&self, node_idx: NodeIndex) -> SrcLocation {
        let (start_token, end_token) = self.ast.get_node_token_span(node_idx);
        let start_token = self.ast.get_token(start_token);
        let end_token = self.ast.get_token(end_token);
        let span_start = start_token.start;
        let span_end = end_token.start + end_token.length;
        SrcLocation {
            byte_start: span_start,
            byte_len: span_end - span_start,
        }
    }

    fn set_dbg_loc(&mut self, node_idx: NodeIndex) -> Option<HirIdx> {
        let loc = self.get_loc(node_idx);
        if Some(loc) == self.loc {
            return None;
        }
        self.loc = Some(loc);
        Some(self.push_node(Hir::DbgLocation(loc)))
    }

    fn push_node(&mut self, node: Hir) -> HirIdx {
        let idx = HirIdx::new(self.nodes.len() as u32);
        self.nodes.push(node);
        idx
    }

    fn patch_node(&mut self, node_idx: HirIdx, node: Hir) {
        self.nodes[node_idx.get() as usize] = node;
    }

    fn push_packed<T: Packable>(&mut self, value: T) -> ExtraIndex<T> {
        let idx = ExtraIndex::new(self.extra.len());
        let mut stream = PackedStreamWriter::new(&mut self.extra);
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

        let start = ExtraIndex::new(self.extra.len());
        let mut stream = PackedStreamWriter::new(&mut self.extra);
        stream.write_slice(values);
        let end = ExtraIndex::new(self.extra.len() - 1);
        Some(ExtraIndexRange(start, end))
    }

    fn build(self) -> HirChunk {
        HirChunk {
            nodes: self.nodes.into(),
            extra: self.extra.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilationError;

impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("compilation failed")
    }
}

impl std::error::Error for CompilationError {}

#[derive(Debug, Clone, Copy)]
pub enum HirRangeIterator {
    Two([Option<HirIdx>; 2]),
    Many(Option<ExtraIndexRange<HirIdx>>),
}

impl HirRangeIterator {
    pub fn is_empty(&self) -> bool {
        match self {
            HirRangeIterator::Two([first, _]) => first.is_none(),
            HirRangeIterator::Many(range) => range.is_none(),
        }
    }

    pub fn iter<'a>(self, hir: HirChunkView<'a>) -> impl Iterator<Item = HirIdx> + 'a {
        struct Wrapper<'a> {
            iter: HirRangeIterator,
            hir: HirChunkView<'a>,
        }
        impl Iterator for Wrapper<'_> {
            type Item = HirIdx;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next(self.hir)
            }
        }
        Wrapper { iter: self, hir }
    }

    pub fn next(&mut self, hir: HirChunkView<'_>) -> Option<HirIdx> {
        match self {
            HirRangeIterator::Two([None, _]) => None,
            HirRangeIterator::Two([Some(idx), rest]) => {
                let idx = *idx;
                *self = Self::Two([*rest, None]);
                Some(idx)
            }
            HirRangeIterator::Many(None) => None,
            HirRangeIterator::Many(Some(range)) => {
                let idx = range.next();
                idx.map(|idx| hir.get_packed(idx))
            }
        }
    }
}

impl From<[Option<HirIdx>; 2]> for HirRangeIterator {
    fn from(value: [Option<HirIdx>; 2]) -> Self {
        Self::Two(value)
    }
}

impl From<Option<ExtraIndexRange<HirIdx>>> for HirRangeIterator {
    fn from(value: Option<ExtraIndexRange<HirIdx>>) -> Self {
        Self::Many(value)
    }
}

impl From<ExtraIndexRange<HirIdx>> for HirRangeIterator {
    fn from(value: ExtraIndexRange<HirIdx>) -> Self {
        Self::Many(Some(value))
    }
}

pub enum AstRangeIterator {
    Two([Option<NodeIndex>; 2]),
    Many(ExtraIndexRange<NodeIndex>),
}

impl AstRangeIterator {
    fn next(&mut self, ast: &Ast) -> Option<NodeIndex> {
        match self {
            AstRangeIterator::Two([None, _]) => None,
            AstRangeIterator::Two([Some(idx), rest]) => {
                let idx = *idx;
                *self = Self::Two([*rest, None]);
                Some(idx)
            }
            AstRangeIterator::Many(members) => {
                let idx = members.next();
                idx.map(|idx| ast.get_packed(idx))
            }
        }
    }
}

pub fn lower_ast(cu: Arc<Cu>, ast_idx: TypedIndex<AstInfo>) -> Result<HirChunk, CompilationError> {
    let pool = cu.pool();
    let ast_info = ast_idx.get_from_pool(pool);
    let ast_id = ast_info.id;

    let ast = pool.get_ast(ast_id);
    let mut builder = Builder::new(cu.clone(), ast);
    lower_ast_root(&mut builder, NodeIndex::ROOT, ast_idx)?;

    Ok(builder.build())
}

fn lower_ast_root(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    ast_idx: TypedIndex<AstInfo>,
) -> Result<HirIdx, CompilationError> {
    let node = builder.ast.get_node(node_idx);
    let members = match node.data {
        NodeData::Root(members) => members.unwrap(),
        _ => unreachable!(),
    };

    // The first members in the list are the attributes and annotations.
    // Iterate them until we find the #[`struct`] attribute or a non-attribute.
    let mut struct_attribute = None;
    for member in members {
        let member = builder.ast.get_packed(member);
        let member_node = builder.ast.get_node(member);
        match member_node.data {
            NodeData::Root(_) => unreachable!(),
            NodeData::OuterAnnotation(_, _) => unreachable!(),
            NodeData::InnerAnnotation(_, _) => {}
            NodeData::RootStructAttribute(_, _)
            | NodeData::RootStructConstAttribute(_, _)
            | NodeData::RootStructLayoutAttribute(_, _)
            | NodeData::RootStructLayoutConstAttribute(_, _) => {
                struct_attribute = Some(member);
                break;
            }
            _ => break,
        }
    }

    let block = builder.push_node(Hir::Noop);
    if let Some(_attr) = struct_attribute {
        todo!()
    } else {
        let mut nodes = vec![];
        nodes.push(builder.push_node(Hir::AstInfo(ast_idx)));
        nodes.extend(lower_ast_root_namespace(builder, node_idx)?);
        nodes.push(builder.push_node(Hir::BreakInline(*nodes.last().unwrap())));
        let nodes = builder.push_packed_list(&nodes);
        builder.patch_node(block, Hir::InlineBlock(nodes));
    };
    Ok(block)
}

fn lower_ast_root_namespace(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let mut ir_idxs = vec![];
    ir_idxs.extend(builder.set_dbg_loc(node_idx));

    let node = builder.ast.get_node(node_idx);
    let mut members = match node.data {
        NodeData::Root(members) => AstRangeIterator::Many(members.unwrap()),
        _ => unreachable!(),
    };

    let mut ir_members = vec![];
    let ir_idx = builder.push_node(Hir::Noop);
    ir_idxs.push(ir_idx);
    while let Some(member) = members.next(builder.ast) {
        ir_members.extend(lower_ast_stmt(builder, member)?);
    }
    let ir_members = builder.push_packed_list(&ir_members);
    let ir_members = builder.push_packed(ir_members);
    builder.patch_node(ir_idx, Hir::Namespace(NameStrategy::Parent, ir_members));

    Ok(ir_idxs)
}

fn lower_ast_stmt(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = builder.ast.get_node(node_idx);
    match node.data {
        NodeData::Const(_) => lower_ast_const_stmt(builder, node_idx),
        NodeData::Run(_) => todo!(),
        NodeData::Defer(_) => todo!(),
        NodeData::Decl(_, _) => lower_ast_decl_stmt(builder, node_idx),
        NodeData::ExprSemicolon(_) => lower_ast_expr_stmt(builder, node_idx),
        _ => unreachable!(),
    }
}

fn lower_ast_const_stmt(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let mut stmt_members = vec![];
    stmt_members.extend(builder.set_dbg_loc(node_idx));

    let node = builder.ast.get_node(node_idx);
    let node_idx = match node.data {
        NodeData::Const(node_idx) => node_idx,
        _ => unreachable!(),
    };

    let ir_idx = builder.push_node(Hir::Noop);
    stmt_members.push(ir_idx);

    let mut block_members = vec![];
    let node = builder.ast.get_node(node_idx);
    match node.data {
        NodeData::Label(_) => todo!(),
        NodeData::BlockTwo(stmt1, stmt2) | NodeData::BlockTwoSemicolon(stmt1, stmt2) => {
            let mut stmts = AstRangeIterator::Two([stmt1, stmt2]);
            while let Some(stmt) = stmts.next(builder.ast) {
                block_members.extend(lower_ast_stmt(builder, stmt)?);
            }
        }
        NodeData::Block(stmts) | NodeData::BlockSemicolon(stmts) => {
            let mut stmts = AstRangeIterator::Many(stmts);
            while let Some(stmt) = stmts.next(builder.ast) {
                block_members.extend(lower_ast_stmt(builder, stmt)?);
            }
        }
        _ => unreachable!(),
    }

    let block_members = builder.push_packed_list(&block_members);
    builder.patch_node(ir_idx, Hir::ConstBlock(block_members));

    Ok(stmt_members)
}

fn lower_ast_decl_stmt(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = builder.ast.get_node(node_idx);
    let (decl_list, init_list) = match node.data {
        NodeData::Decl(decl_list, init_expr) => (decl_list, init_expr),
        _ => unreachable!("{node:?}"),
    };
    let crate::ast::DeclList {
        decl_type,
        prototypes,
        type_expr,
    } = builder.ast.get_packed(decl_list);
    let init_list = builder.ast.get_packed(init_list);
    assert!(!prototypes.is_empty());
    assert!(!init_list.is_empty());

    if prototypes.len() < init_list.len() {
        let loc = builder.get_loc(node_idx);
        let span = loc.into_range();
        let report = [Level::ERROR.primary_title("too many initializers").element(
            Snippet::source(builder.ast.get_source())
                .annotation(AnnotationKind::Context.span(span)),
        )];
        builder.cu.emit_report(&report, ReportKind::Error);
        return Err(CompilationError);
    }

    let mut ir_idxs = vec![];
    ir_idxs.extend(builder.set_dbg_loc(node_idx));

    let mut ir_members = vec![];
    let ir_idx = builder.push_node(Hir::Noop);
    ir_idxs.push(ir_idx);

    let mut lazy_members = vec![];
    let lazy_idx = builder.push_node(Hir::Noop);
    ir_members.push(lazy_idx);

    if let Some(type_expr) = type_expr {
        lazy_members.extend(lower_ast_expr(builder, type_expr, ExprKind::RValue)?);
        let ty = *lazy_members.last().unwrap();
        let coerced = builder.push_node(Hir::AsImmLhs(Index::TY_TYPE, ty));
        lazy_members.push(coerced);
        lazy_members.push(builder.push_node(Hir::PushDeclType(coerced)));
    }

    for (i, prototype) in prototypes.enumerate() {
        let crate::ast::DeclProto {
            align_expr,
            annotations,
            ..
        } = builder.ast.get_packed(prototype);

        if let Some(align_expr) = align_expr {
            lazy_members.extend(lower_ast_expr(builder, align_expr, ExprKind::RValue)?);
            let align = *lazy_members.last().unwrap();
            let coerced = builder.push_node(Hir::AsImmLhs(Index::TY_ANY_INT, align));
            lazy_members.push(coerced);
            lazy_members.push(builder.push_node(Hir::PushDeclAlign(i as u32, coerced)));
        }

        if let Some(annotations) = annotations {
            for annotation in annotations {
                let annotation = builder.ast.get_packed(annotation);
                let annotation = builder.ast.get_node(annotation);
                let annotation = match annotation.data {
                    NodeData::OuterAnnotation(expr, _) => expr,
                    _ => unreachable!("{annotation:?}"),
                };
                lazy_members.extend(lower_ast_expr(builder, annotation, ExprKind::RValue)?);
                let annotation = *lazy_members.last().unwrap();
                lazy_members.push(builder.push_node(Hir::PushDeclAnnotation(i as u32, annotation)));
            }
        }
    }

    for init_expr in init_list {
        let init_expr = builder.ast.get_packed(init_expr);
        lazy_members.extend(lower_ast_expr(builder, init_expr, ExprKind::RValueInit)?);
        let init_expr = *lazy_members.last().unwrap();
        lazy_members.push(builder.push_node(Hir::InitDecl(init_expr)));
    }
    lazy_members.push(builder.push_node(Hir::EnsureDeclsInit));

    for prototype in prototypes {
        let crate::ast::DeclProto {
            is_pub,
            is_var,
            is_inline,
            ident,
            ..
        } = builder.ast.get_packed(prototype);
        let ident_str = builder.ast.get_ident(ident);

        if is_primitive(ident_str) {
            let loc = builder.get_loc(node_idx);
            let span = loc.into_range();
            let report = [Level::ERROR
                .primary_title(format!(r##"cannot shadow primitive `{ident_str}`, try using the raw identifier `#"{ident_str}"`"##))
                .element(
                    Snippet::source(builder.ast.get_source())
                        .annotation(AnnotationKind::Context.span(span)),
                )];
            builder.cu.emit_report(&report, ReportKind::Error);
            return Err(CompilationError);
        }

        let decl = builder.push_packed(DeclInfo {
            decl_type,
            prototype: DeclProto {
                is_pub,
                is_var,
                is_inline,
                ident,
            },
        });
        ir_members.push(builder.push_node(Hir::Declaration(decl)));
    }

    let lazy_members = builder.push_packed_list(&lazy_members);
    builder.patch_node(lazy_idx, Hir::LazyBlock(lazy_members));

    let ir_members = builder.push_packed_list(&ir_members);
    builder.patch_node(ir_idx, Hir::Declarations(ir_members));

    Ok(ir_idxs)
}

fn lower_ast_expr_stmt(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = builder.ast.get_node(node_idx);
    let expr = match node.data {
        NodeData::ExprSemicolon(expr) => expr,
        _ => unreachable!("{node:?}"),
    };

    lower_ast_expr(builder, expr, ExprKind::Statement)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ExprKind {
    Statement,
    LValue,
    RValue,
    RValueInit,
    RValueRet,
}

fn lower_ast_expr(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    kind: ExprKind,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = builder.ast.get_node(node_idx);
    match node.data {
        NodeData::Range(_, _) => todo!(),
        NodeData::Binary(_, _) => lower_ast_expr_binary(builder, node_idx, kind),
        NodeData::Unary(_) => lower_ast_expr_unary(builder, node_idx, kind),
        NodeData::Asm(_, _) => todo!(),
        NodeData::Jump(_, _) => todo!(),
        NodeData::Const(_) => todo!(),
        NodeData::Label(_) => todo!(),
        NodeData::TryBlock(_) => todo!(),
        NodeData::Fn(_, _) => todo!(),
        NodeData::BlockTwo(_, _) => todo!(),
        NodeData::Block(_) => todo!(),
        NodeData::ExprInit(_, _) => todo!(),
        NodeData::EnumLiteral(_) => todo!(),
        NodeData::TokenInit(_) => todo!(),
        NodeData::ErrorUnionType(_, _) => todo!(),
        NodeData::Pack(_) => todo!(),
        NodeData::OptionType(_) => todo!(),
        NodeData::Slice(_, _) => todo!(),
        NodeData::SinglePointerSimple(_, _) | NodeData::SinglePointer(_, _) => todo!(),
        NodeData::MultiPointerSimple(_, _) | NodeData::MultiPointer(_, _) => todo!(),
        NodeData::Vector(_, _) => todo!(),
        NodeData::MatrixSimple(_, _) | NodeData::Matrix(_, _) => todo!(),
        NodeData::ArraySimple(_, _) | NodeData::Array(_, _) => todo!(),
        NodeData::Ident => lower_ast_expr_ident(builder, node_idx, kind),
        NodeData::CoreIdent => todo!(),
        NodeData::BuiltinIdent => todo!(),
        NodeData::WhereExpr(_) => todo!(),
        NodeData::SelfType => todo!(),
        NodeData::SelfIdent => todo!(),
        NodeData::Unreachable => todo!(),
        NodeData::CharLiteral => todo!(),
        NodeData::FloatLiteral => todo!(),
        NodeData::IntLiteral => lower_ast_expr_int_literal(builder, node_idx),
        NodeData::StringLiteral => lower_ast_expr_string_literal(builder, node_idx),
        NodeData::RawStringLiteral(_, _) => todo!(),
        NodeData::Container(_, _) | NodeData::ContainerConst(_, _) => {
            lower_ast_expr_container(builder, node_idx, kind)
        }
        NodeData::Index(_, _) => todo!(),
        NodeData::TypeBinarySuffix(_, _) => todo!(),
        NodeData::TypeUnarySuffix(_) => todo!(),
        NodeData::Call1(_, _)
        | NodeData::Call1Comma(_, _)
        | NodeData::Call(_, _)
        | NodeData::CallComma(_, _) => lower_ast_expr_call(builder, node_idx, kind),
        _ => unreachable!("{node:?}"),
    }
}

fn lower_ast_expr_binary(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    kind: ExprKind,
) -> Result<Vec<HirIdx>, CompilationError> {
    let wrapped = |builder: &mut Builder<'_>| {
        let node = builder.ast.get_node(node_idx);
        let token = builder.ast.get_token(node.main_token);
        let (lhs, rhs) = match node.data {
            NodeData::Binary(lhs, rhs) => (lhs, rhs),
            _ => unreachable!(),
        };

        let lhs_kind = match token.tag {
            Token!(*=)
            | Token!(*|=)
            | Token!(*%=)
            | Token!(/=)
            | Token!(%=)
            | Token!(+=)
            | Token!(+|=)
            | Token!(+%=)
            | Token!(-=)
            | Token!(-|=)
            | Token!(-%=)
            | Token!(<<=)
            | Token!(<<|=)
            | Token!(>>=)
            | Token!(&=)
            | Token!(^=)
            | Token!(|=)
            | Token!(=) => ExprKind::LValue,
            _ => ExprKind::RValue,
        };

        let mut nodes = vec![];
        nodes.extend(lower_ast_expr(builder, lhs, lhs_kind)?);
        let lhs = *nodes.last().unwrap();

        nodes.extend(lower_ast_expr(builder, rhs, ExprKind::RValue)?);
        let rhs = *nodes.last().unwrap();

        nodes.extend(builder.set_dbg_loc(node_idx));
        let node = match token.tag {
            Token!(*=) => Hir::MulAssign(lhs, rhs),
            Token!(*|=) => Hir::MulSatAssign(lhs, rhs),
            Token!(*%=) => Hir::MulWrapAssign(lhs, rhs),
            Token!(/=) => Hir::DivAssign(lhs, rhs),
            Token!(%=) => Hir::RemAssign(lhs, rhs),
            Token!(+=) => Hir::AddAssign(lhs, rhs),
            Token!(+|=) => Hir::AddSatAssign(lhs, rhs),
            Token!(+%=) => Hir::AddWrapAssign(lhs, rhs),
            Token!(-=) => Hir::SubAssign(lhs, rhs),
            Token!(-|=) => Hir::SubSatAssign(lhs, rhs),
            Token!(-%=) => Hir::SubWrapAssign(lhs, rhs),
            Token!(<<=) => Hir::ShlAssign(lhs, rhs),
            Token!(<<|=) => Hir::ShlSatAssign(lhs, rhs),
            Token!(>>=) => Hir::ShrAssign(lhs, rhs),
            Token!(&=) => Hir::BitAndAssign(lhs, rhs),
            Token!(^=) => Hir::BitXorAssign(lhs, rhs),
            Token!(|=) => Hir::BitOrAssign(lhs, rhs),
            Token!(=) => {
                let coerced = builder.push_node(Hir::AsPtrChild(lhs, rhs));
                nodes.push(coerced);
                Hir::Assign(lhs, coerced)
            }

            Token!(or) => Hir::LogicalOr(lhs, rhs),
            Token!(and) => Hir::LogicalAnd(lhs, rhs),

            Token!(==) => Hir::Eq(lhs, rhs),
            Token!(!=) => Hir::NotEq(lhs, rhs),
            Token!(<) => Hir::Lt(lhs, rhs),
            Token!(<=) => Hir::LtEq(lhs, rhs),
            Token!(>) => Hir::Gt(lhs, rhs),
            Token!(>=) => Hir::GtEq(lhs, rhs),
            Token!(<=>) => Hir::Cmp(lhs, rhs),

            Token!(&) => Hir::BitAnd(lhs, rhs),
            Token!(^) => Hir::BitXor(lhs, rhs),
            Token!(|) => Hir::BitOr(lhs, rhs),
            Token!(or_else) => Hir::OrElse(lhs, rhs),
            Token!(catch) => Hir::Catch(lhs, rhs),

            Token!(<<) => Hir::Shl(lhs, rhs),
            Token!(<<|) => Hir::ShlSat(lhs, rhs),
            Token!(>>) => Hir::Shr(lhs, rhs),

            Token!(+) => Hir::Add(lhs, rhs),
            Token!(+|) => Hir::AddSat(lhs, rhs),
            Token!(+%) => Hir::AddWrap(lhs, rhs),
            Token!(-) => Hir::Sub(lhs, rhs),
            Token!(-|) => Hir::SubSat(lhs, rhs),
            Token!(-%) => Hir::SubWrap(lhs, rhs),
            Token!(++) => Hir::Concat(lhs, rhs),

            Token!(*) => Hir::Mul(lhs, rhs),
            Token!(*|) => Hir::MulSat(lhs, rhs),
            Token!(*%) => Hir::MulWrap(lhs, rhs),
            Token!(/) => Hir::Div(lhs, rhs),
            Token!(%) => Hir::Rem(lhs, rhs),
            _ => unreachable!(),
        };
        nodes.push(builder.push_node(node));

        Ok(nodes)
    };

    let mut ir_idxs = vec![];
    ir_idxs.extend(builder.set_dbg_loc(node_idx));

    if kind == ExprKind::Statement {
        let mut ir_members = vec![];
        let ir_idx = builder.push_node(Hir::Noop);
        ir_idxs.push(ir_idx);
        ir_members.extend(wrapped(builder)?);
        ir_members.push(builder.push_node(Hir::BreakInline(*ir_members.last().unwrap())));
        let ir_members = builder.push_packed_list(&ir_members);
        builder.patch_node(ir_idx, Hir::InlineBlock(ir_members));
    } else {
        ir_idxs.extend(wrapped(builder)?);
    }

    Ok(ir_idxs)
}

fn lower_ast_expr_unary(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    kind: ExprKind,
) -> Result<Vec<HirIdx>, CompilationError> {
    let wrapped = |builder: &mut Builder<'_>| {
        let node = builder.ast.get_node(node_idx);
        let token = builder.ast.get_token(node.main_token);
        let expr = match node.data {
            NodeData::Unary(expr) => expr,
            _ => unreachable!(),
        };

        let mut nodes = vec![];
        nodes.extend(lower_ast_expr(builder, expr, ExprKind::RValue)?);
        let expr = *nodes.last().unwrap();

        let node = match token.tag {
            Token!(!) => Hir::Not(expr),
            Token!(-) => Hir::Neg(expr),
            Token!(~) => Hir::BitNot(expr),
            Token!(-%) => Hir::NegWrap(expr),
            Token!(...) => Hir::Destructure(expr),
            _ => unreachable!(),
        };
        nodes.extend(builder.set_dbg_loc(node_idx));
        nodes.push(builder.push_node(node));

        Ok(nodes)
    };

    let mut ir_idxs = vec![];
    ir_idxs.extend(builder.set_dbg_loc(node_idx));

    if kind == ExprKind::Statement {
        let mut ir_members = vec![];
        let ir_idx = builder.push_node(Hir::Noop);
        ir_idxs.push(ir_idx);
        ir_members.extend(wrapped(builder)?);
        ir_members.push(builder.push_node(Hir::BreakInline(*ir_members.last().unwrap())));
        let ir_members = builder.push_packed_list(&ir_members);
        builder.patch_node(ir_idx, Hir::InlineBlock(ir_members));
    } else {
        ir_idxs.extend(wrapped(builder)?);
    }

    Ok(ir_idxs)
}

fn lower_ast_expr_ident(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    kind: ExprKind,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = *builder.ast.get_node(node_idx);
    match node.data {
        NodeData::Ident => {}
        _ => unreachable!("{node:?}"),
    }
    let ident = builder.ast.get_ident(node.main_token);

    let mut nodes = vec![];
    nodes.extend(builder.set_dbg_loc(node_idx));
    match ident {
        "any_int" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_ANY_INT))),
        "any_float" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_ANY_FLOAT))),
        "any_opaque" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_ANY_OPAQUE))),
        "any_type" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_ANY_TYPE))),
        "bool" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_BOOL))),
        "char" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_CHAR))),
        "int" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_INT))),
        "uint" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_UINT))),
        "intptr" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_INTPTR))),
        "uintptr" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_UINTPTR))),
        "no_return" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_NO_RETURN))),
        "rawptr" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_RAWPTR))),
        "string" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_STRING))),
        "cstring" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_CSTRING))),
        "void" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_VOID))),
        "type" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_TYPE))),
        "type_id" => nodes.push(builder.push_node(Hir::Immediate(Index::TY_TYPE_ID))),
        "null" => nodes.push(builder.push_node(Hir::Immediate(Index::VAL_NULL))),
        "undefined" => nodes.push(builder.push_node(Hir::Immediate(Index::VAL_UNDEFINED))),
        "true" => nodes.push(builder.push_node(Hir::Immediate(Index::VAL_TRUE))),
        "false" => nodes.push(builder.push_node(Hir::Immediate(Index::VAL_FALSE))),
        _ if ident.starts_with("b")
            && let Ok(width) = ident[1..].parse::<u16>() =>
        {
            match width {
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_BOOL))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_B8))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_B16))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_B32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_B64))),
                _ => nodes.push(builder.push_node(Hir::Bool { width })),
            }
        }
        _ if ident.starts_with("i")
            && ident.ends_with("le")
            && let Ok(width) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            match width {
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I1LE))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I8LE))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I16LE))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I32LE))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I64LE))),
                128 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I128LE))),
                _ => nodes.push(builder.push_node(Hir::Int {
                    width,
                    is_signed: true,
                    is_little_endian: Some(true),
                })),
            }
        }
        _ if ident.starts_with("i")
            && ident.ends_with("be")
            && let Ok(width) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            match width {
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I1BE))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I8BE))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I16BE))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I32BE))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I64BE))),
                128 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I128BE))),
                _ => nodes.push(builder.push_node(Hir::Int {
                    width,
                    is_signed: true,
                    is_little_endian: Some(false),
                })),
            }
        }
        _ if ident.starts_with("i")
            && let Ok(width) = ident[1..].parse::<u16>() =>
        {
            match width {
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I1))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I8))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I16))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I64))),
                128 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_I128))),
                _ => nodes.push(builder.push_node(Hir::Int {
                    width,
                    is_signed: true,
                    is_little_endian: None,
                })),
            }
        }
        _ if ident.starts_with("u")
            && ident.ends_with("le")
            && let Ok(width) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            match width {
                0 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U0LE))),
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U1LE))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U8LE))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U16LE))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U32LE))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U64LE))),
                128 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U128LE))),
                _ => nodes.push(builder.push_node(Hir::Int {
                    width,
                    is_signed: false,
                    is_little_endian: Some(true),
                })),
            }
        }
        _ if ident.starts_with("u")
            && ident.ends_with("be")
            && let Ok(width) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            match width {
                0 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U0BE))),
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U1BE))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U8BE))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U16BE))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U32BE))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U64BE))),
                128 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U128BE))),
                _ => nodes.push(builder.push_node(Hir::Int {
                    width,
                    is_signed: false,
                    is_little_endian: Some(false),
                })),
            }
        }
        _ if ident.starts_with("u")
            && let Ok(width) = ident[1..].parse::<u16>() =>
        {
            match width {
                0 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U0))),
                1 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U1))),
                8 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U8))),
                16 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U16))),
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U64))),
                128 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_U128))),
                _ => nodes.push(builder.push_node(Hir::Int {
                    width,
                    is_signed: false,
                    is_little_endian: None,
                })),
            }
        }
        _ if ident.starts_with("f")
            && ident.ends_with("le")
            && let Ok(width) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            match width {
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_F32LE))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_F64LE))),
                _ => nodes.push(builder.push_node(Hir::Float {
                    width,
                    is_little_endian: Some(true),
                })),
            }
        }
        _ if ident.starts_with("f")
            && ident.ends_with("be")
            && let Ok(width) = ident[1..ident.len() - 2].parse::<u16>() =>
        {
            match width {
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_F32BE))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_F64BE))),
                _ => nodes.push(builder.push_node(Hir::Float {
                    width,
                    is_little_endian: Some(false),
                })),
            }
        }
        _ if ident.starts_with("f")
            && let Ok(width) = ident[1..].parse::<u16>() =>
        {
            match width {
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_F32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_F64))),
                _ => nodes.push(builder.push_node(Hir::Float {
                    width,
                    is_little_endian: None,
                })),
            }
        }
        _ if ident.starts_with("complex")
            && let Ok(width) = ident[7..].parse::<u16>() =>
        {
            match width {
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_COMPLEX32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_COMPLEX64))),
                _ => nodes.push(builder.push_node(Hir::Complex { width })),
            }
        }
        _ if ident.starts_with("quat")
            && let Ok(width) = ident[4..].parse::<u16>() =>
        {
            match width {
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_QUAT32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_QUAT64))),
                _ => nodes.push(builder.push_node(Hir::Quat { width })),
            }
        }
        _ if ident.starts_with("dquat")
            && let Ok(width) = ident[5..].parse::<u16>() =>
        {
            match width {
                32 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_DQUAT32))),
                64 => nodes.push(builder.push_node(Hir::Immediate(Index::TY_DQUAT64))),
                _ => nodes.push(builder.push_node(Hir::DQuat { width })),
            }
        }
        _ => match kind {
            ExprKind::Statement | ExprKind::RValue | ExprKind::RValueInit | ExprKind::RValueRet => {
                nodes.push(builder.push_node(Hir::LoadIdent(node.main_token)))
            }
            ExprKind::LValue => nodes.push(builder.push_node(Hir::LoadIdentPtr(node.main_token))),
        },
    }

    Ok(nodes)
}

fn lower_ast_expr_int_literal(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = *builder.ast.get_node(node_idx);
    match node.data {
        NodeData::IntLiteral => {}
        _ => unreachable!("{node:?}"),
    }

    let mut nodes = vec![];
    nodes.extend(builder.set_dbg_loc(node_idx));
    nodes.push(builder.push_node(Hir::IntLiteral(node.main_token)));
    Ok(nodes)
}

fn lower_ast_expr_string_literal(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
) -> Result<Vec<HirIdx>, CompilationError> {
    let node = *builder.ast.get_node(node_idx);
    match node.data {
        NodeData::StringLiteral => {}
        _ => unreachable!("{node:?}"),
    }

    let mut nodes = vec![];
    nodes.extend(builder.set_dbg_loc(node_idx));
    nodes.push(builder.push_node(Hir::StringLiteral(node.main_token)));
    Ok(nodes)
}

fn lower_ast_expr_container(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    kind: ExprKind,
) -> Result<Vec<HirIdx>, CompilationError> {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    enum ContainerKind {
        Enum,
        Namespace,
        Opaque,
        Struct,
        Union,
        Primitive,
    }

    let name_strategy = match kind {
        ExprKind::Statement | ExprKind::LValue | ExprKind::RValue => NameStrategy::Anon,
        ExprKind::RValueInit => NameStrategy::Decl,
        ExprKind::RValueRet => NameStrategy::Parent,
    };

    let node = builder.ast.get_node(node_idx);
    let is_const = matches!(node.data, NodeData::ContainerConst(_, _));
    let container_kind = match builder.ast.get_token(node.main_token).tag {
        Token!(enum) => ContainerKind::Enum,
        Token!(namespace) => ContainerKind::Namespace,
        Token!(opaque) => ContainerKind::Opaque,
        Token!(struct) => ContainerKind::Struct,
        Token!(union) => ContainerKind::Union,
        Token!(#primitive) => ContainerKind::Primitive,
        _ => unreachable!(),
    };
    let (layout, block) = match node.data {
        NodeData::Container(layout, block) => (layout, block),
        NodeData::ContainerConst(layout, block) => (layout, block),
        _ => unreachable!(),
    };
    let mut members = match builder.ast.get_node(block).data {
        NodeData::BlockTwo(first, second) => AstRangeIterator::Two([first, second]),
        NodeData::BlockTwoSemicolon(first, second) => AstRangeIterator::Two([first, second]),
        NodeData::Block(members) => AstRangeIterator::Many(members),
        NodeData::BlockSemicolon(members) => AstRangeIterator::Many(members),
        _ => unreachable!(),
    };

    if is_const
        && matches!(
            container_kind,
            ContainerKind::Namespace | ContainerKind::Primitive
        )
    {
        let loc = builder.get_loc(node_idx);
        let span = loc.into_range();
        let report = [Level::ERROR
            .primary_title(match container_kind {
                ContainerKind::Namespace => "namespace can not be marked as `const`",
                ContainerKind::Primitive => "primitive can not be marked as `const`",
                _ => unreachable!(),
            })
            .element(
                Snippet::source(builder.ast.get_source())
                    .annotation(AnnotationKind::Context.span(span)),
            )];
        builder.cu.emit_report(&report, ReportKind::Error);
        return Err(CompilationError);
    }

    if layout.is_some() && container_kind == ContainerKind::Namespace {
        let loc = builder.get_loc(node_idx);
        let span = loc.into_range();
        let report = [Level::ERROR
            .primary_title("namespace layout not permitted")
            .element(
                Snippet::source(builder.ast.get_source())
                    .annotation(AnnotationKind::Context.span(span)),
            )];
        builder.cu.emit_report(&report, ReportKind::Error);
        return Err(CompilationError);
    }

    if layout.is_none() && container_kind == ContainerKind::Primitive {
        let loc = builder.get_loc(node_idx);
        let span = loc.into_range();
        let report = [Level::ERROR.primary_title("expected primitive id").element(
            Snippet::source(builder.ast.get_source())
                .annotation(AnnotationKind::Context.span(span)),
        )];
        builder.cu.emit_report(&report, ReportKind::Error);
        return Err(CompilationError);
    }

    let mut expr_members = vec![];
    expr_members.extend(builder.set_dbg_loc(node_idx));

    let ir_idx = builder.push_node(Hir::Noop);
    expr_members.push(ir_idx);

    let mut block_members = vec![];
    let layout: Option<HirIdx> = match layout {
        Some(_) => {
            debug_assert_ne!(container_kind, ContainerKind::Namespace);
            todo!();
        }
        None => None,
    };

    while let Some(member) = members.next(builder.ast) {
        block_members.extend(lower_ast_stmt(builder, member)?);
    }

    let block_members = builder.push_packed_list(&block_members);
    let node = match container_kind {
        ContainerKind::Enum => todo!(),
        ContainerKind::Namespace => {
            Hir::Namespace(name_strategy, builder.push_packed(block_members))
        }
        ContainerKind::Opaque => todo!(),
        ContainerKind::Struct => todo!(),
        ContainerKind::Union => todo!(),
        ContainerKind::Primitive => todo!(),
    };
    builder.patch_node(ir_idx, node);

    Ok(expr_members)
}

fn lower_ast_expr_call(
    builder: &mut Builder<'_>,
    node_idx: NodeIndex,
    kind: ExprKind,
) -> Result<Vec<HirIdx>, CompilationError> {
    let wrapped = |builder: &mut Builder<'_>| {
        let node = builder.ast.get_node(node_idx);
        let (callee, mut args_nodes) = match node.data {
            NodeData::Call1(callee, arg) | NodeData::Call1Comma(callee, arg) => {
                (callee, AstRangeIterator::Two([arg, None]))
            }
            NodeData::Call(callee, args) | NodeData::CallComma(callee, args) => {
                let args = builder.ast.get_packed(args);
                (callee, AstRangeIterator::Many(args))
            }
            _ => unreachable!(),
        };

        let callee_node = *builder.ast.get_node(callee);
        if let NodeData::BuiltinIdent = callee_node.data {
            let builtin = builder.ast.get_builtin_ident(callee_node.main_token);
            let builtin = match BuiltInFunction::from_string(builtin) {
                Some(x) => x,
                None => {
                    let loc = builder.get_loc(callee);
                    let span = loc.into_range();
                    let report = [Level::ERROR
                        .primary_title(format!("unknown builtin function: `{builtin}`"))
                        .element(
                            Snippet::source(builder.ast.get_source())
                                .annotation(AnnotationKind::Context.span(span)),
                        )];
                    builder.cu.emit_report(&report, ReportKind::Error);
                    return Err(CompilationError);
                }
            };

            let mut nodes = vec![];
            let mut args = vec![];
            while let Some(arg) = args_nodes.next(builder.ast) {
                let arg_node = builder.ast.get_node(arg);
                match arg_node.data {
                    NodeData::Ident if builder.ast.get_ident(arg_node.main_token) == "_" => {
                        let ctx = builder
                            .push_packed_list(&args)
                            .map(|x| builder.push_packed(x));
                        nodes.extend(builder.set_dbg_loc(arg));
                        let default_arg = builder.push_node(Hir::BuiltInArgDefault(builtin, ctx));
                        nodes.push(default_arg);
                        args.push(default_arg);
                    }
                    NodeData::Unary(_)
                        if builder.ast.get_token(arg_node.main_token).tag == Token!(...) =>
                    {
                        todo!()
                    }
                    _ => {
                        let ctx = builder
                            .push_packed_list(&args)
                            .map(|x| builder.push_packed(x));
                        nodes.extend(lower_ast_expr(builder, arg, ExprKind::RValue)?);
                        let arg_type = builder.push_node(Hir::BuiltInArgTypeOf(builtin, ctx));
                        let arg = *nodes.last().unwrap();
                        let coerced_arg = builder.push_node(Hir::As(arg_type, arg));
                        nodes.push(arg_type);
                        nodes.push(coerced_arg);
                        args.push(coerced_arg);
                    }
                }
            }

            let args = builder
                .push_packed_list(&args)
                .map(|x| builder.push_packed(x));
            nodes.extend(builder.set_dbg_loc(node_idx));
            nodes.push(builder.push_node(Hir::BuiltInCall(builtin, args)));

            Ok(nodes)
        } else {
            todo!()
        }
    };

    let mut ir_idxs = vec![];
    ir_idxs.extend(builder.set_dbg_loc(node_idx));

    if kind == ExprKind::Statement {
        let mut ir_members = vec![];
        let ir_idx = builder.push_node(Hir::Noop);
        ir_idxs.push(ir_idx);
        ir_members.extend(wrapped(builder)?);
        ir_members.push(builder.push_node(Hir::BreakInline(*ir_members.last().unwrap())));
        let ir_members = builder.push_packed_list(&ir_members);
        builder.patch_node(ir_idx, Hir::InlineBlock(ir_members));
    } else {
        ir_idxs.extend(wrapped(builder)?);
    }

    Ok(ir_idxs)
}
