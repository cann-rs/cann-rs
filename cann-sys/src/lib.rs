//! Huawei Ascend CANN SDK 的原始 FFI 绑定。
//!
//! 提供与 `libascendcl` C 库交互所需的类型、常量和函数声明。
//!
//! ## 特性
//! - `ffi` —— 启用后链接 `libascendcl.so` 并暴露 FFI 函数声明。
//!   默认关闭，允许在无 NPU 驱动的环境下编译类型和常量测试。

/// ACL 基础运行时类型与错误码。
pub mod acl_base_rt;
/// ACL 数据类型/格式枚举。
pub mod acl_datatype;
/// ACL 设备与 SOC 管理。
pub mod acl_device;
/// ACL 运行时错误码常量。
pub mod acl_error_code;
/// GE 图引擎（aclgrph*，C++ shim 桥接）。
pub mod acl_grph;
/// ACL 内存管理原语。
pub mod acl_memory;
/// ACL 张量/标量基础类型（acl_meta.h）。
pub mod acl_meta;
/// ACL 运行时 FFI 函数声明与版本查询常量。
pub mod acl_rt;
/// aclnn 算子（首批：Matmul/Softmax/RMSNorm）。
pub mod aclnn_ops;

pub use acl_base_rt::*;
pub use acl_error_code::*;
pub use acl_rt::*;
