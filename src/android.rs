//! JNI entry points for the Android client shell.
//!
//! The Android foreground service owns the returned handle. Methods return
//! JSON strings so the first bridge stays small and forwards the same
//! structured snapshots used by other frontends.

use std::{ptr, sync::Arc};

use jni::{
    JNIEnv,
    objects::{JClass, JString},
    sys::{jlong, jstring},
};
use tokio::runtime::Runtime;

use crate::mobile::MobileDaemon;

struct NativeDaemon {
    runtime: Runtime,
    daemon: Arc<MobileDaemon>,
}

fn string<'local>(env: &mut JNIEnv<'local>, value: impl AsRef<str>) -> jstring {
    env.new_string(value.as_ref())
        .map_or(ptr::null_mut(), |value| value.into_raw())
}

fn result_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|error| format!(r#"{{"ok":false,"error":"{error}"}}"#))
}

fn error_json(error: impl std::fmt::Display) -> String {
    serde_json::json!({"ok": false, "error": error.to_string()}).to_string()
}

fn with_handle<T>(handle: jlong, operation: impl FnOnce(&NativeDaemon) -> T) -> Option<T> {
    if handle == 0 {
        return None;
    }
    // Handles are created by Box::into_raw and are released by shutdown.
    let daemon = unsafe { &*(handle as *const NativeDaemon) };
    Some(operation(daemon))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_devptp_MobileNative_start(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jlong {
    let runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => return 0,
    };
    let daemon = match runtime.block_on(MobileDaemon::start()) {
        Ok(daemon) => daemon,
        Err(_) => return 0,
    };
    Box::into_raw(Box::new(NativeDaemon { runtime, daemon })) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_devptp_MobileNative_snapshot(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jstring {
    let Some(result) = with_handle(handle, |native| {
        result_json(&native.runtime.block_on(native.daemon.snapshot()))
    }) else {
        return string(&mut env, &error_json("invalid native handle"));
    };
    string(&mut env, result)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_devptp_MobileNative_connect(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    ticket: JString<'_>,
) -> jstring {
    let ticket = match env.get_string(&ticket) {
        Ok(ticket) => ticket.to_string_lossy().into_owned(),
        Err(error) => return string(&mut env, &error_json(error)),
    };
    let Some(result) = with_handle(handle, |native| {
        match native.runtime.block_on(native.daemon.connect(ticket)) {
            Ok(()) => r#"{"ok":true}"#.to_string(),
            Err(error) => error_json(error),
        }
    }) else {
        return string(&mut env, &error_json("invalid native handle"));
    };
    string(&mut env, result)
}

fn call_operation(
    mut env: JNIEnv<'_>,
    handle: jlong,
    operation: impl FnOnce(&NativeDaemon) -> crate::error::Result<()>,
) -> jstring {
    let Some(result) = with_handle(handle, |native| match operation(native) {
        Ok(()) => r#"{"ok":true}"#.to_string(),
        Err(error) => error_json(error),
    }) else {
        return string(&mut env, &error_json("invalid native handle"));
    };
    string(&mut env, result)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_devptp_MobileNative_disconnect(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jstring {
    call_operation(env, handle, |native| {
        native.runtime.block_on(native.daemon.disconnect())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_devptp_MobileNative_ping(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) -> jstring {
    call_operation(env, handle, |native| {
        native.runtime.block_on(native.daemon.ping())
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_devptp_MobileNative_shutdown(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    if handle == 0 {
        return;
    }
    let native = unsafe { Box::from_raw(handle as *mut NativeDaemon) };
    native.runtime.block_on(native.daemon.shutdown());
}
