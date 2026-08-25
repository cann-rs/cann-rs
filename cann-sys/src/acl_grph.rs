//! GE 图引擎（aclgrph*）FFI 绑定 —— C++ shim 桥接层。
//!
//! GE 的 `aclgrph*` 系列是 **C++ API**（`ge::aclgrphParseONNX` 等，见
//! `include/parser/onnx_parser.h` 与 `include/ge/ge_ir_build.h`），参数与返回涉及
//! `std::map<ge::AscendString, ge::AscendString>`、`ge::Graph&` 等 C++ 类型，
//! 无法直接以 `extern "C"` 声明。本模块经由 `src/ge_shim.cc`（build.rs 在 ffi 档
//! 编译为静态库 `libge_shim.a` 并链接）导出的 C 形状函数绑定；`ge::Graph` 实例
//! 以不透明句柄（`aclgrphGraph`）跨边界传递。
//!
//! 对应头文件：
//! - `include/parser/onnx_parser.h`（aclgrphParseONNX / aclgrphParseONNXFromMem）
//! - `include/ge/ge_ir_build.h`（aclgrphBuildModel / aclgrphSaveModel）
//! - `include/graph/ge_error_codes.h`（graphStatus 类型与错误码常量）
//!
//! 句柄语义：解析成功的图实例由 shim 内部线程局部注册表持有（`ge::Graph` 内部是
//! shared_ptr，值语义 ABI 不稳定，不能裸指针传递），因此句柄**仅在同一线程内有效**；
//! `cann_grph_destroy` 负责释放，同一句柄不得重复释放。

use std::ffi::c_void;
// `c_char` 仅被 ffi 档的 shim 函数声明使用
#[cfg(cann_sys_ffi)]
use std::ffi::c_char;

/// GE 图引擎 API 返回码。
///
/// 对应 C 类型 `uint32_t`（`include/graph/ge_error_codes.h` 中
/// `using graphStatus = uint32_t;`）；`GRAPH_SUCCESS`（0）表示成功，
/// 其余值按 `ge_error_codes.h` 抄录。
#[allow(non_camel_case_types)]
pub type graphStatus = u32;

/// GE 图句柄（不透明）。
///
/// 由 shim 的 `cann_grph_parse_onnx_from_*` 成功返回时生成、
/// `cann_grph_destroy` 释放一次；底层为 C++ `ge::Graph` 实例。
#[allow(non_camel_case_types)]
pub type aclgrphGraph = c_void;

// 以下常量按 `include/graph/ge_error_codes.h` 抄录（CANN 8.5.0），
// 数值与命名保持头文件原样。

/// 操作成功。
// ge_error_codes.h: const graphStatus GRAPH_SUCCESS = 0;
pub const GRAPH_SUCCESS: graphStatus = 0;

/// 操作失败（通用）。
// ge_error_codes.h: const graphStatus GRAPH_FAILED = 0xFFFFFFFF;
pub const GRAPH_FAILED: graphStatus = 0xFFFF_FFFF;

/// 图未发生变更。
// ge_error_codes.h: const graphStatus GRAPH_NOT_CHANGED = 1343242304;
pub const GRAPH_NOT_CHANGED: graphStatus = 1343242304;

/// 参数非法。
// ge_error_codes.h: const graphStatus GRAPH_PARAM_INVALID = 50331649;
pub const GRAPH_PARAM_INVALID: graphStatus = 50331649;

/// 节点缺少常量输入。
// ge_error_codes.h: const graphStatus GRAPH_NODE_WITHOUT_CONST_INPUT = 50331648;
pub const GRAPH_NODE_WITHOUT_CONST_INPUT: graphStatus = 50331648;

/// 节点需要重新遍历。
// ge_error_codes.h: const graphStatus GRAPH_NODE_NEED_REPASS = 50331647;
pub const GRAPH_NODE_NEED_REPASS: graphStatus = 50331647;

/// IR 定义非法。
// ge_error_codes.h: const graphStatus GRAPH_INVALID_IR_DEF = 50331646;
pub const GRAPH_INVALID_IR_DEF: graphStatus = 50331646;

/// 算子缺少 IR 数据类型推断规则。
// ge_error_codes.h: const graphStatus OP_WITHOUT_IR_DATATYPE_INFER_RULE = 50331645;
pub const OP_WITHOUT_IR_DATATYPE_INFER_RULE: graphStatus = 50331645;

/// 参数超出范围。
// ge_error_codes.h: const graphStatus GRAPH_PARAM_OUT_OF_RANGE = 50331644;
pub const GRAPH_PARAM_OUT_OF_RANGE: graphStatus = 50331644;

/// 内存操作失败。
// ge_error_codes.h: const graphStatus GRAPH_MEM_OPERATE_FAILED = 50331539;
pub const GRAPH_MEM_OPERATE_FAILED: graphStatus = 50331539;

/// 空指针。
// ge_error_codes.h: const graphStatus GRAPH_NULL_PTR = 50331538;
pub const GRAPH_NULL_PTR: graphStatus = 50331538;

/// 内存拷贝失败。
// ge_error_codes.h: const graphStatus GRAPH_MEMCPY_FAILED = 50331537;
pub const GRAPH_MEMCPY_FAILED: graphStatus = 50331537;

/// 内存置位失败。
// ge_error_codes.h: const graphStatus GRAPH_MEMSET_FAILED = 50331536;
pub const GRAPH_MEMSET_FAILED: graphStatus = 50331536;

/// 数学计算失败。
// ge_error_codes.h: const graphStatus GRAPH_MATH_CAL_FAILED = 50331429;
pub const GRAPH_MATH_CAL_FAILED: graphStatus = 50331429;

/// 加法溢出。
// ge_error_codes.h: const graphStatus GRAPH_ADD_OVERFLOW = 50331428;
pub const GRAPH_ADD_OVERFLOW: graphStatus = 50331428;

/// 乘法溢出。
// ge_error_codes.h: const graphStatus GRAPH_MUL_OVERFLOW = 50331427;
pub const GRAPH_MUL_OVERFLOW: graphStatus = 50331427;

/// 向上取整溢出（头文件命名原样保留）。
// ge_error_codes.h: const graphStatus GRAPH_RoundUp_Overflow = 50331426;
#[allow(non_upper_case_globals)]
pub const GRAPH_RoundUp_Overflow: graphStatus = 50331426;

// `libge_shim.a` 导出的 C 形状函数声明，仅在启用 `ffi` 特性时编译
// （build.rs 在 ffi 档把 src/ge_shim.cc 编译为静态库并链接 GE 归属库）。
// 签名与 ge_shim.cc 中 `extern "C"` 定义逐项对应；返回值为 graphStatus 原值
// （shim 直接回传 ge 错误码，不做转换）。
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    /// C 函数原名：`cann_grph_parse_onnx_from_file`（shim 导出，桥接
    /// `ge::aclgrphParseONNX`）。官方锚点：`include/parser/onnx_parser.h`
    ///
    /// 从 ONNX 模型文件解析计算图。成功时 `*handle_out` 写入图句柄并返回
    /// `GRAPH_SUCCESS`；失败时写入 NULL 并返回对应错误码。
    /// parser 配置项（parser_params）在 shim 内传空表，L1 阶段不暴露。
    ///
    /// # Safety
    /// - `path` 必须指向 NUL 结尾的有效 C 字符串（模型文件路径），且非 NULL。
    /// - `handle_out` 必须指向有效的可写输出槽位，且非 NULL。
    /// - 返回 `GRAPH_SUCCESS` 后，`*handle_out` 的句柄所有权转移给调用方：
    ///   须由 `cann_grph_destroy` 恰好释放一次，不得重复释放或遗漏。
    /// - 句柄仅在同一线程内有效（shim 内部为 thread_local 注册表），
    ///   跨线程使用句柄是未定义行为。
    pub fn cann_grph_parse_onnx_from_file(
        path: *const c_char,
        handle_out: *mut *mut aclgrphGraph,
    ) -> graphStatus;

    /// C 函数原名：`cann_grph_parse_onnx_from_mem`（shim 导出，桥接
    /// `ge::aclgrphParseONNXFromMem`）。官方锚点：`include/parser/onnx_parser.h`
    ///
    /// 从内存中的 ONNX 模型字节解析计算图。成功/失败语义与句柄所有权同
    /// [`cann_grph_parse_onnx_from_file`]。
    ///
    /// # Safety
    /// - `buffer` 必须指向 `size` 字节可读内存（模型数据），且非 NULL、size 非 0。
    /// - `handle_out` 必须指向有效的可写输出槽位，且非 NULL。
    /// - 返回 `GRAPH_SUCCESS` 后，`*handle_out` 的句柄所有权转移给调用方：
    ///   须由 `cann_grph_destroy` 恰好释放一次，不得重复释放或遗漏。
    /// - 句柄仅在同一线程内有效（shim 内部为 thread_local 注册表），
    ///   跨线程使用句柄是未定义行为。
    pub fn cann_grph_parse_onnx_from_mem(
        buffer: *const c_char,
        size: usize,
        handle_out: *mut *mut aclgrphGraph,
    ) -> graphStatus;

    /// C 函数原名：`cann_grph_build_model`（shim 导出，桥接
    /// `ge::aclgrphBuildModel` + `ge::aclgrphSaveModel`）。
    /// 官方锚点：`include/ge/ge_ir_build.h`
    ///
    /// 编译句柄指向的图并保存为 .om 模型文件。build_options 在 shim 内传空表；
    /// shim 不调用 `aclgrphBuildInitialize`（8.x 构建前无需显式初始化）。
    ///
    /// # Safety
    /// - `handle` 必须来自 `cann_grph_parse_onnx_from_*` 成功返回、尚未销毁且
    ///   在同一线程上使用的句柄。
    /// - `save_path` 必须指向 NUL 结尾的有效 C 字符串（输出文件路径），且非 NULL。
    pub fn cann_grph_build_model(
        handle: *mut aclgrphGraph,
        save_path: *const c_char,
    ) -> graphStatus;

    /// C 函数原名：`cann_grph_destroy`（shim 导出，对应 `ge::Graph` 生命周期管理）。
    ///
    /// 释放图句柄：从 shim 注册表移除并析构底层 `ge::Graph` 实例。
    /// 句柄不存在或已释放返回 `GRAPH_PARAM_INVALID`。
    ///
    /// # Safety
    /// - `handle` 必须来自 `cann_grph_parse_onnx_from_*` 成功返回的句柄且尚未释放；
    ///   同一句柄不得重复调用本函数（重复释放是未定义行为）。
    /// - 句柄须与创建时在同一线程上使用。
    pub fn cann_grph_destroy(handle: *mut aclgrphGraph) -> graphStatus;
}

/// 无需 FFI 的类型级测试（无 SDK 环境下也运行）。
///
/// `graphStatus`、`GRAPH_*` 常量与不透明句柄类型在无 `ffi` 特性时同样编译；
/// 这里锁定类型定义与常量取值（出处：`include/graph/ge_error_codes.h`）。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_defs() {
        let _: graphStatus = 0u32;
        // 句柄在 shim 侧为 `void*`，Rust 侧用 `*mut` 指针表示。
        let _: *mut aclgrphGraph = std::ptr::null_mut();
    }

    #[test]
    fn test_status_constants() {
        // 取值与 ge_error_codes.h 一致（CANN 8.5.0）
        assert_eq!(GRAPH_SUCCESS, 0);
        assert_eq!(GRAPH_FAILED, 0xFFFF_FFFF);
        assert_eq!(GRAPH_NOT_CHANGED, 1343242304);
        assert_eq!(GRAPH_PARAM_INVALID, 50331649);
        assert_eq!(GRAPH_NODE_WITHOUT_CONST_INPUT, 50331648);
        assert_eq!(GRAPH_NODE_NEED_REPASS, 50331647);
        assert_eq!(GRAPH_INVALID_IR_DEF, 50331646);
        assert_eq!(OP_WITHOUT_IR_DATATYPE_INFER_RULE, 50331645);
        assert_eq!(GRAPH_PARAM_OUT_OF_RANGE, 50331644);
        assert_eq!(GRAPH_MEM_OPERATE_FAILED, 50331539);
        assert_eq!(GRAPH_NULL_PTR, 50331538);
        assert_eq!(GRAPH_MEMCPY_FAILED, 50331537);
        assert_eq!(GRAPH_MEMSET_FAILED, 50331536);
        assert_eq!(GRAPH_MATH_CAL_FAILED, 50331429);
        assert_eq!(GRAPH_ADD_OVERFLOW, 50331428);
        assert_eq!(GRAPH_MUL_OVERFLOW, 50331427);
        assert_eq!(GRAPH_RoundUp_Overflow, 50331426);
    }

    #[test]
    fn test_success_is_zero() {
        // graphStatus 约定：0 = 成功，其余为错误码（与头文件 GRAPH_SUCCESS 一致）
        assert_eq!(GRAPH_SUCCESS, 0);
        assert_ne!(GRAPH_FAILED, GRAPH_SUCCESS);
    }

    #[test]
    fn test_handle_pointer_word_size() {
        // 不透明句柄须可无损存储于一个指针宽度的字中
        assert_eq!(
            std::mem::size_of::<*mut aclgrphGraph>(),
            std::mem::size_of::<usize>()
        );
    }
}

// 链接级验证：验证 shim 静态库符号已解析（只取函数地址，不调用函数体，
// 不触碰 SDK 运行时——实际解析 ONNX 需要 GE/NPU 环境，真机验证移交开发板），
// 默认忽略。
#[cfg(all(cann_sys_ffi, test))]
mod ffi_tests {
    use super::*;

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_shim_symbols() {
        // 函数地址非零即说明符号已在链接产物中解析
        assert_ne!(cann_grph_parse_onnx_from_file as *const () as usize, 0);
        assert_ne!(cann_grph_parse_onnx_from_mem as *const () as usize, 0);
        assert_ne!(cann_grph_build_model as *const () as usize, 0);
        assert_ne!(cann_grph_destroy as *const () as usize, 0);
    }
}
