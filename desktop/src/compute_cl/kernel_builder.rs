use crate::compute_cl::buffer::Buffer;
use crate::compute_cl::num::Num;
use crate::compute_cl::program::Program;
use cl_sys::*;
use cl3::kernel::create_kernel;
use crate::compute_cl::kernel::Kernel;
use crate::compute_cl::error::Error;

pub struct KernelBuilder {
    kernel: Kernel,
    arg_idx: u32,
}

impl KernelBuilder {
    pub fn new(kernel: Kernel) -> Self {
        Self { kernel, arg_idx: 0 }
    }

    pub fn add_mem<T>(mut self, arg: &Buffer<T>) -> Result<Self, Error> {
        let index = self.arg_idx;
        self.arg_idx += 1;
        let mem:cl_mem = arg.mem();

        let status = unsafe { clSetKernelArg(self.kernel.cl_kernel(), index, std::mem::size_of::<cl_mem>(), (&mem) as *const cl_mem as *const c_void)};
        Error::result(move ||self,status, ||format!("Failed setting memory buffer as argument {}",index))
    }
    pub fn add_value<T>(mut self, arg: T) -> Result<Self, Error> {
        let index = self.arg_idx;
        self.arg_idx += 1;
        let status = unsafe { clSetKernelArg(self.kernel.cl_kernel(), index, std::mem::size_of::<T>(), &arg as *const T as *const c_void)};
        Error::result(move ||self,status, ||format!("Failed setting {} as argument {}",std::any::type_name::<T>(),index))
    }
    pub fn done(self) -> Kernel {
        self.kernel
    }
}
