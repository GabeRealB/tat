#![feature(adt_const_params)]
#![feature(trim_prefix_suffix)]

use std::error::Error;

pub mod ast;
pub mod lexer;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
