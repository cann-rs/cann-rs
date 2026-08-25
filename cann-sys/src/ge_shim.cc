// GE 图引擎 C++ shim —— 桥接 GE 的 C++ aclgrph* API 为 extern "C"。
//
// 背景：GE 的 aclgrph* 系列（aclgrphParseONNX / aclgrphBuildModel 等）是 C++ API
// （namespace ge，参数含 std::map<ge::AscendString, ge::AscendString> 与 ge::Graph
// 引用），Rust 无法直接 extern "C" 声明。本文件由 build.rs 在 ffi 档编译为静态库
// libge_shim.a 并链接，向 Rust 侧（cann-sys/src/acl_grph.rs）导出简化后的 C 形状：
//   - ge::Graph 内部是 shared_ptr，值语义 ABI 不稳定，不能跨边界裸传：以不透明
//     void* 句柄承载，句柄注册于线程局部注册表（thread_local，无需锁），
//     因此句柄仅在同一线程内有效（parse/build/destroy 须同线程，cann 层保证）；
//   - parser_params / build_options 在 shim 内传空表（L1 阶段不暴露配置项）。
//
// 符号归属（本机 CANN 8.5.0 lib64，nm -D 验证）：
//   - ge::aclgrphParseONNX / aclgrphParseONNXFromMem → libfmk_onnx_parser.so
//   - ge::aclgrphBuildModel / ge::aclgrphSaveModel → libge_compiler.so
// 头文件：include/parser/onnx_parser.h、include/ge/ge_ir_build.h、include/graph/*.h
// 注：本文件为 C++（.cc），shim 外层的构建逻辑见 cann-sys/build.rs build_ge_shim()。

#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <string>

#include "ge/ge_ir_build.h"
#include "parser/onnx_parser.h"

// graphStatus 定义在 namespace ge 内（include/graph/ge_error_codes.h）：
// `using graphStatus = uint32_t;`。extern "C" 出口统一用它作为返回类型。
using ge::graphStatus;

namespace {

// 图句柄注册表：句柄 → 图实例（shared_ptr 持有，随句柄销毁释放）。
// thread_local：天然免锁；代价是句柄仅在同一线程内有效。
thread_local std::map<void*, std::shared_ptr<ge::Graph>> g_graph_registry;
thread_local std::uintptr_t g_next_handle = 1;

// 把解析得到的图登记进注册表并返回不透明句柄（所有权转移给调用方）。
void* register_graph(ge::Graph&& graph) {
    void* handle = reinterpret_cast<void*>(g_next_handle++);
    g_graph_registry.emplace(handle, std::make_shared<ge::Graph>(std::move(graph)));
    return handle;
}

// 按句柄查图实例；句柄不存在返回 nullptr。
std::shared_ptr<ge::Graph>* lookup_graph(void* handle) {
    auto it = g_graph_registry.find(handle);
    return it == g_graph_registry.end() ? nullptr : &it->second;
}

}  // namespace

extern "C" {

// 从 ONNX 模型文件解析计算图（桥接 ge::aclgrphParseONNX，parser_params 传空表）。
// 成功：*handle_out 写入图句柄（所有权归调用方，须 cann_grph_destroy 释放一次）；
// 失败：*handle_out 置 NULL，返回 ge 错误码原值。
graphStatus cann_grph_parse_onnx_from_file(const char* path, void** handle_out) {
    if (path == nullptr || handle_out == nullptr) {
        return ge::GRAPH_PARAM_INVALID;
    }
    *handle_out = nullptr;
    try {
        ge::Graph graph;
        std::map<ge::AscendString, ge::AscendString> parser_params;
        graphStatus st = ge::aclgrphParseONNX(path, parser_params, graph);
        if (st != ge::GRAPH_SUCCESS) {
            return st;
        }
        *handle_out = register_graph(std::move(graph));
        return ge::GRAPH_SUCCESS;
    } catch (...) {
        // 例外不允许跨越 extern "C" 边界
        return ge::GRAPH_FAILED;
    }
}

// 从内存中的 ONNX 模型字节解析计算图（桥接 ge::aclgrphParseONNXFromMem）。
// 成功/失败语义同上。
graphStatus cann_grph_parse_onnx_from_mem(const char* buffer, size_t size,
                                          void** handle_out) {
    if (buffer == nullptr || size == 0 || handle_out == nullptr) {
        return ge::GRAPH_PARAM_INVALID;
    }
    *handle_out = nullptr;
    try {
        ge::Graph graph;
        std::map<ge::AscendString, ge::AscendString> parser_params;
        graphStatus st = ge::aclgrphParseONNXFromMem(buffer, size, parser_params, graph);
        if (st != ge::GRAPH_SUCCESS) {
            return st;
        }
        *handle_out = register_graph(std::move(graph));
        return ge::GRAPH_SUCCESS;
    } catch (...) {
        return ge::GRAPH_FAILED;
    }
}

// 编译句柄指向的图并保存为 .om 模型文件
// （桥接 ge::aclgrphBuildModel + ge::aclgrphSaveModel，build_options 传空表）。
// 注：不调用 ge::aclgrphBuildInitialize —— 8.x 构建前无需显式初始化；
// 若目标环境要求，可在 cann 层 Session 前置处理。
graphStatus cann_grph_build_model(void* handle, const char* save_path) {
    if (handle == nullptr || save_path == nullptr) {
        return ge::GRAPH_PARAM_INVALID;
    }
    try {
        std::shared_ptr<ge::Graph>* graph = lookup_graph(handle);
        if (graph == nullptr) {
            return ge::GRAPH_PARAM_INVALID;
        }
        // 用非废弃重载：build_options 为 AscendString map（空表）；
        // 空 map 构造只涉及内联函数，不引入 AscendString 外部符号。
        std::map<ge::AscendString, ge::AscendString> build_options;
        ge::ModelBufferData model;
        graphStatus st = ge::aclgrphBuildModel(**graph, build_options, model);
        if (st != ge::GRAPH_SUCCESS) {
            return st;
        }
        if (model.data == nullptr || model.length == 0) {
            return ge::GRAPH_FAILED;
        }
        // const char_t* 重载为非废弃形式（char_t = char）
        return ge::aclgrphSaveModel(save_path, model);
    } catch (...) {
        return ge::GRAPH_FAILED;
    }
}

// 释放图句柄：从注册表移除并析构底层 ge::Graph 实例。
// 句柄不存在或已释放返回 GRAPH_PARAM_INVALID。
graphStatus cann_grph_destroy(void* handle) {
    if (handle == nullptr) {
        return ge::GRAPH_PARAM_INVALID;
    }
    return g_graph_registry.erase(handle) == 1 ? ge::GRAPH_SUCCESS
                                               : ge::GRAPH_PARAM_INVALID;
}

}  // extern "C"
