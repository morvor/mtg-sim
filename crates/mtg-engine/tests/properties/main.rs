//! Test binary for `tests/properties/`: metamorphic properties of whole games (see
//! `mtg_engine::properties` and `docs/PROPERTIES.md`). Every other `.rs` file in this
//! directory is included as a module automatically (see build.rs).

#![allow(unused_imports)]

include!(concat!(env!("OUT_DIR"), "/test_mods_properties.rs"));
