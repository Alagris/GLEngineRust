extern crate sdl2;
extern crate gl;
extern crate image;
extern crate genmesh;
extern crate rand;
extern crate nalgebra_glm as glm;
#[macro_use] extern crate failure;
#[macro_use] extern crate render_gl_derive;
use std::ffi::CString;
use gl::Gl;
use resources::Resources;
use std::path::Path;
use crate::render_gl::Program;

mod render_gl;
mod resources;
mod scene;

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
//
// fn main() {
//     let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();
//
//     let sdl = sdl2::init().unwrap();
//     let video_subsystem = sdl.video().unwrap();
//     let gl_attr = video_subsystem.gl_attr();
//     gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
//     gl_attr.set_context_version(4, 1);
//     let window = video_subsystem
//         .window("Game", 900, 700)
//         .opengl()
//         .resizable()
//         .build()
//         .unwrap();
//     let gl_context = window.gl_create_context().unwrap();
//     let gl = Gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
//     unsafe {
//         gl.Viewport(0, 0, 900, 700);
//         gl.ClearColor(0.3, 0.3, 0.5, 1.0);
//     }
//
//
//     let shader_program = Program::from_res(
//         &gl, &res, "shaders/triangle"
//     ).unwrap();
//
//     shader_program.set_used();
//
//     let vertices: Vec<f32> = vec![
//         // positions      // colors
//         0.5, -0.5, 0.0,   1.0, 0.0, 0.0,   // bottom right
//         -0.5, -0.5, 0.0,  0.0, 1.0, 0.0,   // bottom left
//         0.0,  0.5, 0.0,   0.0, 0.0, 1.0    // top
//     ];
//     let mut vbo: gl::types::GLuint = 0;
//     unsafe {
//         gl.GenBuffers(1, &mut vbo);
//     }
//     unsafe {
//         gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
//         gl.BufferData(
//             gl::ARRAY_BUFFER, // target
//             (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
//             vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
//             gl::STATIC_DRAW, // usage
//         );
//         gl.BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
//     }
//     let mut vao: gl::types::GLuint = 0;
//     unsafe {
//         gl.GenVertexArrays(1, &mut vao);
//     }
//     unsafe {
//         gl.BindVertexArray(vao);
//
//         gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
//
//         gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
//         gl.VertexAttribPointer(
//             0, // index of the generic vertex attribute ("layout (location = 0)")
//             3, // the number of components per generic vertex attribute
//             gl::FLOAT, // data type
//             gl::FALSE, // normalized (int-to-float conversion)
//             (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
//             std::ptr::null() // offset of the first component
//         );
//
//         gl.EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
//         gl.VertexAttribPointer(
//             1, // index of the generic vertex attribute ("layout (location = 0)")
//             3, // the number of components per generic vertex attribute
//             gl::FLOAT, // data type
//             gl::FALSE, // normalized (int-to-float conversion)
//             (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
//             (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid // offset of the first component
//         );
//
//         gl.BindBuffer(gl::ARRAY_BUFFER, 0);
//         gl.BindVertexArray(0);
//     }
//     let mut event_pump = sdl.event_pump().unwrap();
//     'main: loop {
//         unsafe {
//             gl.Clear(gl::COLOR_BUFFER_BIT);
//         }
//         shader_program.set_used();
//         unsafe {
//             gl.BindVertexArray(vao);
//             gl.DrawArrays(
//                 gl::TRIANGLES, // mode
//                 0, // starting index in the enabled arrays
//                 3 // number of indices to be rendered
//             );
//         }
//         for event in event_pump.poll_iter() {
//             match event {
//                 sdl2::event::Event::Quit { .. } => break 'main,
//                 _ => {}
//             }
//         }
//         window.gl_swap_window();
//         // render window contents here
//     }
// }