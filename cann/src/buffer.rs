//! CANN 内存缓冲区安全封装。
//!
//! `DeviceBuffer` 为设备侧（NPU）内存（`aclrtMalloc`/`aclrtFree`），`HostBuffer` 为
//! 可被设备直接访问的锁页主机内存（`aclrtMallocHost`/`aclrtFreeHost`）。
//!
//! 线程亲和性：设备内存与其归属设备相关，跨线程使用 `DeviceBuffer` 前必须在目标线程
//! `set_device`（见 [`crate::device`]）；锁页主机内存为 host 侧资源，不绑定设备。
//! 设备复位（`reset_device`）前必须先析构全部缓冲区。

use crate::error::Error;
#[cfg(feature = "ffi")]
use c_void;

/// 设备内存分配策略。
///
/// 对应 CANN `aclrtMemMallocPolicy` 枚举的 8 个取值（`ACL_MEM_MALLOC_*`）。
/// 带宽/权限标志（如 `ACL_MEM_TYPE_LOW_BAND_WIDTH`）为可按位组合的独立标志，与策略
/// 正交，暂未在封装层暴露——保持策略与标志分离，需要时可在后续版本扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemFlags {
    /// 优先大页（huge page）内存，大页不足时回退普通内存。
    HugeFirst,
    /// 仅使用大页内存。
    HugeOnly,
    /// 仅使用普通内存。
    NormalOnly,
    /// 优先大页 + `P2P`（点对点）。
    HugeFirstP2P,
    /// 仅大页 + `P2P`。
    HugeOnlyP2P,
    /// 仅普通内存 + `P2P`。
    NormalOnlyP2P,
    /// 仅使用 `1G` 大页。
    Huge1GOnly,
    /// 仅 `1G` 大页 + `P2P`。
    Huge1GOnlyP2P,
}

#[cfg(any(feature = "ffi", test))]
impl MemFlags {
    /// 映射为 CANN 分配策略常量（`ACL_MEM_MALLOC_*`，供 `aclrtMalloc` 使用）。
    pub(crate) fn as_policy(&self) -> cann_sys::acl_memory::aclrtMemMallocPolicy {
        match self {
            MemFlags::HugeFirst => cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_FIRST,
            MemFlags::HugeOnly => cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_ONLY,
            MemFlags::NormalOnly => cann_sys::acl_memory::ACL_MEM_MALLOC_NORMAL_ONLY,
            MemFlags::HugeFirstP2P => cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_FIRST_P2P,
            MemFlags::HugeOnlyP2P => cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_ONLY_P2P,
            MemFlags::NormalOnlyP2P => cann_sys::acl_memory::ACL_MEM_MALLOC_NORMAL_ONLY_P2P,
            MemFlags::Huge1GOnly => cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE1G_ONLY,
            MemFlags::Huge1GOnlyP2P => cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE1G_ONLY_P2P,
        }
    }
}

/// 设备侧（NPU）内存缓冲区（RAII）。
///
/// 构造时调用 `aclrtMalloc` 分配设备内存，析构时调用 `aclrtFree` 释放。用法：
/// `Context::new()` 与 `set_device` 之后分配，作为 kernel/拷贝操作的数据载体；
/// 通过 `as_ptr` 获取裸指针供后续 FFI 层使用。
///
/// 线程亲和性：本类型实现 `Send`（设备内存指针可由任意线程在绑定归属设备后使用/释放），
/// 但**不是** `Sync`——多线程并发读写同一缓冲区的同步由调用方负责。设备复位
/// （`reset_device`）前必须先析构本缓冲区；析构时若调用线程未绑定归属设备，
/// `aclrtFree` 可能失败（错误在 `Drop` 中忽略）。
#[derive(Debug)]
pub struct DeviceBuffer {
    #[cfg(feature = "ffi")]
    ptr: *mut u8,
    #[cfg(feature = "ffi")]
    len: usize,
}

// SAFETY: `DeviceBuffer` 可跨线程传递（`Send`）：C 侧设备内存句柄为裸指针，无借用关系，
// 任意线程在 `set_device` 绑定归属设备后均可读取/释放——但仅限用于归属设备，跨设备
// 使用由调用方保证不越界。多线程并发访问仍需外部同步，因此**不**实现 `Sync`。
#[cfg(feature = "ffi")]
unsafe impl Send for DeviceBuffer {}

#[cfg(feature = "ffi")]
impl DeviceBuffer {
    /// 在设备上分配 `size` 字节内存（对应 `aclrtMalloc`）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`；`flags` 指定分配策略。
    /// 失败时返回 `Err(Error)`（如内存不足 207001/207018 类错误码）。
    ///
    /// 线程亲和性：分配属于当前线程绑定设备的上下文；缓冲区只能在绑定归属设备的
    /// 线程上使用（或先在该线程 `set_device`）。
    pub fn alloc(size: usize, flags: MemFlags) -> Result<Self, Error> {
        let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        // SAFETY: `ptr` 指向有效的 `*mut std::ffi::c_void` 输出槽位；调用前需 `aclInit` +
        // `set_device`（文档已注明）。
        let ret = unsafe { cann_sys::acl_memory::aclrtMalloc(&mut ptr, size, flags.as_policy()) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(DeviceBuffer {
            ptr: ptr.cast::<u8>(),
            len: size,
        })
    }

    /// 返回设备内存起始地址（`*const u8`）。
    ///
    /// 设备内存不可从 host 直接解引用；本指针用于传给 FFI 层（如 `aclrtMemcpy`）。
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// 返回缓冲区大小（字节）。
    pub fn len(&self) -> usize {
        self.len
    }

    /// 返回缓冲区是否为空（`len == 0`）。
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(feature = "ffi")]
impl Drop for DeviceBuffer {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` 来自 `aclrtMalloc` 且未被析构；本类型持有唯一所有权。
        // 注意：设备复位前必须先析构本缓冲区；释放失败（如线程未绑定设备）时错误被忽略。
        let _ = unsafe { cann_sys::acl_memory::aclrtFree(self.ptr.cast::<c_void>()) };
    }
}

/// 锁页（pinned）主机内存缓冲区（RAII）。
///
/// 构造时调用 `aclrtMallocHost` 分配可被设备直接访问的主机内存，析构时调用
/// `aclrtFreeHost` 释放。用法：作为 host↔device 拷贝的源/目的缓冲区（配合
/// `aclrtMemcpy` 系列），`as_ptr`/`as_mut_ptr` 提供 host 侧读写入口。
///
/// 线程亲和性：锁页主机内存为 host 侧资源，不与设备绑定；本类型当前不实现
/// `Send`/`Sync`（保守默认），如需跨线程共享可后续扩展。
#[derive(Debug)]
pub struct HostBuffer {
    #[cfg(feature = "ffi")]
    ptr: *mut u8,
    #[cfg(feature = "ffi")]
    len: usize,
}

#[cfg(feature = "ffi")]
impl HostBuffer {
    /// 在 host 侧分配 `size` 字节锁页内存（对应 `aclrtMallocHost`）。
    ///
    /// 用法：需已完成 `Context::new()`；分配不依赖设备绑定。失败时返回 `Err(Error)`
    /// （如内存不足 207001 类错误码）。
    pub fn alloc(size: usize) -> Result<Self, Error> {
        let mut ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        // SAFETY: `ptr` 指向有效的 `*mut std::ffi::c_void` 输出槽位；调用前需 `aclInit`（文档已注明）。
        let ret = unsafe { cann_sys::acl_memory::aclrtMallocHost(&mut ptr, size) };
        if ret != cann_sys::ACL_SUCCESS {
            return Err(Error::from(ret));
        }
        Ok(HostBuffer {
            ptr: ptr.cast::<u8>(),
            len: size,
        })
    }

    /// 返回主机内存起始地址（只读视角）。
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// 返回主机内存起始地址（可写视角；host 侧内存可安全写入）。
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }

    /// 返回缓冲区大小（字节）。
    pub fn len(&self) -> usize {
        self.len
    }

    /// 返回缓冲区是否为空（`len == 0`）。
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(feature = "ffi")]
impl Drop for HostBuffer {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` 来自 `aclrtMallocHost` 且未被析构；本类型持有唯一所有权。
        // 释放失败时错误被忽略（host 内存释放不依赖设备绑定）。
        let _ = unsafe { cann_sys::acl_memory::aclrtFreeHost(self.ptr.cast::<c_void>()) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
/// 拷贝方向（对应 `ACL_MEMCPY_*` 常量）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemcpyKind {
    /// Host → Device
    HostToDevice,
    /// Device → Host
    DeviceToHost,
    /// Device → Device
    DeviceToDevice,
}

/// 同步拷贝原语（`aclrtMemcpy`，0105）。
///
/// 安全调用方（reinfer ascend 消费层）必须先在 `kernels::mem_check` 完成
/// 方向/边界/归属校验；本原语不重复校验。
///
/// # Safety
/// - `dst`/`src` 按 `kind` 必须为有效的 device/host 指针，`count <= destMax` 且不越源界；
/// - 涉及 device 端时，本线程已 `Context::new()` 并 `set_device`。
pub unsafe fn memcpy(
    kind: MemcpyKind,
    dst: *mut std::ffi::c_void,
    src: *const std::ffi::c_void,
    count: usize,
) -> Result<(), Error> {
    #[cfg(feature = "ffi")]
    {
        use cann_sys::{
            ACL_MEMCPY_DEVICE_TO_DEVICE, ACL_MEMCPY_DEVICE_TO_HOST, ACL_MEMCPY_HOST_TO_DEVICE,
        };
        let k = match kind {
            MemcpyKind::HostToDevice => ACL_MEMCPY_HOST_TO_DEVICE,
            MemcpyKind::DeviceToHost => ACL_MEMCPY_DEVICE_TO_HOST,
            MemcpyKind::DeviceToDevice => ACL_MEMCPY_DEVICE_TO_DEVICE,
        };
        let ret = unsafe { cann_sys::acl_memory::aclrtMemcpy(dst, count, src, count, k) };
        if ret == cann_sys::ACL_SUCCESS {
            Ok(())
        } else {
            Err(Error::from(ret))
        }
    }
    #[cfg(not(feature = "ffi"))]
    {
        let _ = (kind, dst, src, count);
        Err(unavailable())
    }
}

/// 异步拷贝原语（`aclrtMemcpyAsync`，0106；`stream` 上排队执行）。
///
/// # Safety
/// - `dst`/`src` 按 `kind` 必须为有效的 device/host 指针，`count <= destMax` 且不越源界；
/// - `stream` 必须指向有效流（或为默认流语义——校验与 [`memcpy`] 相同）；
/// - 涉及 device 端时，本线程已 `Context::new()` 并 `set_device`。
pub unsafe fn memcpy_async(
    kind: MemcpyKind,
    dst: *mut std::ffi::c_void,
    src: *const std::ffi::c_void,
    count: usize,
    stream: &crate::stream::Stream,
) -> Result<(), Error> {
    #[cfg(feature = "ffi")]
    {
        use cann_sys::{
            ACL_MEMCPY_DEVICE_TO_DEVICE, ACL_MEMCPY_DEVICE_TO_HOST, ACL_MEMCPY_HOST_TO_DEVICE,
        };
        let k = match kind {
            MemcpyKind::HostToDevice => ACL_MEMCPY_HOST_TO_DEVICE,
            MemcpyKind::DeviceToHost => ACL_MEMCPY_DEVICE_TO_HOST,
            MemcpyKind::DeviceToDevice => ACL_MEMCPY_DEVICE_TO_DEVICE,
        };
        let ret = unsafe {
            cann_sys::acl_memory::aclrtMemcpyAsync(dst, count, src, count, k, stream.raw_handle())
        };
        if ret == cann_sys::ACL_SUCCESS {
            Ok(())
        } else {
            Err(Error::from(ret))
        }
    }
    #[cfg(not(feature = "ffi"))]
    {
        let _ = (kind, dst, src, count, stream);
        Err(unavailable())
    }
}

#[cfg(not(feature = "ffi"))]
impl DeviceBuffer {
    /// 分配设备内存（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    pub fn alloc(_size: usize, _flags: MemFlags) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 返回设备内存起始地址（`ffi` 未启用时恒为 NULL）。
    pub fn as_ptr(&self) -> *const u8 {
        std::ptr::null()
    }

    /// 返回缓冲区大小（`ffi` 未启用时恒为 0）。
    pub fn len(&self) -> usize {
        0
    }

    /// 返回缓冲区是否为空（`ffi` 未启用时恒为 true）。
    pub fn is_empty(&self) -> bool {
        true
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl HostBuffer {
    /// 分配锁页主机内存（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    pub fn alloc(_size: usize) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 返回主机内存起始地址（`ffi` 未启用时恒为 NULL）。
    pub fn as_ptr(&self) -> *const u8 {
        std::ptr::null()
    }

    /// 返回主机内存起始地址（`ffi` 未启用时恒为 NULL）。
    pub fn as_mut_ptr(&self) -> *mut u8 {
        std::ptr::null_mut()
    }

    /// 返回缓冲区大小（`ffi` 未启用时恒为 0）。
    pub fn len(&self) -> usize {
        0
    }

    /// 返回缓冲区是否为空（`ffi` 未启用时恒为 true）。
    pub fn is_empty(&self) -> bool {
        true
    }
}

/// `ffi` 未启用时的错误（code 为 -1，非 ACL 码；message 为中文说明）。
#[cfg(not(feature = "ffi"))]
fn unavailable() -> Error {
    Error {
        code: -1,
        message: "cann ffi 特性未启用，请以 --features ffi 构建".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mem_flags_map_to_acl_policies() {
        assert_eq!(
            MemFlags::HugeFirst.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_FIRST
        );
        assert_eq!(
            MemFlags::HugeOnly.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_ONLY
        );
        assert_eq!(
            MemFlags::NormalOnly.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_NORMAL_ONLY
        );
        assert_eq!(
            MemFlags::HugeFirstP2P.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_FIRST_P2P
        );
        assert_eq!(
            MemFlags::HugeOnlyP2P.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE_ONLY_P2P
        );
        assert_eq!(
            MemFlags::NormalOnlyP2P.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_NORMAL_ONLY_P2P
        );
        assert_eq!(
            MemFlags::Huge1GOnly.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE1G_ONLY
        );
        assert_eq!(
            MemFlags::Huge1GOnlyP2P.as_policy(),
            cann_sys::acl_memory::ACL_MEM_MALLOC_HUGE1G_ONLY_P2P
        );
    }

    #[test]
    fn mem_flags_debug_format() {
        assert!(format!("{:?}", MemFlags::HugeFirst).contains("HugeFirst"));
    }

    #[cfg(not(feature = "ffi"))]
    mod fallback {
        use super::*;

        #[test]
        fn device_buffer_alloc_returns_err() {
            assert!(DeviceBuffer::alloc(64, MemFlags::HugeFirst).is_err());
        }

        #[test]
        fn host_buffer_alloc_returns_err() {
            assert!(HostBuffer::alloc(64).is_err());
        }
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;
    use crate::device::{reset_device, set_device};

    const BUF_SIZE: usize = 64;

    #[test]
    #[ignore = "requires NPU driver"]
    fn device_buffer_alloc_free_roundtrip() {
        let _ctx = crate::test_shared_ctx();
        set_device(0).unwrap();
        let buf = DeviceBuffer::alloc(BUF_SIZE, MemFlags::HugeFirst).unwrap();
        assert_eq!(buf.len(), BUF_SIZE);
        assert!(!buf.as_ptr().is_null());
        // 析构顺序：先缓冲区，后复位设备，最后 _ctx 触发 aclFinalize。
        drop(buf);
        reset_device(0).unwrap();
    }

    #[test]
    #[ignore = "requires NPU driver"]
    fn host_buffer_alloc_free_roundtrip() {
        let _ctx = crate::test_shared_ctx();
        set_device(0).unwrap();
        let buf = HostBuffer::alloc(BUF_SIZE).unwrap();
        assert_eq!(buf.len(), BUF_SIZE);
        assert!(!buf.as_ptr().is_null());
        drop(buf);
        reset_device(0).unwrap();
    }
}
