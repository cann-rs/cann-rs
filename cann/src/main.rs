use cann::version::Version;

fn main() {
    match Version::str() {
        Ok(v) => println!("CANN version: {}", v),
        Err(e) => println!("CANN version: not detected ({})", e),
    }
    match Version::num() {
        Ok(n) => println!("CANN version num: {}", n),
        Err(e) => println!("CANN version num: not detected ({})", e),
    }
}
