//! Huawei Ascend CANN NPU SDK 的安全 Rust 封装。
//!
//! 本 crate 基于 `cann-sys` 提供的 FFI 绑定，提供类型安全、内存安全的 CANN API。
//!
//! ## 模块
//! - [`error`] —— CANN 错误类型。
//! - [`version`] —— CANN 版本查询。
//! - [`context`] —— CANN 上下文（RAII）。
//! - [`device`] —— 设备管理。
//! - [`stream`] —— Stream 流。
//! - [`event`] —— Event 事件。
//! - [`buffer`] —— 设备/主机内存缓冲区。
//!
//! ## 线程亲和性
//!
//! ACL 的当前设备按调用线程绑定（`aclrtSetDevice`），跨线程使用设备资源
//! 前必须在目标线程显式 `set_device`。详见各模块文档。

/// 设备/主机内存缓冲区。
pub mod buffer;
/// CANN 上下文（RAII）。
pub mod context;
/// 设备管理。
pub mod device;
/// CANN 错误类型。
pub mod error;
/// Event 事件。
pub mod event;
/// GE 计算图（ONNX 解析与 .om 编译）。
pub mod graph;
/// aclnn 算子（Matmul/Softmax/RmsNorm）。
pub mod op;
/// Stream 流。
pub mod stream;
/// 张量/张量列表/标量等 aclnn 基础类型。
pub mod tensor;
/// CANN 版本查询。
pub mod version;
