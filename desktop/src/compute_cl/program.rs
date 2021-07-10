use crate::compute_cl::kernel_builder::KernelBuilder;
use std::ffi::CString;
use failure::Fail;
use cl_sys::*;
use crate::compute_cl::kernel::Kernel;
use crate::compute_cl::error::Error;

pub struct Program {
    program:cl_program
}

impl Program {
    pub fn new(program:cl_program)->Self{
        Self{program}
    }
    pub fn program(&self)->cl_program{
        self.program
    }
    pub fn kernel_builder(&self, name:&str) -> Result<KernelBuilder, Error> {
        let c_str = CString::new(name).unwrap();
        let mut status: cl_int = CL_INVALID_VALUE;
        let kernel:cl_kernel = unsafe{clCreateKernel(self.program, c_str.as_ptr(),&mut status)};
        Error::result(||KernelBuilder::new(Kernel::new(kernel)),status,||format!("Failed creating OpenCL kernel for \"{}\"",name))
    }
}