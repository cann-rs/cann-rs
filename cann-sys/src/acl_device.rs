//! ACL 设备与 SOC 管理（对应头文件 `acl_rt.h` 的 Device 管理小节）。
//!
//! 提供 `aclrtGetDeviceCount` / `aclrtSetDevice` / `aclrtResetDevice` /
//! `aclrtResetDeviceForce` / `aclrtGetSocName` / `aclrtSynchronizeDevice`
//! 等 L0 设备原语。FFI 声明仅在 `ffi` 特性下编译（`cann_sys_ffi` cfg）。
//!
//! 说明（源自 CANN 8.5 官方文档，详见 `docs/cann-850-catalog.md` §2 核定表）：
//! - **设备引用计数**：`aclrtSetDevice` 每次调用 +1，`aclrtResetDevice` 每次 −1，
//!   归零后才真正释放设备；复位前必须先析构该设备上显式创建的 Stream/Event/Context。
//! - **SOC 名**：`aclrtGetSocName` 无参、返回运行时持有的静态字符串指针
//!   （如 `"Ascend910B3"`），调用方不得释放。
//! - **线程语义**：ACL 的当前设备按线程绑定，跨线程使用前必须显式 `aclrtSetDevice`。

#[cfg(cann_sys_ffi)]
use crate::acl_base_rt::aclError;
#[cfg(cann_sys_ffi)]
use std::ffi::c_char;

// 设备管理 FFI 函数声明：后接属性宏块，用 `//` 避免 unused doc comment。
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    /// 查询设备总数（输出参数）。
    ///
    /// C 函数：`aclError aclrtGetDeviceCount(uint32_t *count)`
    /// 官方文档锚点：`aclcppdevg_03_0045`
    ///
    /// `count` 为输出参数，成功时写入本机可用设备数量。
    ///
    /// # Safety
    /// - `count` 必须指向有效的 `u32` 槽位，且不能为 NULL。
    pub fn aclrtGetDeviceCount(count: *mut u32) -> aclError;

    /// 指定当前线程使用某个设备（调用一次引用计数 +1）。
    ///
    /// C 函数：`aclError aclrtSetDevice(int32_t deviceId)`
    /// 官方文档锚点：`aclcppdevg_03_0039`
    ///
    /// `deviceId` 为设备逻辑 ID，合法范围 0..设备数−1。
    ///
    /// # Safety
    /// - 该调用绑定调用线程的当前设备；`deviceId` 必须是本机存在的设备。
    pub fn aclrtSetDevice(deviceId: i32) -> aclError;

    /// 释放当前线程对某设备的引用（调用一次引用计数 −1，归零才真正释放）。
    ///
    /// C 函数：`aclError aclrtResetDevice(int32_t deviceId)`
    /// 官方文档锚点：`aclcppdevg_03_0040`
    ///
    /// 复位前必须先析构该设备上显式创建的 Stream/Event/Context；
    /// 复位会释放默认 Context/Stream 及其下的所有 Stream。
    ///
    /// # Safety
    /// - 调用前必须确保该设备上不再有活跃的 Stream/Event/Context 句柄。
    pub fn aclrtResetDevice(deviceId: i32) -> aclError;

    /// 强制释放设备，无视引用计数（可选绑定）。
    ///
    /// C 函数：`aclError aclrtResetDeviceForce(int32_t deviceId)`
    /// 官方文档锚点：`aclcppdevg_03_0041`
    ///
    /// 强制释放指定设备；其它进程/线程持有的该设备句柄可能随之失效。
    ///
    /// # Safety
    /// - 仅在明确需要强制清理时调用；调用后不得再使用该设备上的既有句柄。
    pub fn aclrtResetDeviceForce(deviceId: i32) -> aclError;

    /// 获取当前设备对应的 SOC 型号名（返回静态指针，无参）。
    ///
    /// C 函数：`const char *aclrtGetSocName(void)`
    /// 官方文档锚点：`aclcppdevg_03_0048`
    ///
    /// 无参；返回指向运行时持有的静态字符串（如 `"Ascend910B3"`）的指针，
    /// 调用方不得释放，且只能在 `aclInit` 之后调用。
    ///
    /// # Safety
    /// - 返回值可能为 NULL，读取前必须判空。
    /// - 必须在 `aclInit` 成功之后、`aclFinalize` 之前调用。
    pub fn aclrtGetSocName() -> *const c_char;

    /// 同步等待当前线程设备上的所有任务完成（可选绑定）。
    ///
    /// C 函数：`aclError aclrtSynchronizeDevice(void)`
    /// 官方文档锚点：`aclcppdevg_03_0056`
    ///
    /// 阻塞直到当前设备上所有 Stream 的任务执行完成。
    ///
    /// # Safety
    /// - 须先通过 `aclrtSetDevice` 绑定设备后再调用。
    pub fn aclrtSynchronizeDevice() -> aclError;
}

#[cfg(all(cann_sys_ffi, test))]
mod tests {
    use super::*;
    use crate::acl_base_rt::*;
    use crate::acl_rt::{aclFinalize, aclInit};
    use std::ffi::CStr;

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_get_device_count() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        let mut count: u32 = 0;
        // SAFETY: `count` 指向有效的 `u32` 输出槽位。
        let ret = unsafe { aclrtGetDeviceCount(&mut count) };
        assert_eq!(ret, ACL_SUCCESS);
        assert!(count > 0, "device count should be > 0, got {count}");
        // SAFETY: 运行环境已初始化且无未释放资源。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_set_reset_device() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `deviceId = 0` 为设备逻辑 ID，调用后当前线程绑定设备 0。
        let ret = unsafe { aclrtSetDevice(0) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 当前线程已绑定设备 0，且其上无活跃 Stream/Event/Context。
        let ret = unsafe { aclrtResetDevice(0) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 设备已释放，运行环境可安全终结。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_synchronize_device() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `deviceId = 0` 为设备逻辑 ID，调用后当前线程绑定设备 0。
        let ret = unsafe { aclrtSetDevice(0) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 当前线程已绑定设备 0。
        let ret = unsafe { aclrtSynchronizeDevice() };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 当前线程已绑定设备 0，且其上无活跃 Stream/Event/Context。
        let ret = unsafe { aclrtResetDevice(0) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 设备已释放，运行环境可安全终结。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_link_get_soc_name() {
        // SAFETY: `aclInit(NULL)` 使用默认配置初始化运行环境。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 返回值由运行时持有，仅在本测试作用域内读取。
        let soc_name = unsafe { aclrtGetSocName() };
        assert!(!soc_name.is_null(), "soc name should not be null");
        // SAFETY: `soc_name` 已判空，且 `aclrtGetSocName` 保证返回 NUL 结尾的静态字符串。
        let name = unsafe { CStr::from_ptr(soc_name) };
        let name_str = name.to_str().unwrap();
        assert!(!name_str.is_empty(), "soc name should not be empty");
        assert!(
            name_str.starts_with("Ascend"),
            "unexpected soc name: {name_str}"
        );
        // SAFETY: 运行环境已初始化且无未释放资源。
        unsafe { aclFinalize(0) };
    }
}
