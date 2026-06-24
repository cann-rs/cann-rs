use std::ffi::c_int;

// Allow non-CamelCase to match C header names exactly.
#[allow(non_camel_case_types)]
pub type aclError = c_int;

pub const ACL_SUCCESS: aclError = 0;
pub const ACL_ERROR_INVALID_PARAM: aclError = 100_000;
pub const ACL_ERROR_UNINITIALIZE: aclError = 100_001;
pub const ACL_ERROR_REPEAT_INITIALIZE: aclError = 100_002;
pub const ACL_ERROR_INVALID_FILE: aclError = 100_003;
pub const ACL_ERROR_WRITE_FILE: aclError = 100_004;
pub const ACL_ERROR_INVALID_FILE_SIZE: aclError = 100_005;
pub const ACL_ERROR_PARSE_FILE: aclError = 100_006;
pub const ACL_ERROR_FILE_MISSING_ATTR: aclError = 100_007;
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
