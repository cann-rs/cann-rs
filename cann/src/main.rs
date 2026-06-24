//! CANN 版本查询演示程序。
//!
//! 通过 `cann` crate 查询 CANN 版本字符串和版本号。
//! 无 NPU 驱动时显示"not detected"。

use cann::version::Version;

fn main() {
    match Version::str() {
        Ok(v) => println!("CANN 版本: {}", v),
        Err(e) => println!("CANN 版本: 未检测到 ({})", e),
    }
    match Version::num() {
        Ok(n) => println!("CANN 版本号: {}", n),
        Err(e) => println!("CANN 版本号: 未检测到 ({})", e),
    }
}
