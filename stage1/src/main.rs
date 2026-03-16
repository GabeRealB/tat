#![allow(
    incomplete_features,
    clippy::too_many_arguments,
    clippy::missing_safety_doc
)]
#![feature(slice_ptr_get)]
#![feature(adt_const_params)]
#![feature(trim_prefix_suffix)]
#![feature(unsafe_cell_access)]
#![feature(generic_const_exprs)]
#![feature(arbitrary_self_types_pointers)]
#![feature(maybe_uninit_array_assume_init)]

use std::env;
use std::error::Error;
use std::path::PathBuf;

pub mod ast;
pub mod compilation_unit;
pub mod hir;
pub mod intern_pool;
pub mod lexer;
pub mod packed_stream;
pub mod sema;
pub mod target;
pub mod util;

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    let root_path = args.get(1).expect("fo file path provided");
    let root_path = PathBuf::from(root_path).canonicalize().unwrap();

    let cfg = compilation_unit::Config {
        target: None,
        root_file_path: root_path,
    };
    compilation_unit::compile(cfg)
}
