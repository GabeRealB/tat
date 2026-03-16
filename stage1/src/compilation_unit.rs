use std::{
    borrow::Cow,
    cell::UnsafeCell,
    error::Error,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use crate::{
    ast::Ast,
    hir,
    intern_pool::{
        InternPool, KeyAstInfo, KeyFile, KeyHirInfo, KeyModule, RootScope, RootScopeInner, Scope,
    },
    sema::Sema,
    target::Target,
};
use annotate_snippets::{AnnotationKind, Group, Level, Renderer, Snippet, renderer::DecorStyle};
use smol::{Executor, LocalExecutor, Task, channel, future};

#[derive(Debug)]
pub struct Config {
    pub target: Option<Target>,
    pub root_file_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReportKind {
    FatalError,
    Error,
    Note,
}

pub struct Cu {
    host: Target,
    target: Target,
    root_file_path: PathBuf,
    root_directory_path: PathBuf,
    has_error: AtomicBool,
    has_fatal_error: AtomicBool,
    pool: InternPool,
    exe: Executor<'static>,
}

impl Cu {
    fn new(cfg: Config) -> Arc<Self> {
        let target = cfg.target.unwrap_or(Target::get_host_target());
        let root_file_path = cfg.root_file_path;

        let root_directory_path = root_file_path.parent().unwrap().to_path_buf();

        Arc::new(Self {
            host: Target::get_host_target(),
            target,
            root_file_path,
            root_directory_path,
            has_error: Default::default(),
            has_fatal_error: Default::default(),
            pool: InternPool::new(),
            exe: Executor::new(),
        })
    }

    pub fn target(&self) -> &Target {
        &self.target
    }

    pub fn host(&self) -> &Target {
        &self.host
    }

    pub fn pool(&self) -> &InternPool {
        &self.pool
    }

    pub fn emit_report(&self, report: &[Group<'_>], kind: ReportKind) {
        match kind {
            ReportKind::FatalError => {
                self.has_fatal_error
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                self.has_error
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            ReportKind::Error => {
                self.has_error
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            ReportKind::Note => {}
        }

        let renderer = Renderer::styled().decor_style(DecorStyle::Unicode);
        anstream::println!("{}", renderer.render(report));
    }

    pub fn spawn_task<Fut: Future<Output = ()> + 'static>(
        &self,
        f: impl FnOnce() -> Fut + Send + 'static,
    ) -> Task<()> {
        self.exe.spawn(async move {
            LOCAL_EXE.with(move |exe| {
                let task = exe.spawn(f());
                future::block_on(exe.run(task));
            });
        })
    }
}

thread_local! {
    static LOCAL_EXE: LocalExecutor<'static> = const { LocalExecutor::new() };
}

pub fn compile(cfg: Config) -> Result<(), Box<dyn Error>> {
    let cu = Cu::new(cfg);

    let num_threads = std::thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(1);
    let (stop_token, rx) = channel::unbounded::<()>();

    let mut threads = vec![];
    for _ in 0..num_threads {
        threads.push(std::thread::spawn({
            let cu = cu.clone();
            let rx = rx.clone();
            move || {
                future::block_on(cu.exe.run(async {
                    _ = rx.recv().await;
                }))
            }
        }));
    }

    let task = cu.spawn_task({
        let cu = cu.clone();
        move || compilation_task(stop_token, cu)
    });
    future::block_on(task);

    for thread in threads.drain(..) {
        thread.join().unwrap();
    }
    Ok(())
}

async fn compilation_task(stop_token: channel::Sender<()>, cu: Arc<Cu>) {
    cu.pool.init_pool().await;

    let mut root_file = smol::fs::read(&cu.root_file_path).await.unwrap();
    root_file.push(0);

    let ast = Ast::parse(root_file.into_boxed_slice());

    let mut has_errors = false;
    let mut report = vec![];
    for error in &ast.errors {
        has_errors |= !error.is_warn;
        let desc: Cow<'_, str> = error.data.description();

        let title = if !error.is_warn {
            Level::ERROR.primary_title(desc)
        } else {
            Level::WARNING.primary_title(desc)
        };

        let token = ast.get_token(error.token);
        let span_start = token.start as usize;
        let span_end = span_start + token.length as usize;
        let span = span_start..span_end;
        let source = unsafe { str::from_utf8_unchecked(&ast.source) };
        report.push(
            title.element(Snippet::source(source).annotation(AnnotationKind::Context.span(span))),
        );
    }
    if !report.is_empty() {
        cu.emit_report(
            &report,
            if has_errors {
                ReportKind::FatalError
            } else {
                ReportKind::Note
            },
        );
    }

    if has_errors {
        stop_token.send(()).await.unwrap();
        return;
    }

    println!("Ast root: \n\n{ast}");

    let mut root_directory_path = cu
        .root_directory_path
        .as_path()
        .as_os_str()
        .as_encoded_bytes()
        .to_vec();
    root_directory_path.push(b'\0');
    let root_directory_path = str::from_utf8(&root_directory_path).unwrap();

    let root_file_path = cu
        .root_file_path
        .strip_prefix(&cu.root_directory_path)
        .unwrap();
    let mut root_file_path = root_file_path.as_os_str().as_encoded_bytes().to_vec();
    root_file_path.push(b'\0');
    let root_file_path = str::from_utf8(&root_file_path).unwrap();

    let pool = cu.pool.get_or_init_local_pool().await;
    let root_module_name = pool.intern_cstring("root\0").await;
    let root_directory_path = pool.intern_cstring(root_directory_path).await;
    let root_file_path = pool.intern_cstring(root_file_path).await;
    let root_module = pool
        .intern_root_module(KeyModule {
            root_directory_path,
            root_file_path,
            name: root_module_name,
            is_core: false,
        })
        .await;

    let root_file = pool
        .intern_file(KeyFile {
            file_path: root_file_path,
            qualified_name: root_module_name,
            module: root_module,
        })
        .await;

    let pool = cu.pool.get_or_init_local_pool().await;
    let ast_id = pool.intern_ast(ast).await;
    let ast_info = pool
        .intern_ast_info(KeyAstInfo {
            file: root_file,
            id: ast_id,
        })
        .await;

    let hir = match hir::lower_ast(cu.clone(), ast_info) {
        Ok(x) => x,
        Err(_) => return,
    };
    let hir_id = pool.intern_hir(hir).await;
    let hir_info = pool
        .intern_hir_info(KeyHirInfo {
            ast_info,
            id: hir_id,
        })
        .await;

    let hir = cu.pool().get_hir_chunk(hir_id);
    println!("HIR {hir_id:?}: \n\n{}", hir.display(cu.pool()));

    let scope = RootScope {
        hir_info,
        lock: Default::default(),
        inner: UnsafeCell::new(RootScopeInner {
            resolving: true,
            resolved: false,
            result: None,
            waiters: vec![],
            sema: UnsafeCell::new(Sema::new(&cu)),
        }),
    };
    let scope_id = pool.intern_rscope(scope).await;
    let scope = scope_id.get_from_pool(cu.pool());
    let inner = unsafe { scope.inner.as_ref_unchecked() };
    let sema = unsafe { inner.sema.as_ref_unchecked() };
    match sema.eval_as_root_scope(Scope::Root(scope_id)).await {
        Ok(_) => {}
        Err(_) => {
            stop_token.send(()).await.unwrap();
            return;
        }
    }

    // resolve_ns_task(cu, ty_ns).await;
    stop_token.send(()).await.unwrap();
}
