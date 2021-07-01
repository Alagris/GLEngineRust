use crate::render_gl::Program;
use crate::resources::Resources;
use sdl2::video::Window;
use sdl2::{Sdl, TimerSubsystem};

pub fn run(gl: gl::Gl, res: Resources, sdl: Sdl, window: Window, timer:TimerSubsystem) -> Result<(), failure::Error> {
    let shader_program = Program::from_res(&gl, &res, "shaders/triangle").unwrap();

    shader_program.set_used();

    let vertices: Vec<f32> = vec![
        // positions      // colors
        0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
        -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
        0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // top
    ];
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
        gl.GenBuffers(1, &mut vbo);
    }
    unsafe {
        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl.BufferData(
            gl::ARRAY_BUFFER,                                                       // target
            (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
            vertices.as_ptr() as *const gl::types::GLvoid, // pointer to data
            gl::STATIC_DRAW,                               // usage
        );
        gl.BindBuffer(gl::ARRAY_BUFFER, 0); // unbind the buffer
    }
    let mut vao: gl::types::GLuint = 0;
    unsafe {
        gl.GenVertexArrays(1, &mut vao);
    }
    unsafe {
        gl.BindVertexArray(vao);

        gl.BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl.EnableVertexAttribArray(0); // this is "layout (location = 0)" in vertex shader
        gl.VertexAttribPointer(
            0,         // index of the generic vertex attribute ("layout (location = 0)")
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            std::ptr::null(),                                     // offset of the first component
        );

        gl.EnableVertexAttribArray(1); // this is "layout (location = 0)" in vertex shader
        gl.VertexAttribPointer(
            1,         // index of the generic vertex attribute ("layout (location = 0)")
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE, // normalized (int-to-float conversion)
            (6 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
            (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid, // offset of the first component
        );

        gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        gl.BindVertexArray(0);
    }
    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
        shader_program.set_used();
        unsafe {
            gl.BindVertexArray(vao);
            gl.DrawArrays(
                gl::TRIANGLES, // mode
                0,             // starting index in the enabled arrays
                3,             // number of indices to be rendered
            );
        }
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                _ => {}
            }
        }
        window.gl_swap_window();
        // render window contents here
    }
    Ok(())
}
