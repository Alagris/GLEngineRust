use std::fmt::{Formatter, Display};
use std::marker::PhantomData;
use ocl::core::{DeviceInfoResult, DeviceInfo, ArgVal, ContextProperties};
use std::fs::File;
use std::io::Write;
use crate::compute_cl::buffer::Buffer;
use crate::compute_cl::kernel_builder::KernelBuilder;
use crate::compute_cl::num::Num;
use sdl2::video::GLContext;
use self::ocl::core::{GlContextInfoResult, CommandQueue, Context, Error, DeviceId};
use self::ocl::{MemFlags, Queue};
use std::ffi::CString;
use crate::compute_cl::program::ClGlProgram;
use crate::resources::Resources;
use failure::err_msg;

extern crate ocl;

#[derive(Clone)]
pub struct ClGlContext {
    context:Context,
    queue:CommandQueue,
    device:DeviceId
}

impl ClGlContext {

    pub fn new(gl_context:&GLContext) -> Result<Self, Error> {
        println!("Initialising OpenCL context");
        let mut properties = ocl::builders::ContextProperties::new();
        let raw = unsafe{gl_context.raw()};
        let platform_id = ocl::core::default_platform()?;
        let props = ContextProperties::new().platform(platform_id).gl_context(raw);
        let device = match ocl::core::get_gl_context_info_khr(&props,ocl::core::GlContextInfo::CurrentDevice)?{
            GlContextInfoResult::CurrentDevice(device_id) => device_id,
            GlContextInfoResult::Devices(_) => unreachable!()
        };
        let context = ocl::core::create_context(Some(&properties),
                                           &[device], None, None)?;
        let queue = ocl::core::create_command_queue(&context, &device, None)?;
        Ok(Self{context,queue, device})
    }

    pub fn compile(&self, src:CString) -> Result<ClGlProgram, Error> {
        ClGlProgram::new(&self,src)
    }

    pub fn compile_from_res(&self, res:&Resources, resource_name:&str) -> Result<ClGlProgram, failure::Error> {
        ClGlProgram::new(&self,res.load_cstring(resource_name)?).map_err(err_msg)
    }

    pub fn buffer_from_slice<T:Num>(&self, flags:MemFlags, slice:&[T]) -> Result<Buffer<T>, ocl::core::Error> {
        Buffer::from_slice(self.context(),flags,slice)
    }
    pub unsafe fn buffer_empty<T:Num>(&self, flags:MemFlags, len:usize) -> Result<Buffer<T>, ocl::core::Error> {
        Buffer::empty(self.context(),flags,len)
    }
    pub fn queue(&self) -> &CommandQueue{
        &self.queue
    }
    pub fn context(&self) -> &Context{
        &self.context
    }
    pub fn device(&self) -> DeviceId{
        self.device
    }
    pub fn buffer_filled<T:Num>(&self, flags:MemFlags, len:usize, fill_val:T) -> Result<Buffer<T>, ocl::core::Error> {
        let mut buff = unsafe{self.buffer_empty(flags,len)}?;
        buff.fill(self.queue(),fill_val)?;
        Ok(buff)
    }



}

