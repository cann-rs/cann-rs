//! Huawei Ascend CANN SDK 的原始 FFI 绑定。
//!
//! 提供与 `libascendcl` C 库交互所需的类型、常量和函数声明。
//!
//! ## 特性
//! - `ffi` —— 启用后链接 `libascendcl.so` 并暴露 FFI 函数声明。
//!   默认关闭，允许在无 NPU 驱动的环境下编译类型和常量测试。

/// ACL 基础运行时类型与错误码。
pub mod acl_base_rt;
/// ACL 运行时 FFI 函数声明与版本查询常量。
pub mod acl_rt;

pub use acl_base_rt::*;
pub use acl_rt::*;
