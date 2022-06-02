# revpi-rs

This crate lets you control your [RevolutionPi](https://revolutionpi.com/) with rust

## PiControl

revpi-rs provides multiple ways to interface with piControl, the easiest being the `PiControl` struct:

```rust
use revpi::picontrol::{PiControl, Value};

let pi = PiControl::new().unwrap();
pi.set_value("RevPiLED", Value::Byte(42)).unwrap();
let val = pi.get_value("Core_Temperature").unwrap();
println!("{}", val); // e.g. 42
```

With the `macro` feature turned on, there's also a macro to automatically create getters and setters from the config:

```rust
use revpi::revpi;

revpi!(RevPi);

let pi = RevPi::new().unwrap();
pi.set_RevPiLED(42).unwrap();
let val = pi.get_Core_Temperature().unwrap();
println!("{}", val); // e.g. 42
```

Both examples do the same thing, just in different ways

## RSC

Types to read and write the rsc file format are provided with the feature `rsc`, which is enabled by default.
These types implement serde's Serialize and Deserialize traits so they can be used with serde_json:

```rust
use revpi::rsc::RSC;
use serde_json;
use std::fs::File;

let f = File::open("/etc/revpi/config.rsc").unwrap();
let rsc: RSC = serde_json::from_reader(f).unwrap();
println!("{:?}", rsc);
```
