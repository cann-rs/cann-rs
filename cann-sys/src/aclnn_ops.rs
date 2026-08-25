//! aclnn 算子 FFI 绑定（首批：Matmul / Softmax / RMSNorm）。
//!
//! 对应 CANN 头文件：
//! - `include/aclnnop/aclnn_matmul.h`
//! - `include/aclnnop/aclnn_softmax.h`
//! - `include/aclnnop/aclnn_rms_norm.h`
//!
//! 提供 aclnn 两段式算子 API 的原始声明：第一段 `*GetWorkspaceSize` 根据输入张量
//! 计算 workspace 大小并生成算子执行器（`aclOpExecutor`）；第二段（同名函数）在
//! 指定 stream 上消费执行器完成计算。签名已按本地 CANN 8.5.0 SDK 头文件逐项核实
//! （见 docs/specs/0002-l1-aclnn/plan.md verify-list）。

#[cfg(cann_sys_ffi)]
use std::ffi::c_char;
use std::ffi::{c_int, c_void};

/// aclnn 算子 API 返回码。
///
/// 对应 C 类型 `int32_t`（`include/aclnn/acl_meta.h` 中 `typedef int32_t aclnnStatus;`），
/// 遵循 CANN 的约定：`ACLNN_SUCCESS` 表示成功，其他值为具体错误码。
#[allow(non_camel_case_types)]
pub type aclnnStatus = c_int;

/// 操作成功。
///
/// 取值 0 与 `include/aclnn/acl_meta.h` 中的 `constexpr aclnnStatus OK = 0;`
/// 一致；`ACLNN_SUCCESS` 名称是各 `include/aclnnop/aclnn_*.h` 头文件 `@return`
/// 注释中约定的成功返回名（头文件未定义该宏，值为 0）。
pub const ACLNN_SUCCESS: aclnnStatus = 0;

/// aclnn 算子张量句柄（不透明）。
///
/// 对应 C 类型 `aclTensor`（`include/aclnn/acl_meta.h`），在 Rust 侧以
/// `c_void` 指针表示；实际张量由 `aclCreateTensor` 等 API 创建。
#[allow(non_camel_case_types)]
pub type aclTensor = c_void;

/// aclnn 算子执行器句柄（不透明）。
///
/// 对应 C 类型 `aclOpExecutor`（`include/aclnn/aclnn_base.h`），由两段式 API 的
/// 第一段（`*GetWorkspaceSize`）生成、第二段（同名 Launch 函数）消费；
/// 其生命周期必须与对应 workspace 内存同长。
#[allow(non_camel_case_types)]
pub type aclOpExecutor = c_void;

// `libascendcl` aclnn 两段式算子 FFI 函数声明，仅在启用 `ffi` 特性时编译。
// 签名已对照本地 CANN 8.5.0 头文件核实（aclnnop/aclnn_matmul.h、aclnn_softmax.h、
// aclnn_rms_norm.h）；本机 libascendcl.so 是否含 aclnn 符号由 build.rs 探测门控。
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    /// aclnn 运行环境初始化（进程级单次；对应 `aclnnInit(NULL)`）。
    ///
    /// # Safety
    /// - `configPath` 传 NULL 使用默认配置；须在使用任何 aclnn 算子 API 前成功调用。
    pub fn aclnnInit(configPath: *const c_char) -> aclnnStatus;

    /// aclnn 运行环境释放（对应 `aclnnFinalize()`）。
    ///
    /// # Safety
    /// - 须在全部 aclnn 算子执行完成、不再使用时调用一次。
    pub fn aclnnFinalize() -> aclnnStatus;

    /// C 函数原名：`aclnnMatmulGetWorkspaceSize`（两段式第一段）。
    /// 官方锚点：`include/aclnnop/aclnn_matmul.h`
    ///
    /// 矩阵乘（Matmul）两段式 API 的第一段：根据 `self`、`mat2`、`out` 的
    /// 形状与 `cubeMathType` 计算执行所需的 workspace 大小，并把算子执行器
    /// 写入 `executor`。
    ///
    /// # 参数
    /// - `self`：左矩阵张量（输入），数据类型支持 float16/bfloat16，格式支持 ND。
    /// - `mat2`：右矩阵张量（输入），数据类型支持 float16/bfloat16。
    /// - `out`：结果张量（输出），数据类型支持 float16/bfloat16。
    /// - `cubeMathType`：指定 Cube 单元的计算逻辑，Host 侧整型 `int8_t`。
    /// - `workspaceSize`：输出参数，写入需要在 NPU device 侧申请的 workspace 大小（字节）。
    /// - `executor`：输出参数，写入算子执行器句柄。
    ///
    /// # Safety
    /// - `self`/`mat2` 必须指向由 `aclCreateTensor` 创建且未被销毁的合法张量。
    /// - `out` 必须指向可写的合法张量。
    /// - `workspaceSize`/`executor` 必须指向有效的输出槽位，且不能为 NULL。
    /// - 成功后必须按 `workspaceSize` 返回值分配 workspace 内存，并保持其与
    ///   `executor` 同生命周期，随后调用 `aclnnMatmul`（第二段）消费；
    ///   本调用不负责分配/释放 workspace。
    pub fn aclnnMatmulGetWorkspaceSize(
        self_: *const aclTensor,
        mat2: *const aclTensor,
        out: *mut aclTensor,
        cubeMathType: i8,
        workspaceSize: *mut u64,
        executor: *mut *mut aclOpExecutor,
    ) -> aclnnStatus;

    /// C 函数原名：`aclnnMatmul`（两段式第二段）。
    /// 官方锚点：`include/aclnnop/aclnn_matmul.h`
    ///
    /// 矩阵乘两段式 API 的第二段：在指定 `stream` 上执行由第一段
    /// `aclnnMatmulGetWorkspaceSize` 生成的执行器，结果写入 `out`。
    ///
    /// # 参数
    /// - `workspace`：NPU device 侧 workspace 内存起始地址。
    /// - `workspaceSize`：workspace 大小（字节），须等于第一段输出的值。
    /// - `executor`：第一段生成的算子执行器。
    /// - `stream`：执行计算使用的 acl stream。
    ///
    /// # Safety
    /// - `workspace` 必须指向按第一段 `workspaceSize` 大小分配且尚未释放的
    ///   device 内存；workspace 的分配与释放由调用方负责。
    /// - `executor` 必须来自同一次第一段调用，且其生命周期须与 `workspace`
    ///   同长（执行期间二者均不得释放）。
    /// - `stream` 必须是 `aclrtCreateStream` 创建且未销毁的合法流句柄。
    pub fn aclnnMatmul(
        workspace: *mut c_void,
        workspaceSize: u64,
        executor: *mut aclOpExecutor,
        stream: *mut c_void,
    ) -> aclnnStatus;

    /// C 函数原名：`aclnnSoftmaxGetWorkspaceSize`（两段式第一段）。
    /// 官方锚点：`include/aclnnop/aclnn_softmax.h`
    ///
    /// Softmax 两段式 API 的第一段：根据 `self` 与 `dim` 计算执行所需的
    /// workspace 大小，并把算子执行器写入 `executor`。
    ///
    /// # 参数
    /// - `self`：输入张量。
    /// - `dim`：softmax 归一化所在的维度。
    /// - `out`：结果张量（输出）。
    /// - `workspaceSize`：输出参数，写入需要在 NPU device 侧申请的 workspace 大小（字节）。
    /// - `executor`：输出参数，写入算子执行器句柄。
    ///
    /// # Safety
    /// - `self` 必须指向由 `aclCreateTensor` 创建且未被销毁的合法张量。
    /// - `out` 必须指向可写的合法张量。
    /// - `workspaceSize`/`executor` 必须指向有效的输出槽位，且不能为 NULL。
    /// - 成功后必须按 `workspaceSize` 返回值分配 workspace 内存，并保持其与
    ///   `executor` 同生命周期，随后调用 `aclnnSoftmax`（第二段）消费；
    ///   本调用不负责分配/释放 workspace。
    pub fn aclnnSoftmaxGetWorkspaceSize(
        self_: *const aclTensor,
        dim: i64,
        out: *mut aclTensor,
        workspaceSize: *mut u64,
        executor: *mut *mut aclOpExecutor,
    ) -> aclnnStatus;

    /// C 函数原名：`aclnnSoftmax`（两段式第二段）。
    /// 官方锚点：`include/aclnnop/aclnn_softmax.h`
    ///
    /// Softmax 两段式 API 的第二段：在指定 `stream` 上执行由第一段
    /// `aclnnSoftmaxGetWorkspaceSize` 生成的执行器，结果写入 `out`。
    ///
    /// # 参数
    /// - `workspace`：NPU device 侧 workspace 内存起始地址。
    /// - `workspaceSize`：workspace 大小（字节），须等于第一段输出的值。
    /// - `executor`：第一段生成的算子执行器。
    /// - `stream`：执行计算使用的 acl stream。
    ///
    /// # Safety
    /// - `workspace` 必须指向按第一段 `workspaceSize` 大小分配且尚未释放的
    ///   device 内存；workspace 的分配与释放由调用方负责。
    /// - `executor` 必须来自同一次第一段调用，且其生命周期须与 `workspace`
    ///   同长（执行期间二者均不得释放）。
    /// - `stream` 必须是 `aclrtCreateStream` 创建且未销毁的合法流句柄。
    pub fn aclnnSoftmax(
        workspace: *mut c_void,
        workspaceSize: u64,
        executor: *mut aclOpExecutor,
        stream: *mut c_void,
    ) -> aclnnStatus;

    /// C 函数原名：`aclnnRmsNormGetWorkspaceSize`（两段式第一段）。
    /// 官方锚点：`include/aclnnop/aclnn_rms_norm.h`
    ///
    /// RMSNorm 两段式 API 的第一段：根据 `x`、`gamma` 与 `epsilon` 计算执行
    /// 所需的 workspace 大小，并把算子执行器写入 `executor`。
    ///
    /// # 参数
    /// - `x`：输入张量。
    /// - `gamma`：归一化缩放权重张量。
    /// - `epsilon`：防止除零的小常数（double）。
    /// - `yOut`：归一化结果张量（C 侧为 `const aclTensor*`，语义上为输出）。
    /// - `rstdOut`：输出张量，写入每个样本的 1/sqrt(方差 + epsilon)。
    /// - `workspaceSize`：输出参数，写入需要在 NPU device 侧申请的 workspace 大小（字节）。
    /// - `executor`：输出参数，写入算子执行器句柄。
    ///
    /// # Safety
    /// - `x`/`gamma` 必须指向由 `aclCreateTensor` 创建且未被销毁的合法张量。
    /// - `yOut`/`rstdOut` 必须指向形状匹配的可写合法张量。
    /// - `workspaceSize`/`executor` 必须指向有效的输出槽位，且不能为 NULL。
    /// - 成功后必须按 `workspaceSize` 返回值分配 workspace 内存，并保持其与
    ///   `executor` 同生命周期，随后调用 `aclnnRmsNorm`（第二段）消费；
    ///   本调用不负责分配/释放 workspace。
    pub fn aclnnRmsNormGetWorkspaceSize(
        x: *const aclTensor,
        gamma: *const aclTensor,
        epsilon: f64,
        yOut: *const aclTensor,
        rstdOut: *const aclTensor,
        workspaceSize: *mut u64,
        executor: *mut *mut aclOpExecutor,
    ) -> aclnnStatus;

    /// C 函数原名：`aclnnRmsNorm`（两段式第二段）。
    /// 官方锚点：`include/aclnnop/aclnn_rms_norm.h`
    ///
    /// RMSNorm 两段式 API 的第二段：在指定 `stream` 上执行由第一段
    /// `aclnnRmsNormGetWorkspaceSize` 生成的执行器，结果写入 `yOut`/`rstdOut`。
    ///
    /// # 参数
    /// - `workspace`：NPU device 侧 workspace 内存起始地址。
    /// - `workspaceSize`：workspace 大小（字节），须等于第一段输出的值。
    /// - `executor`：第一段生成的算子执行器。
    /// - `stream`：执行计算使用的 acl stream。
    ///
    /// # Safety
    /// - `workspace` 必须指向按第一段 `workspaceSize` 大小分配且尚未释放的
    ///   device 内存；workspace 的分配与释放由调用方负责。
    /// - `executor` 必须来自同一次第一段调用，且其生命周期须与 `workspace`
    ///   同长（执行期间二者均不得释放）。
    /// - `stream` 必须是 `aclrtCreateStream` 创建且未销毁的合法流句柄。
    pub fn aclnnRmsNorm(
        workspace: *mut c_void,
        workspaceSize: u64,
        executor: *mut aclOpExecutor,
        stream: *mut c_void,
    ) -> aclnnStatus;
}

/// 无需 FFI 的类型级测试（无 SDK 环境下也运行）。
///
/// `aclnnStatus` 与成功常量、不透明句柄类型在无 `ffi` 特性时同样编译；
/// 这里锁定类型定义与常量取值。
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_defs() {
        let _: aclnnStatus = 0;
        let _: aclnnStatus = ACLNN_SUCCESS;
        // 句柄在 C 侧均为 `void*`，Rust 侧用 `*const/*mut` 指针表示。
        let _: *const aclTensor = std::ptr::null();
        let _: *const aclOpExecutor = std::ptr::null();
    }

    #[test]
    fn test_aclnn_success() {
        // 取值与 acl_meta.h 的 `constexpr aclnnStatus OK = 0;` 一致
        assert_eq!(ACLNN_SUCCESS, 0);
    }

    #[test]
    fn test_handle_pointer_word_size() {
        // 不透明句柄须可无损存储于一个指针宽度的字中
        assert_eq!(
            std::mem::size_of::<*const aclTensor>(),
            std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<*const aclOpExecutor>(),
            std::mem::size_of::<usize>()
        );
    }
}

// 真机 smoke 测试：验证 aclnn 两段式符号已链接（链接级验证，不调用函数体，
// 因此不触碰 SDK 运行时），需要已链接 libascendcl 的环境，默认忽略。
#[cfg(all(cann_sys_ffi, test))]
mod ffi_tests {
    use super::*;

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_aclnn_matmul_symbols() {
        // 函数地址非零即说明符号已在链接产物中解析。
        assert_ne!(aclnnMatmulGetWorkspaceSize as *const () as usize, 0);
        assert_ne!(aclnnMatmul as *const () as usize, 0);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_aclnn_softmax_symbols() {
        assert_ne!(aclnnSoftmaxGetWorkspaceSize as *const () as usize, 0);
        assert_ne!(aclnnSoftmax as *const () as usize, 0);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_aclnn_rms_norm_symbols() {
        assert_ne!(aclnnRmsNormGetWorkspaceSize as *const () as usize, 0);
        assert_ne!(aclnnRmsNorm as *const () as usize, 0);
    }
}
