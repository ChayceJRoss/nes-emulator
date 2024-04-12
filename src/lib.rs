//! # # NES Emulator in Rust
//!This is emulator made following the tutorial outlined by: 
//![Rafael Bagmanov - NES Emulator](https://bugzmanov.github.io/nes_ebook/chapter_1.html).

//!# Build Instructions 
//!To build this project ```cargo run```. To see documentation for the API run ```cargo doc --open```


extern crate lazy_static;

pub mod cpu;
pub mod opcodes;