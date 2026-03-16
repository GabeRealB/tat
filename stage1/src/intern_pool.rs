use std::{
    cell::{Cell, UnsafeCell},
    fmt::Debug,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    task::Waker,
    thread::ThreadId,
};

use bitflags::bitflags;
use num_bigint::BigInt;
use rapidhash::{RapidHashMap, fast::RapidHasher};
use smol::lock::{Mutex, RwLock, RwLockWriteGuard};

use crate::{
    ast::{Ast, DeclType},
    hir::{HirChunk, HirIdx, SrcLocation},
    sema::{Sema, SemaInner},
    util::NonMaxU32,
};

pub struct ListIndex<T>(NonMaxU32, PhantomData<fn() -> T>);

impl<T> ListIndex<T> {
    pub const fn new(index: u32) -> Self {
        assert!(index < u32::MAX, "index must be less than u32::MAX");
        unsafe { Self::new_unchecked(index) }
    }

    const unsafe fn new_unchecked(index: u32) -> Self {
        unsafe { Self(NonMaxU32::new_unchecked(index), PhantomData) }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

impl<T> Debug for ListIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ListIndex").field(&self.0).finish()
    }
}

impl<T> Clone for ListIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ListIndex<T> {}

impl<T> PartialEq for ListIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for ListIndex<T> {}

impl<T> PartialOrd for ListIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for ListIndex<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for ListIndex<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct List<T> {
    segments: [AtomicPtr<MaybeUninit<T>>; 31],
}

impl<T> List<T> {
    fn new() -> Self {
        let mut segments = MaybeUninit::<[AtomicPtr<MaybeUninit<T>>; 31]>::uninit();
        let segments = unsafe {
            let ptr = segments.as_mut_ptr() as *mut AtomicPtr<MaybeUninit<T>>;
            for i in 0..31 {
                *ptr.add(i) = AtomicPtr::new(std::ptr::null_mut());
            }
            MaybeUninit::assume_init(segments)
        };
        Self { segments }
    }

    fn write_element(&self, idx: ListIndex<T>, value: T) {
        self.write_element_raw(idx.get(), value);
    }

    fn write_element_raw(&self, idx: u32, value: T) {
        let ptr = self.at_or_uninit(idx);
        unsafe { (*ptr).write(value) };
    }

    fn at_or_uninit(&self, idx: u32) -> *mut MaybeUninit<T> {
        let level = (idx + 1).ilog2();
        let level_idx = idx - ((1 << level) - 1);
        let segment = self.get_or_init_segment(level as usize);
        unsafe { segment.as_mut_ptr().add(level_idx as usize) }
    }

    unsafe fn at_ref(&self, idx: u32) -> &T {
        unsafe { self.at_or_uninit(idx).as_ref_unchecked().assume_init_ref() }
    }

    fn get_or_init_segment(&self, level: usize) -> *mut [MaybeUninit<T>] {
        let capacity = 1usize << level;
        let mut segment = self.segments[level].load(Ordering::Acquire);
        if segment.is_null() {
            let new_alloc = Box::<[T]>::new_uninit_slice(capacity);
            let new_alloc = Box::into_raw(new_alloc);
            segment = match self.segments[level].compare_exchange(
                std::ptr::null_mut(),
                new_alloc.as_mut_ptr(),
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => new_alloc.as_mut_ptr(),
                Err(alloc) => {
                    _ = unsafe { Box::from_raw(new_alloc) };
                    alloc
                }
            };
        }

        std::ptr::slice_from_raw_parts_mut(segment, capacity)
    }

    fn get_segment(&self, level: usize) -> Option<*mut [MaybeUninit<T>]> {
        let segment = self.segments[level].load(Ordering::Acquire);
        if segment.is_null() {
            return None;
        }

        let capacity = 1usize << level;
        let segment = std::ptr::slice_from_raw_parts_mut(segment, capacity);
        Some(segment)
    }
}

unsafe impl<T: Send> Send for List<T> {}
unsafe impl<T: Sync> Sync for List<T> {}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        for lvl in 0..31 {
            let segment = match self.get_segment(lvl) {
                Some(x) => x,
                None => break,
            };
            _ = unsafe { Box::from_raw(segment) }
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AstId(ListIndex<Ast>);

impl AstId {
    pub fn get_from_pool(self, ip: &InternPool) -> &Ast {
        ip.get_ast(self)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HirId(ListIndex<HirChunk>);

impl HirId {
    pub fn get_from_pool(self, ip: &InternPool) -> &HirChunk {
        ip.get_hir_chunk(self)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RScopeId(ListIndex<RootScope>);

impl RScopeId {
    pub fn get_from_pool(self, ip: &InternPool) -> &RootScope {
        ip.get_rscope(self)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UDScopeId(ListIndex<UnorderedDeclScope>);

impl UDScopeId {
    pub fn get_from_pool(self, ip: &InternPool) -> &UnorderedDeclScope {
        ip.get_udscope(self)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LDScopeId(ListIndex<LazyDeclScope>);

impl LDScopeId {
    pub fn get_from_pool(self, ip: &InternPool) -> &LazyDeclScope {
        ip.get_ldscope(self)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclId(ListIndex<Decl>);

impl DeclId {
    pub fn get_from_pool(self, ip: &InternPool) -> &Decl {
        ip.get_decl(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scope {
    Root(RScopeId),
    Type(UDScopeId),
    Lazy(LDScopeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capture {
    Decl(DeclId),
}

#[derive(Debug, Default)]
pub struct ResolveNotification {
    resolved: AtomicBool,
    waiters: RwLock<Vec<Waker>>,
}

impl ResolveNotification {
    pub async fn wait<'a>(&self, sema: &'a Sema, inner: &mut RwLockWriteGuard<'a, SemaInner>) {
        enum State<'a> {
            Start,
            LockWaiters(smol::lock::futures::Write<'a, Vec<Waker>>),
        }

        let mut unlocked = false;
        let mut state = State::Start;
        let poll = |ctx: &mut std::task::Context<'_>| {
            if self.resolved.load(Ordering::Acquire) {
                return std::task::Poll::Ready(());
            }

            loop {
                match &mut state {
                    State::Start => {
                        let lock = self.waiters.write();
                        state = State::LockWaiters(lock);
                        continue;
                    }
                    State::LockWaiters(fut) => {
                        let fut = unsafe { std::pin::Pin::new_unchecked(fut) };
                        match fut.poll(ctx) {
                            std::task::Poll::Ready(mut guard) => {
                                if self.resolved.load(Ordering::Acquire) {
                                    break;
                                }

                                if !unlocked {
                                    let inner_ptr = &raw mut *inner;
                                    unsafe { std::ptr::drop_in_place(inner_ptr) };
                                    unlocked = true;
                                }

                                guard.push(ctx.waker().clone());
                                state = State::Start;
                                return std::task::Poll::Pending;
                            }
                            std::task::Poll::Pending => return std::task::Poll::Pending,
                        }
                    }
                };
            }

            std::task::Poll::Ready(())
        };
        let fut = std::future::poll_fn(poll);
        fut.await;

        if unlocked {
            let inner_ptr = &raw mut *inner;
            let guard = sema.inner.write().await;
            unsafe { inner_ptr.write(guard) };
        }
    }

    pub async fn notify(&self) {
        self.resolved.store(true, Ordering::Release);
        let waiters = {
            let mut inner = self.waiters.write().await;
            std::mem::take(&mut *inner)
        };
        for waiter in waiters {
            waiter.wake();
        }
    }
}

#[derive(Debug)]
pub struct RootScope {
    pub hir_info: TypedIndex<HirInfo>,
    pub lock: RwLock<()>,
    pub inner: UnsafeCell<RootScopeInner>,
}

#[derive(Debug)]
pub struct RootScopeInner {
    pub resolving: bool,
    pub resolved: bool,
    pub result: Option<Index>,
    pub waiters: Vec<Waker>,
    pub sema: UnsafeCell<Sema>,
}

#[derive(Debug)]
pub struct UnorderedDeclScope {
    pub hir_info: TypedIndex<HirInfo>,
    pub decl_loc: SrcLocation,
    pub decl_idx: HirIdx,
    pub parent: Scope,
    pub ty: Cell<Option<Index>>,
    pub lock: RwLock<()>,
    pub inner: UnsafeCell<UnorderedDeclScopeInner>,
}

#[derive(Debug)]
pub struct UnorderedDeclScopeInner {
    pub fully_resolved: bool,
    pub parent_fully_resolved: bool,
    pub captures: RapidHashMap<RawCString, Capture>,
    pub decls: RapidHashMap<RawCString, DeclId>,
    pub waiters: Vec<Waker>,
}

#[derive(Debug)]
pub struct LazyDeclScope {
    pub hir_info: TypedIndex<HirInfo>,
    pub decl_loc: SrcLocation,
    pub decl_idx: HirIdx,
    pub parent: UDScopeId,
    pub decls: UnsafeCell<Vec<DeclId>>,
    pub decl_map: UnsafeCell<RapidHashMap<RawCString, DeclId>>,
    pub notification: ResolveNotification,
    pub resolving: AtomicBool,
    pub sema: UnsafeCell<Sema>,
}

#[derive(Debug)]
pub struct Decl {
    pub parent: Scope,
    pub kind: DeclType,
    pub name: RawCString,
    pub is_pub: bool,
    pub is_var: bool,
    pub lock: RwLock<DeclInner>,
}

#[derive(Debug)]
pub struct DeclInner {
    pub resolving: bool,
    pub resolved: bool,
    pub value: Index,
    pub alignment: Option<Index>,
    pub annotations: Vec<Index>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Module {
    pub root_directory_path: RawCString,
    pub root_file_path: RawCString,
    pub name: RawCString,
    pub is_core: bool,
}

impl Interned for Module {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_module(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct File {
    pub file_path: RawCString,
    pub qualified_name: RawCString,
    pub module: TypedIndex<Module>,
}

impl Interned for File {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_file(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstInfo {
    pub file: TypedIndex<File>,
    pub id: AstId,
}

impl Interned for AstInfo {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_ast_info(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirInfo {
    pub ast_info: TypedIndex<AstInfo>,
    pub id: HirId,
}

impl Interned for HirInfo {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_hir_info(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawString {
    pub ptr: NonNull<[u8]>,
}

#[derive(Debug, Clone, Copy)]
pub struct RawCString {
    pub ptr: NonNull<[u8]>,
}

impl RawCString {
    pub unsafe fn as_str<'a>(self) -> &'a str {
        unsafe {
            let str = self.as_str_with_null();
            &str[..str.len() - 1]
        }
    }

    pub unsafe fn as_str_with_null<'a>(self) -> &'a str {
        unsafe {
            let bytes = self.ptr.as_ref();
            str::from_utf8_unchecked(bytes)
        }
    }
}

impl PartialEq for RawCString {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.as_str().eq(other.as_str()) }
    }
}

impl Eq for RawCString {}

impl PartialOrd for RawCString {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RawCString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe { self.as_str().cmp(other.as_str()) }
    }
}

impl Hash for RawCString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.as_str_with_null().hash(state) }
    }
}

bitflags! {
    /// Additional flags for a type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TypeFlags: u16 {
        const Const = 1 << 0;
        const PtrOnly = 1 << 1;
        const Generic = 1 << 2;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeSimple {
    pub id: u32,
    pub name: RawCString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeAnyOpaque {
    pub id: u32,
    pub name: RawCString,
    pub flags: TypeFlags,
}

impl Interned for TypeAnyOpaque {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_any_opaque(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeBool {
    pub id: u32,
    pub name: RawCString,
    pub width: u16,
}

impl Interned for TypeBool {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_bool(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeNaturalInt {
    pub id: u32,
    pub name: RawCString,
    pub is_signed: bool,
}

impl Interned for TypeNaturalInt {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_natural_int(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypePointerInt {
    pub id: u32,
    pub name: RawCString,
    pub is_signed: bool,
}

impl Interned for TypePointerInt {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_pointer_int(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeFixedWidthInt {
    pub id: u32,
    pub name: RawCString,
    pub width: u16,
    pub is_signed: bool,
    pub is_little_endian: Option<bool>,
}

impl Interned for TypeFixedWidthInt {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_fixed_width_int(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeFloat {
    pub id: u32,
    pub name: RawCString,
    pub width: u16,
    pub is_little_endian: Option<bool>,
}

impl Interned for TypeFloat {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_float(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeComplex {
    pub id: u32,
    pub name: RawCString,
    pub width: u16,
}

impl Interned for TypeComplex {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_complex(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeQuat {
    pub id: u32,
    pub name: RawCString,
    pub width: u16,
}

impl Interned for TypeQuat {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_quat(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeDQuat {
    pub id: u32,
    pub name: RawCString,
    pub width: u16,
}

impl Interned for TypeDQuat {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_dquat(idx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNamespace {
    pub id: u32,
    pub name: RawCString,
    pub scope: UDScopeId,
}

impl Interned for TypeNamespace {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_type_namespace(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueBool {
    pub ty: Index,
    pub value: bool,
}

impl Interned for ValueBool {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_value_bool(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueChar {
    pub value: char,
}

impl Interned for ValueChar {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_value_char(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueIntU64 {
    pub ty: Index,
    pub value: u64,
}

impl ValueIntU64 {
    pub fn as_bigint(&self) -> BigInt {
        self.value.into()
    }
}

impl Interned for ValueIntU64 {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_value_int_u64(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueIntI64 {
    pub ty: Index,
    pub value: i64,
}

impl ValueIntI64 {
    pub fn as_bigint(&self) -> BigInt {
        self.value.into()
    }
}

impl Interned for ValueIntI64 {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_value_int_i64(idx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueIntLarge {
    pub ty: Index,
    pub value: NonNull<[u8]>,
}

impl ValueIntLarge {
    pub unsafe fn as_bigint(&self) -> BigInt {
        unsafe { BigInt::from_signed_bytes_le(self.value.as_ref()) }
    }
}

impl Interned for ValueIntLarge {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self {
        ip.get_value_int_large(idx)
    }
}

const PREDEFINED_INDICES: &[(Key, Index)] = &[
    // Types
    (Key::TypeBool(KeyTypeBool { width: 1 }), Index::TY_BOOL),
    (Key::TypeBool(KeyTypeBool { width: 8 }), Index::TY_B8),
    (Key::TypeBool(KeyTypeBool { width: 16 }), Index::TY_B16),
    (Key::TypeBool(KeyTypeBool { width: 32 }), Index::TY_B32),
    (Key::TypeBool(KeyTypeBool { width: 64 }), Index::TY_B64),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 1,
            is_signed: true,
            is_little_endian: None,
        }),
        Index::TY_I1,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 8,
            is_signed: true,
            is_little_endian: None,
        }),
        Index::TY_I8,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 16,
            is_signed: true,
            is_little_endian: None,
        }),
        Index::TY_I16,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 32,
            is_signed: true,
            is_little_endian: None,
        }),
        Index::TY_I32,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 64,
            is_signed: true,
            is_little_endian: None,
        }),
        Index::TY_I64,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 128,
            is_signed: true,
            is_little_endian: None,
        }),
        Index::TY_I128,
    ),
    (
        Key::TypeNaturalInt(KeyTypeNaturalInt { is_signed: true }),
        Index::TY_INT,
    ),
    (
        Key::TypePointerInt(KeyTypePointerInt { is_signed: true }),
        Index::TY_INTPTR,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 0,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U0,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 1,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U1,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 8,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U8,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 16,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U16,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 32,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U32,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 64,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U64,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 128,
            is_signed: false,
            is_little_endian: None,
        }),
        Index::TY_U128,
    ),
    (
        Key::TypeNaturalInt(KeyTypeNaturalInt { is_signed: false }),
        Index::TY_UINT,
    ),
    (
        Key::TypePointerInt(KeyTypePointerInt { is_signed: false }),
        Index::TY_UINTPTR,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 1,
            is_signed: true,
            is_little_endian: Some(true),
        }),
        Index::TY_I1LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 8,
            is_signed: true,
            is_little_endian: Some(true),
        }),
        Index::TY_I8LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 16,
            is_signed: true,
            is_little_endian: Some(true),
        }),
        Index::TY_I16LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 32,
            is_signed: true,
            is_little_endian: Some(true),
        }),
        Index::TY_I32LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 64,
            is_signed: true,
            is_little_endian: Some(true),
        }),
        Index::TY_I64LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 128,
            is_signed: true,
            is_little_endian: Some(true),
        }),
        Index::TY_I128LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 1,
            is_signed: true,
            is_little_endian: Some(false),
        }),
        Index::TY_I1BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 8,
            is_signed: true,
            is_little_endian: Some(false),
        }),
        Index::TY_I8BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 16,
            is_signed: true,
            is_little_endian: Some(false),
        }),
        Index::TY_I16BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 32,
            is_signed: true,
            is_little_endian: Some(false),
        }),
        Index::TY_I32BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 64,
            is_signed: true,
            is_little_endian: Some(false),
        }),
        Index::TY_I64BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 128,
            is_signed: true,
            is_little_endian: Some(false),
        }),
        Index::TY_I128BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 0,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U0LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 1,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U1LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 8,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U8LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 16,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U16LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 32,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U32LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 64,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U64LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 128,
            is_signed: false,
            is_little_endian: Some(true),
        }),
        Index::TY_U128LE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 0,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U0BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 1,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U1BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 8,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U8BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 16,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U16BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 32,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U32BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 64,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U64BE,
    ),
    (
        Key::TypeFixedWidthInt(KeyTypeFixedWidthInt {
            width: 128,
            is_signed: false,
            is_little_endian: Some(false),
        }),
        Index::TY_U128BE,
    ),
    (
        Key::TypeFloat(KeyTypeFloat {
            width: 32,
            is_little_endian: None,
        }),
        Index::TY_F32,
    ),
    (
        Key::TypeFloat(KeyTypeFloat {
            width: 64,
            is_little_endian: None,
        }),
        Index::TY_F64,
    ),
    (
        Key::TypeFloat(KeyTypeFloat {
            width: 32,
            is_little_endian: Some(true),
        }),
        Index::TY_F32LE,
    ),
    (
        Key::TypeFloat(KeyTypeFloat {
            width: 64,
            is_little_endian: Some(true),
        }),
        Index::TY_F64LE,
    ),
    (
        Key::TypeFloat(KeyTypeFloat {
            width: 32,
            is_little_endian: Some(false),
        }),
        Index::TY_F32BE,
    ),
    (
        Key::TypeFloat(KeyTypeFloat {
            width: 64,
            is_little_endian: Some(false),
        }),
        Index::TY_F64BE,
    ),
    (
        Key::TypeComplex(KeyTypeComplex { width: 32 }),
        Index::TY_COMPLEX32,
    ),
    (
        Key::TypeComplex(KeyTypeComplex { width: 64 }),
        Index::TY_COMPLEX64,
    ),
    (Key::TypeQuat(KeyTypeQuat { width: 32 }), Index::TY_QUAT32),
    (Key::TypeQuat(KeyTypeQuat { width: 64 }), Index::TY_QUAT64),
    (
        Key::TypeDQuat(KeyTypeDQuat { width: 32 }),
        Index::TY_DQUAT32,
    ),
    (
        Key::TypeDQuat(KeyTypeDQuat { width: 64 }),
        Index::TY_DQUAT64,
    ),
    (Key::TypeChar, Index::TY_CHAR),
    (Key::TypeString, Index::TY_STRING),
    (Key::TypeCString, Index::TY_CSTRING),
    (Key::TypeRawptr, Index::TY_RAWPTR),
    (Key::TypeVoid, Index::TY_VOID),
    (Key::TypeAnyInt, Index::TY_ANY_INT),
    (Key::TypeAnyFloat, Index::TY_ANY_FLOAT),
    (
        Key::TypeAnyOpaque(KeyTypeAnyOpaque { metadata: None }),
        Index::TY_ANY_OPAQUE,
    ),
    (Key::TypeAnyType, Index::TY_ANY_TYPE),
    (Key::TypeNull, Index::TY_NULL),
    (Key::TypeType, Index::TY_TYPE),
    (Key::TypeTypeId, Index::TY_TYPE_ID),
    (Key::TypeUndefined, Index::TY_UNDEFINED),
    (Key::TypeNoReturn, Index::TY_NO_RETURN),
    // Values
    (Key::NoReturn, Index::VAL_NO_RETURN),
    (Key::Null, Index::VAL_NULL),
    (Key::Void, Index::VAL_VOID),
    (Key::Undefined, Index::VAL_UNDEFINED),
    (
        Key::Bool(KeyBool {
            ty: Index::TY_BOOL,
            value: true,
        }),
        Index::VAL_TRUE,
    ),
    (
        Key::Bool(KeyBool {
            ty: Index::TY_BOOL,
            value: false,
        }),
        Index::VAL_FALSE,
    ),
];

pub struct InternPool {
    pools: Mutex<RapidHashMap<ThreadId, *mut LocalPoolInner>>,

    key_shards: Box<[Mutex<RapidHashMap<Key<'static>, Index>>]>,
    #[allow(clippy::type_complexity)]
    bytes_shards: Box<[Mutex<RapidHashMap<Box<[u8]>, ListIndex<NonNull<[u8]>>>>]>,

    tags: List<KeyTag>,
    sublist_idx: List<u32>,
    values_count: AtomicU32,

    asts: List<Ast>,
    asts_count: AtomicU32,
    hir_chunks: List<HirChunk>,
    hir_chunks_count: AtomicU32,
    rscopes: List<RootScope>,
    rscopes_count: AtomicU32,
    udscopes: List<UnorderedDeclScope>,
    udscopes_count: AtomicU32,
    ldscopes: List<LazyDeclScope>,
    ldscopes_count: AtomicU32,
    decls: List<Decl>,
    decls_count: AtomicU32,

    modules: List<Module>,
    modules_count: AtomicU32,
    files: List<File>,
    files_count: AtomicU32,
    ast_infos: List<AstInfo>,
    ast_infos_count: AtomicU32,
    hir_infos: List<HirInfo>,
    hir_infos_count: AtomicU32,

    types_count: AtomicU32,
    any_int_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    any_float_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    any_opaque_types: List<TypeAnyOpaque>,
    any_opaque_types_count: AtomicU32,
    any_type_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    bool_types: List<TypeBool>,
    bool_types_count: AtomicU32,
    char_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    nat_int_types: List<TypeNaturalInt>,
    nat_int_types_count: AtomicU32,
    pointer_int_types: List<TypePointerInt>,
    pointer_int_types_count: AtomicU32,
    fixed_width_int_types: List<TypeFixedWidthInt>,
    fixed_width_int_types_count: AtomicU32,
    float_types: List<TypeFloat>,
    float_types_count: AtomicU32,
    complex_types: List<TypeComplex>,
    complex_types_count: AtomicU32,
    quat_types: List<TypeQuat>,
    quat_types_count: AtomicU32,
    dquat_types: List<TypeDQuat>,
    dquat_types_count: AtomicU32,
    no_return_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    null_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    rawptr_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    string_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    cstring_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    void_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    type_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    type_id_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    undefined_type: UnsafeCell<MaybeUninit<TypeSimple>>,
    namespace_types: List<TypeNamespace>,
    namespace_types_count: AtomicU32,

    bytes: List<NonNull<[u8]>>,
    bytes_count: AtomicU32,

    bools: List<ValueBool>,
    bools_count: AtomicU32,
    chars: List<ValueChar>,
    chars_count: AtomicU32,
    ints_u64: List<ValueIntU64>,
    ints_u64_count: AtomicU32,
    ints_i64: List<ValueIntI64>,
    ints_i64_count: AtomicU32,
    ints_large: List<ValueIntLarge>,
    ints_large_count: AtomicU32,
    // float32
    // float64
    no_return: (),
    null: (),
    // optional
    void: (),
    // type_id
    undefined: (),
    // typed_undefined
    // aggregates
    // union
}

impl InternPool {
    pub fn new() -> Self {
        let num_cores = std::thread::available_parallelism()
            .map(|cnt| cnt.get())
            .unwrap_or(16);
        let num_shards = num_cores.next_power_of_two();

        let pools = Mutex::new(Default::default());
        let key_shards = (0..num_shards)
            .map(|_| Mutex::new(Default::default()))
            .collect();
        let bytes_shards = (0..num_shards)
            .map(|_| Mutex::new(Default::default()))
            .collect();
        Self {
            pools,
            key_shards,
            bytes_shards,

            tags: Default::default(),
            sublist_idx: Default::default(),
            values_count: Default::default(),

            asts: Default::default(),
            asts_count: Default::default(),
            hir_chunks: Default::default(),
            hir_chunks_count: Default::default(),
            rscopes: Default::default(),
            rscopes_count: Default::default(),
            udscopes: Default::default(),
            udscopes_count: Default::default(),
            ldscopes: Default::default(),
            ldscopes_count: Default::default(),
            decls: Default::default(),
            decls_count: Default::default(),

            modules: Default::default(),
            modules_count: Default::default(),
            files: Default::default(),
            files_count: Default::default(),
            ast_infos: Default::default(),
            ast_infos_count: Default::default(),
            hir_infos: Default::default(),
            hir_infos_count: Default::default(),

            types_count: Default::default(),
            any_int_type: UnsafeCell::new(MaybeUninit::uninit()),
            any_float_type: UnsafeCell::new(MaybeUninit::uninit()),
            any_opaque_types: Default::default(),
            any_opaque_types_count: Default::default(),
            any_type_type: UnsafeCell::new(MaybeUninit::uninit()),
            bool_types: Default::default(),
            bool_types_count: Default::default(),
            char_type: UnsafeCell::new(MaybeUninit::uninit()),
            nat_int_types: Default::default(),
            nat_int_types_count: Default::default(),
            pointer_int_types: Default::default(),
            pointer_int_types_count: Default::default(),
            fixed_width_int_types: Default::default(),
            fixed_width_int_types_count: Default::default(),
            float_types: Default::default(),
            float_types_count: Default::default(),
            complex_types: Default::default(),
            complex_types_count: Default::default(),
            quat_types: Default::default(),
            quat_types_count: Default::default(),
            dquat_types: Default::default(),
            dquat_types_count: Default::default(),
            no_return_type: UnsafeCell::new(MaybeUninit::uninit()),
            null_type: UnsafeCell::new(MaybeUninit::uninit()),
            rawptr_type: UnsafeCell::new(MaybeUninit::uninit()),
            string_type: UnsafeCell::new(MaybeUninit::uninit()),
            cstring_type: UnsafeCell::new(MaybeUninit::uninit()),
            void_type: UnsafeCell::new(MaybeUninit::uninit()),
            type_type: UnsafeCell::new(MaybeUninit::uninit()),
            type_id_type: UnsafeCell::new(MaybeUninit::uninit()),
            undefined_type: UnsafeCell::new(MaybeUninit::uninit()),
            namespace_types: Default::default(),
            namespace_types_count: Default::default(),

            bytes: Default::default(),
            bytes_count: Default::default(),

            bools: Default::default(),
            bools_count: Default::default(),
            chars: Default::default(),
            chars_count: Default::default(),
            ints_u64: Default::default(),
            ints_u64_count: Default::default(),
            ints_i64: Default::default(),
            ints_i64_count: Default::default(),
            ints_large: Default::default(),
            ints_large_count: Default::default(),
            // float32
            // float64
            no_return: (),
            null: (),
            // optional
            void: (),
            // type_id
            undefined: (),
            // typed_undefined
            // aggregates
            // union
        }
    }

    pub async fn init_pool(&self) {
        assert!(self.values_count.load(Ordering::Relaxed) == 0);
        let local = self.get_or_init_local_pool().await;
        for &(k, idx) in PREDEFINED_INDICES {
            assert_eq!(local.intern_value(k).await, idx)
        }
    }

    pub async fn get_or_init_local_pool(&self) -> LocalPool {
        let thread_id = std::thread::current().id();
        let mut pools = self.pools.lock().await;
        match pools.get(&thread_id) {
            Some(&pool) => LocalPool { inner: pool },
            None => {
                let pool = Box::new(LocalPoolInner::new(self));
                let pool = Box::into_raw(pool);
                pools.insert(thread_id, pool);
                LocalPool { inner: pool }
            }
        }
    }

    pub fn get_tag(&self, idx: Index) -> KeyTag {
        unsafe { (*self.tags.at_or_uninit(idx.get())).assume_init() }
    }

    pub fn get_ast(&self, id: AstId) -> &Ast {
        unsafe {
            let idx = id.0;
            self.asts.at_ref(idx.get())
        }
    }

    pub fn get_hir_chunk(&self, id: HirId) -> &HirChunk {
        unsafe {
            let idx = id.0;
            self.hir_chunks.at_ref(idx.get())
        }
    }

    pub fn get_rscope(&self, id: RScopeId) -> &RootScope {
        unsafe {
            let idx = id.0;
            self.rscopes.at_ref(idx.get())
        }
    }

    pub fn get_udscope(&self, id: UDScopeId) -> &UnorderedDeclScope {
        unsafe {
            let idx = id.0;
            self.udscopes.at_ref(idx.get())
        }
    }

    pub fn get_ldscope(&self, id: LDScopeId) -> &LazyDeclScope {
        unsafe {
            let idx = id.0;
            self.ldscopes.at_ref(idx.get())
        }
    }

    pub fn get_decl(&self, id: DeclId) -> &Decl {
        unsafe {
            let idx = id.0;
            self.decls.at_ref(idx.get())
        }
    }

    pub fn get_module(&self, idx: Index) -> &Module {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::Module);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.modules.at_ref(list_idx)
        }
    }

    pub fn get_file(&self, idx: Index) -> &File {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::File);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.files.at_ref(list_idx)
        }
    }

    pub fn get_ast_info(&self, idx: Index) -> &AstInfo {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::AstInfo);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.ast_infos.at_ref(list_idx)
        }
    }

    pub fn get_hir_info(&self, idx: Index) -> &HirInfo {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::HirInfo);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.hir_infos.at_ref(list_idx)
        }
    }

    pub fn get_type_any_int(&self) -> &TypeSimple {
        unsafe { self.any_int_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_any_float(&self) -> &TypeSimple {
        unsafe {
            self.any_float_type
                .get()
                .as_ref_unchecked()
                .assume_init_ref()
        }
    }

    pub fn get_type_any_opaque(&self, idx: Index) -> &TypeAnyOpaque {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeAnyOpaque);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.any_opaque_types.at_ref(list_idx)
        }
    }

    pub fn get_type_any_type(&self) -> &TypeSimple {
        unsafe {
            self.any_type_type
                .get()
                .as_ref_unchecked()
                .assume_init_ref()
        }
    }

    pub fn get_type_bool(&self, idx: Index) -> &TypeBool {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeBool);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.bool_types.at_ref(list_idx)
        }
    }

    pub fn get_type_char(&self) -> &TypeSimple {
        unsafe { self.char_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_natural_int(&self, idx: Index) -> &TypeNaturalInt {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeNaturalInt);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.nat_int_types.at_ref(list_idx)
        }
    }

    pub fn get_type_pointer_int(&self, idx: Index) -> &TypePointerInt {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypePointerInt);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.pointer_int_types.at_ref(list_idx)
        }
    }

    pub fn get_type_fixed_width_int(&self, idx: Index) -> &TypeFixedWidthInt {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeFixedWidthInt);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.fixed_width_int_types.at_ref(list_idx)
        }
    }

    pub fn get_type_float(&self, idx: Index) -> &TypeFloat {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeFloat);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.float_types.at_ref(list_idx)
        }
    }

    pub fn get_type_complex(&self, idx: Index) -> &TypeComplex {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeComplex);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.complex_types.at_ref(list_idx)
        }
    }

    pub fn get_type_quat(&self, idx: Index) -> &TypeQuat {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeQuat);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.quat_types.at_ref(list_idx)
        }
    }

    pub fn get_type_dquat(&self, idx: Index) -> &TypeDQuat {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeDQuat);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.dquat_types.at_ref(list_idx)
        }
    }

    pub fn get_type_namespace(&self, idx: Index) -> &TypeNamespace {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::TypeNamespace);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.namespace_types.at_ref(list_idx)
        }
    }

    pub fn get_type_no_return(&self) -> &TypeSimple {
        unsafe {
            self.no_return_type
                .get()
                .as_ref_unchecked()
                .assume_init_ref()
        }
    }

    pub fn get_type_null(&self) -> &TypeSimple {
        unsafe { self.null_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_rawptr(&self) -> &TypeSimple {
        unsafe { self.rawptr_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_string(&self) -> &TypeSimple {
        unsafe { self.string_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_cstring(&self) -> &TypeSimple {
        unsafe { self.cstring_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_void(&self) -> &TypeSimple {
        unsafe { self.void_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_type(&self) -> &TypeSimple {
        unsafe { self.type_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_type_id(&self) -> &TypeSimple {
        unsafe { self.type_id_type.get().as_ref_unchecked().assume_init_ref() }
    }

    pub fn get_type_undefined(&self) -> &TypeSimple {
        unsafe {
            self.undefined_type
                .get()
                .as_ref_unchecked()
                .assume_init_ref()
        }
    }

    pub fn get_value_bool(&self, idx: Index) -> &ValueBool {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::Bool);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.bools.at_ref(list_idx)
        }
    }

    pub fn get_value_char(&self, idx: Index) -> &ValueChar {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::Char);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.chars.at_ref(list_idx)
        }
    }

    pub fn get_value_int_u64(&self, idx: Index) -> &ValueIntU64 {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::IntU64);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.ints_u64.at_ref(list_idx)
        }
    }

    pub fn get_value_int_i64(&self, idx: Index) -> &ValueIntI64 {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::IntI64);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.ints_i64.at_ref(list_idx)
        }
    }

    pub fn get_value_int_large(&self, idx: Index) -> &ValueIntLarge {
        unsafe {
            debug_assert_eq!(self.tags.at_ref(idx.get()), &KeyTag::IntLarge);
            let list_idx = *self.sublist_idx.at_ref(idx.get());
            self.ints_large.at_ref(list_idx)
        }
    }

    pub fn is_type(&self, idx: Index) -> bool {
        let tag = self.get_tag(idx);
        matches!(
            tag,
            KeyTag::TypeAnyInt
                | KeyTag::TypeAnyFloat
                | KeyTag::TypeAnyOpaque
                | KeyTag::TypeAnyType
                | KeyTag::TypeBool
                | KeyTag::TypeChar
                | KeyTag::TypeNaturalInt
                | KeyTag::TypePointerInt
                | KeyTag::TypeFixedWidthInt
                | KeyTag::TypeFloat
                | KeyTag::TypeComplex
                | KeyTag::TypeQuat
                | KeyTag::TypeDQuat
                | KeyTag::TypeNamespace
                | KeyTag::TypeNoReturn
                | KeyTag::TypeNull
                | KeyTag::TypeSinglePointer
                | KeyTag::TypeMultiPointer
                | KeyTag::TypeRawptr
                | KeyTag::TypeSlice
                | KeyTag::TypeString
                | KeyTag::TypeCString
                | KeyTag::TypeVoid
                | KeyTag::TypeVector
                | KeyTag::TypeMatrix
                | KeyTag::TypeType
                | KeyTag::TypeTypeId
                | KeyTag::TypeUndefined
        )
    }

    pub fn type_is_ptr_only(&self, idx: Index) -> bool {
        let tag = self.get_tag(idx);
        match tag {
            KeyTag::TypeAnyInt => false,
            KeyTag::TypeAnyFloat => false,
            KeyTag::TypeAnyOpaque => {
                let ty = self.get_type_any_opaque(idx);
                ty.flags.contains(TypeFlags::PtrOnly)
            }
            KeyTag::TypeAnyType => false,
            KeyTag::TypeBool => false,
            KeyTag::TypeChar => false,
            KeyTag::TypeNaturalInt => false,
            KeyTag::TypePointerInt => false,
            KeyTag::TypeFixedWidthInt => false,
            KeyTag::TypeFloat => false,
            KeyTag::TypeComplex => false,
            KeyTag::TypeQuat => false,
            KeyTag::TypeDQuat => false,
            KeyTag::TypeNamespace => false,
            KeyTag::TypeNoReturn => false,
            KeyTag::TypeNull => false,
            KeyTag::TypeSinglePointer => false,
            KeyTag::TypeMultiPointer => false,
            KeyTag::TypeRawptr => false,
            KeyTag::TypeSlice => true,
            KeyTag::TypeString => true,
            KeyTag::TypeCString => true,
            KeyTag::TypeVoid => false,
            KeyTag::TypeVector => false,
            KeyTag::TypeMatrix => false,
            KeyTag::TypeType => false,
            KeyTag::TypeTypeId => false,
            KeyTag::TypeUndefined => false,
            _ => unreachable!(),
        }
    }

    pub fn type_is_generic(&self, idx: Index) -> bool {
        let tag = self.get_tag(idx);
        match tag {
            KeyTag::TypeAnyInt => false,
            KeyTag::TypeAnyFloat => false,
            KeyTag::TypeAnyOpaque => {
                let ty = self.get_type_any_opaque(idx);
                ty.flags.contains(TypeFlags::Generic)
            }
            KeyTag::TypeAnyType => true,
            KeyTag::TypeBool => false,
            KeyTag::TypeChar => false,
            KeyTag::TypeNaturalInt => false,
            KeyTag::TypePointerInt => false,
            KeyTag::TypeFixedWidthInt => false,
            KeyTag::TypeFloat => false,
            KeyTag::TypeComplex => false,
            KeyTag::TypeQuat => false,
            KeyTag::TypeDQuat => false,
            KeyTag::TypeNamespace => false,
            KeyTag::TypeNoReturn => false,
            KeyTag::TypeNull => false,
            KeyTag::TypeSinglePointer => todo!(),
            KeyTag::TypeMultiPointer => todo!(),
            KeyTag::TypeRawptr => false,
            KeyTag::TypeSlice => todo!(),
            KeyTag::TypeString => false,
            KeyTag::TypeCString => false,
            KeyTag::TypeVoid => false,
            KeyTag::TypeVector => todo!(),
            KeyTag::TypeMatrix => todo!(),
            KeyTag::TypeType => false,
            KeyTag::TypeTypeId => false,
            KeyTag::TypeUndefined => false,
            _ => unreachable!(),
        }
    }

    pub fn type_is_const(&self, idx: Index) -> bool {
        let tag = self.get_tag(idx);
        match tag {
            KeyTag::TypeAnyInt => true,
            KeyTag::TypeAnyFloat => true,
            KeyTag::TypeAnyOpaque => {
                let ty = self.get_type_any_opaque(idx);
                ty.flags.contains(TypeFlags::Const)
            }
            KeyTag::TypeAnyType => true,
            KeyTag::TypeBool => false,
            KeyTag::TypeChar => false,
            KeyTag::TypeNaturalInt => false,
            KeyTag::TypePointerInt => false,
            KeyTag::TypeFixedWidthInt => false,
            KeyTag::TypeFloat => false,
            KeyTag::TypeComplex => false,
            KeyTag::TypeQuat => false,
            KeyTag::TypeDQuat => false,
            KeyTag::TypeNamespace => true,
            KeyTag::TypeNoReturn => true,
            KeyTag::TypeNull => true,
            KeyTag::TypeSinglePointer => todo!(),
            KeyTag::TypeMultiPointer => todo!(),
            KeyTag::TypeRawptr => false,
            KeyTag::TypeSlice => todo!(),
            KeyTag::TypeString => false,
            KeyTag::TypeCString => false,
            KeyTag::TypeVoid => false,
            KeyTag::TypeVector => todo!(),
            KeyTag::TypeMatrix => todo!(),
            KeyTag::TypeType => true,
            KeyTag::TypeTypeId => false,
            KeyTag::TypeUndefined => true,
            _ => unreachable!(),
        }
    }

    pub fn get_type_of(&self, idx: Index) -> Index {
        let tag = self.get_tag(idx);
        match tag {
            KeyTag::Module | KeyTag::File | KeyTag::AstInfo | KeyTag::HirInfo => unreachable!(),
            KeyTag::TypeAnyInt
            | KeyTag::TypeAnyFloat
            | KeyTag::TypeAnyOpaque
            | KeyTag::TypeAnyType
            | KeyTag::TypeBool
            | KeyTag::TypeChar
            | KeyTag::TypeNaturalInt
            | KeyTag::TypePointerInt
            | KeyTag::TypeFixedWidthInt
            | KeyTag::TypeFloat
            | KeyTag::TypeComplex
            | KeyTag::TypeQuat
            | KeyTag::TypeDQuat
            | KeyTag::TypeNamespace
            | KeyTag::TypeNoReturn
            | KeyTag::TypeNull
            | KeyTag::TypeSinglePointer
            | KeyTag::TypeMultiPointer
            | KeyTag::TypeRawptr
            | KeyTag::TypeSlice
            | KeyTag::TypeString
            | KeyTag::TypeCString
            | KeyTag::TypeVoid
            | KeyTag::TypeVector
            | KeyTag::TypeMatrix
            | KeyTag::TypeType
            | KeyTag::TypeTypeId
            | KeyTag::TypeUndefined => Index::TY_TYPE,
            KeyTag::Bool => self.get_value_bool(idx).ty,
            KeyTag::Char => Index::TY_CHAR,
            KeyTag::IntU64 => self.get_value_int_u64(idx).ty,
            KeyTag::IntI64 => self.get_value_int_i64(idx).ty,
            KeyTag::IntLarge => self.get_value_int_large(idx).ty,
            KeyTag::Float => todo!(),
            KeyTag::NoReturn => Index::TY_NO_RETURN,
            KeyTag::Null => Index::TY_NULL,
            KeyTag::Optional => todo!(),
            KeyTag::Void => Index::TY_VOID,
            KeyTag::TypeId => Index::TY_TYPE_ID,
            KeyTag::Undefined => Index::TY_UNDEFINED,
            KeyTag::TypedUndefined => todo!(),
            KeyTag::Aggregate => todo!(),
            KeyTag::Union => todo!(),
        }
    }

    pub fn get_type_name(&self, idx: Index) -> RawCString {
        let tag = self.get_tag(idx);
        match tag {
            KeyTag::TypeAnyInt => self.get_type_any_int().name,
            KeyTag::TypeAnyFloat => self.get_type_any_float().name,
            KeyTag::TypeAnyOpaque => self.get_type_any_opaque(idx).name,
            KeyTag::TypeAnyType => self.get_type_any_type().name,
            KeyTag::TypeBool => self.get_type_bool(idx).name,
            KeyTag::TypeChar => self.get_type_char().name,
            KeyTag::TypeNaturalInt => self.get_type_natural_int(idx).name,
            KeyTag::TypePointerInt => self.get_type_pointer_int(idx).name,
            KeyTag::TypeFixedWidthInt => self.get_type_fixed_width_int(idx).name,
            KeyTag::TypeFloat => self.get_type_float(idx).name,
            KeyTag::TypeComplex => self.get_type_complex(idx).name,
            KeyTag::TypeQuat => self.get_type_quat(idx).name,
            KeyTag::TypeDQuat => self.get_type_dquat(idx).name,
            KeyTag::TypeNamespace => self.get_type_namespace(idx).name,
            KeyTag::TypeNoReturn => self.get_type_no_return().name,
            KeyTag::TypeNull => self.get_type_null().name,
            KeyTag::TypeSinglePointer => todo!(),
            KeyTag::TypeMultiPointer => todo!(),
            KeyTag::TypeRawptr => self.get_type_rawptr().name,
            KeyTag::TypeSlice => todo!(),
            KeyTag::TypeString => self.get_type_string().name,
            KeyTag::TypeCString => self.get_type_cstring().name,
            KeyTag::TypeVoid => self.get_type_void().name,
            KeyTag::TypeVector => todo!(),
            KeyTag::TypeMatrix => todo!(),
            KeyTag::TypeType => self.get_type_type().name,
            KeyTag::TypeTypeId => self.get_type_type_id().name,
            KeyTag::TypeUndefined => self.get_type_undefined().name,
            KeyTag::Module
            | KeyTag::File
            | KeyTag::AstInfo
            | KeyTag::HirInfo
            | KeyTag::Bool
            | KeyTag::Char
            | KeyTag::IntU64
            | KeyTag::IntI64
            | KeyTag::IntLarge
            | KeyTag::Float
            | KeyTag::NoReturn
            | KeyTag::Null
            | KeyTag::Optional
            | KeyTag::Void
            | KeyTag::TypeId
            | KeyTag::Undefined
            | KeyTag::TypedUndefined
            | KeyTag::Aggregate
            | KeyTag::Union => unreachable!("not a type"),
        }
    }

    pub fn display_index(&self, idx: Index) -> impl std::fmt::Display + '_ {
        #[derive(Clone, Copy)]
        struct Helper<'a> {
            pool: &'a InternPool,
            idx: Index,
        }
        impl std::fmt::Display for Helper<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.pool.fmt_index(f, self.idx)
            }
        }
        Helper { pool: self, idx }
    }

    pub fn fmt_index(&self, f: &mut (dyn std::fmt::Write + '_), idx: Index) -> std::fmt::Result {
        match self.get_tag(idx) {
            KeyTag::Module => unsafe {
                let module = self.get_module(idx);
                let root_directory_path = module.root_directory_path.as_str();
                let root_file_path = module.root_file_path.as_str();
                let name = module.name.as_str();
                write!(
                    f,
                    "%module(dir := {:?}, file := {:?}, name := {:?})",
                    root_directory_path, root_file_path, name
                )
            },
            KeyTag::File => unsafe {
                let file = self.get_file(idx);
                let file_path = file.file_path.as_str();
                write!(f, "%file(path := {:?}, module :=", file_path)?;
                self.fmt_index(f, file.module.0)?;
                write!(f, ")")
            },
            KeyTag::AstInfo => {
                let info = self.get_ast_info(idx);
                write!(f, "%ast(file := ")?;
                self.fmt_index(f, info.file.0)?;
                write!(f, ")")
            }
            KeyTag::HirInfo => {
                let info = self.get_hir_info(idx);
                write!(f, "%hir(ast := ")?;
                self.fmt_index(f, info.ast_info.0)?;
                write!(f, ")")
            }
            KeyTag::TypeAnyInt
            | KeyTag::TypeAnyFloat
            | KeyTag::TypeAnyOpaque
            | KeyTag::TypeAnyType
            | KeyTag::TypeBool
            | KeyTag::TypeChar
            | KeyTag::TypeNaturalInt
            | KeyTag::TypePointerInt
            | KeyTag::TypeFixedWidthInt
            | KeyTag::TypeFloat
            | KeyTag::TypeComplex
            | KeyTag::TypeQuat
            | KeyTag::TypeDQuat
            | KeyTag::TypeNamespace
            | KeyTag::TypeNoReturn
            | KeyTag::TypeNull
            | KeyTag::TypeSinglePointer
            | KeyTag::TypeMultiPointer
            | KeyTag::TypeRawptr
            | KeyTag::TypeSlice
            | KeyTag::TypeString
            | KeyTag::TypeCString
            | KeyTag::TypeVoid
            | KeyTag::TypeVector
            | KeyTag::TypeMatrix
            | KeyTag::TypeType
            | KeyTag::TypeTypeId
            | KeyTag::TypeUndefined => unsafe {
                let ty_name = self.get_type_name(idx).as_str();
                write!(f, "type({})", ty_name)
            },
            KeyTag::Bool => unsafe {
                let value = self.get_value_bool(idx);
                let ty_name = self.get_type_name(value.ty).as_str();
                write!(f, "{}({})", ty_name, value.value)
            },
            KeyTag::Char => {
                let value = self.get_value_char(idx);
                write!(f, "char({:?})", value.value)
            }
            KeyTag::IntU64 => unsafe {
                let value = self.get_value_int_u64(idx);
                let ty_name = self.get_type_name(value.ty).as_str();
                write!(f, "{}({})", ty_name, value.value)
            },
            KeyTag::IntI64 => unsafe {
                let value = self.get_value_int_i64(idx);
                let ty_name = self.get_type_name(value.ty).as_str();
                write!(f, "{}({})", ty_name, value.value)
            },
            KeyTag::IntLarge => unsafe {
                let value = self.get_value_int_large(idx);
                let ty_name = self.get_type_name(value.ty).as_str();
                let int = value.as_bigint();
                write!(f, "{}({})", ty_name, int)
            },
            KeyTag::Float => todo!(),
            KeyTag::NoReturn => write!(f, "no_return"),
            KeyTag::Null => write!(f, "null"),
            KeyTag::Optional => todo!(),
            KeyTag::Void => write!(f, "void"),
            KeyTag::TypeId => todo!(),
            KeyTag::Undefined => write!(f, "undefined"),
            KeyTag::TypedUndefined => todo!(),
            KeyTag::Aggregate => todo!(),
            KeyTag::Union => todo!(),
        }
    }
}

impl Default for InternPool {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for InternPool {}
unsafe impl Sync for InternPool {}

impl Drop for InternPool {
    fn drop(&mut self) {
        let local_pools = self.pools.lock_blocking();

        unsafe {
            let drop_value = |idx: u32| match self.tags.at_ref(idx) {
                KeyTag::Module
                | KeyTag::File
                | KeyTag::AstInfo
                | KeyTag::HirInfo
                | KeyTag::TypeAnyInt
                | KeyTag::TypeAnyFloat
                | KeyTag::TypeAnyOpaque
                | KeyTag::TypeAnyType
                | KeyTag::TypeBool
                | KeyTag::TypeChar
                | KeyTag::TypeNaturalInt
                | KeyTag::TypePointerInt
                | KeyTag::TypeFixedWidthInt
                | KeyTag::TypeFloat
                | KeyTag::TypeComplex
                | KeyTag::TypeQuat
                | KeyTag::TypeDQuat
                | KeyTag::TypeNamespace
                | KeyTag::TypeNoReturn
                | KeyTag::TypeNull
                | KeyTag::TypeSinglePointer
                | KeyTag::TypeMultiPointer
                | KeyTag::TypeRawptr
                | KeyTag::TypeSlice
                | KeyTag::TypeString
                | KeyTag::TypeCString
                | KeyTag::TypeVoid
                | KeyTag::TypeVector
                | KeyTag::TypeMatrix
                | KeyTag::TypeType
                | KeyTag::TypeTypeId
                | KeyTag::TypeUndefined
                | KeyTag::Bool
                | KeyTag::Char
                | KeyTag::IntU64
                | KeyTag::IntI64
                | KeyTag::IntLarge
                | KeyTag::Float
                | KeyTag::NoReturn
                | KeyTag::Null
                | KeyTag::Optional
                | KeyTag::Void
                | KeyTag::TypeId
                | KeyTag::Undefined
                | KeyTag::TypedUndefined
                | KeyTag::Aggregate
                | KeyTag::Union => {}
            };

            let mut min_bytes_idx = 0u32;
            let mut min_values_idx = 0u32;
            for &pool in local_pools.values() {
                let pool = &*pool;
                let bytes_idx = pool.bytes_count.get() as u32;
                if bytes_idx != 0 {
                    min_bytes_idx = min_bytes_idx.min(bytes_idx);
                }

                let values_idx = pool.values_count.get() as u32;
                if values_idx != 0 {
                    min_values_idx = min_values_idx.min(values_idx);
                }
            }

            for idx in 0..min_values_idx {
                drop_value(idx);
            }

            for &pool in local_pools.values() {
                let pool = &*pool;
                let values_idx = pool.values_count.get() as u32;
                for idx in values_idx & LocalPoolInner::VALUES_BATCH_LEN..values_idx {
                    drop_value(idx);
                }

                let bytes_idx = pool.bytes_count.get() as u32;
                for idx in bytes_idx & LocalPoolInner::BYTES_BATCH_LEN..bytes_idx {
                    let ptr = self.bytes.at_ref(idx).as_ptr();
                    drop(Box::from_raw(ptr));
                }
            }

            for idx in 0..min_bytes_idx {
                let ptr = self.bytes.at_ref(idx).as_ptr();
                drop(Box::from_raw(ptr));
            }

            for idx in 0..self.asts_count.load(Ordering::Relaxed) {
                let ptr = self.asts.at_or_uninit(idx);
                (*ptr).assume_init_drop();
            }

            for idx in 0..self.hir_chunks_count.load(Ordering::Relaxed) {
                let ptr = self.hir_chunks.at_or_uninit(idx);
                (*ptr).assume_init_drop();
            }

            for idx in 0..self.rscopes_count.load(Ordering::Relaxed) {
                let ptr = self.rscopes.at_or_uninit(idx);
                (*ptr).assume_init_drop();
            }

            for idx in 0..self.udscopes_count.load(Ordering::Relaxed) {
                let ptr = self.udscopes.at_or_uninit(idx);
                (*ptr).assume_init_drop();
            }

            for idx in 0..self.ldscopes_count.load(Ordering::Relaxed) {
                let ptr = self.ldscopes.at_or_uninit(idx);
                (*ptr).assume_init_drop();
            }

            for idx in 0..self.decls_count.load(Ordering::Relaxed) {
                let ptr = self.decls.at_or_uninit(idx);
                (*ptr).assume_init_drop();
            }
        }
    }
}

struct LocalPoolInner {
    global: *const InternPool,

    values_count: Cell<u64>,
    bytes_count: Cell<u64>,

    bools_count: Cell<u32>,
    chars_count: Cell<u32>,
    ints_u64_count: Cell<u32>,
    ints_i64_count: Cell<u32>,
    ints_large_count: Cell<u32>,
}

impl LocalPoolInner {
    const BYTES_BATCH_LEN: u32 = 16;
    const VALUES_BATCH_LEN: u32 = 1024;

    fn new(global: *const InternPool) -> Self {
        Self {
            global,

            values_count: Default::default(),
            bytes_count: Default::default(),

            bools_count: Default::default(),
            chars_count: Default::default(),
            ints_u64_count: Default::default(),
            ints_i64_count: Default::default(),
            ints_large_count: Default::default(),
        }
    }

    async fn intern_ast(&self, ast: Ast) -> AstId {
        unsafe {
            let global = &*self.global;
            let idx = global.asts_count.fetch_add(1, Ordering::Relaxed);
            (*global.asts.at_or_uninit(idx)).write(ast);
            AstId(ListIndex::new_unchecked(idx))
        }
    }

    async fn intern_hir(&self, hir: HirChunk) -> HirId {
        unsafe {
            let global = &*self.global;
            let idx = global.hir_chunks_count.fetch_add(1, Ordering::Relaxed);
            (*global.hir_chunks.at_or_uninit(idx)).write(hir);
            HirId(ListIndex::new_unchecked(idx))
        }
    }

    async fn intern_rscope(&self, scope: RootScope) -> RScopeId {
        unsafe {
            let global = &*self.global;
            let idx = global.rscopes_count.fetch_add(1, Ordering::Relaxed);
            (*global.rscopes.at_or_uninit(idx)).write(scope);
            RScopeId(ListIndex::new_unchecked(idx))
        }
    }

    async fn intern_udscope(&self, uds: UnorderedDeclScope) -> UDScopeId {
        unsafe {
            let global = &*self.global;
            let idx = global.udscopes_count.fetch_add(1, Ordering::Relaxed);
            (*global.udscopes.at_or_uninit(idx)).write(uds);
            UDScopeId(ListIndex::new_unchecked(idx))
        }
    }

    async fn intern_ldscope(&self, lds: LazyDeclScope) -> LDScopeId {
        unsafe {
            let global = &*self.global;
            let idx = global.ldscopes_count.fetch_add(1, Ordering::Relaxed);
            (*global.ldscopes.at_or_uninit(idx)).write(lds);
            LDScopeId(ListIndex::new_unchecked(idx))
        }
    }

    async fn intern_decl(&self, decl: Decl) -> DeclId {
        unsafe {
            let global = &*self.global;
            let idx = global.decls_count.fetch_add(1, Ordering::Relaxed);
            (*global.decls.at_or_uninit(idx)).write(decl);
            DeclId(ListIndex::new_unchecked(idx))
        }
    }

    async fn intern_string(&self, string: &str) -> RawString {
        let bytes = string.as_bytes();
        let interned_idx = self.intern_bytes(bytes).await;

        unsafe {
            let global = &*self.global;
            let interned = global
                .bytes
                .at_or_uninit(interned_idx.get())
                .read()
                .assume_init();
            RawString { ptr: interned }
        }
    }

    async fn intern_cstring(&self, string: &str) -> RawCString {
        assert!(!string.is_empty() && string.as_bytes()[string.len() - 1] == 0);
        let bytes = string.as_bytes();
        let interned_idx = self.intern_bytes(bytes).await;

        unsafe {
            let global = &*self.global;
            let interned = global
                .bytes
                .at_or_uninit(interned_idx.get())
                .read()
                .assume_init();
            RawCString { ptr: interned }
        }
    }

    async fn intern_bytes(&self, bytes: &[u8]) -> ListIndex<NonNull<[u8]>> {
        let global = unsafe { &*self.global };

        let mut hasher = RapidHasher::default_const();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();
        let shard_idx = (hash & (global.bytes_shards.len() as u64 - 1)) as usize;

        let mut shard = global.bytes_shards[shard_idx].lock().await;
        if let Some(idx) = shard.get(bytes) {
            return *idx;
        }

        if self
            .bytes_count
            .get()
            .is_multiple_of(Self::BYTES_BATCH_LEN as u64)
        {
            self.bytes_count.set(
                global
                    .bytes_count
                    .fetch_add(Self::BYTES_BATCH_LEN, Ordering::Relaxed) as u64,
            );
            assert!(self.bytes_count.get() < u32::MAX as u64)
        }

        unsafe {
            let idx = ListIndex::new_unchecked(self.bytes_count.get() as u32);
            let boxed = Box::<[u8]>::from(bytes);
            shard.insert(boxed.clone(), idx);
            global
                .bytes
                .write_element(idx, NonNull::new_unchecked(Box::into_raw(boxed)));

            self.bytes_count.update(|v| v + 1);
            idx
        }
    }

    async fn intern_value(&self, value: Key<'_>) -> Index {
        let global = unsafe { &*self.global };

        let mut hasher = RapidHasher::default_const();
        value.hash(&mut hasher);
        let hash = hasher.finish();
        let shard_idx = (hash & (global.key_shards.len() as u64 - 1)) as usize;

        let mut shard = global.key_shards[shard_idx].lock().await;
        if let Some(idx) = shard.get(&value) {
            return *idx;
        }

        if self
            .values_count
            .get()
            .is_multiple_of(Self::VALUES_BATCH_LEN as u64)
        {
            self.values_count.set(
                global
                    .values_count
                    .fetch_add(Self::VALUES_BATCH_LEN, Ordering::Relaxed) as u64,
            );
            assert!(self.values_count.get() < u32::MAX as u64);
        }

        let idx = unsafe { Index::new_unchecked(self.values_count.get() as u32) };
        let tag_idx = unsafe { ListIndex::<KeyTag>::new_unchecked(self.values_count.get() as u32) };
        self.values_count.update(|v| v + 1);

        match value {
            Key::Module(
                key @ KeyModule {
                    root_directory_path,
                    root_file_path,
                    name,
                    is_core,
                },
            ) => {
                let list_idx = global.modules_count.fetch_add(1, Ordering::Relaxed);
                global.modules.write_element_raw(
                    list_idx,
                    Module {
                        root_directory_path,
                        root_file_path,
                        name,
                        is_core,
                    },
                );

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::Module);
                shard.insert(Key::Module(key), idx);
            }
            Key::File(
                key @ KeyFile {
                    file_path,
                    qualified_name,
                    module,
                },
            ) => {
                let list_idx = global.files_count.fetch_add(1, Ordering::Relaxed);
                global.files.write_element_raw(
                    list_idx,
                    File {
                        file_path,
                        qualified_name,
                        module,
                    },
                );

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::File);
                shard.insert(Key::File(key), idx);
            }
            Key::AstInfo(key @ KeyAstInfo { file, id }) => {
                let list_idx = global.ast_infos_count.fetch_add(1, Ordering::Relaxed);
                global
                    .ast_infos
                    .write_element_raw(list_idx, AstInfo { file, id });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::AstInfo);
                shard.insert(Key::AstInfo(key), idx);
            }
            Key::HirInfo(key @ KeyHirInfo { ast_info, id }) => {
                let list_idx = global.hir_infos_count.fetch_add(1, Ordering::Relaxed);
                global
                    .hir_infos
                    .write_element_raw(list_idx, HirInfo { ast_info, id });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::HirInfo);
                shard.insert(Key::HirInfo(key), idx);
            }

            Key::TypeAnyInt => unsafe {
                let name = "any_int\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.any_int_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeAnyInt);
                shard.insert(Key::TypeAnyInt, idx);
            },
            Key::TypeAnyFloat => unsafe {
                let name = "any_float\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.any_float_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeAnyFloat);
                shard.insert(Key::TypeAnyFloat, idx);
            },
            Key::TypeAnyOpaque(key @ KeyTypeAnyOpaque { metadata }) => unsafe {
                let (prefix, suffix, flags) = match metadata {
                    Some(metadata_idx) => {
                        let metadata_tag = global.get_tag(metadata_idx);
                        let flags = match metadata_tag {
                            KeyTag::TypeAnyInt => TypeFlags::Const,
                            KeyTag::TypeAnyFloat => TypeFlags::Const,
                            KeyTag::TypeAnyOpaque => unreachable!(),
                            KeyTag::TypeAnyType => TypeFlags::Const | TypeFlags::Generic,
                            KeyTag::TypeBool => TypeFlags::empty(),
                            KeyTag::TypeChar => TypeFlags::empty(),
                            KeyTag::TypeNaturalInt => TypeFlags::empty(),
                            KeyTag::TypePointerInt => TypeFlags::empty(),
                            KeyTag::TypeFixedWidthInt => TypeFlags::empty(),
                            KeyTag::TypeFloat => TypeFlags::empty(),
                            KeyTag::TypeComplex => TypeFlags::empty(),
                            KeyTag::TypeQuat => TypeFlags::empty(),
                            KeyTag::TypeDQuat => TypeFlags::empty(),
                            KeyTag::TypeNamespace => unreachable!(),
                            KeyTag::TypeNoReturn => TypeFlags::Const,
                            KeyTag::TypeNull => TypeFlags::Const,
                            KeyTag::TypeSinglePointer => todo!(),
                            KeyTag::TypeMultiPointer => todo!(),
                            KeyTag::TypeRawptr => TypeFlags::empty(),
                            KeyTag::TypeSlice => unreachable!(),
                            KeyTag::TypeString => unreachable!(),
                            KeyTag::TypeCString => unreachable!(),
                            KeyTag::TypeVoid => unreachable!(),
                            KeyTag::TypeVector => todo!(),
                            KeyTag::TypeMatrix => todo!(),
                            KeyTag::TypeType => TypeFlags::Const,
                            KeyTag::TypeTypeId => TypeFlags::empty(),
                            KeyTag::TypeUndefined => TypeFlags::Const,
                            KeyTag::Module
                            | KeyTag::File
                            | KeyTag::AstInfo
                            | KeyTag::HirInfo
                            | KeyTag::Bool
                            | KeyTag::Char
                            | KeyTag::IntU64
                            | KeyTag::IntI64
                            | KeyTag::IntLarge
                            | KeyTag::Float
                            | KeyTag::NoReturn
                            | KeyTag::Null
                            | KeyTag::Optional
                            | KeyTag::Void
                            | KeyTag::TypeId
                            | KeyTag::Undefined
                            | KeyTag::TypedUndefined
                            | KeyTag::Aggregate
                            | KeyTag::Union => unreachable!(),
                        };

                        let suffix = format!("({})", global.get_type_name(idx).as_str());
                        ("#", suffix, flags | TypeFlags::PtrOnly)
                    }
                    None => ("", "".to_string(), TypeFlags::PtrOnly),
                };

                let name = format!("{prefix}any_opaque{suffix}\0");
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global
                    .any_opaque_types_count
                    .fetch_add(1, Ordering::Relaxed);
                global
                    .any_opaque_types
                    .write_element_raw(list_idx, TypeAnyOpaque { id, name, flags });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeAnyOpaque);
                shard.insert(Key::TypeAnyOpaque(key), idx);
            },
            Key::TypeAnyType => unsafe {
                let name = "any_type\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.any_type_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeAnyType);
                shard.insert(Key::TypeAnyType, idx);
            },
            Key::TypeBool(key @ KeyTypeBool { width }) => {
                assert!(width != 0);
                let name = match width {
                    1 => "bool\0".to_string(),
                    _ => format!("b{width}\0"),
                };
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.bool_types_count.fetch_add(1, Ordering::Relaxed);
                global
                    .bool_types
                    .write_element_raw(list_idx, TypeBool { id, name, width });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeBool);
                shard.insert(Key::TypeBool(key), idx);
            }
            Key::TypeChar => unsafe {
                let name = "char\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.char_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeChar);
                shard.insert(Key::TypeChar, idx);
            },
            Key::TypeNaturalInt(key @ KeyTypeNaturalInt { is_signed }) => {
                let name = if is_signed { "int\0" } else { "uint\0" };
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.nat_int_types_count.fetch_add(1, Ordering::Relaxed);
                global.nat_int_types.write_element_raw(
                    list_idx,
                    TypeNaturalInt {
                        id,
                        name,
                        is_signed,
                    },
                );

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeNaturalInt);
                shard.insert(Key::TypeNaturalInt(key), idx);
            }
            Key::TypePointerInt(key @ KeyTypePointerInt { is_signed }) => {
                let name = if is_signed { "intptr\0" } else { "uintptr\0" };
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global
                    .pointer_int_types_count
                    .fetch_add(1, Ordering::Relaxed);
                global.pointer_int_types.write_element_raw(
                    list_idx,
                    TypePointerInt {
                        id,
                        name,
                        is_signed,
                    },
                );

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypePointerInt);
                shard.insert(Key::TypePointerInt(key), idx);
            }
            Key::TypeFixedWidthInt(
                key @ KeyTypeFixedWidthInt {
                    width,
                    is_signed,
                    is_little_endian,
                },
            ) => {
                let prefix = if is_signed { "i" } else { "u" };
                let suffix = match is_little_endian {
                    Some(true) => "le",
                    Some(false) => "be",
                    None => "",
                };
                let name = format!("{prefix}{width}{suffix}\0");
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global
                    .fixed_width_int_types_count
                    .fetch_add(1, Ordering::Relaxed);
                global.fixed_width_int_types.write_element_raw(
                    list_idx,
                    TypeFixedWidthInt {
                        id,
                        name,
                        width,
                        is_signed,
                        is_little_endian,
                    },
                );

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global
                    .tags
                    .write_element(tag_idx, KeyTag::TypeFixedWidthInt);
                shard.insert(Key::TypeFixedWidthInt(key), idx);
            }
            Key::TypeFloat(
                key @ KeyTypeFloat {
                    width,
                    is_little_endian,
                },
            ) => {
                assert!(width == 32 || width == 64);
                let suffix = match is_little_endian {
                    Some(true) => "le",
                    Some(false) => "be",
                    None => "",
                };
                let name = format!("f{width}{suffix}\0");
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.float_types_count.fetch_add(1, Ordering::Relaxed);
                global.float_types.write_element_raw(
                    list_idx,
                    TypeFloat {
                        id,
                        name,
                        width,
                        is_little_endian,
                    },
                );

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeFloat);
                shard.insert(Key::TypeFloat(key), idx);
            }
            Key::TypeComplex(key @ KeyTypeComplex { width }) => {
                assert!(width == 32 || width == 64);
                let name = format!("complex{width}\0");
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.complex_types_count.fetch_add(1, Ordering::Relaxed);
                global
                    .complex_types
                    .write_element_raw(list_idx, TypeComplex { id, name, width });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeComplex);
                shard.insert(Key::TypeComplex(key), idx);
            }
            Key::TypeQuat(key @ KeyTypeQuat { width }) => {
                assert!(width == 32 || width == 64);
                let name = format!("quat{width}\0");
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.quat_types_count.fetch_add(1, Ordering::Relaxed);
                global
                    .quat_types
                    .write_element_raw(list_idx, TypeQuat { id, name, width });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeQuat);
                shard.insert(Key::TypeQuat(key), idx);
            }
            Key::TypeDQuat(key @ KeyTypeDQuat { width }) => {
                assert!(width == 32 || width == 64);
                let name = format!("dquat{width}\0");
                let name = self.intern_cstring(&name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.dquat_types_count.fetch_add(1, Ordering::Relaxed);
                global
                    .dquat_types
                    .write_element_raw(list_idx, TypeDQuat { id, name, width });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeDQuat);
                shard.insert(Key::TypeDQuat(key), idx);
            }
            Key::TypeNamespace(key @ KeyTypeNamespace { scope, name }) => {
                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                let list_idx = global.namespace_types_count.fetch_add(1, Ordering::Relaxed);
                global
                    .namespace_types
                    .write_element_raw(list_idx, TypeNamespace { id, name, scope });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::TypeNamespace);
                shard.insert(Key::TypeNamespace(key), idx);
            }
            Key::TypeNoReturn => unsafe {
                let name = "no_return\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.no_return_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeNoReturn);
                shard.insert(Key::TypeNoReturn, idx);
            },
            Key::TypeNull => unsafe {
                let name = "null\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.null_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeNull);
                shard.insert(Key::TypeNull, idx);
            },
            Key::TypeRawptr => unsafe {
                let name = "rawptr\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.rawptr_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeRawptr);
                shard.insert(Key::TypeRawptr, idx);
            },
            Key::TypeString => unsafe {
                let name = "string\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.string_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeString);
                shard.insert(Key::TypeString, idx);
            },
            Key::TypeCString => unsafe {
                let name = "cstring\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.cstring_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeCString);
                shard.insert(Key::TypeCString, idx);
            },
            Key::TypeVoid => unsafe {
                let name = "void\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.void_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeVoid);
                shard.insert(Key::TypeVoid, idx);
            },
            Key::TypeType => unsafe {
                let name = "type\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.type_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeType);
                shard.insert(Key::TypeType, idx);
            },
            Key::TypeTypeId => unsafe {
                let name = "type_id\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.type_id_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeTypeId);
                shard.insert(Key::TypeTypeId, idx);
            },
            Key::TypeUndefined => unsafe {
                let name = "undefined\0";
                let name = self.intern_cstring(name).await;

                let id = global.types_count.fetch_add(1, Ordering::Relaxed);
                (*global.undefined_type.get()).write(TypeSimple { id, name });

                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::TypeUndefined);
                shard.insert(Key::TypeUndefined, idx);
            },

            Key::Bool(key @ KeyBool { ty, value }) => {
                if self
                    .bools_count
                    .get()
                    .is_multiple_of(Self::VALUES_BATCH_LEN as _)
                {
                    self.bools_count.set(
                        global
                            .bools_count
                            .fetch_add(Self::VALUES_BATCH_LEN, Ordering::Relaxed),
                    );
                }
                let list_idx = self.bools_count.get();
                self.bools_count.set(list_idx + 1);
                global
                    .bools
                    .write_element_raw(list_idx, ValueBool { ty, value });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::Bool);
                shard.insert(Key::Bool(key), idx);
            }
            Key::Char(key @ KeyChar { value }) => {
                if self
                    .chars_count
                    .get()
                    .is_multiple_of(Self::VALUES_BATCH_LEN as _)
                {
                    self.chars_count.set(
                        global
                            .chars_count
                            .fetch_add(Self::VALUES_BATCH_LEN, Ordering::Relaxed),
                    );
                }
                let list_idx = self.chars_count.get();
                self.chars_count.set(list_idx + 1);
                global
                    .chars
                    .write_element_raw(list_idx, ValueChar { value });

                global.sublist_idx.write_element_raw(idx.get(), list_idx);
                global.tags.write_element(tag_idx, KeyTag::Char);
                shard.insert(Key::Char(key), idx);
            }
            Key::Int(KeyInt { ty, storage }) => match storage {
                KeyIntStorage::U64(value) => {
                    if self
                        .ints_u64_count
                        .get()
                        .is_multiple_of(Self::VALUES_BATCH_LEN as _)
                    {
                        self.ints_u64_count.set(
                            global
                                .ints_u64_count
                                .fetch_add(Self::VALUES_BATCH_LEN, Ordering::Relaxed),
                        );
                    }
                    let list_idx = self.ints_u64_count.get();
                    self.ints_u64_count.set(list_idx + 1);
                    global
                        .ints_u64
                        .write_element_raw(list_idx, ValueIntU64 { ty, value });

                    global.sublist_idx.write_element_raw(idx.get(), list_idx);
                    global.tags.write_element(tag_idx, KeyTag::IntU64);
                    shard.insert(
                        Key::Int(KeyInt {
                            ty,
                            storage: KeyIntStorage::U64(value),
                        }),
                        idx,
                    );
                }
                KeyIntStorage::I64(value) => {
                    if self
                        .ints_i64_count
                        .get()
                        .is_multiple_of(Self::VALUES_BATCH_LEN as _)
                    {
                        self.ints_i64_count.set(
                            global
                                .ints_i64_count
                                .fetch_add(Self::VALUES_BATCH_LEN, Ordering::Relaxed),
                        );
                    }
                    let list_idx = self.ints_i64_count.get();
                    self.ints_i64_count.set(list_idx + 1);
                    global
                        .ints_i64
                        .write_element_raw(list_idx, ValueIntI64 { ty, value });

                    global.sublist_idx.write_element_raw(idx.get(), list_idx);
                    global.tags.write_element(tag_idx, KeyTag::IntI64);
                    shard.insert(
                        Key::Int(KeyInt {
                            ty,
                            storage: KeyIntStorage::I64(value),
                        }),
                        idx,
                    );
                }
                KeyIntStorage::BigInt(KeyBigIntStorage { bytes }) => {
                    if self
                        .ints_large_count
                        .get()
                        .is_multiple_of(Self::VALUES_BATCH_LEN as _)
                    {
                        self.ints_large_count.set(
                            global
                                .ints_large_count
                                .fetch_add(Self::VALUES_BATCH_LEN, Ordering::Relaxed),
                        );
                    }
                    let list_idx = self.ints_large_count.get();
                    self.ints_large_count.set(list_idx + 1);

                    let bytes = self.intern_bytes(bytes).await;
                    let value = unsafe { *global.bytes.at_ref(bytes.get()) };
                    let bytes = unsafe { value.as_ref() };
                    global
                        .ints_large
                        .write_element_raw(list_idx, ValueIntLarge { ty, value });

                    global.sublist_idx.write_element_raw(idx.get(), list_idx);
                    global.tags.write_element(tag_idx, KeyTag::IntLarge);
                    shard.insert(
                        Key::Int(KeyInt {
                            ty,
                            storage: KeyIntStorage::BigInt(KeyBigIntStorage { bytes }),
                        }),
                        idx,
                    );
                }
            },
            Key::Float(key_float) => todo!(),
            Key::Ptr => todo!(),
            Key::PtrWide(key_ptr_wide) => todo!(),
            Key::NoReturn => {
                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::NoReturn);
                shard.insert(Key::NoReturn, idx);
            }
            Key::Null => {
                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::Null);
                shard.insert(Key::Null, idx);
            }
            Key::Optional(key_optional) => todo!(),
            Key::Void => {
                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::Void);
                shard.insert(Key::Void, idx);
            }
            Key::TypeId(key_type_id) => todo!(),
            Key::Undefined => {
                global.sublist_idx.write_element_raw(idx.get(), 0);
                global.tags.write_element(tag_idx, KeyTag::Undefined);
                shard.insert(Key::Undefined, idx);
            }
            Key::TypedUndefined(key_typed_undefined) => todo!(),
            Key::EnumLiteral => todo!(),
            Key::EnumTag(key_enum_tag) => todo!(),
            Key::Aggregate(key_aggregate) => todo!(),
            Key::Union(key_union) => todo!(),
        }

        idx
    }
}

#[derive(Debug)]
pub struct LocalPool {
    inner: *mut LocalPoolInner,
}

impl LocalPool {
    pub async fn intern_ast(&self, ast: Ast) -> AstId {
        unsafe { (*self.inner.cast_const()).intern_ast(ast).await }
    }

    pub async fn intern_hir(&self, hir: HirChunk) -> HirId {
        unsafe { (*self.inner.cast_const()).intern_hir(hir).await }
    }

    pub async fn intern_rscope(&self, scope: RootScope) -> RScopeId {
        unsafe { (*self.inner.cast_const()).intern_rscope(scope).await }
    }

    pub async fn intern_udscope(&self, scope: UnorderedDeclScope) -> UDScopeId {
        unsafe { (*self.inner.cast_const()).intern_udscope(scope).await }
    }

    pub async fn intern_ldscope(&self, scope: LazyDeclScope) -> LDScopeId {
        unsafe { (*self.inner.cast_const()).intern_ldscope(scope).await }
    }

    pub async fn intern_decl(&self, decl: Decl) -> DeclId {
        unsafe { (*self.inner.cast_const()).intern_decl(decl).await }
    }

    pub async fn intern_string(&self, string: &str) -> RawString {
        unsafe { (*self.inner.cast_const()).intern_string(string).await }
    }

    pub async fn intern_cstring(&self, string: &str) -> RawCString {
        unsafe { (*self.inner.cast_const()).intern_cstring(string).await }
    }

    pub async fn intern_value(&self, value: Key<'_>) -> Index {
        unsafe { (*self.inner.cast_const()).intern_value(value).await }
    }

    pub async fn intern_root_module(&self, value: KeyModule) -> TypedIndex<Module> {
        let value = Key::Module(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_file(&self, value: KeyFile) -> TypedIndex<File> {
        let value = Key::File(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_ast_info(&self, value: KeyAstInfo) -> TypedIndex<AstInfo> {
        let value = Key::AstInfo(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_hir_info(&self, value: KeyHirInfo) -> TypedIndex<HirInfo> {
        let value = Key::HirInfo(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_type_namespace(
        &self,
        value: KeyTypeNamespace,
    ) -> TypedIndex<TypeNamespace> {
        let value = Key::TypeNamespace(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_value_bool(&self, value: KeyBool) -> TypedIndex<ValueBool> {
        let value = Key::Bool(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_value_char(&self, value: KeyChar) -> TypedIndex<ValueChar> {
        let value = Key::Char(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_value_int_i64(&self, value: KeyInt<'_>) -> TypedIndex<ValueIntI64> {
        debug_assert!(matches!(value.storage, KeyIntStorage::I64(_)));
        let value = Key::Int(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_value_int_u64(&self, value: KeyInt<'_>) -> TypedIndex<ValueIntU64> {
        debug_assert!(matches!(value.storage, KeyIntStorage::U64(_)));
        let value = Key::Int(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }

    pub async fn intern_value_int_large(&self, value: KeyInt<'_>) -> TypedIndex<ValueIntLarge> {
        debug_assert!(matches!(value.storage, KeyIntStorage::BigInt(_)));
        let value = Key::Int(value);
        let idx = self.intern_value(value).await;
        unsafe { TypedIndex::from_raw(idx) }
    }
}

pub trait Interned {
    fn get_from_pool(idx: Index, ip: &InternPool) -> &Self;
}

pub struct TypedIndex<T: Interned>(Index, PhantomData<for<'a> fn(&'a InternPool) -> &'a T>);

impl<T: Interned> TypedIndex<T> {
    pub const unsafe fn from_raw(idx: Index) -> Self {
        Self(idx, PhantomData)
    }

    pub const fn into_raw(self) -> Index {
        self.0
    }

    pub fn get_from_pool(self, ip: &InternPool) -> &T {
        T::get_from_pool(self.0, ip)
    }
}

impl<T: Interned> Debug for TypedIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypedIndex").field(&self.0).finish()
    }
}

impl<T: Interned> Clone for TypedIndex<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Interned> Copy for TypedIndex<T> {}

impl<T: Interned> PartialEq for TypedIndex<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Interned> Eq for TypedIndex<T> {}

impl<T: Interned> PartialOrd for TypedIndex<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Interned> Ord for TypedIndex<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Interned> Hash for TypedIndex<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(NonMaxU32);

impl Index {
    pub const TY_BOOL: Self = Self::new(0).unwrap();
    pub const TY_B8: Self = Self::new(1).unwrap();
    pub const TY_B16: Self = Self::new(2).unwrap();
    pub const TY_B32: Self = Self::new(3).unwrap();
    pub const TY_B64: Self = Self::new(4).unwrap();
    pub const TY_I1: Self = Self::new(5).unwrap();
    pub const TY_I8: Self = Self::new(6).unwrap();
    pub const TY_I16: Self = Self::new(7).unwrap();
    pub const TY_I32: Self = Self::new(8).unwrap();
    pub const TY_I64: Self = Self::new(9).unwrap();
    pub const TY_I128: Self = Self::new(10).unwrap();
    pub const TY_INT: Self = Self::new(11).unwrap();
    pub const TY_INTPTR: Self = Self::new(12).unwrap();
    pub const TY_U0: Self = Self::new(13).unwrap();
    pub const TY_U1: Self = Self::new(14).unwrap();
    pub const TY_U8: Self = Self::new(15).unwrap();
    pub const TY_U16: Self = Self::new(16).unwrap();
    pub const TY_U32: Self = Self::new(17).unwrap();
    pub const TY_U64: Self = Self::new(18).unwrap();
    pub const TY_U128: Self = Self::new(19).unwrap();
    pub const TY_UINT: Self = Self::new(20).unwrap();
    pub const TY_UINTPTR: Self = Self::new(21).unwrap();
    pub const TY_I1LE: Self = Self::new(22).unwrap();
    pub const TY_I8LE: Self = Self::new(23).unwrap();
    pub const TY_I16LE: Self = Self::new(24).unwrap();
    pub const TY_I32LE: Self = Self::new(25).unwrap();
    pub const TY_I64LE: Self = Self::new(26).unwrap();
    pub const TY_I128LE: Self = Self::new(27).unwrap();
    pub const TY_I1BE: Self = Self::new(28).unwrap();
    pub const TY_I8BE: Self = Self::new(29).unwrap();
    pub const TY_I16BE: Self = Self::new(30).unwrap();
    pub const TY_I32BE: Self = Self::new(31).unwrap();
    pub const TY_I64BE: Self = Self::new(32).unwrap();
    pub const TY_I128BE: Self = Self::new(33).unwrap();
    pub const TY_U0LE: Self = Self::new(34).unwrap();
    pub const TY_U1LE: Self = Self::new(35).unwrap();
    pub const TY_U8LE: Self = Self::new(36).unwrap();
    pub const TY_U16LE: Self = Self::new(37).unwrap();
    pub const TY_U32LE: Self = Self::new(38).unwrap();
    pub const TY_U64LE: Self = Self::new(39).unwrap();
    pub const TY_U128LE: Self = Self::new(40).unwrap();
    pub const TY_U0BE: Self = Self::new(41).unwrap();
    pub const TY_U1BE: Self = Self::new(42).unwrap();
    pub const TY_U8BE: Self = Self::new(43).unwrap();
    pub const TY_U16BE: Self = Self::new(44).unwrap();
    pub const TY_U32BE: Self = Self::new(45).unwrap();
    pub const TY_U64BE: Self = Self::new(46).unwrap();
    pub const TY_U128BE: Self = Self::new(47).unwrap();
    pub const TY_F32: Self = Self::new(48).unwrap();
    pub const TY_F64: Self = Self::new(49).unwrap();
    pub const TY_F32LE: Self = Self::new(50).unwrap();
    pub const TY_F64LE: Self = Self::new(51).unwrap();
    pub const TY_F32BE: Self = Self::new(52).unwrap();
    pub const TY_F64BE: Self = Self::new(53).unwrap();
    pub const TY_COMPLEX32: Self = Self::new(54).unwrap();
    pub const TY_COMPLEX64: Self = Self::new(55).unwrap();
    pub const TY_QUAT32: Self = Self::new(56).unwrap();
    pub const TY_QUAT64: Self = Self::new(57).unwrap();
    pub const TY_DQUAT32: Self = Self::new(58).unwrap();
    pub const TY_DQUAT64: Self = Self::new(59).unwrap();
    pub const TY_CHAR: Self = Self::new(60).unwrap();
    pub const TY_STRING: Self = Self::new(61).unwrap();
    pub const TY_CSTRING: Self = Self::new(62).unwrap();
    pub const TY_RAWPTR: Self = Self::new(63).unwrap();
    pub const TY_VOID: Self = Self::new(64).unwrap();
    pub const TY_ANY_INT: Self = Self::new(65).unwrap();
    pub const TY_ANY_FLOAT: Self = Self::new(66).unwrap();
    pub const TY_ANY_OPAQUE: Self = Self::new(67).unwrap();
    pub const TY_ANY_TYPE: Self = Self::new(68).unwrap();
    pub const TY_NULL: Self = Self::new(69).unwrap();
    pub const TY_TYPE: Self = Self::new(70).unwrap();
    pub const TY_TYPE_ID: Self = Self::new(71).unwrap();
    pub const TY_UNDEFINED: Self = Self::new(72).unwrap();
    pub const TY_NO_RETURN: Self = Self::new(73).unwrap();
    // pub const TY_STRING_PTR: Self = Self::new(73).unwrap();
    // pub const TY_CSTRING_PTR: Self = Self::new(74).unwrap();

    const TYPES_END: u32 = 74;

    pub const VAL_NO_RETURN: Self = Self::new(Self::TYPES_END).unwrap();
    pub const VAL_NULL: Self = Self::new(Self::TYPES_END + 1).unwrap();
    pub const VAL_VOID: Self = Self::new(Self::TYPES_END + 2).unwrap();
    pub const VAL_UNDEFINED: Self = Self::new(Self::TYPES_END + 3).unwrap();
    pub const VAL_TRUE: Self = Self::new(Self::TYPES_END + 4).unwrap();
    pub const VAL_FALSE: Self = Self::new(Self::TYPES_END + 5).unwrap();

    pub const fn new(index: u32) -> Option<Self> {
        match NonMaxU32::new(index) {
            Some(inner) => Some(Self(inner)),
            None => None,
        }
    }

    const unsafe fn new_unchecked(index: u32) -> Self {
        unsafe { Self(NonMaxU32::new_unchecked(index)) }
    }

    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyTag {
    Module,
    File,
    AstInfo,
    HirInfo,

    TypeAnyInt,
    TypeAnyFloat,
    TypeAnyOpaque,
    TypeAnyType,
    TypeBool,
    TypeChar,
    TypeNaturalInt,
    TypePointerInt,
    TypeFixedWidthInt,
    TypeFloat,
    TypeComplex,
    TypeQuat,
    TypeDQuat,
    TypeNamespace,
    TypeNoReturn,
    TypeNull,
    TypeSinglePointer,
    TypeMultiPointer,
    TypeRawptr,
    TypeSlice,
    TypeString,
    TypeCString,
    TypeVoid,
    TypeVector,
    TypeMatrix,
    TypeType,
    TypeTypeId,
    TypeUndefined,

    Bool,
    Char,
    IntU64,
    IntI64,
    IntLarge,
    Float,
    NoReturn,
    Null,
    Optional,
    Void,
    TypeId,
    Undefined,
    TypedUndefined,
    Aggregate,
    Union,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key<'a> {
    Module(KeyModule),
    File(KeyFile),
    AstInfo(KeyAstInfo),
    HirInfo(KeyHirInfo),

    TypeAnyInt,
    TypeAnyFloat,
    TypeAnyOpaque(KeyTypeAnyOpaque),
    TypeAnyType,
    TypeBool(KeyTypeBool),
    TypeChar,
    TypeNaturalInt(KeyTypeNaturalInt),
    TypePointerInt(KeyTypePointerInt),
    TypeFixedWidthInt(KeyTypeFixedWidthInt),
    TypeFloat(KeyTypeFloat),
    TypeComplex(KeyTypeComplex),
    TypeQuat(KeyTypeQuat),
    TypeDQuat(KeyTypeDQuat),
    TypeNamespace(KeyTypeNamespace),
    TypeNoReturn,
    TypeNull,
    TypeRawptr,
    TypeString,
    TypeCString,
    TypeVoid,
    TypeType,
    TypeTypeId,
    TypeUndefined,

    Bool(KeyBool),
    Char(KeyChar),
    Int(KeyInt<'a>),
    Float(KeyFloat),
    Ptr,
    PtrWide(KeyPtrWide),
    NoReturn,
    Null,
    Optional(KeyOptional),
    Void,
    TypeId(KeyTypeId),
    Undefined,
    TypedUndefined(KeyTypedUndefined),
    EnumLiteral,
    EnumTag(KeyEnumTag),
    Aggregate(KeyAggregate<'a>),
    Union(KeyUnion),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyModule {
    pub root_directory_path: RawCString,
    pub root_file_path: RawCString,
    pub name: RawCString,
    pub is_core: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyFile {
    pub file_path: RawCString,
    pub qualified_name: RawCString,
    pub module: TypedIndex<Module>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyAstInfo {
    pub file: TypedIndex<File>,
    pub id: AstId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyHirInfo {
    pub ast_info: TypedIndex<AstInfo>,
    pub id: HirId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeAnyOpaque {
    pub metadata: Option<Index>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeBool {
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeNaturalInt {
    pub is_signed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypePointerInt {
    pub is_signed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeFixedWidthInt {
    pub width: u16,
    pub is_signed: bool,
    pub is_little_endian: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeFloat {
    pub width: u16,
    pub is_little_endian: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeComplex {
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeQuat {
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeDQuat {
    pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeNamespace {
    pub scope: UDScopeId,
    pub name: RawCString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBool {
    pub ty: Index,
    pub value: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyChar {
    pub value: char,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyInt<'a> {
    pub ty: Index,
    pub storage: KeyIntStorage<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyIntStorage<'a> {
    U64(u64),
    I64(i64),
    BigInt(KeyBigIntStorage<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBigIntStorage<'a> {
    pub bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyFloat {
    pub ty: Index,
    pub storage: KeyFloatStorage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyPtr {
    pub ty: Index,
    pub byte_offset: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyPtrWide {
    pub ty: Index,
    pub ptr: Index,
    pub metadata: Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyFloatStorage {
    F32([u8; 4]),
    F64([u8; 8]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyOptional {
    pub ty: Index,
    pub value: Option<Index>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypeId {
    pub value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyTypedUndefined {
    pub ty: Index,
    pub value: Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyEnumLiteral {
    pub value: RawCString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyEnumTag {
    pub ty: Index,
    pub value: Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyAggregate<'a> {
    pub ty: Index,
    pub storage: KeyAggregateStorage<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyAggregateStorage<'a> {
    Bytes(ListIndex<RawString>),
    Elements(&'a [Index]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyUnion {
    pub ty: Index,
    pub tag: Option<Index>,
    pub value: Index,
}
