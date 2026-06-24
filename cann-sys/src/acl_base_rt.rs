//! ACL 基础运行时类型与错误码定义。
//!
//! 对应 CANN 头文件 `acl_base_rt.h`，提供 `aclError` 类型和常用错误码。

use std::ffi::c_int;

/// ACL 操作返回码。
///
/// 对应 C 类型 `int`，遵循 CANN 的错误码约定：`ACL_SUCCESS` 表示成功，其他值为具体错误。
#[allow(non_camel_case_types)]
pub type aclError = c_int;

/// 操作成功。
pub const ACL_SUCCESS: aclError = 0;
/// 参数无效。
pub const ACL_ERROR_INVALID_PARAM: aclError = 100_000;
/// 模块未初始化。
pub const ACL_ERROR_UNINITIALIZE: aclError = 100_001;
/// 重复初始化。
pub const ACL_ERROR_REPEAT_INITIALIZE: aclError = 100_002;
/// 文件无效（通常因缺少 NPU 驱动导致）。
pub const ACL_ERROR_INVALID_FILE: aclError = 100_003;
/// 文件写入失败。
pub const ACL_ERROR_WRITE_FILE: aclError = 100_004;
/// 文件大小无效。
pub const ACL_ERROR_INVALID_FILE_SIZE: aclError = 100_005;
/// 文件解析失败。
pub const ACL_ERROR_PARSE_FILE: aclError = 100_006;
/// 文件缺少属性。
pub const ACL_ERROR_FILE_MISSING_ATTR: aclError = 100_007;
/// 文件属性无效。
pub const ACL_ERROR_FILE_ATTR_INVALID: aclError = 100_008;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_defs() {
        let _: aclError = 0;
    }

    #[test]
    fn test_acl_success() {
        assert_eq!(ACL_SUCCESS, 0);
    }

    #[test]
    fn test_acl_error_invalid_file() {
        assert_eq!(ACL_ERROR_INVALID_FILE, 100_003);
    }
}
