#![feature(array_map)]
extern crate sdl2;
extern crate gl;
extern crate image;
extern crate genmesh;
extern crate rand;
extern crate num_traits;
extern crate nalgebra_glm as glm;
#[macro_use] extern crate failure;
#[macro_use] extern crate render_gl_derive;
#[macro_use] extern crate num_derive;

use std::ffi::CString;
use gl::Gl;
use resources::Resources;
use std::path::Path;
use crate::render_gl::Program;

mod render_gl;
mod resources;
mod scene;
mod demos;
mod blocks;

fn main() {
    if let Err(e) = scene::run() {
        println!("Error! {}", failure_to_string(e));
    }
}


pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e
        .iter_chain()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
    {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}