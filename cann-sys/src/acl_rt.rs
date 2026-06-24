//! ACL 运行时 FFI 函数声明与版本查询常量。
//!
//! 对应 CANN 头文件 `acl_rt.h`，提供版本查询相关的 FFI 函数声明和常量。

use crate::acl_base_rt::aclError;
use std::ffi::c_char;

/// 版本字符串缓冲区大小（字节）。
pub const ACL_PKG_VERSION_MAX_SIZE: usize = 128;
/// 版本各部分字符串缓冲区大小（字节）。
pub const ACL_PKG_VERSION_PARTS_MAX_SIZE: usize = 64;

/// CANN 软件包名称枚举。
///
/// 对应 C 枚举 `aclCANNPackageName`，用于标识需要查询版本的组件。
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum aclCANNPackageName {
    /// CANN 主包。
    ACL_PKG_NAME_CANN,
    /// 运行时组件。
    ACL_PKG_NAME_RUNTIME,
    /// 编译器组件。
    ACL_PKG_NAME_COMPILER,
    /// HCCL 通信库。
    ACL_PKG_NAME_HCCL,
    /// 工具包。
    ACL_PKG_NAME_TOOLKIT,
    /// OPP 算子包。
    ACL_PKG_NAME_OPP,
    /// OPP 算子内核包。
    ACL_PKG_NAME_OPP_KERNEL,
    /// 驱动。
    ACL_PKG_NAME_DRIVER,
}

// `libascendcl` FFI 函数声明，仅在启用 `ffi` 特性时编译。
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    // 初始化 CANN 运行环境，必须在其他 ACL API 之前调用。
    // # 安全性
    // - `configPath` 可以传入 NULL 使用默认配置。
    pub fn aclInit(configPath: *const c_char) -> aclError;

    // 释放 CANN 运行环境资源。
    // # 安全性
    // - `deviceId` 传入 0 即可。
    pub fn aclFinalize(deviceId: i32) -> aclError;

    // 查询指定软件包的版本字符串。
    // # 安全性
    // - `pkgName` 必须是有效的 NUL 结尾 C 字符串。
    // - `versionStr` 必须指向至少 `ACL_PKG_VERSION_MAX_SIZE` 字节的缓冲区。
    pub fn aclsysGetVersionStr(pkgName: *const c_char, versionStr: *mut c_char) -> aclError;

    // 查询指定软件包的版本号（整数形式）。
    // # 安全性
    // - `pkgName` 必须是有效的 NUL 结尾 C 字符串。
    // - `versionNum` 必须指向有效的 `i32`。
    pub fn aclsysGetVersionNum(pkgName: *const c_char, versionNum: *mut i32) -> aclError;

    // 查询 ACL 运行时组件版本（需先调用 `aclInit`）。
    // # 安全性
    // - `majorVersion`、`minorVersion`、`patchVersion` 必须指向有效的 `i32`。
    // - 要求先调用 `aclInit()`。
    pub fn aclrtGetVersion(
        majorVersion: *mut i32,
        minorVersion: *mut i32,
        patchVersion: *mut i32,
    ) -> aclError;
}

#[cfg(all(cann_sys_ffi, test))]
mod tests {
    use super::*;
    use crate::acl_base_rt::*;
    use std::ffi::CString;

    #[test]
    fn test_version_max_size() {
        assert_eq!(ACL_PKG_VERSION_MAX_SIZE, 128);
    }

    #[test]
    fn test_version_parts_max_size() {
        assert_eq!(ACL_PKG_VERSION_PARTS_MAX_SIZE, 64);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_sys_get_version_str() {
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let pkg_name = CString::new("CANN").unwrap();
        let mut buf = [0u8; ACL_PKG_VERSION_MAX_SIZE];
        let ret =
            unsafe { aclsysGetVersionStr(pkg_name.as_ptr(), buf.as_mut_ptr() as *mut c_char) };
        unsafe { aclFinalize(0) };
        assert_eq!(ret, ACL_SUCCESS);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_sys_get_version_str_content() {
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let pkg_name = CString::new("CANN").unwrap();
        let mut buf = [0u8; ACL_PKG_VERSION_MAX_SIZE];
        let ret =
            unsafe { aclsysGetVersionStr(pkg_name.as_ptr(), buf.as_mut_ptr() as *mut c_char) };
        unsafe { aclFinalize(0) };
        assert_eq!(ret, ACL_SUCCESS);
        let version = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const c_char) };
        let version_str = version.to_str().unwrap();
        assert!(
            !version_str.is_empty(),
            "version string should not be empty"
        );
        assert!(
            version_str.contains('.'),
            "version string should contain dots"
        );
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_sys_get_version_num() {
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let pkg_name = CString::new("CANN").unwrap();
        let mut num: i32 = 0;
        let ret = unsafe { aclsysGetVersionNum(pkg_name.as_ptr(), &mut num) };
        unsafe { aclFinalize(0) };
        assert_eq!(ret, ACL_SUCCESS);
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_sys_get_version_num_plausible() {
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let pkg_name = CString::new("CANN").unwrap();
        let mut num: i32 = 0;
        let ret = unsafe { aclsysGetVersionNum(pkg_name.as_ptr(), &mut num) };
        unsafe { aclFinalize(0) };
        assert_eq!(ret, ACL_SUCCESS);
        assert!(
            (80_000_000..100_000_000).contains(&num),
            "version num {} out of expected range for CANN 8.x or 9.x",
            num
        );
    }
}
