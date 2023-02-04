//! Wrappers and executors for Rust functions.

use std::any::Any;
use std::future::Future;
use std::panic;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::ffi::{IntoDart, MessagePort};
use anyhow::Result;
use futures_util::FutureExt;

use crate::rust2dart::{Rust2Dart, TaskCallback};
use crate::support::WireSyncReturn;
use crate::{spawn, thread, SyncReturn};

/// The types of return values for a particular Rust function.
#[derive(Copy, Clone)]
pub enum FfiCallMode {
    /// The default mode, returns a Dart `Future<T>`.
    Normal,
    /// Used by `SyncReturn<T>` to skip spawning workers.
    Sync,
    /// Returns a Dart `Stream<T>`.
    Stream,
}

/// Supporting information to identify a function's operating mode.
#[derive(Clone)]
pub struct WrapInfo {
    /// A Dart `SendPort`. [None] if the mode is [FfiCallMode::Sync].
    pub port: Option<MessagePort>,
    /// Usually the name of the function.
    pub debug_name: &'static str,
    /// The call mode of this function.
    pub mode: FfiCallMode,
}
/// Provide your own handler to customize how to execute your function calls, etc.
pub trait Handler {
    /// Prepares the arguments, executes a Rust function and sets up its return value.
    ///
    /// Why separate `PrepareFn` and `TaskFn`: because some things cannot be [`Send`] (e.g. raw
    /// pointers), so those can be done in `PrepareFn`, while the real work is done in `TaskFn` with [`Send`].
    ///
    /// The generated code depends on the fact that `PrepareFn` is synchronous to maintain
    /// correctness, therefore implementors of [`Handler`] must also uphold this property.
    ///
    /// If a Rust function returns [`SyncReturn`], it must be called with
    /// [`wrap_sync`](Handler::wrap_sync) instead.
    fn wrap<PrepareFn, TaskFn, TaskRet>(&self, wrap_info: WrapInfo, prepare: PrepareFn)
    where
        PrepareFn: FnOnce() -> TaskFn + UnwindSafe,
        TaskFn: FnOnce(TaskCallback) -> Result<TaskRet> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart;

    fn wrap_async<PrepareFn, TaskFut, TaskRet>(&self, wrap_info: WrapInfo, prepare: PrepareFn)
    where
        PrepareFn: FnOnce() -> TaskFut + UnwindSafe,
        TaskFut: Future<Output = Result<TaskRet>> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart;

    /// Same as [`wrap`][Handler::wrap], but the Rust function must return a [SyncReturn] and
    /// need not implement [Send].
    fn wrap_sync<SyncTaskFn, TaskRet>(
        &self,
        wrap_info: WrapInfo,
        sync_task: SyncTaskFn,
    ) -> WireSyncReturn
    where
        SyncTaskFn: FnOnce() -> Result<SyncReturn<TaskRet>> + UnwindSafe,
        TaskRet: IntoDart;
}

/// The simple handler uses a simple thread pool to execute tasks.
pub struct SimpleHandler<E: Executor, EH: ErrorHandler> {
    executor: E,
    error_handler: EH,
}

impl<E: Executor, H: ErrorHandler> SimpleHandler<E, H> {
    /// Create a new default handler.
    pub fn new(executor: E, error_handler: H) -> Self {
        SimpleHandler {
            executor,
            error_handler,
        }
    }
}

/// The default handler used by the generated code.
pub type DefaultHandler =
    SimpleHandler<ThreadPoolExecutor<ReportDartErrorHandler>, ReportDartErrorHandler>;

impl Default for DefaultHandler {
    fn default() -> Self {
        Self::new(
            ThreadPoolExecutor::new(ReportDartErrorHandler),
            ReportDartErrorHandler,
        )
    }
}

impl<E: Executor, EH: ErrorHandler> Handler for SimpleHandler<E, EH> {
    fn wrap<PrepareFn, TaskFn, TaskRet>(&self, wrap_info: WrapInfo, prepare: PrepareFn)
    where
        PrepareFn: FnOnce() -> TaskFn + UnwindSafe,
        TaskFn: FnOnce(TaskCallback) -> Result<TaskRet> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart,
    {
        // NOTE This extra [catch_unwind] **SHOULD** be put outside **ALL** code!
        // Why do this: As nomicon says, unwind across languages is undefined behavior (UB).
        // Therefore, we should wrap a [catch_unwind] outside of *each and every* line of code
        // that can cause panic. Otherwise we may touch UB.
        // Why do not report error or something like that if this outer [catch_unwind] really
        // catches something: Because if we report error, that line of code itself can cause panic
        // as well. Then that new panic will go across language boundary and cause UB.
        // ref https://doc.rust-lang.org/nomicon/unwinding.html
        let _ = panic::catch_unwind(move || {
            let wrap_info2 = wrap_info.clone();
            if let Err(error) = panic::catch_unwind(move || {
                let task = prepare();
                self.executor.execute(wrap_info2, task);
            }) {
                self.error_handler
                    .handle_error(wrap_info.port.unwrap(), Error::Panic(error));
            }
        });
    }

    fn wrap_async<PrepareFn, TaskFut, TaskRet>(&self, wrap_info: WrapInfo, prepare: PrepareFn)
    where
        PrepareFn: FnOnce() -> TaskFut + UnwindSafe,
        TaskFut: Future<Output = Result<TaskRet>> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart,
    {
        // NOTE This extra [catch_unwind] **SHOULD** be put outside **ALL** code!
        // For reason, see comments in [wrap]
        let _ = panic::catch_unwind(move || {
            let wrap_info2 = wrap_info.clone();
            let task = prepare();
            self.executor.execute_async(wrap_info2, task);
        });
    }

    fn wrap_sync<SyncTaskFn, TaskRet>(
        &self,
        wrap_info: WrapInfo,
        sync_task: SyncTaskFn,
    ) -> WireSyncReturn
    where
        TaskRet: IntoDart,
        SyncTaskFn: FnOnce() -> Result<SyncReturn<TaskRet>> + UnwindSafe,
    {
        // NOTE This extra [catch_unwind] **SHOULD** be put outside **ALL** code!
        // For reason, see comments in [wrap]
        panic::catch_unwind(move || {
            let catch_unwind_result = panic::catch_unwind(move || {
                match self.executor.execute_sync(wrap_info, sync_task) {
                    Ok(data) => wire_sync_from_data(data.0, true),
                    Err(err) => self
                        .error_handler
                        .handle_error_sync(Error::ResultError(err)),
                }
            });
            catch_unwind_result
                .unwrap_or_else(|error| self.error_handler.handle_error_sync(Error::Panic(error)))
        })
        .unwrap_or_else(|_| wire_sync_from_data(None::<()>, false))
    }
}

/// An executor model for Rust functions.
///
/// For example, the default model is [ThreadPoolExecutor]
/// which runs each function in a separate thread.
pub trait Executor: RefUnwindSafe {
    /// Executes a Rust function and transforms its return value into a Dart-compatible
    /// value, i.e. types that implement [`IntoDart`].
    fn execute<TaskFn, TaskRet>(&self, wrap_info: WrapInfo, task: TaskFn)
    where
        TaskFn: FnOnce(TaskCallback) -> Result<TaskRet> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart;

    /// TODO
    fn execute_async<TaskFut, TaskRet>(&self, wrap_info: WrapInfo, task: TaskFut)
    where
        TaskFut: Future<Output = Result<TaskRet>> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart;

    /// Executes a Rust function that returns a [SyncReturn].
    fn execute_sync<SyncTaskFn, TaskRet>(
        &self,
        wrap_info: WrapInfo,
        sync_task: SyncTaskFn,
    ) -> Result<SyncReturn<TaskRet>>
    where
        SyncTaskFn: FnOnce() -> Result<SyncReturn<TaskRet>> + UnwindSafe,
        TaskRet: IntoDart;
}

/// The default executor used.
/// It creates an internal thread pool, and each call to a Rust function is
/// handled by a different thread.
pub struct ThreadPoolExecutor<EH: ErrorHandler> {
    error_handler: EH,
}

impl<EH: ErrorHandler> ThreadPoolExecutor<EH> {
    /// Create a new executor backed by a thread pool.
    pub fn new(error_handler: EH) -> Self {
        ThreadPoolExecutor { error_handler }
    }
}

use once_cell::sync::Lazy;

static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(thread::WORKERS_COUNT)
        .max_blocking_threads(thread::WORKERS_COUNT)
        .build()
        .expect("Failed to build tokio runtime")
});

impl<EH: ErrorHandler> Executor for ThreadPoolExecutor<EH> {
    fn execute<TaskFn, TaskRet>(&self, wrap_info: WrapInfo, task: TaskFn)
    where
        TaskFn: FnOnce(TaskCallback) -> Result<TaskRet> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart,
    {
        let eh = self.error_handler;
        let eh2 = self.error_handler;

        let WrapInfo { port, mode, .. } = wrap_info;

        spawn!(|port: Option<MessagePort>| {
            let port2 = port.as_ref().cloned();
            let thread_result = panic::catch_unwind(move || {
                let port2 = port2.expect("(worker) thread");
                #[allow(clippy::clone_on_copy)]
                let rust2dart = Rust2Dart::new(port2.clone());

                let ret = task(TaskCallback::new(rust2dart.clone())).map(IntoDart::into_dart);

                match ret {
                    Ok(result) => {
                        match mode {
                            FfiCallMode::Normal => {
                                rust2dart.success(result);
                            }
                            FfiCallMode::Stream => {
                                // nothing - ignore the return value of a Stream-typed function
                            }
                            FfiCallMode::Sync => {
                                panic!("FfiCallMode::Sync should not call execute, please call execute_sync instead")
                            }
                        }
                    }
                    Err(error) => {
                        eh2.handle_error(port2, Error::ResultError(error));
                    }
                };
            });

            if let Err(error) = thread_result {
                eh.handle_error(port.expect("(worker) eh"), Error::Panic(error));
            }
        });
    }

    fn execute_async<TaskFut, TaskRet>(&self, wrap_info: WrapInfo, task: TaskFut)
    where
        TaskFut: Future<Output = Result<TaskRet>> + Send + UnwindSafe + 'static,
        TaskRet: IntoDart,
    {
        let eh = self.error_handler;
        let eh2 = self.error_handler;

        let WrapInfo { port, mode, .. } = wrap_info;

        let port = port.expect("execute_async must always be given a MessagePort");
        let port2 = port;

        let maybe_spawn_panic = panic::catch_unwind(move || {
            let eh = eh2;
            let port = port2;
            let rt = &*RUNTIME;

            let wrapped_task = task.catch_unwind().map(move |res| {
                let output = match res {
                    Ok(Ok(output)) => output,
                    Ok(Err(err)) => return eh.handle_error(port, Error::ResultError(err)),
                    Err(err) => return eh.handle_error(port, Error::Panic(err)),
                };

                match mode {
                    FfiCallMode::Normal => {
                        Rust2Dart::new(port).success(output);
                    }
                    FfiCallMode::Stream => (),
                    FfiCallMode::Sync => panic!(
                        "FfiCallMode::Sync should not call execute_async, \
                             please call execute_sync instead"
                    ),
                };
            });

            rt.spawn(wrapped_task);
        });

        if let Err(err) = maybe_spawn_panic {
            eh.handle_error(port, Error::Panic(err));
        }
    }

    fn execute_sync<SyncTaskFn, TaskRet>(
        &self,
        _wrap_info: WrapInfo,
        sync_task: SyncTaskFn,
    ) -> Result<SyncReturn<TaskRet>>
    where
        SyncTaskFn: FnOnce() -> Result<SyncReturn<TaskRet>> + UnwindSafe,
        TaskRet: IntoDart,
    {
        sync_task()
    }
}

/// Errors that occur from normal code execution.
#[derive(Debug)]
pub enum Error {
    /// Errors from an [anyhow::Error].
    ResultError(anyhow::Error),
    /// Exceptional errors from panicking.
    Panic(Box<dyn Any + Send>),
}

impl Error {
    /// The identifier of the type of error.
    pub fn code(&self) -> &'static str {
        match self {
            Error::ResultError(_) => "RESULT_ERROR",
            Error::Panic(_) => "PANIC_ERROR",
        }
    }

    /// The message of the error.
    pub fn message(&self) -> String {
        match self {
            Error::ResultError(e) => format!("{:?}", e),
            Error::Panic(panic_err) => match panic_err.downcast_ref::<&'static str>() {
                Some(s) => *s,
                None => match panic_err.downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<dyn Any>",
                },
            }
            .to_string(),
        }
    }
}

/// A handler model that sends back the error to a Dart `SendPort`.
///
/// For example, instead of using the default [`ReportDartErrorHandler`],
/// you could implement your own handler that logs each error to stderr,
/// or to an external logging service.
pub trait ErrorHandler: UnwindSafe + RefUnwindSafe + Copy + Send + 'static {
    /// The default error handler.
    fn handle_error(&self, port: MessagePort, error: Error);

    /// Special handler only used for synchronous code.
    fn handle_error_sync(&self, error: Error) -> WireSyncReturn;
}

/// The default error handler used by generated code.
#[derive(Clone, Copy)]
pub struct ReportDartErrorHandler;

impl ErrorHandler for ReportDartErrorHandler {
    fn handle_error(&self, port: MessagePort, error: Error) {
        Rust2Dart::new(port).error(error.code().to_string(), error.message());
    }

    fn handle_error_sync(&self, error: Error) -> WireSyncReturn {
        wire_sync_from_data(format!("{}: {}", error.code(), error.message()), false)
    }
}

fn wire_sync_from_data<T: IntoDart>(data: T, success: bool) -> WireSyncReturn {
    let sync_return = vec![data.into_dart(), success.into_dart()].into_dart();

    #[cfg(not(wasm))]
    return crate::support::new_leak_box_ptr(sync_return);

    #[cfg(wasm)]
    return sync_return;
}
