//! GE 图引擎安全封装：ONNX 模型解析与 .om 模型编译。
//!
//! [`Graph`] 为 `cann_grph_parse_onnx_from_*` 句柄的 RAII 封装（解析 ONNX 模型），
//! [`Session`] 提供"解析 → 编译"更高层门面（委托 [`Graph::build_and_save`]）。
//! 底层为 cann-sys `acl_grph` 模块经 C++ shim（`ge_shim.cc`）桥接的 GE C++ API。
//!
//! 线程亲和性：GE 图句柄**仅在同一线程内有效**（shim 内部为 thread_local 注册表，
//! 见 cann-sys `acl_grph` 模块文档）；本模块类型不实现 `Send`/`Sync`，跨线程使用
//! 图对象是未定义行为。

use crate::error::Error;
#[cfg(feature = "ffi")]
use std::ffi::{CString, c_char, c_void};
use std::path::Path;

/// ONNX 计算图（RAII）。
///
/// 构造时调用 `cann_grph_parse_onnx_from_file`/`cann_grph_parse_onnx_from_mem`
/// 解析 ONNX 模型，析构时调用 `cann_grph_destroy` 释放句柄。
///
/// 线程亲和性：句柄仅在同一线程内有效，不得跨线程使用（不实现 `Send`/`Sync`）；
/// 设备复位前必须先析构本图。
#[derive(Debug)]
pub struct Graph {
    #[cfg(all(feature = "ffi", cann_sdk_has_aclgrph))]
    handle: *mut c_void,
}

#[cfg(all(feature = "ffi", cann_sdk_has_aclgrph))]
impl Graph {
    /// 从 ONNX 模型文件解析计算图（对应 `cann_grph_parse_onnx_from_file`）。
    ///
    /// 用法：`path` 为模型文件路径；失败（文件不存在/解析失败等）返回 `Err(Error)`。
    /// 成功句柄仅在同一线程内有效。
    pub fn from_onnx(path: &Path) -> Result<Self, Error> {
        let c_path = CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_| Error {
            code: -1,
            message: "Graph::from_onnx: 路径含 NUL 字节，无法转为 C 字符串".to_string(),
        })?;
        let mut handle: *mut c_void = std::ptr::null_mut();
        // SAFETY: `c_path` 为 NUL 结尾的合法 C 字符串；`handle` 为有效输出槽位；
        // 成功后句柄所有权转移给本类型（由 Drop 恰好释放一次）。
        let ret = unsafe {
            cann_sys::acl_grph::cann_grph_parse_onnx_from_file(c_path.as_ptr(), &mut handle)
        };
        if ret != cann_sys::acl_grph::GRAPH_SUCCESS {
            return Err(graph_error(ret));
        }
        Ok(Graph { handle })
    }

    /// 从内存中的 ONNX 模型字节解析计算图（对应 `cann_grph_parse_onnx_from_mem`）。
    ///
    /// 用法：`bytes` 为 ONNX 模型完整字节（非空）；失败返回 `Err(Error)`。
    /// 成功句柄仅在同一线程内有效。
    pub fn from_mem(bytes: &[u8]) -> Result<Self, Error> {
        let mut handle: *mut c_void = std::ptr::null_mut();
        // SAFETY: `bytes` 为长度 `size` 的合法可读切片；`handle` 为有效输出槽位；
        // 成功后句柄所有权转移给本类型（由 Drop 恰好释放一次）。
        let ret = unsafe {
            cann_sys::acl_grph::cann_grph_parse_onnx_from_mem(
                bytes.as_ptr().cast::<c_char>(),
                bytes.len(),
                &mut handle,
            )
        };
        if ret != cann_sys::acl_grph::GRAPH_SUCCESS {
            return Err(graph_error(ret));
        }
        Ok(Graph { handle })
    }

    /// 编译本图并保存为 .om 模型（对应 `cann_grph_build_model`）。
    ///
    /// 用法：`out_path` 为输出 .om 文件路径；须与本图在同一线程上调用。
    /// 失败时返回 `Err(Error)`（如图未就绪或输出路径不可写）。
    pub fn build_and_save(&self, out_path: &Path) -> Result<(), Error> {
        let c_path = CString::new(out_path.as_os_str().as_encoded_bytes()).map_err(|_| Error {
            code: -1,
            message: "Graph::build_and_save: 路径含 NUL 字节，无法转为 C 字符串".to_string(),
        })?;
        // SAFETY: `self.handle` 为解析成功、尚未销毁且同线程使用的图句柄；
        // `c_path` 为 NUL 结尾的合法 C 字符串。
        let ret =
            unsafe { cann_sys::acl_grph::cann_grph_build_model(self.handle, c_path.as_ptr()) };
        if ret != cann_sys::acl_grph::GRAPH_SUCCESS {
            return Err(graph_error(ret));
        }
        Ok(())
    }
}

#[cfg(all(feature = "ffi", cann_sdk_has_aclgrph))]
impl Drop for Graph {
    fn drop(&mut self) {
        // SAFETY: `self.handle` 来自 `cann_grph_parse_onnx_from_*` 且未被析构；
        // 本类型持有唯一所有权；须与创建时在同一线程析构。
        let _ = unsafe { cann_sys::acl_grph::cann_grph_destroy(self.handle) };
    }
}

/// SDK 无 GE 图引擎（CANN 7.x）：`ffi` 启用但无 `aclgrph*` 时的降级实现。
#[cfg(all(feature = "ffi", not(cann_sdk_has_aclgrph)))]
impl Graph {
    /// 从 ONNX 模型文件解析计算图（需要 GE 图引擎，CANN 8.x+）。
    ///
    /// 当前 SDK 无 GE 图引擎时返回 `Err(Error)`。
    #[allow(clippy::new_without_default)]
    pub fn from_onnx(_path: &Path) -> Result<Self, Error> {
        Err(unsupported_graph())
    }

    /// 从内存中的 ONNX 模型字节解析计算图（需要 GE 图引擎，CANN 8.x+）。
    pub fn from_mem(_bytes: &[u8]) -> Result<Self, Error> {
        Err(unsupported_graph())
    }

    /// 编译并保存 .om 模型（需要 GE 图引擎，CANN 8.x+）。
    #[allow(clippy::unused_self)]
    pub fn build_and_save(&self, _out_path: &Path) -> Result<(), Error> {
        Err(unsupported_graph())
    }
}

#[cfg(all(feature = "ffi", not(cann_sdk_has_aclgrph)))]
impl Session {
    /// 从 ONNX 模型文件创建会话（需要 GE 图引擎，CANN 8.x+）。
    pub fn from_onnx(_path: &Path) -> Result<Self, Error> {
        Err(unsupported_graph())
    }

    /// 从内存中的 ONNX 模型字节创建会话（需要 GE 图引擎，CANN 8.x+）。
    pub fn from_mem(_bytes: &[u8]) -> Result<Self, Error> {
        Err(unsupported_graph())
    }

    /// 编译并保存 .om 模型（需要 GE 图引擎，CANN 8.x+）。
    #[allow(clippy::unused_self)]
    pub fn build_and_save(&self, _out_path: &Path) -> Result<(), Error> {
        Err(unsupported_graph())
    }
}

#[cfg(all(feature = "ffi", not(cann_sdk_has_aclgrph)))]
fn unsupported_graph() -> Error {
    Error {
        code: -1,
        message: "当前 CANN SDK 无 GE 图引擎（aclgrph* 需 CANN 8.x+），Graph/Session 不可用"
            .to_string(),
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 GE 库，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Graph {
    /// 从 ONNX 模型文件解析计算图（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 GE 码）。
    pub fn from_onnx(_path: &Path) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 从内存中的 ONNX 模型字节解析计算图（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn from_mem(_bytes: &[u8]) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 编译并保存 .om 模型（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn build_and_save(&self, _out_path: &Path) -> Result<(), Error> {
        Err(unavailable())
    }
}

/// GE 图会话：持有解析后的计算图，提供 build/save 门面。
///
/// 与 [`Graph`] 的关系：`Session` 拥有一个 [`Graph`]，析构时随图一并释放；
/// 语义等价于 `Graph` + `build_and_save`，作为"解析 → 编译"链路的更高层入口。
///
/// 线程亲和性：与 [`Graph`] 相同，仅限创建线程使用。
#[derive(Debug)]
pub struct Session {
    #[cfg(all(feature = "ffi", cann_sdk_has_aclgrph))]
    graph: Graph,
}

#[cfg(all(feature = "ffi", cann_sdk_has_aclgrph))]
impl Session {
    /// 从 ONNX 模型文件创建会话（解析计算图）。
    ///
    /// 失败（文件不存在/解析失败等）返回 `Err(Error)`。
    pub fn from_onnx(path: &Path) -> Result<Self, Error> {
        Graph::from_onnx(path).map(|graph| Session { graph })
    }

    /// 从内存中的 ONNX 模型字节创建会话（解析计算图）。
    ///
    /// 失败（模型无效等）返回 `Err(Error)`。
    pub fn from_mem(bytes: &[u8]) -> Result<Self, Error> {
        Graph::from_mem(bytes).map(|graph| Session { graph })
    }

    /// 编译会话中的图并保存为 .om 模型。
    ///
    /// 用法：`out_path` 为输出 .om 文件路径；须与会话在同一线程上调用。
    pub fn build_and_save(&self, out_path: &Path) -> Result<(), Error> {
        self.graph.build_and_save(out_path)
    }
}

/// 无 `ffi` 特性时的降级实现：不链接 GE 库，统一返回"未启用"错误。
#[cfg(not(feature = "ffi"))]
impl Session {
    /// 从 ONNX 模型文件创建会话（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`（code 为 -1，非 GE 码）。
    pub fn from_onnx(_path: &Path) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 从内存中的 ONNX 模型字节创建会话（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    pub fn from_mem(_bytes: &[u8]) -> Result<Self, Error> {
        Err(unavailable())
    }

    /// 编译并保存 .om 模型（需要 `ffi` 特性）。
    ///
    /// 未启用 `ffi` 特性时返回 `Err(Error)`。
    #[allow(clippy::unused_self)]
    pub fn build_and_save(&self, _out_path: &Path) -> Result<(), Error> {
        Err(unavailable())
    }
}

/// 将 GE `graphStatus` 返回码转换为 `Error`（fail-closed：非 0 一律按错误处理）。
///
/// 正式的 `From<graphStatus>` 映射由 L1-5 任务在 [`crate::error`] 统一提供，
/// 在此之前先在此处直接构造。
#[cfg(all(feature = "ffi", cann_sdk_has_aclgrph))]
fn graph_error(ret: cann_sys::acl_grph::graphStatus) -> Error {
    Error {
        code: ret as i32,
        message: format!("graph 调用失败: {ret}"),
    }
}

/// `ffi` 未启用时的错误（code 为 -1，非 GE 码；message 为中文说明）。
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
        assert!(Graph::from_onnx(Path::new("/tmp/model.onnx")).is_err());
        assert!(Graph::from_mem(&[]).is_err());
        assert!(Session::from_onnx(Path::new("/tmp/model.onnx")).is_err());
        assert!(Session::from_mem(&[]).is_err());
    }
}

#[cfg(all(feature = "ffi", test, cann_sdk_has_aclgrph))]
mod ffi_smoke {
    use super::*;

    /// 链接冒烟：不存在的模型文件应返回错误（不触碰 NPU 驱动）。
    #[test]
    #[ignore = "requires NPU driver"]
    fn from_onnx_missing_file_returns_err() {
        let missing = Path::new("/nonexistent/cann-rs/never.onnx");
        assert!(Graph::from_onnx(missing).is_err());
        assert!(Session::from_onnx(missing).is_err());
    }
}

#[cfg(all(feature = "ffi", test, not(cann_sdk_has_aclgrph)))]
mod ffi_no_ge_tests {
    use super::*;

    #[test]
    fn graph_unavailable_on_sdk_without_ge() {
        assert!(Graph::from_onnx(Path::new("/x.onnx")).is_err());
        assert!(Session::from_onnx(Path::new("/x.onnx")).is_err());
    }
}
