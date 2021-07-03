use crate::resources::Resources;
use failure::err_msg;
use std::ffi::CStr;
use std::path::Path;

use crate::render_gl::gl_error::drain_gl_errors;

pub const GL_VER_MINOR: u8 = 1;
pub const GL_VER_MAJOR: u8 = 4;
pub fn supported_since(major: u8, minor: u8) -> bool {
    major < GL_VER_MAJOR || (major == GL_VER_MAJOR && minor <= GL_VER_MINOR)
}

extern "system" fn message_callback(
    _source: gl::types::GLenum,
    type_of_error: gl::types::GLenum,
    _id: gl::types::GLuint,
    severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const gl::types::GLchar,
    _user_param: *mut gl::types::GLvoid,
) {
    let type_of_error = match type_of_error {
        gl::NO_ERROR => String::from("GL_NO_ERROR"),
        gl::INVALID_VALUE => String::from("GL_INVALID_VALUE"),
        gl::INVALID_OPERATION => String::from("GL_INVALID_OPERATION"),
        gl::STACK_OVERFLOW => String::from("GL_STACK_OVERFLOW"),
        gl::STACK_UNDERFLOW => String::from("GL_STACK_UNDERFLOW"),
        gl::OUT_OF_MEMORY => String::from("GL_OUT_OF_MEMORY"),
        gl::INVALID_FRAMEBUFFER_OPERATION => String::from("GL_INVALID_FRAMEBUFFER_OPERATION"),
        gl::CONTEXT_LOST => String::from("GL_CONTEXT_LOST"),
        gl::DEBUG_SOURCE_API => String::from("DEBUG_SOURCE_API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => String::from("DEBUG_SOURCE_WINDOW_SYSTEM"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => String::from("DEBUG_SOURCE_SHADER_COMPILER"),
        gl::DEBUG_SOURCE_THIRD_PARTY => String::from("DEBUG_SOURCE_THIRD_PARTY"),
        gl::DEBUG_SOURCE_APPLICATION => String::from("DEBUG_SOURCE_APPLICATION"),
        gl::DEBUG_SOURCE_OTHER => String::from("DEBUG_SOURCE_OTHER"),
        gl::DEBUG_TYPE_ERROR => String::from("DEBUG_TYPE_ERROR"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => String::from("DEBUG_TYPE_DEPRECATED_BEHAVIOR"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => String::from("DEBUG_TYPE_UNDEFINED_BEHAVIOR"),
        gl::DEBUG_TYPE_PORTABILITY => String::from("DEBUG_TYPE_PORTABILITY"),
        gl::DEBUG_TYPE_PERFORMANCE => String::from("DEBUG_TYPE_PERFORMANCE"),
        gl::DEBUG_TYPE_MARKER => String::from("DEBUG_TYPE_MARKER"),
        gl::DEBUG_TYPE_PUSH_GROUP => String::from("DEBUG_TYPE_PUSH_GROUP"),
        gl::DEBUG_TYPE_POP_GROUP => String::from("DEBUG_TYPE_POP_GROUP"),
        gl::DEBUG_TYPE_OTHER => String::from("DEBUG_TYPE_OTHER"),
        x => x.to_string(),
    };
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => String::from("HIGH"),
        gl::DEBUG_SEVERITY_MEDIUM => String::from("MEDIUM"),
        gl::DEBUG_SEVERITY_LOW => String::from("LOW"),
        gl::DEBUG_SEVERITY_NOTIFICATION => String::from("NOTIFICATION"),
        x => x.to_string(),
    };
    let msg = if let Ok(s) = unsafe { CStr::from_ptr(message) }.to_str() {
        s
    } else {
        "invalid c string!"
    };
    println!(
        "GL CALLBACK: type = {}, severity = {}, message = {}\n",
        type_of_error, severity, msg
    );
}

pub fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();
    #[cfg(target_os = "maxos")]{
        sdl2::hint::set("SDL_HINT_MAC_CTRL_CLICK_EMULATE_RIGHT_CLICK","1");
    }
    let sdl = sdl2::init().map_err(err_msg)?;

    let video_subsystem = sdl.video().map_err(err_msg)?;
    let timer = sdl.timer().map_err(err_msg)?;
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(GL_VER_MAJOR, GL_VER_MINOR);
    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl()
        .resizable()
        .build()?;
    sdl.mouse().set_relative_mouse_mode(true);
    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });
    drain_gl_errors(&gl);
    unsafe {
        gl.DepthFunc(gl::LESS);
        drain_gl_errors(&gl);
        gl.Enable(gl::DEPTH_TEST);
        drain_gl_errors(&gl);
        if supported_since(4, 3) {
            gl.Enable(gl::DEBUG_OUTPUT);
            drain_gl_errors(&gl);
            gl.DebugMessageCallback(Some(message_callback), 0 as *const gl::types::GLvoid);
            drain_gl_errors(&gl);
        }
    }

    crate::demos::block_world::run(gl, res, sdl, window, timer)
}
