use std::fmt::{Formatter, Display};
use std::marker::PhantomData;
use std::fs::File;
use std::io::Write;
// use crate::compute_cl::buffer::Buffer;
// use crate::compute_cl::kernel_builder::KernelBuilder;
// use crate::compute_cl::num::Num;
use sdl2::video::GLContext;
use std::ffi::{CString, CStr};
// use crate::compute_cl::program::ClGlProgram;
use crate::resources::Resources;
use failure::err_msg;
use std::mem::MaybeUninit;
use crate::compute_cl::error::ClGlError;

#[derive(Clone)]
pub struct ClGlContext {
    context:cl_sys::cl_context,
    queue:cl_sys::cl_command_queue,
    device:cl_sys::cl_device_id
}

impl ClGlContext {

    pub  fn new(gl_context:&GLContext) -> Result<Self, ClGlError> {
        println!("Initialising OpenCL context");
        let platforms:Vec<cl_sys::cl_platform_id> = cl3::platform::get_platform_ids()?;
        let platform:cl_sys::cl_platform_id = platforms.first().cloned().ok_or(cl_sys::CL_PLATFORM_NOT_FOUND_KHR)?;

        let clGetGLContextInfoKHR = unsafe{cl_sys::clGetExtensionFunctionAddressForPlatform(platform,  b"clGetGLContextInfoKHR\0" as *const i8) as cl_sys::clGetGLContextInfoKHR_fn};

        let gl_context = unsafe{gl_context.raw()};
        let mut props:[cl_sys::cl_context_properties;5] = [
                //OpenCL platform
                cl_sys::CL_CONTEXT_PLATFORM as cl_sys::cl_context_properties, platform as cl_sys::cl_context_properties,
                //OpenGL context
                cl_sys::CL_GL_CONTEXT_KHR,   gl_context as cl_sys::cl_context_properties,
                //HDC used to create the OpenGL context
                // cl_sys::CL_WGL_HDC_KHR,   hDC as cl_sys::cl_context_properties,
                0
        ];
        let mut device: MaybeUninit<cl_sys::cl_device_id> = MaybeUninit::uninit();
        (*clGetGLContextInfoKHR)(props.as_mut_ptr(),cl_sys::CL_CURRENT_DEVICE_FOR_GL_CONTEXT_KHR,std::mem::size_of::<cl_sys::cl_device_id>(),device,std::ptr::null());
        let device:cl_sys::cl_device_id = unsafe{device.assume_init()};
        // cl3::gl::get_gl_context_info_khr()
        // let device = match ?{
        //     GlContextInfoResult::CurrentDevice(device_id) => device_id,
        //     GlContextInfoResult::Devices(_) => unreachable!()
        // };
        // let context = ocl::core::create_context(Some(&properties),
        //                                    &[device], None, None)?;
        // let queue = ocl::core::create_command_queue(&context, &device, None)?;
        // Ok(Self{context,queue, device})
        panic!("Good job")
    }
    //
    // pub fn compile(&self, src:CString) -> Result<ClGlProgram, Error> {
    //     ClGlProgram::new(&self,src)
    // }
    //
    // pub fn compile_from_res(&self, res:&Resources, resource_name:&str) -> Result<ClGlProgram, failure::Error> {
    //     ClGlProgram::new(&self,res.load_cstring(resource_name)?).map_err(err_msg)
    // }
    //
    // pub fn buffer_from_slice<T:Num>(&self, flags:MemFlags, slice:&[T]) -> Result<Buffer<T>, ocl::core::Error> {
    //     Buffer::from_slice(self.context(),flags,slice)
    // }
    // pub unsafe fn buffer_empty<T:Num>(&self, flags:MemFlags, len:usize) -> Result<Buffer<T>, ocl::core::Error> {
    //     Buffer::empty(self.context(),flags,len)
    // }
    // pub fn queue(&self) -> &CommandQueue{
    //     &self.queue
    // }
    // pub fn context(&self) -> &Context{
    //     &self.context
    // }
    // pub fn device(&self) -> DeviceId{
    //     self.device
    // }
    // pub fn buffer_filled<T:Num>(&self, flags:MemFlags, len:usize, fill_val:T) -> Result<Buffer<T>, ocl::core::Error> {
    //     let mut buff = unsafe{self.buffer_empty(flags,len)}?;
    //     buff.fill(self.queue(),fill_val)?;
    //     Ok(buff)
    // }



}

