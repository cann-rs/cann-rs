//! ACL 运行时 FFI 函数声明与版本查询常量。
//!
//! 对应 CANN 头文件 `acl_rt.h`，提供版本查询相关的 FFI 函数声明和常量。

#[cfg(cann_sys_ffi)]
use crate::acl_base_rt::aclError;
#[cfg(cann_sys_ffi)]
use std::ffi::{c_char, c_void};

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

/// CANN 包版本结构（对应 `aclCANNPackageVersion`，acl_rt.h）。
#[cfg(cann_sys_ffi)]
#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct aclCANNPackageVersion {
    /// 版本字符串（如 "8.5.0"）。
    pub version: [c_char; ACL_PKG_VERSION_MAX_SIZE],
    /// 主版本。
    pub majorVersion: [c_char; ACL_PKG_VERSION_PARTS_MAX_SIZE],
    /// 次版本。
    pub minorVersion: [c_char; ACL_PKG_VERSION_PARTS_MAX_SIZE],
    /// 发布版本。
    pub releaseVersion: [c_char; ACL_PKG_VERSION_PARTS_MAX_SIZE],
    /// 补丁版本。
    pub patchVersion: [c_char; ACL_PKG_VERSION_PARTS_MAX_SIZE],
    /// 保留。
    pub reserved: [c_char; ACL_PKG_VERSION_MAX_SIZE],
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
    // 存在性门控：aclsys* 为 CANN 8.x+ 符号；7.x 无（版本查询回退 aclrtGetVersion）。
    #[cfg(cann_sys_has_aclsysGetVersionStr)]
    pub fn aclsysGetVersionStr(pkgName: *const c_char, versionStr: *mut c_char) -> aclError;

    // 查询指定软件包的版本号（整数形式）。
    // # 安全性
    // - `pkgName` 必须是有效的 NUL 结尾 C 字符串。
    // - `versionNum` 必须指向有效的 `i32`。
    #[cfg(cann_sys_has_aclsysGetVersionStr)]
    pub fn aclsysGetVersionNum(pkgName: *const c_char, versionNum: *mut i32) -> aclError;

    // 旧版包版本查询（已废弃但 pyacl 沿用；以枚举取包名）。
    // # 安全性
    // - `name` 必须为合法包名枚举值。
    // - `version` 必须指向有效的 `aclCANNPackageVersion` 结构。
    #[cfg(cann_sys_has_aclsysGetVersionStr)]
    pub fn aclsysGetCANNVersion(
        name: aclCANNPackageName,
        version: *mut aclCANNPackageVersion,
    ) -> aclError;

    // 查询 ACL 运行时组件版本（需先调用 `aclInit`）。
    // # 安全性
    // - `majorVersion`、`minorVersion`、`patchVersion` 必须指向有效的 `i32`。
    // - 要求先调用 `aclInit()`。
    pub fn aclrtGetVersion(
        majorVersion: *mut i32,
        minorVersion: *mut i32,
        patchVersion: *mut i32,
    ) -> aclError;

    // ---- Stream / Event 管理（C 侧句柄均为 `void*`，Rust 侧用 `*mut c_void`） ----

    /// 创建 Stream 流对象（输出参数）。
    ///
    /// C 函数：`aclError aclrtCreateStream(aclrtStream *stream)`
    /// 官方文档锚点：`aclcppdevg_03_0066`
    ///
    /// `stream` 为输出参数：调用成功后写入新创建的流句柄。
    /// 句柄不再使用时必须通过 `aclrtDestroyStream` 释放；
    /// 流数量有上限（不同产品 512~2048，见官方文档）。
    ///
    /// # Safety
    /// - `stream` 必须指向有效的 `*mut c_void` 输出槽位，且不能为 NULL；
    ///   调用前无需初始化该槽位。
    pub fn aclrtCreateStream(stream: *mut *mut c_void) -> aclError;

    /// 销毁 Stream 流对象。
    ///
    /// C 函数：`aclError aclrtDestroyStream(aclrtStream stream)`
    /// 官方文档锚点：`aclcppdevg_03_0070`
    ///
    /// `stream` 为 `aclrtCreateStream` 创建的句柄；
    /// 销毁后不得再向该流提交任务或等待。
    ///
    /// # Safety
    /// - `stream` 必须是 `aclrtCreateStream` 成功返回的句柄；
    ///   调用时该流上不得存在正在等待的同步操作。
    pub fn aclrtDestroyStream(stream: *mut c_void) -> aclError;

    /// 同步等待指定 Stream 上的任务全部完成（可选绑定）。
    ///
    /// C 函数：`aclError aclrtSynchronizeStream(aclrtStream stream)`
    /// 官方文档锚点：`aclcppdevg_03_0076`
    ///
    /// `stream` 为流句柄；调用阻塞直到该流上所有任务执行完成。
    ///
    /// # Safety
    /// - `stream` 必须是有效的流句柄；不得在流自身的回调函数中调用。
    pub fn aclrtSynchronizeStream(stream: *mut c_void) -> aclError;

    /// 查询 Stream 上任务执行状态（可选绑定）。
    ///
    /// C 函数：`aclError aclrtStreamQuery(aclrtStream stream, uint32_t *status)`
    /// 官方文档锚点：`aclcppdevg_03_0075`
    ///
    /// `status` 为输出参数，写入流状态（0 表示该流上所有任务已完成，
    /// 非 0 表示任务仍在执行或执行失败）。
    ///
    /// # Safety
    /// - `stream` 必须是有效的流句柄。
    /// - `status` 必须指向有效的 `u32` 槽位，且不能为 NULL。
    pub fn aclrtStreamQuery(stream: *mut c_void, status: *mut u32) -> aclError;

    /// 创建 Event 事件对象（输出参数）。
    ///
    /// C 函数：`aclError aclrtCreateEvent(aclrtEvent *event)`
    /// 官方文档锚点：`aclcppdevg_03_0079`
    ///
    /// `event` 为输出参数：调用成功后写入新创建的事件句柄。
    /// 句柄不再使用时必须通过 `aclrtDestroyEvent` 释放；
    /// 事件数量有上限（不同产品 1023~65536，见官方文档）。
    ///
    /// # Safety
    /// - `event` 必须指向有效的 `*mut c_void` 输出槽位，且不能为 NULL；
    ///   调用前无需初始化该槽位。
    pub fn aclrtCreateEvent(event: *mut *mut c_void) -> aclError;

    /// 在指定 Stream 上记录一个 Event（事件置位）。
    ///
    /// C 函数：`aclError aclrtRecordEvent(aclrtEvent event, aclrtStream stream)`
    /// 官方文档锚点：`aclcppdevg_03_0083`
    ///
    /// `stream` 传 NULL 表示当前线程默认流；`event` 记录在该流上，
    /// 后续可通过 `aclrtSynchronizeEvent` / `aclrtStreamWaitEvent` 等待该点。
    ///
    /// # Safety
    /// - `event` 必须是 `aclrtCreateEvent` 成功返回的句柄。
    /// - `stream` 必须是有效的流句柄，或为 NULL（默认流）。
    pub fn aclrtRecordEvent(event: *mut c_void, stream: *mut c_void) -> aclError;

    /// 同步等待 Event 事件发生。
    ///
    /// C 函数：`aclError aclrtSynchronizeEvent(aclrtEvent event)`
    /// 官方文档锚点：`aclcppdevg_03_0088`
    ///
    /// `event` 为事件句柄；调用阻塞直到该事件被记录。
    ///
    /// # Safety
    /// - `event` 必须是 `aclrtCreateEvent` 成功返回的句柄；
    ///   该事件必须已通过 `aclrtRecordEvent` 记录。
    pub fn aclrtSynchronizeEvent(event: *mut c_void) -> aclError;

    /// 销毁 Event 事件对象。
    ///
    /// C 函数：`aclError aclrtDestroyEvent(aclrtEvent event)`
    /// 官方文档锚点：`aclcppdevg_03_0082`
    ///
    /// `event` 为 `aclrtCreateEvent` 创建的句柄；销毁后不得再等待该事件。
    ///
    /// # Safety
    /// - `event` 必须是 `aclrtCreateEvent` 成功返回的句柄，
    ///   且销毁时不存在对该事件的并发等待。
    pub fn aclrtDestroyEvent(event: *mut c_void) -> aclError;

    /// 指定 Stream 等待 Event 事件（可选绑定）。
    ///
    /// C 函数：`aclError aclrtStreamWaitEvent(aclrtStream stream, aclrtEvent event)`
    /// 官方文档锚点：`aclcppdevg_03_0091`
    ///
    /// `stream` 阻塞等待 `event` 被记录后再继续执行后续任务。
    ///
    /// # Safety
    /// - `stream` 必须是有效的流句柄。
    /// - `event` 必须是 `aclrtCreateEvent` 成功返回的句柄。
    pub fn aclrtStreamWaitEvent(stream: *mut c_void, event: *mut c_void) -> aclError;
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

    #[cfg(cann_sys_has_aclsysGetVersionStr)]
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

    #[cfg(cann_sys_has_aclsysGetVersionStr)]
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

    #[cfg(cann_sys_has_aclsysGetVersionStr)]
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

    #[cfg(cann_sys_has_aclsysGetVersionStr)]
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

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_create_destroy_stream() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut stream: *mut c_void = std::ptr::null_mut();
        // SAFETY: `stream` 指向有效的 `*mut c_void` 输出槽位，用于接收新流句柄。
        let ret = unsafe { aclrtCreateStream(&mut stream) };
        assert_eq!(ret, ACL_SUCCESS);
        assert!(!stream.is_null(), "created stream should not be null");
        // SAFETY: `stream` 是 `aclrtCreateStream` 成功返回的合法句柄，且无并发同步。
        let ret = unsafe { aclrtDestroyStream(stream) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 流已销毁，运行环境可安全终结。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_record_synchronize_event() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut stream: *mut c_void = std::ptr::null_mut();
        // SAFETY: `stream` 指向有效的 `*mut c_void` 输出槽位，用于接收新流句柄。
        let ret = unsafe { aclrtCreateStream(&mut stream) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut event: *mut c_void = std::ptr::null_mut();
        // SAFETY: `event` 指向有效的 `*mut c_void` 输出槽位，用于接收新事件句柄。
        let ret = unsafe { aclrtCreateEvent(&mut event) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `event` 为合法事件句柄；`stream` 为合法流句柄（非默认流）。
        let ret = unsafe { aclrtRecordEvent(event, stream) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `event` 已在本流上记录，可安全等待。
        let ret = unsafe { aclrtSynchronizeEvent(event) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `event` 为合法句柄且无并发等待。
        let ret = unsafe { aclrtDestroyEvent(event) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `stream` 为合法句柄且无并发同步。
        let ret = unsafe { aclrtDestroyStream(stream) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 流与事件均已销毁，运行环境可安全终结。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_stream_wait_event_and_query() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut stream: *mut c_void = std::ptr::null_mut();
        // SAFETY: `stream` 指向有效的 `*mut c_void` 输出槽位，用于接收新流句柄。
        let ret = unsafe { aclrtCreateStream(&mut stream) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut event: *mut c_void = std::ptr::null_mut();
        // SAFETY: `event` 指向有效的 `*mut c_void` 输出槽位，用于接收新事件句柄。
        let ret = unsafe { aclrtCreateEvent(&mut event) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `event` 为合法事件句柄；`stream` 为合法流句柄。
        let ret = unsafe { aclrtRecordEvent(event, stream) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `stream` 与 `event` 均为合法句柄，等待同一流上的事件。
        let ret = unsafe { aclrtStreamWaitEvent(stream, event) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut status: u32 = 0;
        // SAFETY: `stream` 为合法流句柄；`status` 指向有效的 `u32` 输出槽位。
        let ret = unsafe { aclrtStreamQuery(stream, &mut status) };
        assert_eq!(ret, ACL_SUCCESS);
        assert_eq!(status, 0, "stream should be idle after wait, got {status}");
        // SAFETY: `event` 为合法句柄且无并发等待。
        let ret = unsafe { aclrtDestroyEvent(event) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `stream` 为合法句柄且无并发同步。
        let ret = unsafe { aclrtDestroyStream(stream) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 流与事件均已销毁，运行环境可安全终结。
        unsafe { aclFinalize(0) };
    }
}

/// 无需 FFI 的类型级测试（无 SDK 环境下也运行）。
///
/// 流/事件句柄在 C 侧均为 `void*`，Rust 侧统一用 `*mut c_void` 表示；
/// 这里锁定句柄指针宽度的假设：句柄可无损存储于一个指针宽度的字中。
#[cfg(test)]
mod type_tests {
    #[test]
    fn test_handle_pointer_word_size() {
        assert_eq!(
            std::mem::size_of::<*mut std::ffi::c_void>(),
            std::mem::size_of::<usize>(),
            "opaque handles must fit in a pointer-sized word"
        );
    }
}
