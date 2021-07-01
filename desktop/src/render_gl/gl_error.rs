use failure::Backtrace;

pub fn drain_gl_errors(gl:&gl::Gl){
    loop{
        let err = unsafe{gl.GetError()};
        if err == gl::NO_ERROR{
            break
        }else{
            eprintln!("{}",error_to_string(err));
            eprintln!("{:?}", Backtrace::new());
        }

    }
}
pub fn error_to_string(gl_error:gl::types::GLenum)->&'static str{
    match gl_error{
        gl::INVALID_ENUM=>"GL_INVALID_ENUM",
        gl::INVALID_VALUE => "GL_INVALID_VALUE",
        gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
        gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
        gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
        gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        gl::CONTEXT_LOST => "GL_CONTEXT_LOST",
        // gl::TABLE_TOO_LARGE => "GL_TABLE_TOO_LARGE1",
        _ => "<unknown error code>"
    }
}