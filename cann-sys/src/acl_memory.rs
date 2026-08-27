//! ACL 内存管理原语（对应头文件 `acl_rt.h` 内存管理/数据传输小节）。
//!
//! 提供 `aclrtMalloc`/`aclrtFree`/`aclrtMallocHost`/`aclrtFreeHost`/`aclrtMemcpy`
//! 及 `ACL_MEM_MALLOC_*` / `ACL_MEMCPY_*` 常量。
//!
//! 注意：`ACL_MEM_*` 常量的数值尚未与头文件核对（verify-list 未清，
//! 见 `docs/cann-850-catalog.md` §2）。

use std::ffi::c_int;

/// 设备内存分配策略。
///
/// 对应 C 类型 `int`（C 枚举 `aclrtMemMallocPolicy` 的底层类型），
/// 取值见 `ACL_MEM_MALLOC_*` 常量（CANN 8.5 头文件 `acl_rt.h` 已核对）。
#[allow(non_camel_case_types)]
pub type aclrtMemMallocPolicy = c_int;

/// 分配策略：优先使用大页（huge）内存，大页不足时回退普通内存。
pub const ACL_MEM_MALLOC_HUGE_FIRST: aclrtMemMallocPolicy = 0;
/// 分配策略：仅使用大页内存。
pub const ACL_MEM_MALLOC_HUGE_ONLY: aclrtMemMallocPolicy = 1;
/// 分配策略：仅使用普通内存。
pub const ACL_MEM_MALLOC_NORMAL_ONLY: aclrtMemMallocPolicy = 2;
/// 分配策略：优先大页 + P2P（点对点）。
pub const ACL_MEM_MALLOC_HUGE_FIRST_P2P: aclrtMemMallocPolicy = 3;
/// 分配策略：仅大页 + P2P。
pub const ACL_MEM_MALLOC_HUGE_ONLY_P2P: aclrtMemMallocPolicy = 4;
/// 分配策略：仅普通内存 + P2P。
pub const ACL_MEM_MALLOC_NORMAL_ONLY_P2P: aclrtMemMallocPolicy = 5;
/// 分配策略：仅使用 1G 大页。
pub const ACL_MEM_MALLOC_HUGE1G_ONLY: aclrtMemMallocPolicy = 6;
/// 分配策略：仅 1G 大页 + P2P。
pub const ACL_MEM_MALLOC_HUGE1G_ONLY_P2P: aclrtMemMallocPolicy = 7;
/// 内存类型：低带宽（标志位，可与上述策略按位组合）。
pub const ACL_MEM_TYPE_LOW_BAND_WIDTH: aclrtMemMallocPolicy = 0x0100;
/// 内存类型：高带宽（标志位）。
pub const ACL_MEM_TYPE_HIGH_BAND_WIDTH: aclrtMemMallocPolicy = 0x1000;
/// 内存访问权限标志：用户空间只读。
pub const ACL_MEM_ACCESS_USER_SPACE_READONLY: aclrtMemMallocPolicy = 0x100000;

/// 内存拷贝方向。
///
/// 对应 C 类型 `int`（C 枚举 `aclrtMemcpyKind` 的底层类型），
/// 取值见 `ACL_MEMCPY_*` 常量（CANN 8.5 头文件 `acl_rt.h` 已核对）。
#[allow(non_camel_case_types)]
pub type aclrtMemcpyKind = c_int;

/// 拷贝方向：host 到 host。
pub const ACL_MEMCPY_HOST_TO_HOST: aclrtMemcpyKind = 0;
/// 拷贝方向：host 到 device。
pub const ACL_MEMCPY_HOST_TO_DEVICE: aclrtMemcpyKind = 1;
/// 拷贝方向：device 到 host。
pub const ACL_MEMCPY_DEVICE_TO_HOST: aclrtMemcpyKind = 2;
/// 拷贝方向：device 到 device。
pub const ACL_MEMCPY_DEVICE_TO_DEVICE: aclrtMemcpyKind = 3;
/// 拷贝方向：由系统根据源/目的地址自动判断。
pub const ACL_MEMCPY_DEFAULT: aclrtMemcpyKind = 4;
/// 拷贝方向：host → 中转 buffer → device。
pub const ACL_MEMCPY_HOST_TO_BUF_TO_DEVICE: aclrtMemcpyKind = 5;
/// 拷贝方向：同一设备内部 device 间。
pub const ACL_MEMCPY_INNER_DEVICE_TO_DEVICE: aclrtMemcpyKind = 6;
/// 拷贝方向：跨设备 device 间。
pub const ACL_MEMCPY_INTER_DEVICE_TO_DEVICE: aclrtMemcpyKind = 7;

#[cfg(cann_sys_ffi)]
use crate::acl_base_rt::aclError;
#[cfg(cann_sys_ffi)]
use std::ffi::c_void;

// `libascendcl` 内存管理 FFI 函数声明，仅在启用 `ffi` 特性时编译。
// 签名已对照官方 CANN 8.5 文档核实（aclcppdevg_03_0095/0100/0101/0102/0105），
// 见 docs/cann-850-catalog.md §2 verify-list。
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    /// C 函数原名：`aclrtMalloc`（aclcppdevg_03_0095）。
    ///
    /// 在设备（NPU）上分配 `size` 字节的设备侧内存，分配策略由 `policy` 指定。
    ///
    /// # 参数
    /// - `devPtr`：输出参数，指向接收设备内存地址的指针变量。
    /// - `size`：申请的内存大小（字节）。
    /// - `policy`：分配策略，取 `ACL_MEM_MALLOC_*` 常量。
    ///
    /// # Safety
    /// - `devPtr` 必须指向有效的 `*mut c_void` 变量。
    /// - 调用前需完成 `aclInit`，且当前线程已设置设备（`aclrtSetDevice`）。
    pub fn aclrtMalloc(
        devPtr: *mut *mut c_void,
        size: usize,
        policy: aclrtMemMallocPolicy,
    ) -> aclError;

    /// C 函数原名：`aclrtFree`（aclcppdevg_03_0100）。
    ///
    /// 释放 `aclrtMalloc` 分配的设备内存。
    ///
    /// # 参数
    /// - `devPtr`：`aclrtMalloc` 返回的设备内存地址。
    ///
    /// # Safety
    /// - `devPtr` 必须是由 `aclrtMalloc` 分配且尚未释放的地址（或 NULL）。
    /// - 重复释放同一地址、或释放非 `aclrtMalloc` 分配的内存，行为未定义。
    pub fn aclrtFree(devPtr: *mut c_void) -> aclError;

    /// C 函数原名：`aclrtMallocHost`（aclcppdevg_03_0101）。
    ///
    /// 在 host 侧分配可被 device 访问的锁定（pinned）内存。
    ///
    /// # 参数
    /// - `hostPtr`：输出参数，指向接收 host 内存地址的指针变量。
    /// - `size`：申请的内存大小（字节）。
    ///
    /// # Safety
    /// - `hostPtr` 必须指向有效的 `*mut c_void` 变量。
    /// - 调用前需完成 `aclInit`。
    pub fn aclrtMallocHost(hostPtr: *mut *mut c_void, size: usize) -> aclError;

    /// C 函数原名：`aclrtFreeHost`（aclcppdevg_03_0102）。
    ///
    /// 释放 `aclrtMallocHost` 分配的 host 内存。
    ///
    /// # 参数
    /// - `hostPtr`：`aclrtMallocHost` 返回的 host 内存地址。
    ///
    /// # Safety
    /// - `hostPtr` 必须是由 `aclrtMallocHost` 分配且尚未释放的地址（或 NULL）。
    /// - 重复释放同一地址、或释放非 `aclrtMallocHost` 分配的内存，行为未定义。
    pub fn aclrtFreeHost(hostPtr: *mut c_void) -> aclError;

    /// C 函数原名：`aclrtMemcpy`（aclcppdevg_03_0105）。
    ///
    /// 同步内存拷贝，`kind` 指定拷贝方向（host/device 组合）。
    ///
    /// # 参数
    /// - `dst`：目的地址。
    /// - `destMax`：目的缓冲区大小（字节）。
    /// - `src`：源地址。
    /// - `count`：拷贝的字节数，不得超过 `destMax`。
    /// - `kind`：拷贝方向，取 `ACL_MEMCPY_*` 常量。
    ///
    /// # Safety
    /// - `dst`/`src` 必须指向有效的、大小至少为 `count` 字节的内存区域。
    /// - `count` 不得大于 `destMax`。
    /// - 涉及 device 内存时，调用前需完成 `aclInit` 且当前线程已设置设备。
    /// - 源与目的重叠时行为未定义。
    pub fn aclrtMemcpy(
        dst: *mut c_void,
        destMax: usize,
        src: *const c_void,
        count: usize,
        kind: aclrtMemcpyKind,
    ) -> aclError;

    /// C 函数原名：`aclrtMemcpyAsync`（aclcppdevg_03_0106）。
    ///
    /// # Safety：与 `aclrtMemcpy` 相同；`stream` 为有效流句柄（NULL = 默认流）。
    pub fn aclrtMemcpyAsync(
        dst: *mut c_void,
        destMax: usize,
        src: *const c_void,
        count: usize,
        kind: aclrtMemcpyKind,
        stream: *mut c_void,
    ) -> aclError;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_aliases() {
        let _: aclrtMemMallocPolicy = ACL_MEM_MALLOC_HUGE_FIRST;
        let _: aclrtMemcpyKind = ACL_MEMCPY_HOST_TO_HOST;
        let _: aclrtMemMallocPolicy = 0;
        let _: aclrtMemcpyKind = 1;
    }

    #[test]
    fn test_mem_malloc_policy_constants() {
        // 数值按 CANN 8.5.0 acl_rt.h `aclrtMemMallocPolicy` 枚举核对
        assert_eq!(ACL_MEM_MALLOC_HUGE_FIRST, 0);
        assert_eq!(ACL_MEM_MALLOC_HUGE_ONLY, 1);
        assert_eq!(ACL_MEM_MALLOC_NORMAL_ONLY, 2);
        assert_eq!(ACL_MEM_MALLOC_HUGE_FIRST_P2P, 3);
        assert_eq!(ACL_MEM_MALLOC_HUGE_ONLY_P2P, 4);
        assert_eq!(ACL_MEM_MALLOC_NORMAL_ONLY_P2P, 5);
        assert_eq!(ACL_MEM_MALLOC_HUGE1G_ONLY, 6);
        assert_eq!(ACL_MEM_MALLOC_HUGE1G_ONLY_P2P, 7);
        assert_eq!(ACL_MEM_TYPE_LOW_BAND_WIDTH, 0x0100);
        assert_eq!(ACL_MEM_TYPE_HIGH_BAND_WIDTH, 0x1000);
        assert_eq!(ACL_MEM_ACCESS_USER_SPACE_READONLY, 0x100000);
    }

    #[test]
    fn test_memcpy_kind_constants() {
        // 数值按 CANN 8.5.0 acl_rt.h `aclrtMemcpyKind` 枚举核对
        assert_eq!(ACL_MEMCPY_HOST_TO_HOST, 0);
        assert_eq!(ACL_MEMCPY_HOST_TO_DEVICE, 1);
        assert_eq!(ACL_MEMCPY_DEVICE_TO_HOST, 2);
        assert_eq!(ACL_MEMCPY_DEVICE_TO_DEVICE, 3);
        assert_eq!(ACL_MEMCPY_DEFAULT, 4);
        assert_eq!(ACL_MEMCPY_HOST_TO_BUF_TO_DEVICE, 5);
        assert_eq!(ACL_MEMCPY_INNER_DEVICE_TO_DEVICE, 6);
        assert_eq!(ACL_MEMCPY_INTER_DEVICE_TO_DEVICE, 7);
    }
}

// 真机 smoke 测试：调用 FFI 函数，需要 NPU 驱动，默认忽略。
#[cfg(all(cann_sys_ffi, test))]
mod ffi_tests {
    use super::*;
    use crate::acl_base_rt::ACL_SUCCESS;
    use crate::acl_rt::{aclFinalize, aclInit};
    use std::ffi::c_void;

    const BUF_SIZE: usize = 64;

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_rt_malloc_free_roundtrip() {
        // SAFETY: `aclInit` 接受 NULL 使用默认配置，初始化后必须 `aclFinalize` 配对。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);

        let mut dev_ptr: *mut c_void = std::ptr::null_mut();
        // SAFETY: `dev_ptr` 指向有效的输出变量；需设备上下文，真机在已设置设备的线程运行。
        let ret = unsafe { aclrtMalloc(&mut dev_ptr, BUF_SIZE, ACL_MEM_MALLOC_HUGE_FIRST) };
        assert_eq!(ret, ACL_SUCCESS);
        assert!(!dev_ptr.is_null());

        // SAFETY: `dev_ptr` 来自本次 `aclrtMalloc`，未重复释放。
        let ret = unsafe { aclrtFree(dev_ptr) };
        assert_eq!(ret, ACL_SUCCESS);

        // SAFETY: 与 `aclInit` 配对，释放运行环境资源。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_rt_malloc_host_free_roundtrip() {
        // SAFETY: `aclInit` 接受 NULL 使用默认配置，初始化后必须 `aclFinalize` 配对。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);

        let mut host_ptr: *mut c_void = std::ptr::null_mut();
        // SAFETY: `host_ptr` 指向有效的输出变量。
        let ret = unsafe { aclrtMallocHost(&mut host_ptr, BUF_SIZE) };
        assert_eq!(ret, ACL_SUCCESS);
        assert!(!host_ptr.is_null());

        // SAFETY: `host_ptr` 来自本次 `aclrtMallocHost`，未重复释放。
        let ret = unsafe { aclrtFreeHost(host_ptr) };
        assert_eq!(ret, ACL_SUCCESS);

        // SAFETY: 与 `aclInit` 配对，释放运行环境资源。
        unsafe { aclFinalize(0) };
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn test_rt_memcpy_host_device_roundtrip() {
        // SAFETY: `aclInit` 接受 NULL 使用默认配置，初始化后必须 `aclFinalize` 配对。
        let ret = unsafe { aclInit(std::ptr::null()) };
        assert_eq!(ret, ACL_SUCCESS);

        let mut src_ptr: *mut c_void = std::ptr::null_mut();
        let mut dst_ptr: *mut c_void = std::ptr::null_mut();
        let mut dev_ptr: *mut c_void = std::ptr::null_mut();

        // SAFETY: `src_ptr`/`dst_ptr` 指向有效的输出变量。
        let ret = unsafe { aclrtMallocHost(&mut src_ptr, BUF_SIZE) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `dst_ptr` 指向有效的输出变量。
        let ret = unsafe { aclrtMallocHost(&mut dst_ptr, BUF_SIZE) };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: `dev_ptr` 指向有效的输出变量；需设备上下文，真机在已设置设备的线程运行。
        let ret = unsafe { aclrtMalloc(&mut dev_ptr, BUF_SIZE, ACL_MEM_MALLOC_HUGE_FIRST) };
        assert_eq!(ret, ACL_SUCCESS);

        // SAFETY: `src_ptr` 指向 `aclrtMallocHost` 分配的 64 字节内存，可写。
        let src = unsafe { std::slice::from_raw_parts_mut(src_ptr.cast::<u8>(), BUF_SIZE) };
        for (i, b) in src.iter_mut().enumerate() {
            *b = i as u8;
        }

        // SAFETY: 三段缓冲区大小均为 64 字节，`count` 不超过 `destMax`；
        // host 缓冲来自 `aclrtMallocHost`，device 缓冲来自 `aclrtMalloc`。
        let ret = unsafe {
            aclrtMemcpy(
                dev_ptr,
                BUF_SIZE,
                src_ptr,
                BUF_SIZE,
                ACL_MEMCPY_HOST_TO_DEVICE,
            )
        };
        assert_eq!(ret, ACL_SUCCESS);
        // SAFETY: 同上；方向为 device 到 host，`dst` 缓冲区未与 `src` 重叠。
        let ret = unsafe {
            aclrtMemcpy(
                dst_ptr,
                BUF_SIZE,
                dev_ptr,
                BUF_SIZE,
                ACL_MEMCPY_DEVICE_TO_HOST,
            )
        };
        assert_eq!(ret, ACL_SUCCESS);

        // SAFETY: `dst_ptr` 指向 `aclrtMallocHost` 分配的 64 字节内存，可读。
        let dst = unsafe { std::slice::from_raw_parts(dst_ptr.cast::<u8>(), BUF_SIZE) };
        assert_eq!(dst, src);

        // SAFETY: 以下指针均来自本次分配且未释放。
        let ret = unsafe { aclrtFreeHost(dst_ptr) };
        assert_eq!(ret, ACL_SUCCESS);
        let ret = unsafe { aclrtFreeHost(src_ptr) };
        assert_eq!(ret, ACL_SUCCESS);
        let ret = unsafe { aclrtFree(dev_ptr) };
        assert_eq!(ret, ACL_SUCCESS);

        // SAFETY: 与 `aclInit` 配对，释放运行环境资源。
        unsafe { aclFinalize(0) };
    }
}
