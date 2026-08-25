//! ACL 张量/标量等 aclnn 基础类型绑定。
//!
//! 对应 CANN 头文件 `include/aclnn/acl_meta.h`（CANN 8.5.0）。
//! 提供不透明句柄类型（`aclTensor`/`aclScalar`/`aclTensorList`/`aclIntArray`/
//! `aclFloatArray`/`aclBoolArray`）、`aclnnStatus` 返回码类型，以及这些类型的
//! 生命周期/创建/访问器 FFI 声明（全部为 `extern "C"` 链接，签名按本地 SDK 8.5.0
//! 头文件逐项核对，未核对的函数不在此列）。

use std::ffi::c_int;
use std::ffi::c_void;

/// aclnn 算子返回码。
///
/// 对应 C 类型 `int32_t`（acl_meta.h `typedef int32_t aclnnStatus;`）。
/// 0（C 常量 `OK`）表示成功，非 0 为具体错误码。
#[allow(non_camel_case_types)]
pub type aclnnStatus = c_int;

/// aclnn 张量句柄（不透明类型）。
///
/// 对应 acl_meta.h `typedef struct aclTensor aclTensor;`。
#[allow(non_camel_case_types)]
pub type aclTensor = c_void;

/// aclnn 标量句柄（不透明类型）。
///
/// 对应 acl_meta.h `typedef struct aclScalar aclScalar;`。
#[allow(non_camel_case_types)]
pub type aclScalar = c_void;

/// aclnn 张量列表句柄（不透明类型）。
///
/// 对应 acl_meta.h `typedef struct aclTensorList aclTensorList;`。
#[allow(non_camel_case_types)]
pub type aclTensorList = c_void;

/// aclnn 整数数组句柄（不透明类型）。
///
/// 对应 acl_meta.h `typedef struct aclIntArray aclIntArray;`。
#[allow(non_camel_case_types)]
pub type aclIntArray = c_void;

/// aclnn 浮点数组句柄（不透明类型）。
///
/// 对应 acl_meta.h `typedef struct aclFloatArray aclFloatArray;`。
#[allow(non_camel_case_types)]
pub type aclFloatArray = c_void;

/// aclnn 布尔数组句柄（不透明类型）。
///
/// 对应 acl_meta.h `typedef struct aclBoolArray aclBoolArray;`。
#[allow(non_camel_case_types)]
pub type aclBoolArray = c_void;

#[cfg(cann_sys_ffi)]
use crate::acl_datatype::{aclDataType, aclFormat};

// `libascendcl` aclnn 基础类型 FFI 声明（acl_meta.h，均为 `extern "C"`），仅在启用 `ffi` 特性时编译。
// 签名已按本地 CANN 8.5.0 头文件逐项核对；不透明句柄一律以 `c_void` 指针表示。
#[cfg(cann_sys_ffi)]
unsafe extern "C" {
    /// C 函数原名：`aclCreateTensor`（acl_meta.h）。
    ///
    /// 基于视图维度、步长、视图偏移、存储维度与数据地址创建一个 aclnn 张量句柄。
    ///
    /// # 参数
    /// - `viewDims`：视图维度数组，长度 `viewDimsNum`。
    /// - `viewDimsNum`：视图维度个数。
    /// - `dataType`：张量数据类型，取 `aclDataType` 常量。
    /// - `stride`：视图各维步长数组，长度 `viewDimsNum`。
    /// - `offset`：视图相对存储首地址的偏移（元素个数）。
    /// - `format`：张量内存布局格式，取 `aclFormat` 常量。
    /// - `storageDims`：存储维度数组，长度 `storageDimsNum`；可传 NULL（与视图同形）。
    /// - `storageDimsNum`：存储维度个数。
    /// - `tensorData`：张量数据所在内存地址（通常为设备内存）。
    ///
    /// # Safety
    /// - `viewDims`/`stride` 必须指向长度至少 `viewDimsNum` 的合法 `i64` 数组。
    /// - `storageDims` 非 NULL 时必须指向长度至少 `storageDimsNum` 的合法 `i64` 数组。
    /// - `tensorData` 指向的内存必须足以容纳 `storageDims` 所描述的数据（或传 NULL）。
    /// - 返回的句柄必须由 `aclDestroyTensor` 释放，且只能释放一次。
    pub fn aclCreateTensor(
        viewDims: *const i64,
        viewDimsNum: u64,
        dataType: aclDataType,
        stride: *const i64,
        offset: i64,
        format: aclFormat,
        storageDims: *const i64,
        storageDimsNum: u64,
        tensorData: *mut c_void,
    ) -> *mut c_void;

    /// C 函数原名：`aclCreateScalar`（acl_meta.h）。
    ///
    /// 以指定数据类型的值创建 aclnn 标量句柄。
    ///
    /// # 参数
    /// - `value`：指向标量值的指针（按 `dataType` 解释）。
    /// - `dataType`：标量数据类型，取 `aclDataType` 常量。
    ///
    /// # Safety
    /// - `value` 必须指向与 `dataType` 尺寸一致的合法内存（如 `ACL_INT32` 对应 `i32`）。
    /// - 返回的句柄必须由 `aclDestroyScalar` 释放，且只能释放一次。
    pub fn aclCreateScalar(value: *mut c_void, dataType: aclDataType) -> *mut c_void;

    /// C 函数原名：`aclCreateTensorList`（acl_meta.h）。
    ///
    /// 以 `size` 个张量句柄创建 aclnn 张量列表句柄。
    ///
    /// # 参数
    /// - `value`：指向 `size` 个 `aclTensor` 指针的数组（元素可含 NULL）。
    /// - `size`：张量个数。
    ///
    /// # Safety
    /// - `value` 必须指向长度至少 `size` 的合法指针数组。
    /// - 返回的句柄必须由 `aclDestroyTensorList` 释放，且只能释放一次。
    pub fn aclCreateTensorList(value: *const *const c_void, size: u64) -> *mut c_void;

    /// C 函数原名：`aclCreateIntArray`（acl_meta.h）。
    ///
    /// 以 `size` 个 `int64_t` 值创建 aclnn 整数数组句柄。
    ///
    /// # 参数
    /// - `value`：指向 `size` 个 `int64_t` 值的数组。
    /// - `size`：元素个数。
    ///
    /// # Safety
    /// - `value` 必须指向长度至少 `size` 的合法 `i64` 数组。
    /// - 返回的句柄必须由 `aclDestroyIntArray` 释放（属同一家族，见 acl_meta.h 生命周期函数）。
    pub fn aclCreateIntArray(value: *const i64, size: u64) -> *mut c_void;

    /// C 函数原名：`aclCreateFloatArray`（acl_meta.h）。
    ///
    /// 以 `size` 个 `float` 值创建 aclnn 浮点数组句柄。
    ///
    /// # 参数
    /// - `value`：指向 `size` 个 `float` 值的数组。
    /// - `size`：元素个数。
    ///
    /// # Safety
    /// - `value` 必须指向长度至少 `size` 的合法 `f32` 数组。
    pub fn aclCreateFloatArray(value: *const f32, size: u64) -> *mut c_void;

    /// C 函数原名：`aclCreateBoolArray`（acl_meta.h）。
    ///
    /// 以 `size` 个 `bool` 值创建 aclnn 布尔数组句柄。
    ///
    /// # 参数
    /// - `value`：指向 `size` 个 `bool` 值的数组。
    /// - `size`：元素个数。
    ///
    /// # Safety
    /// - `value` 必须指向长度至少 `size` 的合法 `bool` 数组，且每个元素为合法布尔值（0 或 1）。
    pub fn aclCreateBoolArray(value: *const bool, size: u64) -> *mut c_void;

    /// C 函数原名：`aclDestroyTensor`（acl_meta.h）。
    ///
    /// 释放 `aclCreateTensor` 创建的张量句柄。
    ///
    /// # 参数
    /// - `tensor`：待释放的张量句柄。
    ///
    /// # Safety
    /// - `tensor` 必须由 `aclCreateTensor` 返回且尚未释放（或为 NULL）。
    /// - 重复释放同一句柄、或释放非 `aclCreateTensor` 返回的指针，行为未定义。
    pub fn aclDestroyTensor(tensor: *const c_void) -> aclnnStatus;

    /// C 函数原名：`aclDestroyTensorList`（acl_meta.h）。
    ///
    /// 释放 `aclCreateTensorList` 创建的张量列表句柄。
    ///
    /// # 参数
    /// - `array`：待释放的张量列表句柄。
    ///
    /// # Safety
    /// - `array` 必须由 `aclCreateTensorList` 返回且尚未释放（或为 NULL）。
    pub fn aclDestroyTensorList(array: *const c_void) -> aclnnStatus;

    /// C 函数原名：`aclDestroyScalar`（acl_meta.h）。
    ///
    /// 释放 `aclCreateScalar` 创建的标量句柄。
    ///
    /// # 参数
    /// - `scalar`：待释放的标量句柄。
    ///
    /// # Safety
    /// - `scalar` 必须由 `aclCreateScalar` 返回且尚未释放（或为 NULL）。
    pub fn aclDestroyScalar(scalar: *const c_void) -> aclnnStatus;

    /// C 函数原名：`aclGetViewShape`（acl_meta.h）。
    ///
    /// 查询张量视图维度。
    ///
    /// # 参数
    /// - `tensor`：目标张量句柄。
    /// - `viewDims`：输出参数，接收指向视图维度数组的指针（归张量所有）。
    /// - `viewDimsNum`：输出参数，接收维度个数。
    ///
    /// # Safety
    /// - `tensor` 必须为 `aclCreateTensor` 返回且尚未释放的句柄。
    /// - `viewDims`/`viewDimsNum` 必须指向有效的输出变量。
    /// - 返回的 `*viewDims` 内存归张量所有：不得释放，且在 `aclDestroyTensor` 后失效。
    pub fn aclGetViewShape(
        tensor: *const c_void,
        viewDims: *mut *mut i64,
        viewDimsNum: *mut u64,
    ) -> aclnnStatus;

    /// C 函数原名：`aclGetStorageShape`（acl_meta.h）。
    ///
    /// 查询张量存储维度。
    ///
    /// # 参数
    /// - `tensor`：目标张量句柄。
    /// - `storageDims`：输出参数，接收指向存储维度数组的指针（归张量所有）。
    /// - `storageDimsNum`：输出参数，接收存储维度个数。
    ///
    /// # Safety
    /// - `tensor` 必须为 `aclCreateTensor` 返回且尚未释放的句柄。
    /// - `storageDims`/`storageDimsNum` 必须指向有效的输出变量。
    /// - 返回的 `*storageDims` 内存归张量所有：不得释放，且在 `aclDestroyTensor` 后失效。
    pub fn aclGetStorageShape(
        tensor: *const c_void,
        storageDims: *mut *mut i64,
        storageDimsNum: *mut u64,
    ) -> aclnnStatus;

    /// C 函数原名：`aclGetViewStrides`（acl_meta.h）。
    ///
    /// 查询张量视图各维步长。
    ///
    /// # 参数
    /// - `tensor`：目标张量句柄。
    /// - `stridesValue`：输出参数，接收指向步长数组的指针（归张量所有）。
    /// - `stridesNum`：输出参数，接收步长个数。
    ///
    /// # Safety
    /// - `tensor` 必须为 `aclCreateTensor` 返回且尚未释放的句柄。
    /// - `stridesValue`/`stridesNum` 必须指向有效的输出变量。
    /// - 返回的 `*stridesValue` 内存归张量所有：不得释放，且在 `aclDestroyTensor` 后失效。
    pub fn aclGetViewStrides(
        tensor: *const c_void,
        stridesValue: *mut *mut i64,
        stridesNum: *mut u64,
    ) -> aclnnStatus;

    /// C 函数原名：`aclGetViewOffset`（acl_meta.h）。
    ///
    /// 查询张量视图相对存储首地址的偏移（元素个数）。
    ///
    /// # 参数
    /// - `tensor`：目标张量句柄。
    /// - `offset`：输出参数，接收偏移值。
    ///
    /// # Safety
    /// - `tensor` 必须为 `aclCreateTensor` 返回且尚未释放的句柄。
    /// - `offset` 必须指向有效的输出变量。
    pub fn aclGetViewOffset(tensor: *const c_void, offset: *mut i64) -> aclnnStatus;

    /// C 函数原名：`aclGetFormat`（acl_meta.h）。
    ///
    /// 查询张量内存布局格式。
    ///
    /// # 参数
    /// - `tensor`：目标张量句柄。
    /// - `format`：输出参数，接收格式值（`aclFormat`）。
    ///
    /// # Safety
    /// - `tensor` 必须为 `aclCreateTensor` 返回且尚未释放的句柄。
    /// - `format` 必须指向有效的输出变量。
    pub fn aclGetFormat(tensor: *const c_void, format: *mut aclFormat) -> aclnnStatus;

    /// C 函数原名：`aclGetDataType`（acl_meta.h）。
    ///
    /// 查询张量数据类型。
    ///
    /// # 参数
    /// - `tensor`：目标张量句柄。
    /// - `dataType`：输出参数，接收数据类型值（`aclDataType`）。
    ///
    /// # Safety
    /// - `tensor` 必须为 `aclCreateTensor` 返回且尚未释放的句柄。
    /// - `dataType` 必须指向有效的输出变量。
    pub fn aclGetDataType(tensor: *const c_void, dataType: *mut aclDataType) -> aclnnStatus;

    /// C 函数原名：`aclGetTensorListSize`（acl_meta.h）。
    ///
    /// 查询张量列表中的张量个数。
    ///
    /// # 参数
    /// - `tensorList`：目标张量列表句柄。
    /// - `size`：输出参数，接收张量个数。
    ///
    /// # Safety
    /// - `tensorList` 必须为 `aclCreateTensorList` 返回且尚未释放的句柄。
    /// - `size` 必须指向有效的输出变量。
    pub fn aclGetTensorListSize(tensorList: *const c_void, size: *mut u64) -> aclnnStatus;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 不透明句柄类型与 `c_void` 指针同尺寸（别名保持成立）。
    #[test]
    fn test_opaque_types_are_pointer_sized() {
        assert_eq!(size_of::<*const aclTensor>(), size_of::<*const c_void>());
        assert_eq!(size_of::<*const aclScalar>(), size_of::<*const c_void>());
        assert_eq!(
            size_of::<*const aclTensorList>(),
            size_of::<*const c_void>()
        );
        assert_eq!(size_of::<*const aclIntArray>(), size_of::<*const c_void>());
        assert_eq!(
            size_of::<*const aclFloatArray>(),
            size_of::<*const c_void>()
        );
        assert_eq!(size_of::<*const aclBoolArray>(), size_of::<*const c_void>());
    }

    /// `aclnnStatus` 与 C `int32_t` 同尺寸。
    #[test]
    fn test_aclnn_status_is_i32() {
        assert_eq!(size_of::<aclnnStatus>(), size_of::<i32>());
    }
}

// FFI 链接冒烟测试：仅在启用 `ffi` 特性且运行 `cargo test` 时编译。
// 真实调用需要 NPU 驱动与已初始化的 ACL 环境，本机（无驱动）一律 `--ignored` 跳过，
// 仅作编译期/链接期验证。
#[cfg(all(cann_sys_ffi, test))]
mod ffi_tests {
    use super::*;
    use crate::acl_datatype::{ACL_FLOAT, ACL_FORMAT_ND, ACL_INT32};

    /// 链接冒烟：`aclCreateTensor` → `aclGetDataType` → `aclDestroyTensor` 往返。
    #[test]
    #[ignore = "requires NPU driver"]
    fn create_get_destroy_tensor() {
        let dims: [i64; 2] = [2, 2];
        let strides: [i64; 2] = [2, 1];
        let storage: [i64; 2] = [2, 2];
        unsafe {
            // SAFETY: dims/strides/storage 均为长度 2 的合法数组；tensorData 传 NULL（仅创建元数据）。
            let tensor = aclCreateTensor(
                dims.as_ptr(),
                2,
                ACL_FLOAT,
                strides.as_ptr(),
                0,
                ACL_FORMAT_ND,
                storage.as_ptr(),
                2,
                std::ptr::null_mut(),
            );
            let mut dt: aclDataType = -1;
            // SAFETY: tensor 由上面的 aclCreateTensor 返回且尚未释放；dt 为有效输出变量。
            let status = aclGetDataType(tensor, &mut dt);
            assert_eq!(status, 0);
            assert_eq!(dt, ACL_FLOAT);
            // SAFETY: tensor 由上面的 aclCreateTensor 返回，且本测试内仅释放一次。
            let status = aclDestroyTensor(tensor);
            assert_eq!(status, 0);
        }
    }

    /// 链接冒烟：`aclCreateScalar` → `aclDestroyScalar` 往返。
    #[test]
    #[ignore = "requires NPU driver"]
    fn create_destroy_scalar() {
        let mut value: i32 = 1;
        unsafe {
            // SAFETY: value 为合法的可写 i32 变量，内存布局与 ACL_INT32 一致。
            let scalar = aclCreateScalar(std::ptr::addr_of_mut!(value).cast(), ACL_INT32);
            // SAFETY: scalar 由上面的 aclCreateScalar 返回，且本测试内仅释放一次。
            let status = aclDestroyScalar(scalar);
            assert_eq!(status, 0);
        }
    }

    /// 链接冒烟：`aclCreateTensorList` → `aclGetTensorListSize` → `aclDestroyTensorList` 往返。
    #[test]
    #[ignore = "requires NPU driver"]
    fn create_get_destroy_tensor_list() {
        let dims: [i64; 1] = [4];
        let strides: [i64; 1] = [1];
        let storage: [i64; 1] = [4];
        unsafe {
            // SAFETY: dims/strides/storage 均为长度 1 的合法数组；tensorData 传 NULL（仅创建元数据）。
            let t1 = aclCreateTensor(
                dims.as_ptr(),
                1,
                ACL_FLOAT,
                strides.as_ptr(),
                0,
                ACL_FORMAT_ND,
                storage.as_ptr(),
                1,
                std::ptr::null_mut(),
            );
            // SAFETY: t1 由上面的 aclCreateTensor 返回；指针数组长度为 1 且元素有效。
            let tensor_ptr: *const c_void = t1;
            let list = aclCreateTensorList(&tensor_ptr as *const *const c_void, 1);
            let mut size: u64 = 0;
            // SAFETY: list 由上面的 aclCreateTensorList 返回且尚未释放；size 为有效输出变量。
            let status = aclGetTensorListSize(list, &mut size);
            assert_eq!(status, 0);
            assert_eq!(size, 1);
            // SAFETY: list 由上面的 aclCreateTensorList 返回，且本测试内仅释放一次。
            let status = aclDestroyTensorList(list);
            assert_eq!(status, 0);
            // SAFETY: t1 由上面的 aclCreateTensor 返回，且本测试内仅释放一次。
            let status = aclDestroyTensor(t1);
            assert_eq!(status, 0);
        }
    }
}
