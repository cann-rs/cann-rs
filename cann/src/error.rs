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
}
