//! CANN 错误类型。
//!
//! 提供统一的错误表示，支持从 `aclError` / `aclnnStatus` / `graphStatus`
//! 三个错误族转换和 Display 输出。
//!
//! 错误族关系（cann-sys 定义）：
//! - `aclError`（L0 ACL 运行时）与 `aclnnStatus`（aclnn 算子 API）在
//!   cann-sys 中同为 `i32`（`c_int`）别名，`From<aclError>` 的实现即
//!   `From<i32>`，同时覆盖 aclnn 返回码的转换——无法也不应再写一个重复的
//!   `From<aclnnStatus>` 实现（E0119 重复实现冲突）。
//! - `graphStatus`（GE 图引擎）为 `u32`，经 `From<graphStatus>` 按
//!   `ge_error_codes.h` 常量表映射出可读名称后存入 `code`（`i32`）。
//! - 两族码进入 `Error` 后按现有 L0 白名单分类（`is_oom`/`is_recoverable`）：
//!   未知码返回 `false`（fail-closed），由上层按 Fatal 处理。已知限制：
//!   `aclnnStatus` 值域不公开，个别非零码可能与白名单码数值重合（如
//!   507000 恰为 `ACL_ERROR_RT_INTERNAL_ERROR`，命中可恢复白名单），
//!   错误族归属与重试语义应由调用方按错误码来源判断。

use cann_sys::acl_grph::graphStatus;
use cann_sys::aclError;

/// CANN 操作错误。
///
/// 包含错误码和可读的错误描述。
#[derive(Debug)]
pub struct Error {
    /// CANN 原生错误码。
    pub code: aclError,
    /// 错误描述（中文可读文本）。
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CANN 错误 ({}): {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl Error {
    /// 是否为 OOM（内存耗尽）类错误。
    ///
    /// 采用显式码段白名单，fail-closed：只有 0001 SDD Task 7 Error 分类表中
    /// 明确列入 OOM 类的 `ACL_ERROR_RT_*` 码返回 `true`，其余任何码（包括
    /// 同码段相邻的码和未来 SDK 新增码）一律返回 `false`，由上层按 Fatal 处理。
    ///
    /// L1 错误族扩展（0002 SDD Task 5）：`aclnnStatus`（与 `aclError` 同为
    /// `i32`）与 `graphStatus`（`u32` 转 `i32` 存入 `code`）的码进入本白名单
    /// 判断，未知码返回 `false`（Fatal，fail-closed）——L1 错误分类仍以 L0
    /// 白名单为准，不新增判断；如某码需纳入分类须显式评审加白名单。
    /// 数值重合的已知限制见模块文档。
    pub fn is_oom(&self) -> bool {
        // 白名单加码需评审：新增 SDK 错误码默认 Fatal（fail-closed），
        // 必须由人显式评审并加入白名单后才会命中本分类。
        matches!(
            self.code,
            cann_sys::acl_error_code::ACL_ERROR_RT_MEMORY_ALLOCATION
                | cann_sys::acl_error_code::ACL_ERROR_RT_MEMORY_FREE
                | cann_sys::acl_error_code::ACL_ERROR_RT_DEVICE_OOM
        )
    }

    /// 是否为可恢复错误（驱动/上下文重建类）。
    ///
    /// 对应 reinfer `LaunchError::Driver`：驱动或上下文重建后可能恢复，
    /// 调用方据此选择重建策略而非直接 Fatal。白名单收敛到驱动/执行类
    /// 507xxx 中的下列显式码，其余一律返回 `false`（fail-closed）。
    ///
    /// 取舍说明：`ACL_ERROR_RT_NO_DEVICE`（207004，无可用设备）未列入白名单。
    /// 它属于资源/内存类 207xxx 而非驱动类 507xxx，且"无设备"通常需要
    /// 外部插拔/上电设备才能恢复，重建上下文本身无法解决，故默认按 Fatal 处理，
    /// 由上层按资源类错误单独处理或重试，不纳入驱动重建语义。
    ///
    /// L1 错误族扩展（0002 SDD Task 5）：`aclnnStatus`/`graphStatus` 的码
    /// 按现有白名单判断，未知码一律返回 `false`（Fatal，fail-closed），
    /// 取舍与 `is_oom` 相同：L1 错误分类仍以 L0 白名单为准，不新增判断。
    /// 数值重合的已知限制见模块文档（如 aclnn 码 507000 恰为
    /// `ACL_ERROR_RT_INTERNAL_ERROR`，会命中本白名单）。
    pub fn is_recoverable(&self) -> bool {
        // 白名单加码需评审：新增 SDK 错误码默认 Fatal（fail-closed）。
        matches!(
            self.code,
            cann_sys::acl_error_code::ACL_ERROR_RT_INTERNAL_ERROR
                | cann_sys::acl_error_code::ACL_ERROR_RT_DEV_SETUP_ERROR
                | cann_sys::acl_error_code::ACL_ERROR_RT_DRV_INTERNAL_ERROR
                | cann_sys::acl_error_code::ACL_ERROR_RT_AICPU_INTERNAL_ERROR
                | cann_sys::acl_error_code::ACL_ERROR_RT_CONTEXT_RELEASE_ERROR
        )
    }
}

/// 从 `aclError`（L0 ACL 运行时）错误码转换。
///
/// 注：`aclnnStatus` 在 cann-sys 中与 `aclError` 同为 `i32`（`c_int`）别名，
/// 本实现即 `From<i32>`，同时承担 aclnn 返回码的转换：非 0 码经兜底分支
/// 得到含码值的 message；0（`ACLNN_SUCCESS`）为成功语义，由调用方判断，
/// 不构造 Error。
impl From<aclError> for Error {
    fn from(code: aclError) -> Self {
        let message = match code {
            cann_sys::ACL_ERROR_INVALID_PARAM => "参数无效".to_string(),
            cann_sys::ACL_ERROR_UNINITIALIZE => "模块未初始化".to_string(),
            cann_sys::ACL_ERROR_REPEAT_INITIALIZE => "重复初始化".to_string(),
            cann_sys::ACL_ERROR_INVALID_FILE => "文件无效（可能缺少 NPU 驱动）".to_string(),
            cann_sys::ACL_ERROR_WRITE_FILE => "文件写入失败".to_string(),
            cann_sys::ACL_ERROR_INVALID_FILE_SIZE => "文件大小无效".to_string(),
            cann_sys::ACL_ERROR_PARSE_FILE => "文件解析失败".to_string(),
            cann_sys::ACL_ERROR_FILE_MISSING_ATTR => "文件缺少属性".to_string(),
            cann_sys::ACL_ERROR_FILE_ATTR_INVALID => "文件属性无效".to_string(),
            _ => format!("未知 CANN 错误码: {}", code),
        };
        Error { code, message }
    }
}

/// 从 `graphStatus`（GE 图引擎）错误码转换。
///
/// 按 `cann_sys::acl_grph` 抄录的 `ge_error_codes.h` 常量表匹配出可读名称；
/// 未知码 fail-closed，落入含码值的通用 message。`code` 以 `i32` 存储
/// （`graphStatus` 为 `u32`，`GRAPH_FAILED`（`0xFFFFFFFF`）等大于 `i32::MAX`
/// 的值按位回绕为负值，可读名称由 `message` 保留）。`GRAPH_SUCCESS`（0）
/// 为成功语义，由调用方判断，此处仅按常量表给出可读名称。
impl From<graphStatus> for Error {
    fn from(code: graphStatus) -> Self {
        let message = match code {
            cann_sys::acl_grph::GRAPH_SUCCESS => "图引擎操作成功 (GRAPH_SUCCESS)".to_string(),
            cann_sys::acl_grph::GRAPH_FAILED => "图引擎操作失败 (GRAPH_FAILED)".to_string(),
            cann_sys::acl_grph::GRAPH_NOT_CHANGED => "图未发生变更 (GRAPH_NOT_CHANGED)".to_string(),
            cann_sys::acl_grph::GRAPH_PARAM_INVALID => "参数非法 (GRAPH_PARAM_INVALID)".to_string(),
            cann_sys::acl_grph::GRAPH_NODE_WITHOUT_CONST_INPUT => {
                "节点缺少常量输入 (GRAPH_NODE_WITHOUT_CONST_INPUT)".to_string()
            }
            cann_sys::acl_grph::GRAPH_NODE_NEED_REPASS => {
                "节点需要重新遍历 (GRAPH_NODE_NEED_REPASS)".to_string()
            }
            cann_sys::acl_grph::GRAPH_INVALID_IR_DEF => {
                "IR 定义非法 (GRAPH_INVALID_IR_DEF)".to_string()
            }
            cann_sys::acl_grph::OP_WITHOUT_IR_DATATYPE_INFER_RULE => {
                "算子缺少 IR 数据类型推断规则 (OP_WITHOUT_IR_DATATYPE_INFER_RULE)".to_string()
            }
            cann_sys::acl_grph::GRAPH_PARAM_OUT_OF_RANGE => {
                "参数超出范围 (GRAPH_PARAM_OUT_OF_RANGE)".to_string()
            }
            cann_sys::acl_grph::GRAPH_MEM_OPERATE_FAILED => {
                "内存操作失败 (GRAPH_MEM_OPERATE_FAILED)".to_string()
            }
            cann_sys::acl_grph::GRAPH_NULL_PTR => "空指针 (GRAPH_NULL_PTR)".to_string(),
            cann_sys::acl_grph::GRAPH_MEMCPY_FAILED => {
                "内存拷贝失败 (GRAPH_MEMCPY_FAILED)".to_string()
            }
            cann_sys::acl_grph::GRAPH_MEMSET_FAILED => {
                "内存置位失败 (GRAPH_MEMSET_FAILED)".to_string()
            }
            cann_sys::acl_grph::GRAPH_MATH_CAL_FAILED => {
                "数学计算失败 (GRAPH_MATH_CAL_FAILED)".to_string()
            }
            cann_sys::acl_grph::GRAPH_ADD_OVERFLOW => "加法溢出 (GRAPH_ADD_OVERFLOW)".to_string(),
            cann_sys::acl_grph::GRAPH_MUL_OVERFLOW => "乘法溢出 (GRAPH_MUL_OVERFLOW)".to_string(),
            cann_sys::acl_grph::GRAPH_RoundUp_Overflow => {
                "向上取整溢出 (GRAPH_RoundUp_Overflow)".to_string()
            }
            _ => format!("未知图引擎错误码: {}", code),
        };
        Error {
            code: code as aclError,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cann_sys::ACL_ERROR_INVALID_FILE;
    use cann_sys::acl_error_code::{
        ACL_ERROR_RT_AICORE_OVER_FLOW, ACL_ERROR_RT_AICPU_INTERNAL_ERROR,
        ACL_ERROR_RT_CONTEXT_RELEASE_ERROR, ACL_ERROR_RT_DEV_SETUP_ERROR, ACL_ERROR_RT_DEVICE_OOM,
        ACL_ERROR_RT_DRV_INTERNAL_ERROR, ACL_ERROR_RT_INTERNAL_ERROR, ACL_ERROR_RT_LOST_HEARTBEAT,
        ACL_ERROR_RT_MEMORY_ALLOCATION, ACL_ERROR_RT_MEMORY_FREE, ACL_ERROR_RT_NO_DEVICE,
        ACL_ERROR_RT_QUEUE_FULL,
    };
    use cann_sys::acl_grph::{
        GRAPH_ADD_OVERFLOW, GRAPH_FAILED, GRAPH_MEM_OPERATE_FAILED, GRAPH_NULL_PTR,
        GRAPH_PARAM_INVALID, graphStatus,
    };
    use cann_sys::aclnn_ops::aclnnStatus;

    #[test]
    fn test_error_from_invalid_file() {
        let e = Error::from(ACL_ERROR_INVALID_FILE);
        assert_eq!(e.code, ACL_ERROR_INVALID_FILE);
        assert_eq!(e.message, "文件无效（可能缺少 NPU 驱动）");
    }

    #[test]
    fn test_error_display_does_not_panic() {
        let e = Error::from(ACL_ERROR_INVALID_FILE);
        let s = format!("{}", e);
        assert!(s.contains("文件无效"));
    }

    #[test]
    fn test_error_from_unknown_code() {
        let e = Error::from(999_999);
        assert_eq!(e.code, 999_999);
        assert!(e.message.contains("999999"));
    }

    #[test]
    fn test_is_oom_whitelist_hits() {
        for code in [
            ACL_ERROR_RT_MEMORY_ALLOCATION,
            ACL_ERROR_RT_MEMORY_FREE,
            ACL_ERROR_RT_DEVICE_OOM,
        ] {
            assert!(Error::from(code).is_oom(), "码 {code} 应命中 OOM 白名单");
        }
    }

    #[test]
    fn test_is_recoverable_whitelist_hits() {
        for code in [
            ACL_ERROR_RT_INTERNAL_ERROR,
            ACL_ERROR_RT_DEV_SETUP_ERROR,
            ACL_ERROR_RT_DRV_INTERNAL_ERROR,
            ACL_ERROR_RT_AICPU_INTERNAL_ERROR,
            ACL_ERROR_RT_CONTEXT_RELEASE_ERROR,
        ] {
            assert!(
                Error::from(code).is_recoverable(),
                "码 {code} 应命中可恢复白名单"
            );
        }
    }

    #[test]
    fn test_adjacent_non_whitelist_codes_are_fatal() {
        // 白名单相邻但未列入的码：既非 OOM 也非可恢复（fail-closed）
        for code in [
            ACL_ERROR_RT_AICORE_OVER_FLOW, // 207003：OOM 码段相邻
            ACL_ERROR_RT_QUEUE_FULL,       // 207014：资源类相邻
            ACL_ERROR_RT_LOST_HEARTBEAT,   // 507010：可恢复白名单相邻
        ] {
            let e = Error::from(code);
            assert!(!e.is_oom(), "码 {code} 不应命中 OOM 白名单");
            assert!(!e.is_recoverable(), "码 {code} 不应命中可恢复白名单");
        }
    }

    #[test]
    fn test_no_device_is_not_recoverable() {
        // 取舍固化：ACL_ERROR_RT_NO_DEVICE(207004) 属资源类而非驱动类，
        // 不列入可恢复白名单，默认 Fatal。
        assert!(!Error::from(ACL_ERROR_RT_NO_DEVICE).is_recoverable());
        assert!(!Error::from(ACL_ERROR_RT_NO_DEVICE).is_oom());
    }

    #[test]
    fn test_unknown_codes_fail_closed() {
        // 未知码与未列入白名单的码：双双 false（fail-closed）
        for code in [999_999, 507_001, 207_099] {
            let e = Error::from(code);
            assert!(!e.is_oom(), "未知码 {code} 不应命中 OOM 白名单");
            assert!(!e.is_recoverable(), "未知码 {code} 不应命中可恢复白名单");
        }
    }

    #[test]
    fn test_from_aclnn_status_non_zero_failure_codes() {
        // aclnnStatus 与 aclError 同为 i32 别名，由 From<aclError> 承担转换；
        // 非 0 码一律映射为错误，Display 含码值。0（ACLNN_SUCCESS）为成功
        // 语义，由调用方处理，不进入 Error。
        for code in [1_i32, -1, 507_000] {
            let status: aclnnStatus = code;
            let e = Error::from(status);
            let s = format!("{}", e);
            assert!(
                s.contains(&code.to_string()),
                "Display 应含 aclnn 码值 {code}: {s}"
            );
            assert!(!e.is_oom(), "aclnn 码 {code} 不应命中 OOM 白名单");
        }
        // 已知限制（结构性事实，非缺陷）：aclnn 值域不公开，507000 恰与
        // ACL_ERROR_RT_INTERNAL_ERROR 数值重合，命中可恢复白名单；aclnn
        // 失败语义应由调用方按错误码来源判断，不依赖 L0 白名单分类。
        let status: aclnnStatus = 507_000;
        assert!(Error::from(status).is_recoverable());
    }

    #[test]
    fn test_from_graph_status_known_codes() {
        // 已知图码 → Display 含可读名称（常量名），且 is_oom/is_recoverable
        // 均为 false（fail-closed 交叉验证：L1 分类仍以 L0 白名单为准）。
        let cases = [
            (GRAPH_FAILED, "GRAPH_FAILED"),
            (GRAPH_PARAM_INVALID, "GRAPH_PARAM_INVALID"),
            (GRAPH_NULL_PTR, "GRAPH_NULL_PTR"),
            (GRAPH_MEM_OPERATE_FAILED, "GRAPH_MEM_OPERATE_FAILED"),
            (GRAPH_ADD_OVERFLOW, "GRAPH_ADD_OVERFLOW"),
        ];
        for (code, name) in cases {
            let e = Error::from(code);
            let s = format!("{}", e);
            assert!(s.contains(name), "Display 应含图常量名 {name}: {s}");
            assert!(!e.is_oom(), "{name} 不应命中 OOM 白名单");
            assert!(!e.is_recoverable(), "{name} 不应命中可恢复白名单");
        }
    }

    #[test]
    fn test_from_graph_status_unknown_code_fail_closed() {
        // 未知图码 → fail-closed 通用 message（含码值），分类双双 false。
        let status: graphStatus = 12_345;
        let e = Error::from(status);
        let s = format!("{}", e);
        assert!(s.contains("12345"), "Display 应含未知码值: {s}");
        assert!(
            s.contains("未知图引擎错误码"),
            "Display 应含通用失败 message: {s}"
        );
        assert!(!e.is_oom(), "未知图码不应命中 OOM 白名单");
        assert!(!e.is_recoverable(), "未知图码不应命中可恢复白名单");
    }
}
