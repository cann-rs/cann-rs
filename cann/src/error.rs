//! CANN 错误类型。
//!
//! 提供统一的错误表示，支持从 `aclError` 转换和 Display 输出。

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
}
