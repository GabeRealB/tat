use std::{
    cell::{Cell, UnsafeCell},
    sync::{Arc, Weak, atomic::Ordering},
};

use annotate_snippets::{AnnotationKind, Level, Snippet};
use rapidhash::RapidHashMap;
use smol::lock::{RwLock, RwLockWriteGuard};

use num_bigint::BigInt;

use crate::{
    ast::{DeclType, NodeIndex},
    compilation_unit::{Cu, ReportKind},
    hir::{
        CompilationError, DeclProto, Hir, HirChunk, HirChunkView, HirIdx, HirRangeIterator,
        SingleDecl, SrcLocation,
    },
    intern_pool::{
        Capture, Decl, DeclId, DeclInner, HirInfo, Index, InternPool, Key, KeyBigIntStorage,
        KeyInt, KeyIntStorage, KeyTag, KeyTypeNamespace, LDScopeId, LazyDeclScope, LocalPool,
        RawCString, Scope, TypeNamespace, TypedIndex, UnorderedDeclScope, UnorderedDeclScopeInner,
    },
    util::NonMaxU32,
};

pub struct Sema {
    pub cu: Weak<Cu>,
    pub inner: RwLock<SemaInner>,
}

impl Sema {
    pub fn new(cu: &Arc<Cu>) -> Self {
        Self {
            cu: Arc::downgrade(cu),
            inner: RwLock::new(SemaInner {
                blocks: Default::default(),
                decls: Default::default(),
                nodes: Default::default(),
                extra: Default::default(),
                constants: Default::default(),

                has_error: false,
                block_idx: 0,
                src_loc: SrcLocation {
                    byte_start: 0,
                    byte_len: 0,
                },
            }),
        }
    }

    pub async fn eval_as_root_scope(&self, scope: Scope) -> Result<(), CompilationError> {
        let cu = self.cu.upgrade().expect("invalid compilation unit");
        let mut inner = self.inner.write().await;
        assert!(inner.blocks.is_empty(), "sema already evaluating");

        let rscope = match scope {
            Scope::Root(rscope_id) => rscope_id.get_from_pool(cu.pool()),
            _ => unreachable!(),
        };
        let hir_info = rscope.hir_info.get_from_pool(cu.pool());
        let chunk = hir_info.id.get_from_pool(cu.pool()).view();
        let ast_info = hir_info.ast_info.get_from_pool(cu.pool());
        let file = ast_info.file.get_from_pool(cu.pool());
        let qualified_name = file.qualified_name;

        let ast = ast_info.id.get_from_pool(cu.pool());
        let (start, end) = ast.get_node_token_span(NodeIndex::ROOT);
        let start = ast.get_token(start);
        let end = ast.get_token(end);
        let byte_start = start.start;
        let byte_end = end.start + end.length;
        let byte_len = byte_end - byte_start;
        let src_loc = SrcLocation {
            byte_start,
            byte_len,
        };

        inner.src_loc = src_loc;
        inner.blocks.push(Block {
            scope,
            hir_info: rscope.hir_info,
            parent_block: None,
            src_loc,
            qualified_name,

            kind: BlockKind::FileRoot,

            tmp_decl_idx: 0,
            tmp_decls: Default::default(),

            decls: Default::default(),
            inst_map: Default::default(),
            inst_idxs: Default::default(),
        });

        let nodes = HirRangeIterator::Two([Some(HirIdx::ROOT), None]);
        match sema_block(&cu, self, &mut inner, chunk, nodes).await {
            Ok(result) => {
                let waiters = {
                    let _scope_guard = rscope.lock.write().await;
                    let scope_inner = unsafe { rscope.inner.as_mut_unchecked() };
                    scope_inner.resolved = true;
                    scope_inner.result = match result {
                        ControlFlow::None(index) => index,
                    };
                    std::mem::take(&mut scope_inner.waiters)
                };
                for waiter in waiters {
                    waiter.wake();
                }
                Ok(())
            }
            Err(err) => {
                inner.has_error = true;
                let waiters = {
                    let _scope_guard = rscope.lock.write().await;
                    let scope_inner = unsafe { rscope.inner.as_mut_unchecked() };
                    scope_inner.resolved = true;
                    std::mem::take(&mut scope_inner.waiters)
                };
                for waiter in waiters {
                    waiter.wake();
                }
                Err(err)
            }
        }
    }

    pub async fn eval_as_lazy_decl_scope(
        &self,
        scope_id: LDScopeId,
    ) -> Result<(), CompilationError> {
        let cu = self.cu.upgrade().expect("invalid compilation unit");
        let ip = cu.pool();
        let mut inner = self.inner.write().await;
        assert!(inner.blocks.is_empty(), "sema already evaluating");

        let scope = scope_id.get_from_pool(ip);
        let parent_scope = scope.parent.get_from_pool(ip);
        let hir_info = scope.hir_info.get_from_pool(ip);
        let chunk = hir_info.id.get_from_pool(ip).view();

        let parent_ty = parent_scope.ty.get().unwrap();
        let qualified_name = ip.get_type_name(parent_ty);
        let src_loc = scope.decl_loc;

        let mut tmp_decls = vec![];
        let decls = unsafe { scope.decls.as_ref_unchecked() };
        for decl in decls {
            let Decl { name, .. } = decl.get_from_pool(ip);
            tmp_decls.push(TmpDecl {
                name: *name,
                ty: None,
                align: None,
                annotations: Default::default(),
            });
        }

        inner.src_loc = src_loc;
        inner.blocks.push(Block {
            scope: Scope::Lazy(scope_id),
            hir_info: scope.hir_info,
            parent_block: None,
            src_loc,
            qualified_name,

            kind: BlockKind::LazyDecl,

            tmp_decl_idx: 0,
            tmp_decls,

            decls: Default::default(),
            inst_map: Default::default(),
            inst_idxs: Default::default(),
        });

        let mut block_iter = chunk.block_range_iter(scope.decl_idx).iter(chunk);
        let lazy_idx = block_iter.next().unwrap();

        let nodes = chunk.block_range_iter(lazy_idx);
        match sema_block(&cu, self, &mut inner, chunk, nodes).await {
            Ok(result) => {
                assert!(matches!(result, ControlFlow::None(None)));
                scope.notification.notify().await;
                Ok(())
            }
            Err(err) => {
                inner.has_error = true;
                inner.blocks[0].tmp_decls.clear();
                scope.notification.notify().await;
                Err(err)
            }
        }
    }
}

pub struct SemaInner {
    pub blocks: Vec<Block>,
    pub decls: Vec<()>,
    pub nodes: Vec<Hir>,
    pub extra: Vec<u32>,
    pub constants: RapidHashMap<Index, HirIdx>,

    pub has_error: bool,
    pub block_idx: usize,
    pub src_loc: SrcLocation,
}

impl SemaInner {
    pub fn chunk_view(&self) -> HirChunkView<'_> {
        HirChunkView {
            nodes: &self.nodes,
            extra: &self.extra,
        }
    }

    fn emit_fatal_error(&self, cu: &Arc<Cu>, desc: &str) -> Result<(), CompilationError> {
        let ip = cu.pool();
        let hir_info = self.hir_info().get_from_pool(ip);
        let ast_info = hir_info.ast_info.get_from_pool(ip);
        let ast = ast_info.id.get_from_pool(ip);
        let source = ast.get_source();

        let span_start = self.src_loc.byte_start as usize;
        let span_end = span_start + self.src_loc.byte_len as usize;
        let span = span_start..span_end;

        let report = [Level::ERROR
            .primary_title(desc)
            .element(Snippet::source(source).annotation(AnnotationKind::Context.span(span)))];
        cu.emit_report(&report, ReportKind::FatalError);

        Err(CompilationError)
    }

    fn emit_error(&self, cu: &Arc<Cu>, desc: &str) {
        let ip = cu.pool();
        let hir_info = self.hir_info().get_from_pool(ip);
        let ast_info = hir_info.ast_info.get_from_pool(ip);
        let ast = ast_info.id.get_from_pool(ip);
        let source = ast.get_source();

        let span_start = self.src_loc.byte_start as usize;
        let span_end = span_start + self.src_loc.byte_len as usize;
        let span = span_start..span_end;

        let report = [Level::ERROR
            .primary_title(desc)
            .element(Snippet::source(source).annotation(AnnotationKind::Context.span(span)))];
        cu.emit_report(&report, ReportKind::FatalError);
    }

    fn emit_warning(&self, cu: &Arc<Cu>, desc: &str) {
        let ip = cu.pool();
        let hir_info = self.hir_info().get_from_pool(ip);
        let ast_info = hir_info.ast_info.get_from_pool(ip);
        let ast = ast_info.id.get_from_pool(ip);
        let source = ast.get_source();

        let span_start = self.src_loc.byte_start as usize;
        let span_end = span_start + self.src_loc.byte_len as usize;
        let span = span_start..span_end;

        let report = [Level::WARNING
            .primary_title(desc)
            .element(Snippet::source(source).annotation(AnnotationKind::Context.span(span)))];
        cu.emit_report(&report, ReportKind::FatalError);
    }

    fn push_block(&mut self, mut block: Block) {
        assert!(self.blocks.len() < u32::MAX as usize);
        block.parent_block = Some(self.block_idx as u32);
        self.block_idx = self.blocks.len();
        self.src_loc = block.src_loc;
        self.blocks.push(block);
    }

    fn pop_block(&mut self) {
        let block = &mut self.blocks[self.block_idx];
        block.tmp_decls.clear();

        let parent = block.parent_block.expect("block has no parent") as usize;
        self.block_idx = parent;
    }

    fn scope(&self) -> Scope {
        self.blocks[self.block_idx].scope
    }

    fn hir_info(&self) -> TypedIndex<HirInfo> {
        self.blocks[self.block_idx].hir_info
    }

    fn qualified_name(&self) -> RawCString {
        self.blocks[self.block_idx].qualified_name
    }

    fn block_kind(&self) -> BlockKind {
        self.blocks[self.block_idx].kind
    }

    fn block_is_const(&self) -> bool {
        match self.blocks[self.block_idx].kind {
            BlockKind::FileRoot => true,
            BlockKind::Namespace => true,
            BlockKind::LazyDecl => true,
            BlockKind::Const => true,
        }
    }

    fn get_immediate(&self, hir_idx: HirIdx) -> Option<Index> {
        let block = &self.blocks[self.block_idx];
        let mapped = *block.inst_map.get(&hir_idx)?;
        match self.chunk_view().get_node(mapped) {
            Hir::Immediate(value) => Some(value),
            _ => None,
        }
    }

    fn set_src_loc(&mut self, src_loc: SrcLocation) {
        self.src_loc = src_loc;
    }

    fn push_node_raw(&mut self, node: Hir) -> HirIdx {
        let idx = HirIdx::new(self.nodes.len() as u32);
        self.nodes.push(node);
        idx
    }

    fn push_immediate(&mut self, inst_idx: Option<HirIdx>, value: Index) -> HirIdx {
        let idx = match self.constants.get(&value) {
            Some(&idx) => idx,
            None => {
                let idx = self.push_node_raw(Hir::Immediate(value));
                self.constants.insert(value, idx);
                self.blocks[0].inst_idxs.push(idx);
                idx
            }
        };

        if let Some(inst_idx) = inst_idx {
            self.blocks[self.block_idx].inst_map.insert(inst_idx, idx);
        }

        idx
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlockKind {
    FileRoot,
    Namespace,
    LazyDecl,
    Const,
}

#[derive(Debug)]
pub struct TmpDecl {
    name: RawCString,
    ty: Option<Index>,
    align: Option<Index>,
    annotations: Vec<Index>,
}

impl Default for TmpDecl {
    fn default() -> Self {
        Self {
            name: RawCString {
                ptr: "\0".as_bytes().into(),
            },
            ty: Default::default(),
            align: Default::default(),
            annotations: Default::default(),
        }
    }
}

pub struct Block {
    pub scope: Scope,
    pub hir_info: TypedIndex<HirInfo>,
    pub parent_block: Option<u32>,
    pub src_loc: SrcLocation,
    pub qualified_name: RawCString,

    pub kind: BlockKind,

    pub tmp_decl_idx: u32,
    pub tmp_decls: Vec<TmpDecl>,

    pub decls: RapidHashMap<RawCString, Vec<u32>>,
    pub inst_map: RapidHashMap<HirIdx, HirIdx>,
    pub inst_idxs: Vec<HirIdx>,
}

enum ControlFlow {
    None(Option<Index>),
}

async fn sema_block<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    chunk: HirChunkView<'a>,
    nodes: HirRangeIterator,
) -> Result<ControlFlow, CompilationError> {
    for hir_idx in nodes.iter(chunk) {
        let node = chunk.get_node(hir_idx);
        println!("{hir_idx}: {node:?}");
        match node {
            Hir::Noop => {}
            Hir::AstInfo(_) => {}
            Hir::DbgLocation(src_loc) => inner.set_src_loc(src_loc),

            Hir::Namespace(_) => sema_namespace(cu, sema, inner, chunk, hir_idx).await?,
            Hir::Declarations(_) => todo!(),

            Hir::PushDeclType(_) => todo!(),
            Hir::PushDeclAlign(_, _) => todo!(),
            Hir::PushDeclAnnotation(_, _) => todo!(),
            Hir::InitDecl(_) => sema_init_decl(cu, sema, inner, chunk, hir_idx).await?,
            Hir::EnsureDeclsInit => {
                let block_idx = inner.block_idx;
                let block = &mut inner.blocks[block_idx];
                let got = block.tmp_decl_idx as usize;
                let expected = block.tmp_decls.len();
                if got != expected {
                    inner.emit_fatal_error(
                        cu,
                        &format!("expected `{expected}` initializers, got `{got}`"),
                    )?;
                    unreachable!();
                }
                block.tmp_decls.clear();
            }
            Hir::Declaration(_) => unreachable!(),

            Hir::InlineBlock(_) => {
                // An inline block will be inlined directly in the current block.
                let nodes = chunk.block_range_iter(hir_idx);
                match Box::pin(sema_block(cu, sema, inner, chunk, nodes)).await? {
                    ControlFlow::None(None) => {}
                    ControlFlow::None(Some(value)) => {
                        inner.push_immediate(Some(hir_idx), value);
                    }
                };
            }
            Hir::LazyBlock(_) => {}
            Hir::ScopingBlock(_) => todo!(),
            Hir::ConstBlock(_) => todo!(),
            Hir::RunBlock(_) => todo!(),
            Hir::DeferBlock(_) => todo!(),
            Hir::ErrDeferBlock(_) => todo!(),
            Hir::ContDeferBlock(_) => todo!(),

            Hir::BreakInline(hir_idx) => todo!(),
            Hir::BreakLazy(hir_idx) => todo!(),

            Hir::MulAssign(hir_idx, hir_idx1) => todo!(),
            Hir::MulSatAssign(hir_idx, hir_idx1) => todo!(),
            Hir::MulWrapAssign(hir_idx, hir_idx1) => todo!(),
            Hir::DivAssign(hir_idx, hir_idx1) => todo!(),
            Hir::RemAssign(hir_idx, hir_idx1) => todo!(),
            Hir::AddAssign(hir_idx, hir_idx1) => todo!(),
            Hir::AddSatAssign(hir_idx, hir_idx1) => todo!(),
            Hir::AddWrapAssign(hir_idx, hir_idx1) => todo!(),
            Hir::SubAssign(hir_idx, hir_idx1) => todo!(),
            Hir::SubSatAssign(hir_idx, hir_idx1) => todo!(),
            Hir::SubWrapAssign(hir_idx, hir_idx1) => todo!(),
            Hir::ShlAssign(hir_idx, hir_idx1) => todo!(),
            Hir::ShlSatAssign(hir_idx, hir_idx1) => todo!(),
            Hir::ShrAssign(hir_idx, hir_idx1) => todo!(),
            Hir::BitAndAssign(hir_idx, hir_idx1) => todo!(),
            Hir::BitXorAssign(hir_idx, hir_idx1) => todo!(),
            Hir::BitOrAssign(hir_idx, hir_idx1) => todo!(),
            Hir::Assign(hir_idx, hir_idx1) => todo!(),

            Hir::LogicalOr(hir_idx, hir_idx1) => todo!(),
            Hir::LogicalAnd(hir_idx, hir_idx1) => todo!(),

            Hir::Eq(hir_idx, hir_idx1) => todo!(),
            Hir::NotEq(hir_idx, hir_idx1) => todo!(),
            Hir::Lt(hir_idx, hir_idx1) => todo!(),
            Hir::LtEq(hir_idx, hir_idx1) => todo!(),
            Hir::Gt(hir_idx, hir_idx1) => todo!(),
            Hir::GtEq(hir_idx, hir_idx1) => todo!(),
            Hir::Cmp(hir_idx, hir_idx1) => todo!(),

            Hir::BitAnd(hir_idx, hir_idx1) => todo!(),
            Hir::BitXor(hir_idx, hir_idx1) => todo!(),
            Hir::BitOr(hir_idx, hir_idx1) => todo!(),
            Hir::OrElse(hir_idx, hir_idx1) => todo!(),
            Hir::Catch(hir_idx, hir_idx1) => todo!(),

            Hir::Shl(hir_idx, hir_idx1) => todo!(),
            Hir::ShlSat(hir_idx, hir_idx1) => todo!(),
            Hir::Shr(hir_idx, hir_idx1) => todo!(),

            Hir::Add(hir_idx, hir_idx1) => todo!(),
            Hir::AddSat(hir_idx, hir_idx1) => todo!(),
            Hir::AddWrap(hir_idx, hir_idx1) => todo!(),

            Hir::Sub(hir_idx, hir_idx1) => todo!(),
            Hir::SubSat(hir_idx, hir_idx1) => todo!(),
            Hir::SubWrap(hir_idx, hir_idx1) => todo!(),
            Hir::Concat(hir_idx, hir_idx1) => todo!(),

            Hir::Mul(hir_idx, hir_idx1) => todo!(),
            Hir::MulSat(hir_idx, hir_idx1) => todo!(),
            Hir::MulWrap(hir_idx, hir_idx1) => todo!(),
            Hir::Div(hir_idx, hir_idx1) => todo!(),
            Hir::Rem(hir_idx, hir_idx1) => todo!(),

            Hir::Not(hir_idx) => todo!(),
            Hir::Neg(hir_idx) => todo!(),
            Hir::BitNot(hir_idx) => todo!(),
            Hir::NegWrap(hir_idx) => todo!(),
            Hir::Destructure(hir_idx) => todo!(),

            Hir::AnyOpaque(hir_idx) => todo!(),
            Hir::Bool { width } => todo!(),
            Hir::Int {
                width,
                is_signed,
                is_little_endian,
            } => todo!(),
            Hir::Float {
                width,
                is_little_endian,
            } => todo!(),
            Hir::Complex { width } => todo!(),
            Hir::Quat { width } => todo!(),
            Hir::DQuat { width } => todo!(),

            Hir::Immediate(idx) => _ = inner.push_immediate(Some(hir_idx), idx),
            Hir::IntLiteral(_) => sema_int_literal(cu, sema, inner, chunk, hir_idx).await?,
            Hir::StringLiteral(token_index) => todo!(),

            Hir::LoadIdent(_) => sema_load_ident(cu, sema, inner, chunk, hir_idx).await?,
            Hir::LoadIdentPtr(token_index) => todo!(),
            Hir::LoadCoreIdent(token_index) => todo!(),
            Hir::LoadCoreIdentPtr(token_index) => todo!(),

            Hir::InjectStmt(token_index) => todo!(),
            Hir::InjectExpr(token_index) => todo!(),

            Hir::As(hir_idx, hir_idx1) => todo!(),
            Hir::AsImmLhs(index, hir_idx) => todo!(),
            Hir::AsPtrChild(hir_idx, hir_idx1) => todo!(),

            Hir::BuiltInCall(built_in_function, extra_index) => todo!(),
            Hir::BuiltInArgTypeOf(built_in_function, extra_index) => todo!(),
            Hir::BuiltInArgDefault(built_in_function, extra_index) => todo!(),

            Hir::BuiltInConstPrint(extra_index_range) => todo!(),
            Hir::BuiltInExportSymbol(extra_index_range) => todo!(),
            Hir::BuiltInImport(extra_index_range) => todo!(),
        }
    }

    Ok(ControlFlow::None(None))
}

async fn sema_container<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    chunk: HirChunkView<'a>,
    nodes: HirRangeIterator,
) -> Result<(), CompilationError> {
    let ip = cu.pool();
    let local_ip = ip.get_or_init_local_pool().await;

    let hir_info = inner.hir_info().get_from_pool(ip);
    let ast_info = hir_info.ast_info.get_from_pool(ip);
    let ast = ast_info.id.get_from_pool(ip);

    let (container_scope_id, container_scope) = match inner.scope() {
        Scope::Type(id) => (id, id.get_from_pool(ip)),
        _ => unreachable!(),
    };

    let mut const_blocks = vec![];
    for hir_idx in nodes.iter(chunk) {
        let node = chunk.get_node(hir_idx);
        match node {
            Hir::Noop => {}
            Hir::AstInfo(_) => {}
            Hir::DbgLocation(src_loc) => inner.set_src_loc(src_loc),
            Hir::Declarations(_) => {
                let scope = LazyDeclScope {
                    hir_info: inner.hir_info(),
                    decl_loc: inner.src_loc,
                    decl_idx: hir_idx,
                    parent: container_scope_id,
                    decls: UnsafeCell::new(Default::default()),
                    decl_map: UnsafeCell::new(Default::default()),
                    notification: Default::default(),
                    resolving: Default::default(),
                    sema: UnsafeCell::new(Sema::new(cu)),
                };
                let scope_id = local_ip.intern_ldscope(scope).await;
                let scope = scope_id.get_from_pool(ip);
                let scope_decls = unsafe { scope.decls.as_mut_unchecked() };
                let scope_decl_map = unsafe { scope.decl_map.as_mut_unchecked() };

                let container_inner = unsafe { container_scope.inner.as_mut_unchecked() };
                let container_decls = &mut container_inner.decls;

                let mut nodes = chunk.block_range_iter(hir_idx).iter(chunk);
                _ = nodes.next().unwrap();
                for decl_idx in nodes {
                    let SingleDecl {
                        decl_type,
                        prototype:
                            DeclProto {
                                is_pub,
                                is_var,
                                is_inline,
                                ident,
                            },
                    } = match chunk.get_node(decl_idx) {
                        Hir::Declaration(decl) => chunk.get_packed(decl),
                        _ => unreachable!(),
                    };
                    let decl_type = match decl_type {
                        DeclType::Normal => match inner.block_kind() {
                            BlockKind::Namespace => DeclType::Static,
                            _ => unreachable!(),
                        },
                        _ => decl_type,
                    };
                    if is_var && decl_type == DeclType::Const {
                        inner.emit_fatal_error(
                            cu,
                            "mutable constant-evaluated declarations are only allowed in functions",
                        )?
                    }
                    if is_inline && (decl_type != DeclType::Normal) {
                        inner.emit_fatal_error(
                            cu,
                            "`inline` qualifier can only be applied to struct and union fields",
                        )?
                    }

                    let name = ast.get_ident(ident);
                    let mut name = name.to_string();
                    name.push('\0');
                    let name = local_ip.intern_cstring(&name).await;
                    if container_decls.contains_key(&name) || scope_decl_map.contains_key(&name) {
                        let name = unsafe { name.as_str() };
                        inner.emit_fatal_error(cu, &format!("duplicate identifier `{name}`"))?
                    }

                    let decl = Decl {
                        parent: Scope::Lazy(scope_id),
                        kind: decl_type,
                        name,
                        is_pub,
                        is_var,
                        lock: RwLock::new(DeclInner {
                            resolving: false,
                            resolved: false,
                            alignment: None,
                            value: Index::VAL_UNDEFINED,
                            annotations: Default::default(),
                        }),
                    };
                    let decl_id = local_ip.intern_decl(decl).await;
                    container_decls.insert(name, decl_id);
                    scope_decls.push(decl_id);
                    scope_decl_map.insert(name, decl_id);
                }
            }
            Hir::ConstBlock(_) => const_blocks.push((inner.src_loc, hir_idx)),
            _ => inner.emit_fatal_error(cu, "statement not allowed in the root of a container")?,
        }
    }

    unsafe {
        let container_inner = container_scope.inner.as_mut_unchecked();
        container_inner.fully_resolved = true;
    }

    for (src_loc, hir_idx) in const_blocks {
        let block = Block {
            scope: inner.scope(),
            hir_info: inner.hir_info(),
            parent_block: None,
            src_loc,
            qualified_name: inner.qualified_name(),

            kind: BlockKind::Const,

            tmp_decl_idx: 0,
            tmp_decls: Default::default(),

            decls: Default::default(),
            inst_map: Default::default(),
            inst_idxs: Default::default(),
        };
        inner.push_block(block);

        let nodes = chunk.block_range_iter(hir_idx);
        match Box::pin(sema_block(cu, sema, inner, chunk, nodes)).await? {
            ControlFlow::None(None) => {}
            ControlFlow::None(_) => unreachable!(),
        }

        inner.pop_block();
    }

    Ok(())
}

async fn sema_namespace<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    chunk: HirChunkView<'a>,
    decl_idx: HirIdx,
) -> Result<(), CompilationError> {
    let ip = cu.pool().get_or_init_local_pool().await;
    let scope = UnorderedDeclScope {
        hir_info: inner.hir_info(),
        decl_loc: inner.src_loc,
        decl_idx,
        parent: inner.scope(),
        ty: Cell::new(None),
        lock: Default::default(),
        inner: UnsafeCell::new(UnorderedDeclScopeInner {
            fully_resolved: false,
            parent_fully_resolved: false,
            captures: Default::default(),
            decls: Default::default(),
            waiters: Default::default(),
        }),
    };
    let scope_id = ip.intern_udscope(scope).await;

    let qualified_name = format!(
        "{}.__namespace_{}\0",
        unsafe { inner.qualified_name().as_str() },
        decl_idx.get()
    );
    let qualified_name = ip.intern_cstring(&qualified_name).await;

    let ns_id = ip
        .intern_type_namespace(KeyTypeNamespace {
            scope: scope_id,
            name: qualified_name,
        })
        .await;
    let scope = scope_id.get_from_pool(cu.pool());
    scope.ty.set(Some(ns_id.into_raw()));

    let block = Block {
        scope: Scope::Type(scope_id),
        hir_info: inner.hir_info(),
        parent_block: None,
        src_loc: inner.src_loc,
        qualified_name,

        kind: BlockKind::Namespace,

        tmp_decl_idx: 0,
        tmp_decls: Default::default(),

        decls: Default::default(),
        inst_map: Default::default(),
        inst_idxs: Default::default(),
    };
    inner.push_block(block);

    let nodes = chunk.block_range_iter(decl_idx);
    sema_container(cu, sema, inner, chunk, nodes).await?;

    inner.pop_block();
    inner.push_immediate(Some(decl_idx), ns_id.into_raw());

    Ok(())
}

async fn sema_init_decl<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    chunk: HirChunkView<'a>,
    hir_idx: HirIdx,
) -> Result<(), CompilationError> {
    let ip = cu.pool();

    let value_idx = match chunk.get_node(hir_idx) {
        Hir::InitDecl(value_idx) => value_idx,
        _ => unreachable!(),
    };

    let value = inner.get_immediate(value_idx);
    if value.is_none() && inner.block_is_const() {
        inner.emit_fatal_error(cu, "expected a constant expression")?
    }

    match value {
        Some(value) => {
            let block_idx = inner.block_idx;
            let block = &mut inner.blocks[block_idx];
            let TmpDecl {
                name,
                ty,
                align,
                annotations,
            } = match block.tmp_decls.get_mut(block.tmp_decl_idx as usize) {
                Some(tmp) => std::mem::take(tmp),
                None => todo!(),
            };

            let value_ty = ip.get_type_of(value);
            if let Some(ty) = ty
                && ty != value_ty
            {
                let ty_name = unsafe { ip.get_type_name(ty).as_str() };
                let value_ty_name = unsafe { ip.get_type_name(value_ty).as_str() };
                inner.emit_fatal_error(
                    cu,
                    &format!("type coersion from `{ty_name}` to `{value_ty_name}` not implemented"),
                )?
            }

            let resolved_name = lookup_name(cu, sema, inner, name, false).await?;
            match resolved_name {
                ResolvedName::Decl(decl_id) => {
                    let decl = decl_id.get_from_pool(ip);
                    let mut decl_inner = decl.lock.write().await;
                    decl_inner.resolved = true;
                    decl_inner.value = value;
                    decl_inner.alignment = align;
                    decl_inner.annotations = annotations;
                }
                ResolvedName::Local(_) => todo!(),
            }

            inner.blocks[block_idx].tmp_decl_idx += 1;
            Ok(())
        }
        None => todo!(),
    }
}

async fn sema_int_literal<'a>(
    cu: &Arc<Cu>,
    _sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    chunk: HirChunkView<'a>,
    hir_idx: HirIdx,
) -> Result<(), CompilationError> {
    let ip = cu.pool();
    let local_ip = ip.get_or_init_local_pool().await;

    let hir_info = inner.hir_info().get_from_pool(ip);
    let ast_info = hir_info.ast_info.get_from_pool(ip);
    let ast = ast_info.id.get_from_pool(ip);

    let literal = match chunk.get_node(hir_idx) {
        Hir::IntLiteral(literal) => literal,
        _ => unreachable!(),
    };
    let literal = ast.get_int_lit(literal).replace("_", "");
    let value = if literal.starts_with("0b") {
        BigInt::parse_bytes(&literal.as_bytes()[2..], 2)
    } else if literal.starts_with("0o") {
        BigInt::parse_bytes(&literal.as_bytes()[2..], 8)
    } else if literal.starts_with("0x") {
        BigInt::parse_bytes(&literal.as_bytes()[2..], 16)
    } else {
        BigInt::parse_bytes(literal.as_bytes(), 10)
    };

    let value = match value {
        Some(v) => v,
        None => {
            inner.emit_fatal_error(cu, &format!("invalid integer literal `{literal}`"))?;
            unreachable!()
        }
    };

    let value = match u64::try_from(&value) {
        Ok(value) => local_ip
            .intern_value_int_u64(KeyInt {
                ty: Index::TY_ANY_INT,
                storage: KeyIntStorage::U64(value),
            })
            .await
            .into_raw(),
        Err(_) => {
            let bytes = value.to_signed_bytes_le();
            local_ip
                .intern_value_int_u64(KeyInt {
                    ty: Index::TY_ANY_INT,
                    storage: KeyIntStorage::BigInt(KeyBigIntStorage { bytes: &bytes }),
                })
                .await
                .into_raw()
        }
    };
    inner.push_immediate(Some(hir_idx), value);

    Ok(())
}

enum ResolvedName {
    Decl(DeclId),
    Local(u32),
}

async fn resolve_lazy_decl_scope<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    scope_id: LDScopeId,
) -> Result<(), CompilationError> {
    let ip = cu.pool();
    let scope = scope_id.get_from_pool(ip);
    if !scope.resolving.swap(true, Ordering::Release) {
        cu.spawn_task({
            let cu = cu.clone();
            async move || {
                let ip = cu.pool();
                let scope = scope_id.get_from_pool(ip);
                let sema = unsafe { scope.sema.as_ref_unchecked() };
                _ = sema.eval_as_lazy_decl_scope(scope_id).await;
            }
        })
        .detach();
    }

    scope.notification.wait(sema, inner).await;
    let scope_sema = unsafe { scope.sema.as_ref_unchecked() };
    let scope_sema_inner = scope_sema.inner.read().await;
    if scope_sema_inner.has_error {
        Err(CompilationError)
    } else {
        Ok(())
    }
}

async fn resolve_decl<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    decl_id: DeclId,
) -> Result<(), CompilationError> {
    let ip = cu.pool();
    let decl = decl_id.get_from_pool(ip);

    {
        let inner = decl.lock.read().await;
        if inner.resolved {
            return Ok(());
        }
    }

    let mut decl_inner = decl.lock.write().await;
    if decl_inner.resolved {
        return Ok(());
    }
    if !decl_inner.resolving {
        decl_inner.resolving = true;
        drop(decl_inner);

        let scope_id = match decl.parent {
            Scope::Lazy(scope_id) => scope_id,
            _ => unreachable!(),
        };
        resolve_lazy_decl_scope(cu, sema, inner, scope_id).await?;
    }

    Ok(())
}

async fn lookup_name<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    name: RawCString,
    resolve: bool,
) -> Result<ResolvedName, CompilationError> {
    let ip = cu.pool();
    let mut block_ids = inner.block_idx;
    loop {
        let block = &inner.blocks[block_ids];
        match block.kind {
            BlockKind::FileRoot => {
                inner.emit_fatal_error(cu, "cannot resolve name at the file root")?
            }
            BlockKind::Namespace => {
                let scope = match inner.scope() {
                    Scope::Type(scope_id) => scope_id.get_from_pool(ip),
                    _ => unreachable!(),
                };

                let _guard = scope.lock.read().await;
                let scope_inner = unsafe { scope.inner.as_ref_unchecked() };
                if let Some(&decl) = scope_inner.decls.get(&name) {
                    drop(_guard);
                    if resolve {
                        resolve_decl(cu, sema, inner, decl).await?;
                    }
                    return Ok(ResolvedName::Decl(decl));
                }
                if let Some(&capture) = scope_inner.captures.get(&name) {
                    drop(_guard);
                    match capture {
                        Capture::Decl(decl) => {
                            if resolve {
                                resolve_decl(cu, sema, inner, decl).await?;
                            }
                            return Ok(ResolvedName::Decl(decl));
                        }
                    }
                }
            }
            BlockKind::LazyDecl => {
                let scope = match inner.scope() {
                    Scope::Lazy(scope_id) => scope_id.get_from_pool(ip),
                    _ => unreachable!(),
                };

                let decl_map = unsafe { scope.decl_map.as_ref_unchecked() };
                if let Some(&decl) = decl_map.get(&name) {
                    if resolve {
                        resolve_decl(cu, sema, inner, decl).await?;
                    }
                    return Ok(ResolvedName::Decl(decl));
                }
            }
            BlockKind::Const => {
                if let Some(decls_idxs) = block.decls.get(&name) {
                    let decls_idx = *decls_idxs.last().unwrap();
                    return Ok(ResolvedName::Local(decls_idx));
                }
            }
        };

        match block.parent_block {
            Some(parent) => block_ids = parent as usize,
            None => break,
        }
    }

    let name = unsafe { name.as_str() };
    inner.emit_fatal_error(cu, &format!("identifier not found `{}`", name))?;
    unreachable!()
}

async fn sema_load_ident<'a>(
    cu: &Arc<Cu>,
    sema: &'a Sema,
    inner: &mut RwLockWriteGuard<'a, SemaInner>,
    chunk: HirChunkView<'a>,
    hir_idx: HirIdx,
) -> Result<(), CompilationError> {
    let ip = cu.pool();
    let local_ip = ip.get_or_init_local_pool().await;

    let hir_info = inner.hir_info().get_from_pool(ip);
    let ast_info = hir_info.ast_info.get_from_pool(ip);
    let ast = ast_info.id.get_from_pool(ip);

    let ident = match chunk.get_node(hir_idx) {
        Hir::LoadIdent(ident) => ident,
        _ => unreachable!(),
    };
    let mut name = ast.get_ident(ident).to_string();
    name.push('\0');
    let name = local_ip.intern_cstring(&name).await;
    let resolved = lookup_name(cu, sema, inner, name, true).await?;
    let value = match resolved {
        ResolvedName::Decl(decl_id) => {
            let decl = decl_id.get_from_pool(ip);
            if inner.block_is_const() {
                match decl.kind {
                    DeclType::Normal => unreachable!(),
                    DeclType::Const => {}
                    DeclType::ThreadLocal => {
                        inner.emit_fatal_error(
                            cu,
                            "`thread_local` declaration is not a constant expression",
                        )?;
                    }
                    DeclType::Static => {
                        if decl.is_var {
                            inner.emit_fatal_error(
                                cu,
                                "`var` declaration is not a constant expression",
                            )?;
                        }
                    }
                }
            }

            let decl_inner = decl.lock.read().await;
            Some(decl_inner.value)
        }
        ResolvedName::Local(_) => todo!(),
    };

    match value {
        Some(value) => {
            inner.push_immediate(Some(hir_idx), value);
        }
        None => todo!(),
    }

    Ok(())
}
