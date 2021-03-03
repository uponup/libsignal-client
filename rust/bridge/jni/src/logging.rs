//
// Copyright 2021 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use jni::objects::{GlobalRef, JClass, JObject, JValue};
use jni::sys::jint;
use jni::{JNIEnv, JavaVM};
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::abort;

// Keep this in sync with SignalProtocolLogger.java, as well as the list below.
#[derive(Clone, Copy)]
enum JavaLogLevel {
    Verbose = 2,
    Debug = 3,
    Info = 4,
    Warn = 5,
    Error = 6,
    Assert = 7,
}

impl From<log::Level> for JavaLogLevel {
    fn from(level: log::Level) -> Self {
        use log::Level::*;
        match level {
            Error => Self::Error,
            Warn => Self::Warn,
            Info => Self::Info,
            Debug => Self::Debug,
            Trace => Self::Verbose,
        }
    }
}

impl From<JavaLogLevel> for jint {
    fn from(level: JavaLogLevel) -> Self {
        level as jint
    }
}

impl From<JavaLogLevel> for JValue<'_> {
    fn from(level: JavaLogLevel) -> Self {
        Self::Int(level.into())
    }
}

impl From<JavaLogLevel> for log::Level {
    fn from(level: JavaLogLevel) -> Self {
        use JavaLogLevel::*;
        match level {
            Error | Assert => Self::Error,
            Warn => Self::Warn,
            Info => Self::Info,
            Debug => Self::Debug,
            Verbose => Self::Trace,
        }
    }
}

struct JniLogger {
    vm: JavaVM,
    logger_class: GlobalRef,
}

impl JniLogger {
    fn new(env: JNIEnv, logger_class: JClass) -> jni::errors::Result<Self> {
        Ok(Self {
            vm: env.get_java_vm()?,
            logger_class: env.new_global_ref(logger_class)?,
        })
    }

    fn log_impl(&self, record: &log::Record) -> jni::errors::Result<()> {
        let env = self.vm.attach_current_thread()?;
        let level: JavaLogLevel = record.level().into();
        let message = format!(
            "{}:{}: {}",
            record.file().unwrap_or("<unknown>"),
            record.line().unwrap_or(0),
            record.args(),
        );
        let args: [JValue; 3] = [
            level.into(),
            env.new_string("libsignal-client")?.into(),
            env.new_string(message)?.into(),
        ];
        let result = env.call_static_method(
            &self.logger_class,
            "log",
            "(ILjava/lang/String;Ljava/lang/String;)V",
            &args,
        );

        let throwable = env.exception_occurred()?;
        if **throwable == *JObject::null() {
            result?;
        } else {
            env.exception_clear()?;
        }
        Ok(())
    }
}

impl log::Log for JniLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.log_impl(record).is_err() {
            // Drop the error; it's not like we can log it!
        }
    }

    fn flush(&self) {}
}

// See https://github.com/rust-lang/rfcs/issues/1389
fn describe_panic(any: &Box<dyn Any + Send>) -> String {
    if let Some(msg) = any.downcast_ref::<&str>() {
        msg.to_string()
    } else if let Some(msg) = any.downcast_ref::<String>() {
        msg.to_string()
    } else {
        "(break on rust_panic to debug)".to_string()
    }
}

/// A low-level version of `run_ffi_safe` that just aborts on errors.
///
/// This is important for logging failures because we might want to log during the normal
/// `run_ffi_safe`. This should *not* be used normally because we don't want to crash the app!
fn abort_on_panic(f: impl FnOnce()) {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|e| {
        eprintln!("fatal error: {}", describe_panic(&e));
        abort();
    });
}

fn set_max_level_from_java_level(max_level: jint) {
    // Keep this in sync with SignalProtocolLogger.java.
    let level = match max_level {
        // The jni crate uses trace! in its own implementation.
        2 => panic!("invalid log level (must be DEBUG or higher for libsignal-client)"),
        3 => JavaLogLevel::Debug,
        4 => JavaLogLevel::Info,
        5 => JavaLogLevel::Warn,
        6 => JavaLogLevel::Error,
        7 => JavaLogLevel::Assert,
        _ => panic!("invalid log level (see SignalProtocolLogger)"),
    };
    assert!(jint::from(level) == max_level);

    log::set_max_level(log::Level::from(level).to_level_filter());
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_signal_client_internal_Native_Logger_1Initialize(
    env: JNIEnv,
    _class: JClass,
    max_level: jint,
    logger_class: JClass,
) {
    abort_on_panic(|| {
        let logger = JniLogger::new(env, logger_class).expect("could not initialize logging");

        match log::set_logger(Box::leak(Box::new(logger))) {
            Ok(_) => {
                set_max_level_from_java_level(max_level);
                log::info!(
                    "Initializing libsignal-client version:{}",
                    env!("CARGO_PKG_VERSION")
                );
            }
            Err(_) => {
                log::warn!("logging already initialized for libsignal-client; ignoring later call");
            }
        }
    });
}

#[no_mangle]
pub unsafe extern "C" fn Java_org_signal_client_internal_Native_Logger_1SetMaxLevel(
    _env: JNIEnv,
    _class: JClass,
    max_level: jint,
) {
    abort_on_panic(|| set_max_level_from_java_level(max_level));
}
