#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![feature(trim_prefix_suffix)]
#![feature(generic_const_exprs)]
#![feature(maybe_uninit_array_assume_init)]

use std::error::Error;

pub mod ast;
pub mod lexer;
pub mod packed_stream;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
