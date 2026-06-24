//! Huawei Ascend CANN NPU SDK 的安全 Rust 封装。
//!
//! 本 crate 基于 `cann-sys` 提供的 FFI 绑定，提供类型安全、内存安全的 CANN API。
//!
//! ## 模块
//! - [`error`] —— CANN 错误类型。
//! - [`version`] —— CANN 版本查询。

/// CANN 错误类型。
pub mod error;
/// CANN 版本查询。
pub mod version;
