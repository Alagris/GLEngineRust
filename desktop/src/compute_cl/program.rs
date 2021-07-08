use crate::compute_cl::context::ClGlContext;
use ocl::core::{Program, Context, Error};
use crate::compute_cl::kernel_builder::KernelBuilder;
use std::ffi::CString;
use failure::Fail;

pub struct ClGlProgram{
    program:Program
}

impl ClGlProgram{
    pub fn new(context:&ClGlContext, src:CString)->Result<Self,Error>{
        let program = ocl::core::create_program_with_source(context.context(), &[src])?;
        ocl::core::build_program(&program, Some(&[context.device()]), &CString::new("")?,None, None)?;
        Ok(Self{program})

    }
    pub fn program(&self)->&Program{
        &self.program
    }
    pub fn kernel_builder<S: AsRef<str>>(&self, name:S) -> Result<KernelBuilder, ocl::core::Error> {
        KernelBuilder::new(self.program(),name)
    }

}