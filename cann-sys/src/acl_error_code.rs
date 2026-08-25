//! ACL 运行时错误码常量。
//!
//! 对应头文件 `include/acl/error_codes/rt_error_codes.h`（CANN 8.5.0）。
//! 数值按头文件抄录，全部带出处注释；本机 SDK 路径：`/usr/local/Ascend/cann-8.5.0`。
//!
//! 码段语义（供 cann 层错误分类）：
//! - `107xxx` 参数/句柄/上下文类（Invalid）
//! - `207xxx` 资源/内存类（含 OOM：207001/207002/207018）
//! - `507xxx` 驱动/执行类（Internal/异常/超时）
//!
use crate::acl_base_rt::aclError;

/// param invalid
// rt_error_codes.h: ACL_ERROR_RT_PARAM_INVALID = 107000
pub const ACL_ERROR_RT_PARAM_INVALID: aclError = 107000;

/// invalid device id
// rt_error_codes.h: ACL_ERROR_RT_INVALID_DEVICEID = 107001
pub const ACL_ERROR_RT_INVALID_DEVICEID: aclError = 107001;

/// current context null
// rt_error_codes.h: ACL_ERROR_RT_CONTEXT_NULL = 107002
pub const ACL_ERROR_RT_CONTEXT_NULL: aclError = 107002;

/// stream not in current context
// rt_error_codes.h: ACL_ERROR_RT_STREAM_CONTEXT = 107003
pub const ACL_ERROR_RT_STREAM_CONTEXT: aclError = 107003;

/// model not in current context
// rt_error_codes.h: ACL_ERROR_RT_MODEL_CONTEXT = 107004
pub const ACL_ERROR_RT_MODEL_CONTEXT: aclError = 107004;

/// stream not in model
// rt_error_codes.h: ACL_ERROR_RT_STREAM_MODEL = 107005
pub const ACL_ERROR_RT_STREAM_MODEL: aclError = 107005;

/// event timestamp invalid
// rt_error_codes.h: ACL_ERROR_RT_EVENT_TIMESTAMP_INVALID = 107006
pub const ACL_ERROR_RT_EVENT_TIMESTAMP_INVALID: aclError = 107006;

/// event timestamp reversal
// rt_error_codes.h: ACL_ERROR_RT_EVENT_TIMESTAMP_REVERSAL = 107007
pub const ACL_ERROR_RT_EVENT_TIMESTAMP_REVERSAL: aclError = 107007;

/// memory address unaligned
// rt_error_codes.h: ACL_ERROR_RT_ADDR_UNALIGNED = 107008
pub const ACL_ERROR_RT_ADDR_UNALIGNED: aclError = 107008;

/// open file failed
// rt_error_codes.h: ACL_ERROR_RT_FILE_OPEN = 107009
pub const ACL_ERROR_RT_FILE_OPEN: aclError = 107009;

/// write file failed
// rt_error_codes.h: ACL_ERROR_RT_FILE_WRITE = 107010
pub const ACL_ERROR_RT_FILE_WRITE: aclError = 107010;

/// error subscribe stream
// rt_error_codes.h: ACL_ERROR_RT_STREAM_SUBSCRIBE = 107011
pub const ACL_ERROR_RT_STREAM_SUBSCRIBE: aclError = 107011;

/// error subscribe thread
// rt_error_codes.h: ACL_ERROR_RT_THREAD_SUBSCRIBE = 107012
pub const ACL_ERROR_RT_THREAD_SUBSCRIBE: aclError = 107012;

/// group not set
// rt_error_codes.h: ACL_ERROR_RT_GROUP_NOT_SET = 107013
pub const ACL_ERROR_RT_GROUP_NOT_SET: aclError = 107013;

/// group not create
// rt_error_codes.h: ACL_ERROR_RT_GROUP_NOT_CREATE = 107014
pub const ACL_ERROR_RT_GROUP_NOT_CREATE: aclError = 107014;

/// callback not register to stream
// rt_error_codes.h: ACL_ERROR_RT_STREAM_NO_CB_REG = 107015
pub const ACL_ERROR_RT_STREAM_NO_CB_REG: aclError = 107015;

/// invalid memory type
// rt_error_codes.h: ACL_ERROR_RT_INVALID_MEMORY_TYPE = 107016
pub const ACL_ERROR_RT_INVALID_MEMORY_TYPE: aclError = 107016;

/// invalid handle
// rt_error_codes.h: ACL_ERROR_RT_INVALID_HANDLE = 107017
pub const ACL_ERROR_RT_INVALID_HANDLE: aclError = 107017;

/// invalid malloc type
// rt_error_codes.h: ACL_ERROR_RT_INVALID_MALLOC_TYPE = 107018
pub const ACL_ERROR_RT_INVALID_MALLOC_TYPE: aclError = 107018;

/// wait timeout
// rt_error_codes.h: ACL_ERROR_RT_WAIT_TIMEOUT = 107019
pub const ACL_ERROR_RT_WAIT_TIMEOUT: aclError = 107019;

/// task timeout
// rt_error_codes.h: ACL_ERROR_RT_TASK_TIMEOUT = 107020
pub const ACL_ERROR_RT_TASK_TIMEOUT: aclError = 107020;

/// not set sysparamopt
// rt_error_codes.h: ACL_ERROR_RT_SYSPARAMOPT_NOT_SET = 107021
pub const ACL_ERROR_RT_SYSPARAMOPT_NOT_SET: aclError = 107021;

/// device task aborting
// rt_error_codes.h: ACL_ERROR_RT_DEVICE_TASK_ABORT = 107022
pub const ACL_ERROR_RT_DEVICE_TASK_ABORT: aclError = 107022;

/// stream aborting
// rt_error_codes.h: ACL_ERROR_RT_STREAM_ABORT = 107023
pub const ACL_ERROR_RT_STREAM_ABORT: aclError = 107023;

/// capture dependency failure
// rt_error_codes.h: ACL_ERROR_RT_CAPTURE_DEPENDENCY = 107024
pub const ACL_ERROR_RT_CAPTURE_DEPENDENCY: aclError = 107024;

/// invalid capture model
// rt_error_codes.h: ACL_ERROR_RT_STREAM_UNJOINED = 107025
pub const ACL_ERROR_RT_STREAM_UNJOINED: aclError = 107025;

/// model is captured
// rt_error_codes.h: ACL_ERROR_RT_MODEL_CAPTURED = 107026
pub const ACL_ERROR_RT_MODEL_CAPTURED: aclError = 107026;

/// stream is captured
// rt_error_codes.h: ACL_ERROR_RT_STREAM_CAPTURED = 107027
pub const ACL_ERROR_RT_STREAM_CAPTURED: aclError = 107027;

/// event is captured
// rt_error_codes.h: ACL_ERROR_RT_EVENT_CAPTURED = 107028
pub const ACL_ERROR_RT_EVENT_CAPTURED: aclError = 107028;

/// stream is not in capture status
// rt_error_codes.h: ACL_ERROR_RT_STREAM_NOT_CAPTURED = 107029
pub const ACL_ERROR_RT_STREAM_NOT_CAPTURED: aclError = 107029;

/// stream is captured, not support current oper
// rt_error_codes.h: ACL_ERROR_RT_CAPTURE_MODE_NOT_SUPPORT = 107030
pub const ACL_ERROR_RT_CAPTURE_MODE_NOT_SUPPORT: aclError = 107030;

/// a disallowed implicit dependency from defalut stream
// rt_error_codes.h: ACL_ERROR_RT_STREAM_CAPTURE_IMPLICIT = 107031
pub const ACL_ERROR_RT_STREAM_CAPTURE_IMPLICIT: aclError = 107031;

/// device task aborting stop before post process
// rt_error_codes.h: ACL_ERROR_RT_TASK_ABORT_STOP = 107035
pub const ACL_ERROR_RT_TASK_ABORT_STOP: aclError = 107035;

/// the capture was not initiated in this stream
// rt_error_codes.h: ACL_ERROR_RT_STREAM_CAPTURE_UNMATCHED = 107036
pub const ACL_ERROR_RT_STREAM_CAPTURE_UNMATCHED: aclError = 107036;

/// the model is still running
// rt_error_codes.h: ACL_ERROR_RT_MODEL_RUNNING = 107037
pub const ACL_ERROR_RT_MODEL_RUNNING: aclError = 107037;

/// the thread of end capture and begin capture is not same
// rt_error_codes.h: ACL_ERROR_RT_STREAM_CAPTURE_WRONG_THREAD = 107038
pub const ACL_ERROR_RT_STREAM_CAPTURE_WRONG_THREAD: aclError = 107038;

/// feature not support
// rt_error_codes.h: ACL_ERROR_RT_FEATURE_NOT_SUPPORT = 207000
pub const ACL_ERROR_RT_FEATURE_NOT_SUPPORT: aclError = 207000;

/// memory allocation error, only used by out of memory
// rt_error_codes.h: ACL_ERROR_RT_MEMORY_ALLOCATION = 207001
pub const ACL_ERROR_RT_MEMORY_ALLOCATION: aclError = 207001;

/// memory free error
// rt_error_codes.h: ACL_ERROR_RT_MEMORY_FREE = 207002
pub const ACL_ERROR_RT_MEMORY_FREE: aclError = 207002;

/// aicore over flow
// rt_error_codes.h: ACL_ERROR_RT_AICORE_OVER_FLOW = 207003
pub const ACL_ERROR_RT_AICORE_OVER_FLOW: aclError = 207003;

/// no device
// rt_error_codes.h: ACL_ERROR_RT_NO_DEVICE = 207004
pub const ACL_ERROR_RT_NO_DEVICE: aclError = 207004;

/// resource alloc fail
// rt_error_codes.h: ACL_ERROR_RT_RESOURCE_ALLOC_FAIL = 207005
pub const ACL_ERROR_RT_RESOURCE_ALLOC_FAIL: aclError = 207005;

/// no permission
// rt_error_codes.h: ACL_ERROR_RT_NO_PERMISSION = 207006
pub const ACL_ERROR_RT_NO_PERMISSION: aclError = 207006;

/// no event resource
// rt_error_codes.h: ACL_ERROR_RT_NO_EVENT_RESOURCE = 207007
pub const ACL_ERROR_RT_NO_EVENT_RESOURCE: aclError = 207007;

/// no stream resource
// rt_error_codes.h: ACL_ERROR_RT_NO_STREAM_RESOURCE = 207008
pub const ACL_ERROR_RT_NO_STREAM_RESOURCE: aclError = 207008;

/// no notify resource
// rt_error_codes.h: ACL_ERROR_RT_NO_NOTIFY_RESOURCE = 207009
pub const ACL_ERROR_RT_NO_NOTIFY_RESOURCE: aclError = 207009;

/// no model resource
// rt_error_codes.h: ACL_ERROR_RT_NO_MODEL_RESOURCE = 207010
pub const ACL_ERROR_RT_NO_MODEL_RESOURCE: aclError = 207010;

/// no cdq resource
// rt_error_codes.h: ACL_ERROR_RT_NO_CDQ_RESOURCE = 207011
pub const ACL_ERROR_RT_NO_CDQ_RESOURCE: aclError = 207011;

/// over limit
// rt_error_codes.h: ACL_ERROR_RT_OVER_LIMIT = 207012
pub const ACL_ERROR_RT_OVER_LIMIT: aclError = 207012;

/// queue is empty
// rt_error_codes.h: ACL_ERROR_RT_QUEUE_EMPTY = 207013
pub const ACL_ERROR_RT_QUEUE_EMPTY: aclError = 207013;

/// queue is full
// rt_error_codes.h: ACL_ERROR_RT_QUEUE_FULL = 207014
pub const ACL_ERROR_RT_QUEUE_FULL: aclError = 207014;

/// repeated init
// rt_error_codes.h: ACL_ERROR_RT_REPEATED_INIT = 207015
pub const ACL_ERROR_RT_REPEATED_INIT: aclError = 207015;

/// aivec over flow
// rt_error_codes.h: ACL_ERROR_RT_AIVEC_OVER_FLOW = 207016
pub const ACL_ERROR_RT_AIVEC_OVER_FLOW: aclError = 207016;

/// common over flow
// rt_error_codes.h: ACL_ERROR_RT_OVER_FLOW = 207017
pub const ACL_ERROR_RT_OVER_FLOW: aclError = 207017;

/// device oom
// rt_error_codes.h: ACL_ERROR_RT_DEVICE_OOM = 207018
pub const ACL_ERROR_RT_DEVICE_OOM: aclError = 207018;

/// not support to update this op
// rt_error_codes.h: ACL_ERROR_RT_FEATURE_NOT_SUPPORT_UPDATE_OP = 207019
pub const ACL_ERROR_RT_FEATURE_NOT_SUPPORT_UPDATE_OP: aclError = 207019;

/// runtime internal error
// rt_error_codes.h: ACL_ERROR_RT_INTERNAL_ERROR = 507000
pub const ACL_ERROR_RT_INTERNAL_ERROR: aclError = 507000;

/// ts internel error
// rt_error_codes.h: ACL_ERROR_RT_TS_ERROR = 507001
pub const ACL_ERROR_RT_TS_ERROR: aclError = 507001;

/// task full in stream
// rt_error_codes.h: ACL_ERROR_RT_STREAM_TASK_FULL = 507002
pub const ACL_ERROR_RT_STREAM_TASK_FULL: aclError = 507002;

/// task empty in stream
// rt_error_codes.h: ACL_ERROR_RT_STREAM_TASK_EMPTY = 507003
pub const ACL_ERROR_RT_STREAM_TASK_EMPTY: aclError = 507003;

/// stream not complete
// rt_error_codes.h: ACL_ERROR_RT_STREAM_NOT_COMPLETE = 507004
pub const ACL_ERROR_RT_STREAM_NOT_COMPLETE: aclError = 507004;

/// end of sequence
// rt_error_codes.h: ACL_ERROR_RT_END_OF_SEQUENCE = 507005
pub const ACL_ERROR_RT_END_OF_SEQUENCE: aclError = 507005;

/// event not complete
// rt_error_codes.h: ACL_ERROR_RT_EVENT_NOT_COMPLETE = 507006
pub const ACL_ERROR_RT_EVENT_NOT_COMPLETE: aclError = 507006;

/// context release error
// rt_error_codes.h: ACL_ERROR_RT_CONTEXT_RELEASE_ERROR = 507007
pub const ACL_ERROR_RT_CONTEXT_RELEASE_ERROR: aclError = 507007;

/// soc version error
// rt_error_codes.h: ACL_ERROR_RT_SOC_VERSION = 507008
pub const ACL_ERROR_RT_SOC_VERSION: aclError = 507008;

/// task type not support
// rt_error_codes.h: ACL_ERROR_RT_TASK_TYPE_NOT_SUPPORT = 507009
pub const ACL_ERROR_RT_TASK_TYPE_NOT_SUPPORT: aclError = 507009;

/// ts lost heartbeat
// rt_error_codes.h: ACL_ERROR_RT_LOST_HEARTBEAT = 507010
pub const ACL_ERROR_RT_LOST_HEARTBEAT: aclError = 507010;

/// model execute failed
// rt_error_codes.h: ACL_ERROR_RT_MODEL_EXECUTE = 507011
pub const ACL_ERROR_RT_MODEL_EXECUTE: aclError = 507011;

/// report timeout
// rt_error_codes.h: ACL_ERROR_RT_REPORT_TIMEOUT = 507012
pub const ACL_ERROR_RT_REPORT_TIMEOUT: aclError = 507012;

/// sys dma error
// rt_error_codes.h: ACL_ERROR_RT_SYS_DMA = 507013
pub const ACL_ERROR_RT_SYS_DMA: aclError = 507013;

/// aicore timeout
// rt_error_codes.h: ACL_ERROR_RT_AICORE_TIMEOUT = 507014
pub const ACL_ERROR_RT_AICORE_TIMEOUT: aclError = 507014;

/// aicore exception
// rt_error_codes.h: ACL_ERROR_RT_AICORE_EXCEPTION = 507015
pub const ACL_ERROR_RT_AICORE_EXCEPTION: aclError = 507015;

/// aicore trap exception
// rt_error_codes.h: ACL_ERROR_RT_AICORE_TRAP_EXCEPTION = 507016
pub const ACL_ERROR_RT_AICORE_TRAP_EXCEPTION: aclError = 507016;

/// aicpu timeout
// rt_error_codes.h: ACL_ERROR_RT_AICPU_TIMEOUT = 507017
pub const ACL_ERROR_RT_AICPU_TIMEOUT: aclError = 507017;

/// aicpu exception
// rt_error_codes.h: ACL_ERROR_RT_AICPU_EXCEPTION = 507018
pub const ACL_ERROR_RT_AICPU_EXCEPTION: aclError = 507018;

/// aicpu datadump response error
// rt_error_codes.h: ACL_ERROR_RT_AICPU_DATADUMP_RSP_ERR = 507019
pub const ACL_ERROR_RT_AICPU_DATADUMP_RSP_ERR: aclError = 507019;

/// aicpu model operate response error
// rt_error_codes.h: ACL_ERROR_RT_AICPU_MODEL_RSP_ERR = 507020
pub const ACL_ERROR_RT_AICPU_MODEL_RSP_ERR: aclError = 507020;

/// profiling error
// rt_error_codes.h: ACL_ERROR_RT_PROFILING_ERROR = 507021
pub const ACL_ERROR_RT_PROFILING_ERROR: aclError = 507021;

/// ipc error
// rt_error_codes.h: ACL_ERROR_RT_IPC_ERROR = 507022
pub const ACL_ERROR_RT_IPC_ERROR: aclError = 507022;

/// model abort normal
// rt_error_codes.h: ACL_ERROR_RT_MODEL_ABORT_NORMAL = 507023
pub const ACL_ERROR_RT_MODEL_ABORT_NORMAL: aclError = 507023;

/// kernel unregistering
// rt_error_codes.h: ACL_ERROR_RT_KERNEL_UNREGISTERING = 507024
pub const ACL_ERROR_RT_KERNEL_UNREGISTERING: aclError = 507024;

/// ringbuffer not init
// rt_error_codes.h: ACL_ERROR_RT_RINGBUFFER_NOT_INIT = 507025
pub const ACL_ERROR_RT_RINGBUFFER_NOT_INIT: aclError = 507025;

/// ringbuffer no data
// rt_error_codes.h: ACL_ERROR_RT_RINGBUFFER_NO_DATA = 507026
pub const ACL_ERROR_RT_RINGBUFFER_NO_DATA: aclError = 507026;

/// kernel lookup error
// rt_error_codes.h: ACL_ERROR_RT_KERNEL_LOOKUP = 507027
pub const ACL_ERROR_RT_KERNEL_LOOKUP: aclError = 507027;

/// kernel register duplicate
// rt_error_codes.h: ACL_ERROR_RT_KERNEL_DUPLICATE = 507028
pub const ACL_ERROR_RT_KERNEL_DUPLICATE: aclError = 507028;

/// debug register failed
// rt_error_codes.h: ACL_ERROR_RT_DEBUG_REGISTER_FAIL = 507029
pub const ACL_ERROR_RT_DEBUG_REGISTER_FAIL: aclError = 507029;

/// debug unregister failed
// rt_error_codes.h: ACL_ERROR_RT_DEBUG_UNREGISTER_FAIL = 507030
pub const ACL_ERROR_RT_DEBUG_UNREGISTER_FAIL: aclError = 507030;

/// label not in current context
// rt_error_codes.h: ACL_ERROR_RT_LABEL_CONTEXT = 507031
pub const ACL_ERROR_RT_LABEL_CONTEXT: aclError = 507031;

/// program register num use out
// rt_error_codes.h: ACL_ERROR_RT_PROGRAM_USE_OUT = 507032
pub const ACL_ERROR_RT_PROGRAM_USE_OUT: aclError = 507032;

/// device setup error
// rt_error_codes.h: ACL_ERROR_RT_DEV_SETUP_ERROR = 507033
pub const ACL_ERROR_RT_DEV_SETUP_ERROR: aclError = 507033;

/// vector core timeout
// rt_error_codes.h: ACL_ERROR_RT_VECTOR_CORE_TIMEOUT = 507034
pub const ACL_ERROR_RT_VECTOR_CORE_TIMEOUT: aclError = 507034;

/// vector core exception
// rt_error_codes.h: ACL_ERROR_RT_VECTOR_CORE_EXCEPTION = 507035
pub const ACL_ERROR_RT_VECTOR_CORE_EXCEPTION: aclError = 507035;

/// vector core trap exception
// rt_error_codes.h: ACL_ERROR_RT_VECTOR_CORE_TRAP_EXCEPTION = 507036
pub const ACL_ERROR_RT_VECTOR_CORE_TRAP_EXCEPTION: aclError = 507036;

/// cdq alloc batch abnormal
// rt_error_codes.h: ACL_ERROR_RT_CDQ_BATCH_ABNORMAL = 507037
pub const ACL_ERROR_RT_CDQ_BATCH_ABNORMAL: aclError = 507037;

/// can not change die mode
// rt_error_codes.h: ACL_ERROR_RT_DIE_MODE_CHANGE_ERROR = 507038
pub const ACL_ERROR_RT_DIE_MODE_CHANGE_ERROR: aclError = 507038;

/// single die mode can not set die
// rt_error_codes.h: ACL_ERROR_RT_DIE_SET_ERROR = 507039
pub const ACL_ERROR_RT_DIE_SET_ERROR: aclError = 507039;

/// invalid die id
// rt_error_codes.h: ACL_ERROR_RT_INVALID_DIEID = 507040
pub const ACL_ERROR_RT_INVALID_DIEID: aclError = 507040;

/// die mode not set
// rt_error_codes.h: ACL_ERROR_RT_DIE_MODE_NOT_SET = 507041
pub const ACL_ERROR_RT_DIE_MODE_NOT_SET: aclError = 507041;

/// aic trap read overflow
// rt_error_codes.h: ACL_ERROR_RT_AICORE_TRAP_READ_OVERFLOW = 507042
pub const ACL_ERROR_RT_AICORE_TRAP_READ_OVERFLOW: aclError = 507042;

/// aic trap write overflow
// rt_error_codes.h: ACL_ERROR_RT_AICORE_TRAP_WRITE_OVERFLOW = 507043
pub const ACL_ERROR_RT_AICORE_TRAP_WRITE_OVERFLOW: aclError = 507043;

/// aiv trap read overflow
// rt_error_codes.h: ACL_ERROR_RT_VECTOR_CORE_TRAP_READ_OVERFLOW = 507044
pub const ACL_ERROR_RT_VECTOR_CORE_TRAP_READ_OVERFLOW: aclError = 507044;

/// aiv trap write overflow
// rt_error_codes.h: ACL_ERROR_RT_VECTOR_CORE_TRAP_WRITE_OVERFLOW = 507045
pub const ACL_ERROR_RT_VECTOR_CORE_TRAP_WRITE_OVERFLOW: aclError = 507045;

/// stream sync time out
// rt_error_codes.h: ACL_ERROR_RT_STREAM_SYNC_TIMEOUT = 507046
pub const ACL_ERROR_RT_STREAM_SYNC_TIMEOUT: aclError = 507046;

/// event sync time out
// rt_error_codes.h: ACL_ERROR_RT_EVENT_SYNC_TIMEOUT = 507047
pub const ACL_ERROR_RT_EVENT_SYNC_TIMEOUT: aclError = 507047;

/// ffts+ timeout
// rt_error_codes.h: ACL_ERROR_RT_FFTS_PLUS_TIMEOUT = 507048
pub const ACL_ERROR_RT_FFTS_PLUS_TIMEOUT: aclError = 507048;

/// ffts+ exception
// rt_error_codes.h: ACL_ERROR_RT_FFTS_PLUS_EXCEPTION = 507049
pub const ACL_ERROR_RT_FFTS_PLUS_EXCEPTION: aclError = 507049;

/// ffts+ trap exception
// rt_error_codes.h: ACL_ERROR_RT_FFTS_PLUS_TRAP_EXCEPTION = 507050
pub const ACL_ERROR_RT_FFTS_PLUS_TRAP_EXCEPTION: aclError = 507050;

/// hdc send msg fail
// rt_error_codes.h: ACL_ERROR_RT_SEND_MSG = 507051
pub const ACL_ERROR_RT_SEND_MSG: aclError = 507051;

/// copy data fail
// rt_error_codes.h: ACL_ERROR_RT_COPY_DATA = 507052
pub const ACL_ERROR_RT_COPY_DATA: aclError = 507052;

/// device MEM ERROR
// rt_error_codes.h: ACL_ERROR_RT_DEVICE_MEM_ERROR = 507053
pub const ACL_ERROR_RT_DEVICE_MEM_ERROR: aclError = 507053;

/// hbm Multi-bit ECC error
// rt_error_codes.h: ACL_ERROR_RT_HBM_MULTI_BIT_ECC_ERROR = 507054
pub const ACL_ERROR_RT_HBM_MULTI_BIT_ECC_ERROR: aclError = 507054;

/// suspect device MEM ERROR
// rt_error_codes.h: ACL_ERROR_RT_SUSPECT_DEVICE_MEM_ERROR = 507055
pub const ACL_ERROR_RT_SUSPECT_DEVICE_MEM_ERROR: aclError = 507055;

/// link ERROR
// rt_error_codes.h: ACL_ERROR_RT_LINK_ERROR = 507056
pub const ACL_ERROR_RT_LINK_ERROR: aclError = 507056;

/// suspect remote ERROR
// rt_error_codes.h: ACL_ERROR_RT_SUSPECT_REMOTE_ERROR = 507057
pub const ACL_ERROR_RT_SUSPECT_REMOTE_ERROR: aclError = 507057;

/// drv internal error
// rt_error_codes.h: ACL_ERROR_RT_DRV_INTERNAL_ERROR = 507899
pub const ACL_ERROR_RT_DRV_INTERNAL_ERROR: aclError = 507899;

/// aicpu internal error
// rt_error_codes.h: ACL_ERROR_RT_AICPU_INTERNAL_ERROR = 507900
pub const ACL_ERROR_RT_AICPU_INTERNAL_ERROR: aclError = 507900;

/// hdc disconnect
// rt_error_codes.h: ACL_ERROR_RT_SOCKET_CLOSE = 507901
pub const ACL_ERROR_RT_SOCKET_CLOSE: aclError = 507901;

/// aicpu info load response error
// rt_error_codes.h: ACL_ERROR_RT_AICPU_INFO_LOAD_RSP_ERR = 507902
pub const ACL_ERROR_RT_AICPU_INFO_LOAD_RSP_ERR: aclError = 507902;

/// capture status is invalidated
// rt_error_codes.h: ACL_ERROR_RT_STREAM_CAPTURE_INVALIDATED = 507903
pub const ACL_ERROR_RT_STREAM_CAPTURE_INVALIDATED: aclError = 507903;

/// hccl operation retry failed
// rt_error_codes.h: ACL_ERROR_RT_COMM_OP_RETRY_FAIL = 507904
pub const ACL_ERROR_RT_COMM_OP_RETRY_FAIL: aclError = 507904;

#[cfg(test)]
mod tests {
    use super::*;

    /// 逐个断言数值与 rt_error_codes.h 一致（防漂移；头文件改动需同步更新本文件）
    #[test]
    fn test_rt_error_constants_match_header() {
        assert_eq!(ACL_ERROR_RT_PARAM_INVALID, 107000);
        assert_eq!(ACL_ERROR_RT_INVALID_DEVICEID, 107001);
        assert_eq!(ACL_ERROR_RT_CONTEXT_NULL, 107002);
        assert_eq!(ACL_ERROR_RT_STREAM_CONTEXT, 107003);
        assert_eq!(ACL_ERROR_RT_MODEL_CONTEXT, 107004);
        assert_eq!(ACL_ERROR_RT_STREAM_MODEL, 107005);
        assert_eq!(ACL_ERROR_RT_EVENT_TIMESTAMP_INVALID, 107006);
        assert_eq!(ACL_ERROR_RT_EVENT_TIMESTAMP_REVERSAL, 107007);
        assert_eq!(ACL_ERROR_RT_ADDR_UNALIGNED, 107008);
        assert_eq!(ACL_ERROR_RT_FILE_OPEN, 107009);
        assert_eq!(ACL_ERROR_RT_FILE_WRITE, 107010);
        assert_eq!(ACL_ERROR_RT_STREAM_SUBSCRIBE, 107011);
        assert_eq!(ACL_ERROR_RT_THREAD_SUBSCRIBE, 107012);
        assert_eq!(ACL_ERROR_RT_GROUP_NOT_SET, 107013);
        assert_eq!(ACL_ERROR_RT_GROUP_NOT_CREATE, 107014);
        assert_eq!(ACL_ERROR_RT_STREAM_NO_CB_REG, 107015);
        assert_eq!(ACL_ERROR_RT_INVALID_MEMORY_TYPE, 107016);
        assert_eq!(ACL_ERROR_RT_INVALID_HANDLE, 107017);
        assert_eq!(ACL_ERROR_RT_INVALID_MALLOC_TYPE, 107018);
        assert_eq!(ACL_ERROR_RT_WAIT_TIMEOUT, 107019);
        assert_eq!(ACL_ERROR_RT_TASK_TIMEOUT, 107020);
        assert_eq!(ACL_ERROR_RT_SYSPARAMOPT_NOT_SET, 107021);
        assert_eq!(ACL_ERROR_RT_DEVICE_TASK_ABORT, 107022);
        assert_eq!(ACL_ERROR_RT_STREAM_ABORT, 107023);
        assert_eq!(ACL_ERROR_RT_CAPTURE_DEPENDENCY, 107024);
        assert_eq!(ACL_ERROR_RT_STREAM_UNJOINED, 107025);
        assert_eq!(ACL_ERROR_RT_MODEL_CAPTURED, 107026);
        assert_eq!(ACL_ERROR_RT_STREAM_CAPTURED, 107027);
        assert_eq!(ACL_ERROR_RT_EVENT_CAPTURED, 107028);
        assert_eq!(ACL_ERROR_RT_STREAM_NOT_CAPTURED, 107029);
        assert_eq!(ACL_ERROR_RT_CAPTURE_MODE_NOT_SUPPORT, 107030);
        assert_eq!(ACL_ERROR_RT_STREAM_CAPTURE_IMPLICIT, 107031);
        assert_eq!(ACL_ERROR_RT_TASK_ABORT_STOP, 107035);
        assert_eq!(ACL_ERROR_RT_STREAM_CAPTURE_UNMATCHED, 107036);
        assert_eq!(ACL_ERROR_RT_MODEL_RUNNING, 107037);
        assert_eq!(ACL_ERROR_RT_STREAM_CAPTURE_WRONG_THREAD, 107038);
        assert_eq!(ACL_ERROR_RT_FEATURE_NOT_SUPPORT, 207000);
        assert_eq!(ACL_ERROR_RT_MEMORY_ALLOCATION, 207001);
        assert_eq!(ACL_ERROR_RT_MEMORY_FREE, 207002);
        assert_eq!(ACL_ERROR_RT_AICORE_OVER_FLOW, 207003);
        assert_eq!(ACL_ERROR_RT_NO_DEVICE, 207004);
        assert_eq!(ACL_ERROR_RT_RESOURCE_ALLOC_FAIL, 207005);
        assert_eq!(ACL_ERROR_RT_NO_PERMISSION, 207006);
        assert_eq!(ACL_ERROR_RT_NO_EVENT_RESOURCE, 207007);
        assert_eq!(ACL_ERROR_RT_NO_STREAM_RESOURCE, 207008);
        assert_eq!(ACL_ERROR_RT_NO_NOTIFY_RESOURCE, 207009);
        assert_eq!(ACL_ERROR_RT_NO_MODEL_RESOURCE, 207010);
        assert_eq!(ACL_ERROR_RT_NO_CDQ_RESOURCE, 207011);
        assert_eq!(ACL_ERROR_RT_OVER_LIMIT, 207012);
        assert_eq!(ACL_ERROR_RT_QUEUE_EMPTY, 207013);
        assert_eq!(ACL_ERROR_RT_QUEUE_FULL, 207014);
        assert_eq!(ACL_ERROR_RT_REPEATED_INIT, 207015);
        assert_eq!(ACL_ERROR_RT_AIVEC_OVER_FLOW, 207016);
        assert_eq!(ACL_ERROR_RT_OVER_FLOW, 207017);
        assert_eq!(ACL_ERROR_RT_DEVICE_OOM, 207018);
        assert_eq!(ACL_ERROR_RT_FEATURE_NOT_SUPPORT_UPDATE_OP, 207019);
        assert_eq!(ACL_ERROR_RT_INTERNAL_ERROR, 507000);
        assert_eq!(ACL_ERROR_RT_TS_ERROR, 507001);
        assert_eq!(ACL_ERROR_RT_STREAM_TASK_FULL, 507002);
        assert_eq!(ACL_ERROR_RT_STREAM_TASK_EMPTY, 507003);
        assert_eq!(ACL_ERROR_RT_STREAM_NOT_COMPLETE, 507004);
        assert_eq!(ACL_ERROR_RT_END_OF_SEQUENCE, 507005);
        assert_eq!(ACL_ERROR_RT_EVENT_NOT_COMPLETE, 507006);
        assert_eq!(ACL_ERROR_RT_CONTEXT_RELEASE_ERROR, 507007);
        assert_eq!(ACL_ERROR_RT_SOC_VERSION, 507008);
        assert_eq!(ACL_ERROR_RT_TASK_TYPE_NOT_SUPPORT, 507009);
        assert_eq!(ACL_ERROR_RT_LOST_HEARTBEAT, 507010);
        assert_eq!(ACL_ERROR_RT_MODEL_EXECUTE, 507011);
        assert_eq!(ACL_ERROR_RT_REPORT_TIMEOUT, 507012);
        assert_eq!(ACL_ERROR_RT_SYS_DMA, 507013);
        assert_eq!(ACL_ERROR_RT_AICORE_TIMEOUT, 507014);
        assert_eq!(ACL_ERROR_RT_AICORE_EXCEPTION, 507015);
        assert_eq!(ACL_ERROR_RT_AICORE_TRAP_EXCEPTION, 507016);
        assert_eq!(ACL_ERROR_RT_AICPU_TIMEOUT, 507017);
        assert_eq!(ACL_ERROR_RT_AICPU_EXCEPTION, 507018);
        assert_eq!(ACL_ERROR_RT_AICPU_DATADUMP_RSP_ERR, 507019);
        assert_eq!(ACL_ERROR_RT_AICPU_MODEL_RSP_ERR, 507020);
        assert_eq!(ACL_ERROR_RT_PROFILING_ERROR, 507021);
        assert_eq!(ACL_ERROR_RT_IPC_ERROR, 507022);
        assert_eq!(ACL_ERROR_RT_MODEL_ABORT_NORMAL, 507023);
        assert_eq!(ACL_ERROR_RT_KERNEL_UNREGISTERING, 507024);
        assert_eq!(ACL_ERROR_RT_RINGBUFFER_NOT_INIT, 507025);
        assert_eq!(ACL_ERROR_RT_RINGBUFFER_NO_DATA, 507026);
        assert_eq!(ACL_ERROR_RT_KERNEL_LOOKUP, 507027);
        assert_eq!(ACL_ERROR_RT_KERNEL_DUPLICATE, 507028);
        assert_eq!(ACL_ERROR_RT_DEBUG_REGISTER_FAIL, 507029);
        assert_eq!(ACL_ERROR_RT_DEBUG_UNREGISTER_FAIL, 507030);
        assert_eq!(ACL_ERROR_RT_LABEL_CONTEXT, 507031);
        assert_eq!(ACL_ERROR_RT_PROGRAM_USE_OUT, 507032);
        assert_eq!(ACL_ERROR_RT_DEV_SETUP_ERROR, 507033);
        assert_eq!(ACL_ERROR_RT_VECTOR_CORE_TIMEOUT, 507034);
        assert_eq!(ACL_ERROR_RT_VECTOR_CORE_EXCEPTION, 507035);
        assert_eq!(ACL_ERROR_RT_VECTOR_CORE_TRAP_EXCEPTION, 507036);
        assert_eq!(ACL_ERROR_RT_CDQ_BATCH_ABNORMAL, 507037);
        assert_eq!(ACL_ERROR_RT_DIE_MODE_CHANGE_ERROR, 507038);
        assert_eq!(ACL_ERROR_RT_DIE_SET_ERROR, 507039);
        assert_eq!(ACL_ERROR_RT_INVALID_DIEID, 507040);
        assert_eq!(ACL_ERROR_RT_DIE_MODE_NOT_SET, 507041);
        assert_eq!(ACL_ERROR_RT_AICORE_TRAP_READ_OVERFLOW, 507042);
        assert_eq!(ACL_ERROR_RT_AICORE_TRAP_WRITE_OVERFLOW, 507043);
        assert_eq!(ACL_ERROR_RT_VECTOR_CORE_TRAP_READ_OVERFLOW, 507044);
        assert_eq!(ACL_ERROR_RT_VECTOR_CORE_TRAP_WRITE_OVERFLOW, 507045);
        assert_eq!(ACL_ERROR_RT_STREAM_SYNC_TIMEOUT, 507046);
        assert_eq!(ACL_ERROR_RT_EVENT_SYNC_TIMEOUT, 507047);
        assert_eq!(ACL_ERROR_RT_FFTS_PLUS_TIMEOUT, 507048);
        assert_eq!(ACL_ERROR_RT_FFTS_PLUS_EXCEPTION, 507049);
        assert_eq!(ACL_ERROR_RT_FFTS_PLUS_TRAP_EXCEPTION, 507050);
        assert_eq!(ACL_ERROR_RT_SEND_MSG, 507051);
        assert_eq!(ACL_ERROR_RT_COPY_DATA, 507052);
        assert_eq!(ACL_ERROR_RT_DEVICE_MEM_ERROR, 507053);
        assert_eq!(ACL_ERROR_RT_HBM_MULTI_BIT_ECC_ERROR, 507054);
        assert_eq!(ACL_ERROR_RT_SUSPECT_DEVICE_MEM_ERROR, 507055);
        assert_eq!(ACL_ERROR_RT_LINK_ERROR, 507056);
        assert_eq!(ACL_ERROR_RT_SUSPECT_REMOTE_ERROR, 507057);
        assert_eq!(ACL_ERROR_RT_DRV_INTERNAL_ERROR, 507899);
        assert_eq!(ACL_ERROR_RT_AICPU_INTERNAL_ERROR, 507900);
        assert_eq!(ACL_ERROR_RT_SOCKET_CLOSE, 507901);
        assert_eq!(ACL_ERROR_RT_AICPU_INFO_LOAD_RSP_ERR, 507902);
        assert_eq!(ACL_ERROR_RT_STREAM_CAPTURE_INVALIDATED, 507903);
        assert_eq!(ACL_ERROR_RT_COMM_OP_RETRY_FAIL, 507904);
    }
}
