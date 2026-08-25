//! Helpers that keep unexpected Rust panics from crossing the N-API boundary.
//!
//! Release artifacts use `panic = "abort"`, so `catch_unwind` only helps debug
//! and test builds. Public bindings must still avoid input-triggered panics.

use std::panic::{AssertUnwindSafe, catch_unwind};

pub const UNEXPECTED_PANIC: &str =
    "internal error: unexpected panic in native binding; the host process stayed alive";

pub fn recover<T>(op: impl FnOnce() -> T, on_panic: impl FnOnce() -> T) -> T {
    match catch_unwind(AssertUnwindSafe(op)) {
        Ok(value) => value,
        Err(_) => on_panic(),
    }
}
