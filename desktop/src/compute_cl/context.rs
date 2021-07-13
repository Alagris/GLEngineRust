use std::fmt::{Formatter, Display};
use std::marker::PhantomData;
use std::fs::File;
use std::io::Write;
// use crate::compute_cl::buffer::Buffer;
// use crate::compute_cl::kernel_builder::KernelBuilder;
// use crate::compute_cl::num::Num;
use sdl2::video::GLContext;
use std::ffi::CString;
// use crate::compute_cl::program::ClGlProgram;
use crate::resources::Resources;
use failure::err_msg;
use crate::render_gl::buffer::{BufferType, BufferUsage};
use crate::compute_cl::error::{Error, ErrCode};
use cl_sys::*;
use crate::compute_cl::buffer::Buffer;
use crate::compute_cl::program::Program;


pub struct Context {
    context: cl_context,
    queue: cl_command_queue,
    device: cl_device_id,
    platform: cl_platform_id,
}

impl Context {
    fn default_platform() -> Result<cl_platform_id, Error> {
        // Get the number of platforms
        // Get the platform ids.
        let mut id: cl_platform_id = std::ptr::null_mut();
        let p: *mut cl_platform_id = (&mut id) as *mut cl_platform_id;
        let status = unsafe {
            clGetPlatformIDs(1, p as *mut *mut c_void, std::ptr::null_mut())
        };
        if id.is_null() {
            Err(Error::new(CL_PLATFORM_NOT_FOUND_KHR, String::from("No platform available")))
        } else {
            Error::result(||id, status, ||String::from("Failed getting platform ID"))
        }
    }

    fn default_device(platform:cl_platform_id) -> Result<cl_device_id, Error> {
        // Get the number of platforms
        // Get the platform ids.
        let mut id: cl_device_id = std::ptr::null_mut();
        let p: *mut cl_device_id = (&mut id) as *mut cl_device_id;
        let status = unsafe {
            clGetDeviceIDs(platform, CL_DEVICE_TYPE_GPU, 1, p,std::ptr::null_mut())
        };
        if id.is_null() {
            Err(Error::new(CL_PLATFORM_NOT_FOUND_KHR, String::from("No device available")))
        } else {
            Error::result(||id, status, ||String::from("Failed getting platform ID"))
        }
    }

    fn gl_props(platform:cl_platform_id, device:cl_device_id, gl_context:cl_context_properties)->[cl_context_properties;7]{
        [
            //OpenCL platform
            cl_sys::CL_CONTEXT_PLATFORM as cl_context_properties, platform as cl_context_properties,
            //OpenGL context
            cl_sys::CL_GL_CONTEXT_KHR,   gl_context,
            //HDC used to create the OpenGL context
            cl_sys::CL_GLX_DISPLAY_KHR,   device as cl_context_properties,
            0
        ]
    }
    fn create_context(device:cl_device_id, properties:&mut [cl_context_properties])->Result<cl_context,Error>{
        let mut status: cl_int = CL_INVALID_VALUE;
        let context:cl_context = unsafe{clCreateContext(properties.as_mut_ptr(),1,&device,None,std::ptr::null_mut(),&mut status)};
        Error::result(||context,status,||String::from("Failed creating OpenCL context"))
    }
    fn create_queue(device:cl_device_id, context:cl_context)->Result<cl_command_queue,Error>{
        let mut status: cl_int = CL_INVALID_VALUE;
        let queue:cl_command_queue = unsafe{clCreateCommandQueue(context,device,CL_QUEUE_PROFILING_ENABLE,&mut status)};
        Error::result(||queue,status,||String::from("Failed creating OpenCL command queue"))
    }
    pub fn new(gl_context: &GLContext) -> Result<Self, Error> {
        println!("Initialising OpenCL context");
        let raw = unsafe { gl_context.raw() };
        println!("Getting default opencl platform");
        let platform = Self::default_platform()?;
        let device = Self::default_device(platform)?;
        let mut props = Self::gl_props(platform,device,raw as cl_context_properties);
        let context:cl_context = Self::create_context(device,&mut props)?;
        let queue:cl_command_queue = Self::create_queue(device,context)?;
        Ok(Self{
            context,
            queue,
            device,
            platform
        })
    }

    pub fn compile(&self, src: &str) -> Result<Program, Error> {
        let mut status: cl_int = CL_INVALID_VALUE;
        let sources = &[src.as_ptr() as *const c_char];
        let lengths = &[src.len()];

        let program:cl_program = unsafe{clCreateProgramWithSource(self.context(), 1,sources.as_ptr(),lengths.as_ptr(),&mut status)};
        if status != 0{
            return Err(Error::new(status, String::from("Failed allocating OpenCL program from source")));
        }
        let options = b"-cl-std=CL1.1 -cl-mad-enable -Werror\0".as_ptr() as *const c_char;
        let status = unsafe{
            cl_sys::clBuildProgram(program,1,&self.device(),options,None,std::ptr::null_mut())
        };
        Error::result(||Program::new(program),status,||{
            if status == CL_BUILD_PROGRAM_FAILURE {
                let mut log_size = 0;
                unsafe{
                    clGetProgramBuildInfo(program, self.device, CL_PROGRAM_BUILD_LOG, 0, std::ptr::null_mut(), &mut log_size);
                }
                let mut log = Vec::<u8>::with_capacity(log_size);
                println!("Log size={}",log_size);
                let log = unsafe{
                    clGetProgramBuildInfo(program, self.device, CL_PROGRAM_BUILD_LOG, log_size, log.as_mut_ptr() as *mut c_void, std::ptr::null_mut());
                    log.set_len(log_size);
                    String::from_utf8_unchecked(log)
                };
                println!("Log={}",log);
                log
            }else{
                String::from("Failed building OpenCL program from source")
            }
        })

    }

    pub fn compile_from_res(&self, res: &Resources, resource_name: &str) -> Result<Program, failure::Error> {
        self.compile(&res.load(resource_name).map_err(err_msg)?).map_err(err_msg)
    }
    pub fn queue(&self) -> cl_command_queue {
        self.queue
    }
    pub fn platform(&self) -> cl_platform_id {
        self.platform
    }
    pub fn context(&self) -> cl_context {
        self.context
    }
    pub fn device(&self) -> cl_device_id {
        self.device
    }

    // pub fn buffer_filled<T: Num>(&self, flags: MemFlags, len: usize, fill_val: T) -> Result<Buffer<T>, ocl::core::Error> {
    //     let mut buff = unsafe { self.buffer_empty(flags, len) }?;
    //     buff.fill(self.queue(), fill_val)?;
    //     Ok(buff)
    // }
    pub fn buffer_from_gl<T, B: BufferType, U: BufferUsage>(&self, buff: &crate::render_gl::buffer::Buffer<B, T, U>, flags: cl_mem_flags) -> Result<Buffer<T>, Error> {
        let mut status: cl_int = CL_INVALID_VALUE;
        let mem:cl_mem = unsafe {clCreateFromGLBuffer(self.context, flags, buff.id(),&mut status) };
        Error::result(||Buffer::new(mem,buff.len(), None),status,||String::from("Failed creating OpenCL buffer from OpenGL buffer object"))
    }
    pub unsafe fn buffer_empty<T>(&self, flags: cl_mem_flags, len: usize) -> Result<Buffer<T>, Error> {
        let mut status: cl_int = CL_INVALID_VALUE;
        let mem:cl_mem = unsafe {clCreateBuffer(self.context, flags, len*std::mem::size_of::<T>(),std::ptr::null_mut(),&mut status) };
        Error::result(||Buffer::new(mem,len, None),status,||String::from("Failed allocating empty OpenCL buffer"))
    }
    pub fn buffer_from_slice<T>(&self, mut flags: cl_mem_flags, host_slice: &[T]) -> Result<Buffer<T>, Error> {
        let mut status: cl_int = CL_INVALID_VALUE;
        let mem:cl_mem = unsafe {clCreateBuffer(self.context, flags, host_slice.len()*std::mem::size_of::<T>(),host_slice.as_ptr() as *mut c_void,&mut status) };
        Error::result(||Buffer::new(mem,host_slice.len(), None),status,||String::from("Failed allocating empty OpenCL buffer"))
    }
}

impl Clone for Context{
    fn clone(&self) -> Self {
        let status = unsafe { clRetainContext(self.context) };
        Error::result(||(),status,||String::from("Failed releasing OpenCL context")).unwrap();
        let status = unsafe { clRetainCommandQueue(self.queue) };
        Error::result(||(),status,||String::from("Failed releasing OpenCL command queue")).unwrap();
        Self{
            context: self.context,
            queue: self.queue,
            device: self.device,
            platform: self.platform
        }
    }
}

impl Drop for Context{
    fn drop(&mut self) {
        let status = unsafe { clReleaseContext(self.context) };
        Error::result(||(),status,||String::from("Failed releasing OpenCL context")).unwrap();
        let status = unsafe { clReleaseCommandQueue(self.queue) };
        Error::result(||(),status,||String::from("Failed releasing OpenCL command queue")).unwrap();
    }
}

