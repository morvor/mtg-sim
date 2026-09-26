//! Test binary for `tests/keywords/`. Every other `.rs` file in this directory is included
//! as a module automatically (see build.rs) — add tests by adding files.

#![allow(unused_imports)]

include!(concat!(env!("OUT_DIR"), "/test_mods_keywords.rs"));
