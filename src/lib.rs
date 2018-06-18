//!
//! Provides input/output for 3D models and materials.
//!
//! Currently, only OBJ is supported.
//!
//! ```
//! # extern crate aitios_asset;
//! use aitios_asset::obj;
//!
//! # fn main() {
//! // Load entities from an OBJ as aitios_scene entities
//! let entities = obj::load("tests/cube.obj")
//!     .unwrap();
//!
//! // Save them back to OBJ/MTL
//! obj::save(
//!     entities.iter(),
//!     Some("tests/cube_with_mtl.obj"),
//!     Some("tests/cube_with_mtl.mtl")
//! ).unwrap();
//! # }
//! ```
//!

extern crate aitios_geom as geom;
extern crate aitios_scene as scene;
extern crate tobj;
extern crate pathdiff;
extern crate failure;
#[macro_use] extern crate failure_derive;

pub mod obj;
pub mod err;
