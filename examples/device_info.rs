//! 输出 CANN 芯片信息示例。
//!
//! 运行（需 `ffi` 特性与 CANN SDK）：
//!
//! ```bash
//! source /usr/local/Ascend/ascend-toolkit/latest/set_env.sh  # 或板子对应工具链
//! cargo run --example device_info --features ffi
//! ```
//!
//! 输出 CANN 软件版本、设备数量与每一台设备的 SOC 型号
//! （如 `Ascend910B1`、`Ascend310B` 等）。

use cann::device;
use cann::Version;
use cann::Context;

fn main() {
    // 1) CANN 软件版本（7.x 回退 aclrtGetVersion，8.x 用 aclsysGetVersionStr）
    match Version::str() {
        Ok(v) => println!("CANN 版本: {v}"),
        Err(e) => println!("CANN 版本: 未检测到 ({e})"),
    }
    match Version::num() {
        Ok(n) => println!("CANN 版本号: {n}"),
        Err(_) => {}
    }

    // 2) 初始化运行环境（进程级单次，幂等）
    let ctx = match Context::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("初始化失败: {e}");
            eprintln!("请检查 NPU 驱动与 CANN SDK（ASCEND_TOOLKIT_HOME）");
            return;
        }
    };

    // 3) 枚举设备与芯片型号
    match device::device_count() {
        Ok(count) => {
            println!("设备数量: {count}");
            for dev in 0..count {
                if let Err(e) = device::set_device(dev) {
                    println!("设备 {dev}: 绑定失败 ({e})");
                    continue;
                }
                match device::soc_name() {
                    Ok(soc) => println!("设备 {dev}: SOC = {soc}"),
                    Err(e) => println!("设备 {dev}: SOC 查询失败 ({e})"),
                }
                let _ = device::reset_device(dev); // 引用计数配对释放
            }
        }
        Err(e) => println!("设备数量查询失败: {e}"),
    }

    drop(ctx); // 进程退出前释放运行环境
}
