//! ACL 数据类型/格式枚举。
//!
//! 对应 CANN 头文件 `include/acl/acl_base_rt.h`（CANN 8.5.0）第 133-188 行，
//! C 枚举 `aclDataType` / `aclFormat` 的整组抄录（底层类型 `int`）。
//! 数值按头文件抄录，全部带出处注释；头文件改动需同步更新本文件。

use std::ffi::c_int;

/// 张量数据类型（C 枚举 `aclDataType`，底层为 `int`）。
///
/// 对应 `acl_base_rt.h` 第 133-163 行枚举，取值见 `ACL_*` 常量。
#[allow(non_camel_case_types)]
pub type aclDataType = c_int;

/// 数据类型未定义。
// acl_base_rt.h: ACL_DT_UNDEFINED = -1
pub const ACL_DT_UNDEFINED: aclDataType = -1;
/// 32 位浮点（FP32）。
// acl_base_rt.h: ACL_FLOAT = 0
pub const ACL_FLOAT: aclDataType = 0;
/// 16 位浮点（FP16）。
// acl_base_rt.h: ACL_FLOAT16 = 1
pub const ACL_FLOAT16: aclDataType = 1;
/// 8 位有符号整数。
// acl_base_rt.h: ACL_INT8 = 2
pub const ACL_INT8: aclDataType = 2;
/// 32 位有符号整数。
// acl_base_rt.h: ACL_INT32 = 3
pub const ACL_INT32: aclDataType = 3;
/// 8 位无符号整数。
// acl_base_rt.h: ACL_UINT8 = 4
pub const ACL_UINT8: aclDataType = 4;
/// 16 位有符号整数。
// acl_base_rt.h: ACL_INT16 = 6
pub const ACL_INT16: aclDataType = 6;
/// 16 位无符号整数。
// acl_base_rt.h: ACL_UINT16 = 7
pub const ACL_UINT16: aclDataType = 7;
/// 32 位无符号整数。
// acl_base_rt.h: ACL_UINT32 = 8
pub const ACL_UINT32: aclDataType = 8;
/// 64 位有符号整数。
// acl_base_rt.h: ACL_INT64 = 9
pub const ACL_INT64: aclDataType = 9;
/// 64 位无符号整数。
// acl_base_rt.h: ACL_UINT64 = 10
pub const ACL_UINT64: aclDataType = 10;
/// 64 位浮点（FP64）。
// acl_base_rt.h: ACL_DOUBLE = 11
pub const ACL_DOUBLE: aclDataType = 11;
/// 布尔。
// acl_base_rt.h: ACL_BOOL = 12
pub const ACL_BOOL: aclDataType = 12;
/// 字符串。
// acl_base_rt.h: ACL_STRING = 13
pub const ACL_STRING: aclDataType = 13;
/// 复数（64 位，2×FP32）。
// acl_base_rt.h: ACL_COMPLEX64 = 16
pub const ACL_COMPLEX64: aclDataType = 16;
/// 复数（128 位，2×FP64）。
// acl_base_rt.h: ACL_COMPLEX128 = 17
pub const ACL_COMPLEX128: aclDataType = 17;
/// bfloat16 浮点。
// acl_base_rt.h: ACL_BF16 = 27
pub const ACL_BF16: aclDataType = 27;
/// 4 位有符号整数。
// acl_base_rt.h: ACL_INT4 = 29
pub const ACL_INT4: aclDataType = 29;
/// 1 位无符号整数。
// acl_base_rt.h: ACL_UINT1 = 30
pub const ACL_UINT1: aclDataType = 30;
/// 复数（32 位）。
// acl_base_rt.h: ACL_COMPLEX32 = 33
pub const ACL_COMPLEX32: aclDataType = 33;
/// 8 位浮点（hiFLOAT8）。
// acl_base_rt.h: ACL_HIFLOAT8 = 34
pub const ACL_HIFLOAT8: aclDataType = 34;
/// 8 位浮点（E5M2）。
// acl_base_rt.h: ACL_FLOAT8_E5M2 = 35
pub const ACL_FLOAT8_E5M2: aclDataType = 35;
/// 8 位浮点（E4M3FN）。
// acl_base_rt.h: ACL_FLOAT8_E4M3FN = 36
pub const ACL_FLOAT8_E4M3FN: aclDataType = 36;
/// 8 位浮点（E8M0）。
// acl_base_rt.h: ACL_FLOAT8_E8M0 = 37
pub const ACL_FLOAT8_E8M0: aclDataType = 37;
/// 6 位浮点（E3M2）。
// acl_base_rt.h: ACL_FLOAT6_E3M2 = 38
pub const ACL_FLOAT6_E3M2: aclDataType = 38;
/// 6 位浮点（E2M3）。
// acl_base_rt.h: ACL_FLOAT6_E2M3 = 39
pub const ACL_FLOAT6_E2M3: aclDataType = 39;
/// 4 位浮点（E2M1）。
// acl_base_rt.h: ACL_FLOAT4_E2M1 = 40
pub const ACL_FLOAT4_E2M1: aclDataType = 40;
/// 4 位浮点（E1M2）。
// acl_base_rt.h: ACL_FLOAT4_E1M2 = 41
pub const ACL_FLOAT4_E1M2: aclDataType = 41;

/// 张量内存布局格式（C 枚举 `aclFormat`，底层为 `int`）。
///
/// 对应 `acl_base_rt.h` 第 166-188 行枚举，取值见 `ACL_FORMAT_*` 常量。
#[allow(non_camel_case_types)]
pub type aclFormat = c_int;

/// 格式未定义。
// acl_base_rt.h: ACL_FORMAT_UNDEFINED = -1
pub const ACL_FORMAT_UNDEFINED: aclFormat = -1;
/// NCHW 四维格式。
// acl_base_rt.h: ACL_FORMAT_NCHW = 0
pub const ACL_FORMAT_NCHW: aclFormat = 0;
/// NHWC 四维格式。
// acl_base_rt.h: ACL_FORMAT_NHWC = 1
pub const ACL_FORMAT_NHWC: aclFormat = 1;
/// 通用 N 维格式。
// acl_base_rt.h: ACL_FORMAT_ND = 2
pub const ACL_FORMAT_ND: aclFormat = 2;
/// 5D 格式（C1=C/16，C0=16）。
// acl_base_rt.h: ACL_FORMAT_NC1HWC0 = 3
pub const ACL_FORMAT_NC1HWC0: aclFormat = 3;
/// 分形 Z（小方块）格式。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_Z = 4
pub const ACL_FORMAT_FRACTAL_Z: aclFormat = 4;
/// NC1HWC0 变体（C0=4）。
// acl_base_rt.h: ACL_FORMAT_NC1HWC0_C04 = 12
pub const ACL_FORMAT_NC1HWC0_C04: aclFormat = 12;
/// HWCN 格式。
// acl_base_rt.h: ACL_FORMAT_HWCN = 16
pub const ACL_FORMAT_HWCN: aclFormat = 16;
/// NDHWC 5D 格式。
// acl_base_rt.h: ACL_FORMAT_NDHWC = 27
pub const ACL_FORMAT_NDHWC: aclFormat = 27;
/// 分形 NZ（大数据块）格式。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_NZ = 29
pub const ACL_FORMAT_FRACTAL_NZ: aclFormat = 29;
/// NCDHW 5D 格式。
// acl_base_rt.h: ACL_FORMAT_NCDHW = 30
pub const ACL_FORMAT_NCDHW: aclFormat = 30;
/// NDC1HWC0 格式。
// acl_base_rt.h: ACL_FORMAT_NDC1HWC0 = 32
pub const ACL_FORMAT_NDC1HWC0: aclFormat = 32;
/// 3D 分形 Z 格式。
// acl_base_rt.h: ACL_FRACTAL_Z_3D = 33
pub const ACL_FRACTAL_Z_3D: aclFormat = 33;
/// NC 二维格式。
// acl_base_rt.h: ACL_FORMAT_NC = 35
pub const ACL_FORMAT_NC: aclFormat = 35;
/// NCL 三维格式。
// acl_base_rt.h: ACL_FORMAT_NCL = 47
pub const ACL_FORMAT_NCL: aclFormat = 47;
/// 分形 NZ（C0=16）。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_NZ_C0_16 = 50
pub const ACL_FORMAT_FRACTAL_NZ_C0_16: aclFormat = 50;
/// 分形 NZ（C0=32）。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_NZ_C0_32 = 51
pub const ACL_FORMAT_FRACTAL_NZ_C0_32: aclFormat = 51;
/// 分形 NZ（C0=2）。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_NZ_C0_2 = 52
pub const ACL_FORMAT_FRACTAL_NZ_C0_2: aclFormat = 52;
/// 分形 NZ（C0=4）。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_NZ_C0_4 = 53
pub const ACL_FORMAT_FRACTAL_NZ_C0_4: aclFormat = 53;
/// 分形 NZ（C0=8）。
// acl_base_rt.h: ACL_FORMAT_FRACTAL_NZ_C0_8 = 54
pub const ACL_FORMAT_FRACTAL_NZ_C0_8: aclFormat = 54;

#[cfg(test)]
mod tests {
    use super::*;

    /// 类型底层为 `c_int`（C 枚举底层类型 `int`）。
    #[test]
    fn test_type_underlying_is_i32() {
        assert_eq!(size_of::<aclDataType>(), size_of::<i32>());
        assert_eq!(size_of::<aclFormat>(), size_of::<i32>());
    }

    /// 逐个断言 aclDataType 数值与 acl_base_rt.h 一致（防漂移；头文件改动需同步更新本文件）。
    #[test]
    fn test_acl_data_type_values_match_header() {
        assert_eq!(ACL_DT_UNDEFINED, -1);
        assert_eq!(ACL_FLOAT, 0);
        assert_eq!(ACL_FLOAT16, 1);
        assert_eq!(ACL_INT8, 2);
        assert_eq!(ACL_INT32, 3);
        assert_eq!(ACL_UINT8, 4);
        assert_eq!(ACL_INT16, 6);
        assert_eq!(ACL_UINT16, 7);
        assert_eq!(ACL_UINT32, 8);
        assert_eq!(ACL_INT64, 9);
        assert_eq!(ACL_UINT64, 10);
        assert_eq!(ACL_DOUBLE, 11);
        assert_eq!(ACL_BOOL, 12);
        assert_eq!(ACL_STRING, 13);
        assert_eq!(ACL_COMPLEX64, 16);
        assert_eq!(ACL_COMPLEX128, 17);
        assert_eq!(ACL_BF16, 27);
        assert_eq!(ACL_INT4, 29);
        assert_eq!(ACL_UINT1, 30);
        assert_eq!(ACL_COMPLEX32, 33);
        assert_eq!(ACL_HIFLOAT8, 34);
        assert_eq!(ACL_FLOAT8_E5M2, 35);
        assert_eq!(ACL_FLOAT8_E4M3FN, 36);
        assert_eq!(ACL_FLOAT8_E8M0, 37);
        assert_eq!(ACL_FLOAT6_E3M2, 38);
        assert_eq!(ACL_FLOAT6_E2M3, 39);
        assert_eq!(ACL_FLOAT4_E2M1, 40);
        assert_eq!(ACL_FLOAT4_E1M2, 41);
    }

    /// 逐个断言 aclFormat 数值与 acl_base_rt.h 一致（防漂移；头文件改动需同步更新本文件）。
    #[test]
    fn test_acl_format_values_match_header() {
        assert_eq!(ACL_FORMAT_UNDEFINED, -1);
        assert_eq!(ACL_FORMAT_NCHW, 0);
        assert_eq!(ACL_FORMAT_NHWC, 1);
        assert_eq!(ACL_FORMAT_ND, 2);
        assert_eq!(ACL_FORMAT_NC1HWC0, 3);
        assert_eq!(ACL_FORMAT_FRACTAL_Z, 4);
        assert_eq!(ACL_FORMAT_NC1HWC0_C04, 12);
        assert_eq!(ACL_FORMAT_HWCN, 16);
        assert_eq!(ACL_FORMAT_NDHWC, 27);
        assert_eq!(ACL_FORMAT_FRACTAL_NZ, 29);
        assert_eq!(ACL_FORMAT_NCDHW, 30);
        assert_eq!(ACL_FORMAT_NDC1HWC0, 32);
        assert_eq!(ACL_FRACTAL_Z_3D, 33);
        assert_eq!(ACL_FORMAT_NC, 35);
        assert_eq!(ACL_FORMAT_NCL, 47);
        assert_eq!(ACL_FORMAT_FRACTAL_NZ_C0_16, 50);
        assert_eq!(ACL_FORMAT_FRACTAL_NZ_C0_32, 51);
        assert_eq!(ACL_FORMAT_FRACTAL_NZ_C0_2, 52);
        assert_eq!(ACL_FORMAT_FRACTAL_NZ_C0_4, 53);
        assert_eq!(ACL_FORMAT_FRACTAL_NZ_C0_8, 54);
    }
}
