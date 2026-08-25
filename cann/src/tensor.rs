//! ACL 张量/张量列表/标量等 aclnn 基础类型的安全封装。
//!
//! `Tensor` 为 `aclCreateTensor`/`aclDestroyTensor` 的 RAII 封装，附带元数据查询
//! （视图形状/数据类型/内存格式）；`TensorList` 与 `Scalar` 为对应句柄的简化 RAII。
//! `DataType`/`Format` 为 CANN `aclDataType`/`aclFormat` 枚举的安全映射。
//!
//! 线程亲和性：张量句柄与创建线程当前绑定的设备上下文相关（`set_device` 见
//! [`crate::device`]），跨线程使用前必须在目标线程 `set_device`。本模块类型不实现
//! `Send`/`Sync`，同一张量的并发操作需要外部同步。

use crate::error::Error;
use cann_sys::acl_datatype as acl;
use std::ffi::c_void;

/// 张量数据类型（安全映射 CANN `aclDataType` 枚举，见 cann-sys `acl_datatype`）。
///
/// 覆盖 aclnn 算子常用的数值类型；未映射的 ACL 类型（如 `ACL_STRING`/复数）
/// 由 [`DataType::from_acl`] 返回 `None`（fail-closed）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// 16 位浮点（FP16）。
    Fp16,
    /// 32 位浮点（FP32）。
    Fp32,
    /// 64 位浮点（FP64）。
    Fp64,
    /// bfloat16 浮点。
    Bf16,
    /// 8 位有符号整数。
    Int8,
    /// 16 位有符号整数。
    Int16,
    /// 32 位有符号整数。
    Int32,
    /// 64 位有符号整数。
    Int64,
    /// 8 位无符号整数。
    Uint8,
    /// 16 位无符号整数。
    Uint16,
    /// 32 位无符号整数。
    Uint32,
    /// 64 位无符号整数。
    Uint64,
    /// 布尔。
    Bool,
    /// 8 位浮点（E5M2）。
    Float8E5m2,
    /// 8 位浮点（E4M3FN）。
    Float8E4m3fn,
}

impl DataType {
    /// 映射为 CANN `aclDataType` 常量（供 `aclCreateTensor` 等 FFI 调用使用）。
    pub fn as_acl(&self) -> acl::aclDataType {
        match self {
            DataType::Fp16 => acl::ACL_FLOAT16,
            DataType::Fp32 => acl::ACL_FLOAT,
            DataType::Fp64 => acl::ACL_DOUBLE,
            DataType::Bf16 => acl::ACL_BF16,
            DataType::Int8 => acl::ACL_INT8,
            DataType::Int16 => acl::ACL_INT16,
            DataType::Int32 => acl::ACL_INT32,
            DataType::Int64 => acl::ACL_INT64,
            DataType::Uint8 => acl::ACL_UINT8,
            DataType::Uint16 => acl::ACL_UINT16,
            DataType::Uint32 => acl::ACL_UINT32,
            DataType::Uint64 => acl::ACL_UINT64,
            DataType::Bool => acl::ACL_BOOL,
            DataType::Float8E5m2 => acl::ACL_FLOAT8_E5M2,
            DataType::Float8E4m3fn => acl::ACL_FLOAT8_E4M3FN,
        }
    }

    /// 从 CANN `aclDataType` 常量反向映射。
    ///
    /// 未映射的类型返回 `None`（fail-closed），由调用方决定如何处理。
    pub fn from_acl(dt: acl::aclDataType) -> Option<Self> {
        match dt {
            acl::ACL_FLOAT16 => Some(DataType::Fp16),
            acl::ACL_FLOAT => Some(DataType::Fp32),
            acl::ACL_DOUBLE => Some(DataType::Fp64),
            acl::ACL_BF16 => Some(DataType::Bf16),
            acl::ACL_INT8 => Some(DataType::Int8),
            acl::ACL_INT16 => Some(DataType::Int16),
            acl::ACL_INT32 => Some(DataType::Int32),
            acl::ACL_INT64 => Some(DataType::Int64),
            acl::ACL_UINT8 => Some(DataType::Uint8),
            acl::ACL_UINT16 => Some(DataType::Uint16),
            acl::ACL_UINT32 => Some(DataType::Uint32),
            acl::ACL_UINT64 => Some(DataType::Uint64),
            acl::ACL_BOOL => Some(DataType::Bool),
            acl::ACL_FLOAT8_E5M2 => Some(DataType::Float8E5m2),
            acl::ACL_FLOAT8_E4M3FN => Some(DataType::Float8E4m3fn),
            _ => None,
        }
    }
}

/// 张量内存布局格式（安全映射 CANN `aclFormat` 枚举，见 cann-sys `acl_datatype`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// NCHW 四维格式。
    Nchw,
    /// NHWC 四维格式。
    Nhwc,
    /// 通用 N 维格式。
    Nd,
    /// 5D 格式（C1=C/16，C0=16）。
    Nc1Hwc0,
    /// 分形 Z（小方块）格式。
    FractalZ,
    /// NC1HWC0 变体（C0=4）。
    Nc1Hwc0C04,
    /// HWCN 格式。
    Hwcn,
    /// NDHWC 5D 格式。
    Ndhwc,
    /// 分形 NZ（大数据块）格式。
    FractalNz,
    /// NCDHW 5D 格式。
    Ncdhw,
    /// NDC1HWC0 格式。
    Ndc1Hwc0,
    /// 3D 分形 Z 格式。
    FractalZ3d,
    /// NC 二维格式。
    Nc,
    /// NCL 三维格式。
    Ncl,
    /// 分形 NZ（C0=16）。
    FractalNzC0_16,
    /// 分形 NZ（C0=32）。
    FractalNzC0_32,
    /// 分形 NZ（C0=2）。
    FractalNzC0_2,
    /// 分形 NZ（C0=4）。
    FractalNzC0_4,
    /// 分形 NZ（C0=8）。
    FractalNzC0_8,
}

impl Format {
    /// 映射为 CANN `aclFormat` 常量（供 `aclCreateTensor` 等 FFI 调用使用）。
    pub fn as_acl(&self) -> acl::aclFormat {
        match self {
            Format::Nchw => acl::ACL_FORMAT_NCHW,
            Format::Nhwc => acl::ACL_FORMAT_NHWC,
            Format::Nd => acl::ACL_FORMAT_ND,
            Format::Nc1Hwc0 => acl::ACL_FORMAT_NC1HWC0,
            Format::FractalZ => acl::ACL_FORMAT_FRACTAL_Z,
            Format::Nc1Hwc0C04 => acl::ACL_FORMAT_NC1HWC0_C04,
            Format::Hwcn => acl::ACL_FORMAT_HWCN,
            Format::Ndhwc => acl::ACL_FORMAT_NDHWC,
            Format::FractalNz => acl::ACL_FORMAT_FRACTAL_NZ,
            Format::Ncdhw => acl::ACL_FORMAT_NCDHW,
            Format::Ndc1Hwc0 => acl::ACL_FORMAT_NDC1HWC0,
            Format::FractalZ3d => acl::ACL_FRACTAL_Z_3D,
            Format::Nc => acl::ACL_FORMAT_NC,
            Format::Ncl => acl::ACL_FORMAT_NCL,
            Format::FractalNzC0_16 => acl::ACL_FORMAT_FRACTAL_NZ_C0_16,
            Format::FractalNzC0_32 => acl::ACL_FORMAT_FRACTAL_NZ_C0_32,
            Format::FractalNzC0_2 => acl::ACL_FORMAT_FRACTAL_NZ_C0_2,
            Format::FractalNzC0_4 => acl::ACL_FORMAT_FRACTAL_NZ_C0_4,
            Format::FractalNzC0_8 => acl::ACL_FORMAT_FRACTAL_NZ_C0_8,
        }
    }

    /// 从 CANN `aclFormat` 常量反向映射。
    ///
    /// 未映射的格式返回 `None`（fail-closed）。
    pub fn from_acl(fmt: acl::aclFormat) -> Option<Self> {
        match fmt {
            acl::ACL_FORMAT_NCHW => Some(Format::Nchw),
            acl::ACL_FORMAT_NHWC => Some(Format::Nhwc),
            acl::ACL_FORMAT_ND => Some(Format::Nd),
            acl::ACL_FORMAT_NC1HWC0 => Some(Format::Nc1Hwc0),
            acl::ACL_FORMAT_FRACTAL_Z => Some(Format::FractalZ),
            acl::ACL_FORMAT_NC1HWC0_C04 => Some(Format::Nc1Hwc0C04),
            acl::ACL_FORMAT_HWCN => Some(Format::Hwcn),
            acl::ACL_FORMAT_NDHWC => Some(Format::Ndhwc),
            acl::ACL_FORMAT_FRACTAL_NZ => Some(Format::FractalNz),
            acl::ACL_FORMAT_NCDHW => Some(Format::Ncdhw),
            acl::ACL_FORMAT_NDC1HWC0 => Some(Format::Ndc1Hwc0),
            acl::ACL_FRACTAL_Z_3D => Some(Format::FractalZ3d),
            acl::ACL_FORMAT_NC => Some(Format::Nc),
            acl::ACL_FORMAT_NCL => Some(Format::Ncl),
            acl::ACL_FORMAT_FRACTAL_NZ_C0_16 => Some(Format::FractalNzC0_16),
            acl::ACL_FORMAT_FRACTAL_NZ_C0_32 => Some(Format::FractalNzC0_32),
            acl::ACL_FORMAT_FRACTAL_NZ_C0_2 => Some(Format::FractalNzC0_2),
            acl::ACL_FORMAT_FRACTAL_NZ_C0_4 => Some(Format::FractalNzC0_4),
            acl::ACL_FORMAT_FRACTAL_NZ_C0_8 => Some(Format::FractalNzC0_8),
            _ => None,
        }
    }
}

/// aclnn 张量（RAII）。
///
/// 构造时调用 `aclCreateTensor` 创建张量元数据（视图维度/步长/偏移/格式等），
/// 析构时调用 `aclDestroyTensor` 释放。`data` 指向的数据内存由调用方另行管理
/// （本类型只拥有元数据句柄，不拥有数据）。
///
/// 线程亲和性：不实现 `Send`/`Sync`；张量属于创建线程当前绑定的设备上下文，
/// 跨线程使用前必须在目标线程 `set_device`。设备复位（`reset_device`）前必须先
/// 析构本张量。
#[derive(Debug)]
pub struct Tensor {
    #[cfg(feature = "ffi")]
    handle: *mut c_void,
}

#[cfg(feature = "ffi")]
impl Tensor {
    /// 创建张量元数据（对应 `aclCreateTensor`）。
    ///
    /// `dims` 为视图各维长度；`dt`/`fmt` 为数据类型与内存格式；`offset` 为视图相对
    /// 存储首地址的元素偏移；`strides` 为各维步长（`None` 时按行主序连续布局处理）；
    /// `storage_dims` 为存储维度（`None` 时与视图同形）；`data` 为张量数据所在内存
    /// 起始地址（通常为 [`crate::buffer::DeviceBuffer`] 等设备内存，可传 NULL 仅建
    /// 元数据）。
    ///
    /// 安全校验：`strides` 提供时其长度必须等于 `dims` 长度（C 侧按 `viewDimsNum`
    /// 读取步长数组，长度不符会越界读取）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`。
    ///
    /// `data` 为裸指针参数（契约要求 `new` 为安全函数）：本封装只将其透传给
    /// `aclCreateTensor`，不在 Rust 侧解引用；指针有效性由调用方保证。
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new(
        dims: &[i64],
        dt: DataType,
        fmt: Format,
        offset: i64,
        strides: Option<&[i64]>,
        storage_dims: Option<&[i64]>,
        data: *mut c_void,
    ) -> Result<Self, Error> {
        if strides.is_some_and(|s| s.len() != dims.len()) {
            return Err(Error {
                code: -1,
                message: "Tensor::new: strides 长度与 dims 长度不一致".to_string(),
            });
        }
        // 注意：`aclCreateTensor` 以 `viewDimsNum` 同时作为 stride 数组长度，
        // stride 长度已在上方校验等于 dims 长度，故此处只需取指针。
        let stride_ptr = match strides {
            Some(s) => s.as_ptr(),
            None => std::ptr::null(),
        };
        let (storage_ptr, storage_num) = match storage_dims {
            Some(s) => (s.as_ptr(), s.len() as u64),
            None => (std::ptr::null(), 0),
        };
        // SAFETY: `dims` 为长度 `viewDimsNum` 的合法切片；`strides` 长度已校验等于
        // `dims` 长度；`storage_dims` 为长度 `storageDimsNum` 的合法切片（或 NULL）；
        // `data` 由调用方保证有效（或 NULL）；返回句柄由本类型唯一持有。
        let handle = unsafe {
            cann_sys::acl_meta::aclCreateTensor(
                dims.as_ptr(),
                dims.len() as u64,
                dt.as_acl(),
                stride_ptr,
                offset,
                fmt.as_acl(),
                storage_ptr,
                storage_num,
                data,
            )
        };
        if handle.is_null() {
            return Err(Error {
                code: -1,
                message: "aclCreateTensor 返回空句柄（参数非法或内存不足）".to_string(),
            });
        }
        Ok(Tensor { handle })
    }

    /// 查询视图各维长度（对应 `aclGetViewShape`，结果拷贝到新的 `Vec`）。
    ///
    /// 用法：张量析构后返回的数组失效，本方法在调用时拷贝，与张量生命周期解耦。
    pub fn shape(&self) -> Result<Vec<i64>, Error> {
        let mut dims: *mut i64 = std::ptr::null_mut();
        let mut num: u64 = 0;
        // SAFETY: `self.handle` 为 `aclCreateTensor` 成功返回且未被析构的句柄；
        // `dims`/`num` 为有效输出槽位。
        let ret = unsafe { cann_sys::acl_meta::aclGetViewShape(self.handle, &mut dims, &mut num) };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        if num == 0 {
            return Ok(Vec::new());
        }
        // SAFETY: 成功时 `dims` 指向由张量持有的 `num` 个 `i64` 元素（归张量所有，
        // 不得释放），此处仅构造只读切片并拷贝。
        let out = unsafe { std::slice::from_raw_parts(dims, num as usize) }.to_vec();
        Ok(out)
    }

    /// 查询数据类型（对应 `aclGetDataType`）。
    pub fn data_type(&self) -> Result<DataType, Error> {
        let mut dt: acl::aclDataType = -1;
        // SAFETY: `self.handle` 为有效张量句柄；`dt` 为有效输出槽位。
        let ret = unsafe { cann_sys::acl_meta::aclGetDataType(self.handle, &mut dt) };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        DataType::from_acl(dt).ok_or_else(|| Error {
            code: -1,
            message: format!("无法识别的 aclDataType: {dt}"),
        })
    }

    /// 查询内存布局格式（对应 `aclGetFormat`）。
    pub fn format(&self) -> Result<Format, Error> {
        let mut fmt: acl::aclFormat = -1;
        // SAFETY: `self.handle` 为有效张量句柄；`fmt` 为有效输出槽位。
        let ret = unsafe { cann_sys::acl_meta::aclGetFormat(self.handle, &mut fmt) };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        Format::from_acl(fmt).ok_or_else(|| Error {
            code: -1,
            message: format!("无法识别的 aclFormat: {fmt}"),
        })
    }

    /// 原始句柄（仅供 crate 内部跨模块使用，如 [`crate::op`] 的算子封装）。
    pub(crate) fn raw_handle(&self) -> *const c_void {
        self.handle
    }
}

#[cfg(feature = "ffi")]
impl Drop for Tensor {
    fn drop(&mut self) {
        // SAFETY: `self.handle` 来自 `aclCreateTensor` 且未被析构；本类型持有唯一
        // 所有权。注意：设备复位（`reset_device`）前必须先析构本张量。
        let _ = unsafe { cann_sys::acl_meta::aclDestroyTensor(self.handle) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Tensor {
    /// 创建张量元数据（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    pub fn new(
        _dims: &[i64],
        _dt: DataType,
        _fmt: Format,
        _offset: i64,
        _strides: Option<&[i64]>,
        _storage_dims: Option<&[i64]>,
        _data: *mut c_void,
    ) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 查询视图各维长度（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn shape(&self) -> Result<Vec<i64>, Error> {
        Err(unavailable())
    }

    /// 查询数据类型（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn data_type(&self) -> Result<DataType, Error> {
        Err(unavailable())
    }

    /// 查询内存布局格式（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn format(&self) -> Result<Format, Error> {
        Err(unavailable())
    }

    /// 原始句柄（`ffi` 未启用时恒为 NULL）。
    ///
    /// 仅 ffi 档由 crate 内部（[`crate::op`]/[`TensorList`]）使用，无 ffi 档无调用点。
    #[allow(clippy::unused_self, dead_code)]
    pub(crate) fn raw_handle(&self) -> *const c_void {
        std::ptr::null()
    }
}

/// aclnn 张量列表（RAII）。
///
/// 构造时调用 `aclCreateTensorList` 创建列表句柄（持有张量指针的副本），析构时调用
/// `aclDestroyTensorList` 释放。列表不拥有其中的张量：列表有效期内各张量必须存活，
/// 张量生命周期由调用方管理。
///
/// 线程亲和性：不实现 `Send`/`Sync`，同 [`Tensor`]。
#[derive(Debug)]
pub struct TensorList {
    #[cfg(feature = "ffi")]
    handle: *mut c_void,
}

#[cfg(feature = "ffi")]
impl TensorList {
    /// 创建张量列表（对应 `aclCreateTensorList`）。
    ///
    /// 列表持有张量指针的副本，不转移所有权；列表有效期内各张量必须存活。
    /// 失败时返回 `Err(Error)`。
    pub fn new(tensors: &[&Tensor]) -> Result<Self, Error> {
        let ptrs: Vec<*const c_void> = tensors.iter().map(|t| t.raw_handle()).collect();
        // SAFETY: `ptrs` 为长度 `tensors.len()` 的合法指针数组，元素均为有效张量
        // 句柄（或 NULL，C 侧允许）；返回句柄由本类型唯一持有。
        let handle =
            unsafe { cann_sys::acl_meta::aclCreateTensorList(ptrs.as_ptr(), ptrs.len() as u64) };
        if handle.is_null() {
            return Err(Error {
                code: -1,
                message: "aclCreateTensorList 返回空句柄".to_string(),
            });
        }
        Ok(TensorList { handle })
    }

    /// 查询列表中的张量个数（对应 `aclGetTensorListSize`）。
    pub fn len(&self) -> Result<u64, Error> {
        let mut size: u64 = 0;
        // SAFETY: `self.handle` 为 `aclCreateTensorList` 成功返回且未被析构的句柄；
        // `size` 为有效输出槽位。
        let ret = unsafe { cann_sys::acl_meta::aclGetTensorListSize(self.handle, &mut size) };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        Ok(size)
    }

    /// 查询列表是否为空（`len() == 0`）。
    pub fn is_empty(&self) -> Result<bool, Error> {
        self.len().map(|n| n == 0)
    }
}

#[cfg(feature = "ffi")]
impl Drop for TensorList {
    fn drop(&mut self) {
        // SAFETY: `self.handle` 来自 `aclCreateTensorList` 且未被析构；
        // 本类型持有唯一所有权。
        let _ = unsafe { cann_sys::acl_meta::aclDestroyTensorList(self.handle) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl TensorList {
    /// 创建张量列表（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn new(_tensors: &[&Tensor]) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 查询张量个数（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn len(&self) -> Result<u64, Error> {
        Err(unavailable())
    }

    /// 查询列表是否为空（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn is_empty(&self) -> Result<bool, Error> {
        self.len().map(|n| n == 0)
    }
}

/// aclnn 标量（RAII）。
///
/// 构造时调用 `aclCreateScalar` 创建标量句柄（拷贝值），析构时调用
/// `aclDestroyScalar` 释放。供后续批次算子（如 Add 等）作为标量参数使用；
/// 当前三算子（Matmul/Softmax/RmsNorm）尚未消费。
///
/// 线程亲和性：不实现 `Send`/`Sync`，同 [`Tensor`]。
#[derive(Debug)]
pub struct Scalar {
    #[cfg(feature = "ffi")]
    handle: *mut c_void,
}

#[cfg(feature = "ffi")]
impl Scalar {
    /// 以 32 位浮点值创建标量（`ACL_FLOAT`）。
    pub fn new_f32(v: f32) -> Result<Self, Error> {
        // SAFETY: 局部变量 `v` 在调用期间有效，内存布局与 `ACL_FLOAT` 一致；
        // `aclCreateScalar` 拷贝值，返回句柄由本类型唯一持有。
        Self::create(acl::ACL_FLOAT, std::ptr::addr_of!(v).cast_mut().cast())
    }

    /// 以 64 位浮点值创建标量（`ACL_DOUBLE`）。
    pub fn new_f64(v: f64) -> Result<Self, Error> {
        // SAFETY: 局部变量 `v` 在调用期间有效，内存布局与 `ACL_DOUBLE` 一致。
        Self::create(acl::ACL_DOUBLE, std::ptr::addr_of!(v).cast_mut().cast())
    }

    /// 以 32 位有符号整数创建标量（`ACL_INT32`）。
    pub fn new_i32(v: i32) -> Result<Self, Error> {
        // SAFETY: 局部变量 `v` 在调用期间有效，内存布局与 `ACL_INT32` 一致。
        Self::create(acl::ACL_INT32, std::ptr::addr_of!(v).cast_mut().cast())
    }

    /// 以 64 位有符号整数创建标量（`ACL_INT64`）。
    pub fn new_i64(v: i64) -> Result<Self, Error> {
        // SAFETY: 局部变量 `v` 在调用期间有效，内存布局与 `ACL_INT64` 一致。
        Self::create(acl::ACL_INT64, std::ptr::addr_of!(v).cast_mut().cast())
    }

    /// 调用 `aclCreateScalar` 的公共路径。
    ///
    /// # Safety 契约
    /// `value` 必须指向与 `dt` 尺寸一致的可读值，且仅在本调用期间有效
    /// （`aclCreateScalar` 会拷贝该值）。
    fn create(dt: acl::aclDataType, value: *mut c_void) -> Result<Self, Error> {
        // SAFETY: 见本方法文档契约，由各 `new_*` 构造函数保证成立。
        let handle = unsafe { cann_sys::acl_meta::aclCreateScalar(value, dt) };
        if handle.is_null() {
            return Err(Error {
                code: -1,
                message: "aclCreateScalar 返回空句柄".to_string(),
            });
        }
        Ok(Scalar { handle })
    }
}

#[cfg(feature = "ffi")]
impl Drop for Scalar {
    fn drop(&mut self) {
        // SAFETY: `self.handle` 来自 `aclCreateScalar` 且未被析构；
        // 本类型持有唯一所有权。
        let _ = unsafe { cann_sys::acl_meta::aclDestroyScalar(self.handle) };
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Scalar {
    /// 以 32 位浮点值创建标量（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn new_f32(_v: f32) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 以 64 位浮点值创建标量（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn new_f64(_v: f64) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 以 32 位有符号整数创建标量（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn new_i32(_v: i32) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 以 64 位有符号整数创建标量（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn new_i64(_v: i64) -> Result<Self, Error> {
        Err(unavailable())
    }
}

/// 将 aclnn 返回码转换为 `Error`（fail-closed：非 0 一律按错误处理）。
///
/// 正式的 `From<aclnnStatus>` 映射由 L1-5 任务在 [`crate::error`] 统一提供，
/// 在此之前先在此处直接构造。
#[cfg(feature = "ffi")]
fn aclnn_error(ret: cann_sys::aclnn_ops::aclnnStatus) -> Error {
    Error {
        code: ret,
        message: format!("aclnn 调用失败: {ret}"),
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

#[cfg(all(test, not(feature = "ffi")))]
mod tests {
    use super::*;

    #[test]
    fn data_type_mapping_roundtrip() {
        for dt in [
            DataType::Fp16,
            DataType::Fp32,
            DataType::Fp64,
            DataType::Bf16,
            DataType::Int8,
            DataType::Int16,
            DataType::Int32,
            DataType::Int64,
            DataType::Uint8,
            DataType::Uint16,
            DataType::Uint32,
            DataType::Uint64,
            DataType::Bool,
            DataType::Float8E5m2,
            DataType::Float8E4m3fn,
        ] {
            assert_eq!(DataType::from_acl(dt.as_acl()), Some(dt), "{dt:?}");
        }
        // 未映射的 ACL 类型 fail-closed：ACL_DT_UNDEFINED(-1) 与 ACL_STRING(13)
        assert_eq!(DataType::from_acl(-1), None);
        assert_eq!(DataType::from_acl(13), None);
    }

    #[test]
    fn format_mapping_roundtrip() {
        for fmt in [
            Format::Nchw,
            Format::Nhwc,
            Format::Nd,
            Format::Nc1Hwc0,
            Format::FractalZ,
            Format::Nc1Hwc0C04,
            Format::Hwcn,
            Format::Ndhwc,
            Format::FractalNz,
            Format::Ncdhw,
            Format::Ndc1Hwc0,
            Format::FractalZ3d,
            Format::Nc,
            Format::Ncl,
            Format::FractalNzC0_16,
            Format::FractalNzC0_32,
            Format::FractalNzC0_2,
            Format::FractalNzC0_4,
            Format::FractalNzC0_8,
        ] {
            assert_eq!(Format::from_acl(fmt.as_acl()), Some(fmt), "{fmt:?}");
        }
        // 未映射的 ACL 格式 fail-closed：ACL_FORMAT_UNDEFINED(-1)
        assert_eq!(Format::from_acl(-1), None);
    }

    #[test]
    fn new_returns_err_without_ffi() {
        assert!(
            Tensor::new(
                &[2, 3],
                DataType::Fp32,
                Format::Nd,
                0,
                Some(&[3, 1]),
                Some(&[2, 3]),
                std::ptr::null_mut(),
            )
            .is_err()
        );
        assert!(TensorList::new(&[]).is_err());
        assert!(Scalar::new_f32(1.5).is_err());
        assert!(Scalar::new_f64(1.5).is_err());
        assert!(Scalar::new_i32(1).is_err());
        assert!(Scalar::new_i64(1).is_err());
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;

    /// 真机冒烟：Tensor 元数据往返（创建 → shape/data_type/format → 析构）。
    #[test]
    #[ignore = "requires NPU driver"]
    fn tensor_metadata_roundtrip() {
        let t = Tensor::new(
            &[2, 3],
            DataType::Fp16,
            Format::Nd,
            0,
            Some(&[3, 1]),
            Some(&[2, 3]),
            std::ptr::null_mut(),
        )
        .unwrap();
        assert_eq!(t.shape().unwrap(), vec![2, 3]);
        assert_eq!(t.data_type().unwrap(), DataType::Fp16);
        assert_eq!(t.format().unwrap(), Format::Nd);
        drop(t); // 析构触发 aclDestroyTensor
    }

    /// 真机冒烟：TensorList 往返（创建 → 大小查询 → 析构）。
    #[test]
    #[ignore = "requires NPU driver"]
    fn tensor_list_roundtrip() {
        let a = Tensor::new(
            &[4],
            DataType::Fp32,
            Format::Nd,
            0,
            Some(&[1]),
            Some(&[4]),
            std::ptr::null_mut(),
        )
        .unwrap();
        let b = Tensor::new(
            &[4],
            DataType::Fp32,
            Format::Nd,
            0,
            Some(&[1]),
            Some(&[4]),
            std::ptr::null_mut(),
        )
        .unwrap();
        let list = TensorList::new(&[&a, &b]).unwrap();
        assert_eq!(list.len().unwrap(), 2);
        assert!(!list.is_empty().unwrap());
        drop(list);
        drop(a);
        drop(b);
    }

    /// 真机冒烟：Scalar 往返（创建 → 析构）。
    #[test]
    #[ignore = "requires NPU driver"]
    fn scalar_roundtrip() {
        let s = Scalar::new_f32(1.5).unwrap();
        drop(s);
        let s = Scalar::new_i64(7).unwrap();
        drop(s);
    }
}
