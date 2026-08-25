//! aclnn 算子安全封装（首批：Matmul / Softmax / RMSNorm）。
//!
//! 统一采用 aclnn 两段式调用模式：`<Op>::new(...)` 调用 `*GetWorkspaceSize` 计算
//! workspace 大小并生成执行器（[`OpExecutor`]），`launch(&Stream)` 在指定 stream 上
//! 消费执行器完成计算。
//!
//! ## workspace 语义
//!
//! aclnn 要求调用方提供 workspace 缓冲区：第一段输出所需大小，第二段传入起始指针。
//! 本封装在 [`OpExecutor`] 内以 host 侧内存持有 workspace（aclnn workspace 规模一般
//! 较小），并按 CANN 要求做 64 字节对齐：分配 `ws_size + 63` 字节后将起始地址手动
//! 对齐到 64 字节边界（`Vec<u8>` 只保证 8 字节对齐）。执行器句柄与 workspace 同
//! 生命周期（同存于 `OpExecutor`，析构时一并释放），满足 acl 的两段式生命周期约束。
//!
//! 线程亲和性：算子不实现 `Send`/`Sync`；`new` 需当前线程已完成 `Context::new()` 与
//! `set_device`（第一段需要设备上下文），`launch` 的 stream 须为当前设备上下文下
//! 创建的有效流。

use crate::error::Error;
use crate::stream::Stream;
use crate::tensor::Tensor;
#[cfg(feature = "ffi")]
use std::ffi::c_void;

/// workspace 对齐要求（CANN 要求 workspace 内存 64 字节对齐）。
#[cfg(feature = "ffi")]
const WORKSPACE_ALIGN: usize = 64;

/// 将地址向上对齐到 `align` 的倍数（要求 `align` 为 2 的幂）。
#[cfg(feature = "ffi")]
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// aclnn 算子执行器（两段式第一段的产物，RAII）。
///
/// 持有 workspace 缓冲区与执行器句柄：`*GetWorkspaceSize` 成功后构造，析构时
/// workspace 内存随 `Vec` 释放。执行器句柄为 C 侧资源，其生命周期必须与 workspace
/// 同长——二者同存于本类型，顺序天然一致。
///
/// workspace 采用 host 侧内存（规模一般较小），起始地址 64 字节对齐
/// （见模块文档"workspace 语义"）。
#[derive(Debug)]
pub struct OpExecutor {
    #[cfg(feature = "ffi")]
    #[allow(dead_code)] // 仅用于持有分配内存，保证 ws_ptr 在存活期内有效；构造后不读取
    ws_buf: Vec<u8>,
    #[cfg(feature = "ffi")]
    ws_ptr: *mut u8,
    #[cfg(feature = "ffi")]
    ws_size: u64,
    #[cfg(feature = "ffi")]
    handle: *mut c_void,
}

#[cfg(feature = "ffi")]
impl OpExecutor {
    /// 执行 `*GetWorkspaceSize` 第一段并构造执行器。
    ///
    /// `get_workspace_size` 为各算子的第一段调用闭包；成功（`ACLNN_SUCCESS`）且
    /// 执行器句柄非空时，按返回大小分配 64 字节对齐的 workspace。
    fn prepare(
        get_workspace_size: impl FnOnce(*mut u64, *mut *mut c_void) -> cann_sys::aclnn_ops::aclnnStatus,
    ) -> Result<Self, Error> {
        let mut ws_size: u64 = 0;
        let mut handle: *mut c_void = std::ptr::null_mut();
        // SAFETY: `ws_size`/`handle` 为有效输出槽位；闭包内传入的输入/输出张量
        // 句柄由各算子 `new` 保证有效（见其调用点）。
        let ret = get_workspace_size(&mut ws_size, &mut handle);
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        if handle.is_null() {
            return Err(Error {
                code: -1,
                message: "GetWorkspaceSize 返回空执行器句柄".to_string(),
            });
        }
        // 分配 ws_size + 63 字节，将起始地址对齐到 64 字节（偏移 ≤ 63，始终落在
        // 分配范围内）。取舍：不用外部对齐库，std 手工对齐即可满足 CANN 要求。
        let capacity = ws_size as usize + (WORKSPACE_ALIGN - 1);
        let mut ws_buf = vec![0u8; capacity];
        let base = ws_buf.as_mut_ptr() as usize;
        let ws_ptr = align_up(base, WORKSPACE_ALIGN) as *mut u8;
        Ok(OpExecutor {
            ws_buf,
            ws_ptr,
            ws_size,
            handle,
        })
    }

    /// workspace 起始地址（64 字节对齐，供第二段 Launch 使用）。
    pub(crate) fn workspace_ptr(&self) -> *mut c_void {
        self.ws_ptr.cast()
    }

    /// workspace 大小（字节，等于第一段输出值）。
    pub(crate) fn workspace_size(&self) -> u64 {
        self.ws_size
    }

    /// 执行器句柄（供第二段 Launch 使用）。
    pub(crate) fn handle(&self) -> *mut c_void {
        self.handle
    }
}

/// 统一的 aclnn 两段式算子抽象。
///
/// 实现者持有各自的输入/输出张量并暴露第一段产物 [`OpExecutor`]；
/// `launch` 由各实现提供（在指定 stream 上消费执行器完成计算）。
pub trait Operator {
    /// 第一段生成的执行器（含 workspace）。
    fn executor(&self) -> &OpExecutor;
}

/// 矩阵乘（Matmul）算子封装。
///
/// 两段式：`new` 调用 `aclnnMatmulGetWorkspaceSize`，`launch` 在指定 stream 上执行，
/// 结果写入 `out`。张量数据类型支持 FP16/BF16，格式支持 ND（CANN 8.5.0 限制）。
///
/// 字段持有输入/输出张量所有权：executor 有效期内张量必须存活，由本类型统一保证。
/// （字段构造后不再读取——所有权/生命周期语义，故 `allow(dead_code)`。）
#[allow(dead_code)]
#[derive(Debug)]
pub struct Matmul {
    a: Tensor,
    b: Tensor,
    out: Tensor,
    exec: OpExecutor,
}

#[cfg(feature = "ffi")]
impl Matmul {
    /// 创建 Matmul 算子（两段式第一段，对应 `aclnnMatmulGetWorkspaceSize`）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`；`a`/`b` 为输入
    /// 张量，`out` 为输出张量（FP16/BF16，ND 格式），`cube_math_type` 指定 Cube
    /// 计算逻辑（0 或 1）。`new` 转移张量所有权（执行器有效期内张量必须存活）。
    /// 失败时返回 `Err(Error)`（如参数非法或设备上下文缺失）。
    pub fn new(a: Tensor, b: Tensor, out: Tensor, cube_math_type: i8) -> Result<Self, Error> {
        let exec = OpExecutor::prepare(|ws, exec| {
            // SAFETY: a/b/out 为 `aclCreateTensor` 创建且未被析构的有效张量；
            // ws/exec 为 `OpExecutor::prepare` 提供的有效输出槽位。
            unsafe {
                cann_sys::aclnn_ops::aclnnMatmulGetWorkspaceSize(
                    a.raw_handle(),
                    b.raw_handle(),
                    out.raw_handle().cast_mut(),
                    cube_math_type,
                    ws,
                    exec,
                )
            }
        })?;
        Ok(Matmul { a, b, out, exec })
    }

    /// 在指定 stream 上执行矩阵乘（两段式第二段，对应 `aclnnMatmul`）。
    ///
    /// 用法：`launch` 前需已完成 `Context::new()` + `set_device`，且 `stream` 为
    /// 当前线程设备上下文下创建的有效流。计算异步执行，结果写入 `out`，
    /// 完成后需 `stream.synchronize()`（或经 `Event`）等待。
    pub fn launch(&self, stream: &Stream) -> Result<(), Error> {
        // SAFETY: `self.exec` 的 workspace 与执行器句柄来自同一次第一段调用且
        // 生命周期同长（同在 `OpExecutor` 中）；`stream` 为有效流句柄。
        let ret = unsafe {
            cann_sys::aclnn_ops::aclnnMatmul(
                self.exec.workspace_ptr(),
                self.exec.workspace_size(),
                self.exec.handle(),
                stream.raw_handle(),
            )
        };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        Ok(())
    }
}

/// Softmax 算子封装。
///
/// 两段式：`new` 调用 `aclnnSoftmaxGetWorkspaceSize`，`launch` 在指定 stream 上
/// 执行；`dim` 为 softmax 归一化所在维度。
/// （字段构造后不再读取——所有权/生命周期语义，故 `allow(dead_code)`。）
#[allow(dead_code)]
#[derive(Debug)]
pub struct Softmax {
    x: Tensor,
    dim: i64,
    out: Tensor,
    exec: OpExecutor,
}

#[cfg(feature = "ffi")]
impl Softmax {
    /// 创建 Softmax 算子（两段式第一段，对应 `aclnnSoftmaxGetWorkspaceSize`）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`；`x` 为输入张量，
    /// `dim` 为归一化所在维度，`out` 为输出张量。转移张量所有权。
    /// 失败时返回 `Err(Error)`。
    pub fn new(x: Tensor, dim: i64, out: Tensor) -> Result<Self, Error> {
        let exec = OpExecutor::prepare(|ws, exec| {
            // SAFETY: x/out 为 `aclCreateTensor` 创建且未被析构的有效张量；
            // ws/exec 为 `OpExecutor::prepare` 提供的有效输出槽位。
            unsafe {
                cann_sys::aclnn_ops::aclnnSoftmaxGetWorkspaceSize(
                    x.raw_handle(),
                    dim,
                    out.raw_handle().cast_mut(),
                    ws,
                    exec,
                )
            }
        })?;
        Ok(Softmax { x, dim, out, exec })
    }

    /// 在指定 stream 上执行 softmax（两段式第二段，对应 `aclnnSoftmax`）。
    ///
    /// 用法同 [`Matmul::launch`]；计算异步执行，结果写入 `out`，完成后需
    /// `stream.synchronize()` 等待。
    pub fn launch(&self, stream: &Stream) -> Result<(), Error> {
        // SAFETY: 同 `Matmul::launch`：workspace/executor 同生命周期，stream 有效。
        let ret = unsafe {
            cann_sys::aclnn_ops::aclnnSoftmax(
                self.exec.workspace_ptr(),
                self.exec.workspace_size(),
                self.exec.handle(),
                stream.raw_handle(),
            )
        };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        Ok(())
    }
}

/// RMSNorm 算子封装。
///
/// 两段式：`new` 调用 `aclnnRmsNormGetWorkspaceSize`，`launch` 在指定 stream 上
/// 执行；结果写入 `y`（归一化结果）与 `rstd`（1/sqrt(方差+eps)）。
/// （字段构造后不再读取——所有权/生命周期语义，故 `allow(dead_code)`。）
#[allow(dead_code)]
#[derive(Debug)]
pub struct RmsNorm {
    x: Tensor,
    gamma: Tensor,
    eps: f64,
    y: Tensor,
    rstd: Tensor,
    exec: OpExecutor,
}

#[cfg(feature = "ffi")]
impl RmsNorm {
    /// 创建 RMSNorm 算子（两段式第一段，对应 `aclnnRmsNormGetWorkspaceSize`）。
    ///
    /// 用法：需已完成 `Context::new()` 且当前线程已 `set_device`；`x` 为输入张量，
    /// `gamma` 为归一化缩放权重张量，`eps` 为防止除零的小常数，`y`/`rstd` 为输出
    /// 张量（对应 C 侧 yOut/rstdOut）。转移张量所有权。失败时返回 `Err(Error)`。
    pub fn new(x: Tensor, gamma: Tensor, eps: f64, y: Tensor, rstd: Tensor) -> Result<Self, Error> {
        let exec = OpExecutor::prepare(|ws, exec| {
            // SAFETY: x/gamma/y/rstd 为 `aclCreateTensor` 创建且未被析构的有效张量；
            // ws/exec 为 `OpExecutor::prepare` 提供的有效输出槽位。
            unsafe {
                cann_sys::aclnn_ops::aclnnRmsNormGetWorkspaceSize(
                    x.raw_handle(),
                    gamma.raw_handle(),
                    eps,
                    y.raw_handle(),
                    rstd.raw_handle(),
                    ws,
                    exec,
                )
            }
        })?;
        Ok(RmsNorm {
            x,
            gamma,
            eps,
            y,
            rstd,
            exec,
        })
    }

    /// 在指定 stream 上执行 RMSNorm（两段式第二段，对应 `aclnnRmsNorm`）。
    ///
    /// 用法同 [`Matmul::launch`]；计算异步执行，结果写入 `y`/`rstd`，完成后需
    /// `stream.synchronize()` 等待。
    pub fn launch(&self, stream: &Stream) -> Result<(), Error> {
        // SAFETY: 同 `Matmul::launch`：workspace/executor 同生命周期，stream 有效。
        let ret = unsafe {
            cann_sys::aclnn_ops::aclnnRmsNorm(
                self.exec.workspace_ptr(),
                self.exec.workspace_size(),
                self.exec.handle(),
                stream.raw_handle(),
            )
        };
        if ret != cann_sys::aclnn_ops::ACLNN_SUCCESS {
            return Err(aclnn_error(ret));
        }
        Ok(())
    }
}

impl Operator for Matmul {
    fn executor(&self) -> &OpExecutor {
        &self.exec
    }
}

impl Operator for Softmax {
    fn executor(&self) -> &OpExecutor {
        &self.exec
    }
}

impl Operator for RmsNorm {
    fn executor(&self) -> &OpExecutor {
        &self.exec
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Matmul {
    /// 创建 Matmul 算子（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    pub fn new(_a: Tensor, _b: Tensor, _out: Tensor, _cube_math_type: i8) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 在指定 stream 上执行矩阵乘（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn launch(&self, _stream: &Stream) -> Result<(), Error> {
        Err(unavailable())
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Softmax {
    /// 创建 Softmax 算子（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    pub fn new(_x: Tensor, _dim: i64, _out: Tensor) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 在指定 stream 上执行 softmax（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn launch(&self, _stream: &Stream) -> Result<(), Error> {
        Err(unavailable())
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 `libascendcl`，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl RmsNorm {
    /// 创建 RMSNorm 算子（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 ACL 码）。
    pub fn new(
        _x: Tensor,
        _gamma: Tensor,
        _eps: f64,
        _y: Tensor,
        _rstd: Tensor,
    ) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 在指定 stream 上执行 RMSNorm（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn launch(&self, _stream: &Stream) -> Result<(), Error> {
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
    fn new_returns_err_without_ffi() {
        // ffi 未启用时 Tensor 为无字段类型，只能以占位值构造
        assert!(Matmul::new(Tensor {}, Tensor {}, Tensor {}, 0).is_err());
        assert!(Softmax::new(Tensor {}, 1, Tensor {}).is_err());
        assert!(RmsNorm::new(Tensor {}, Tensor {}, 1e-5, Tensor {}, Tensor {}).is_err());
    }
}

#[cfg(all(feature = "ffi", test))]
mod ffi_smoke {
    use super::*;
    use crate::buffer::{DeviceBuffer, MemFlags};
    use crate::context::Context;
    use crate::device::{reset_device, set_device};
    use crate::tensor::{DataType, Format};

    const N: i64 = 16;

    /// 构造设备内存张量的测试辅助（FP16，N×N，行主序连续）。
    fn dev_tensor(buf: &DeviceBuffer, dims: [i64; 2]) -> Tensor {
        Tensor::new(
            &dims,
            DataType::Fp16,
            Format::Nd,
            0,
            Some(&[dims[1], 1]),
            Some(&dims),
            buf.as_ptr().cast_mut().cast(),
        )
        .unwrap()
    }

    /// 真机冒烟：Matmul 全链路（Context → set_device → Stream → 设备内存 →
    /// 张量 → new → launch → synchronize → 析构）。
    #[test]
    #[ignore = "requires NPU driver"]
    fn matmul_full_chain() {
        let _ctx = Context::new().unwrap();
        set_device(0).unwrap();
        let stream = Stream::new().unwrap();
        let buf_a = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let buf_b = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let buf_out = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let dims = [N, N];
        let a = dev_tensor(&buf_a, dims);
        let b = dev_tensor(&buf_b, dims);
        let out = dev_tensor(&buf_out, dims);
        let matmul = Matmul::new(a, b, out, 0).unwrap();
        matmul.launch(&stream).unwrap();
        stream.synchronize().unwrap();
        // 析构顺序：算子（释放张量句柄与 workspace）→ 流 → 缓冲 → 复位设备 → _ctx
        drop(matmul);
        drop(stream);
        reset_device(0).unwrap();
    }

    /// 真机冒烟：Softmax 全链路。
    #[test]
    #[ignore = "requires NPU driver"]
    fn softmax_full_chain() {
        let _ctx = Context::new().unwrap();
        set_device(0).unwrap();
        let stream = Stream::new().unwrap();
        let buf_x = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let buf_out = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let dims = [N, N];
        let x = dev_tensor(&buf_x, dims);
        let out = dev_tensor(&buf_out, dims);
        let softmax = Softmax::new(x, 1, out).unwrap();
        softmax.launch(&stream).unwrap();
        stream.synchronize().unwrap();
        drop(softmax);
        drop(stream);
        reset_device(0).unwrap();
    }

    /// 真机冒烟：RMSNorm 全链路（x/y 为 [N,N]，gamma 为 [N]，rstd 为 [N,1]）。
    #[test]
    #[ignore = "requires NPU driver"]
    fn rms_norm_full_chain() {
        let _ctx = Context::new().unwrap();
        set_device(0).unwrap();
        let stream = Stream::new().unwrap();
        let buf_x = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let buf_gamma = DeviceBuffer::alloc((N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let buf_y = DeviceBuffer::alloc((N * N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let buf_rstd = DeviceBuffer::alloc((N * 2) as usize, MemFlags::HugeFirst).unwrap();
        let x = dev_tensor(&buf_x, [N, N]);
        let gamma = Tensor::new(
            &[N],
            DataType::Fp16,
            Format::Nd,
            0,
            Some(&[1]),
            Some(&[N]),
            buf_gamma.as_ptr().cast_mut().cast(),
        )
        .unwrap();
        let y = dev_tensor(&buf_y, [N, N]);
        let rstd = Tensor::new(
            &[N, 1],
            DataType::Fp16,
            Format::Nd,
            0,
            Some(&[1, 1]),
            Some(&[N, 1]),
            buf_rstd.as_ptr().cast_mut().cast(),
        )
        .unwrap();
        let rms = RmsNorm::new(x, gamma, 1e-5, y, rstd).unwrap();
        rms.launch(&stream).unwrap();
        stream.synchronize().unwrap();
        drop(rms);
        drop(stream);
        reset_device(0).unwrap();
    }
}
