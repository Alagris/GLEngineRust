use cl_sys::*;
use crate::compute_cl::error::Error;
use crate::compute_cl::context::Context;

pub struct Kernel{
    kernel:cl_kernel
}

impl Kernel{
    pub fn new(kernel:cl_kernel)->Self{
        Self{kernel}
    }
    pub fn cl_kernel(&self)->cl_kernel{
        self.kernel
    }

    pub fn enq(&self, context:&Context, global_work_dimensions:&[usize]) -> Result<(), Error> {
        if global_work_dimensions.len()>3{
            return Err(Error::new(CL_INVALID_WORK_DIMENSION,format!("OpenCL has at most 3 work dimensions but provided shape is {:?}",global_work_dimensions)));
        }
        let status = unsafe {
            clEnqueueNDRangeKernel(context.queue(), self.kernel, global_work_dimensions.len() as u32, std::ptr::null(), global_work_dimensions.as_ptr(), std::ptr::null(), 0, std::ptr::null(),std::ptr::null_mut())
        };
        Error::result(||(),status,||String::from("Failed to enqueue kernel"))
    }
}
impl Clone for Kernel{
    fn clone(&self) -> Self {
        let status = unsafe{ clRetainKernel(self.kernel) };
        Error::result(||Self{kernel:self.kernel},status,||String::from("Failed to retain OpenCL kernel")).unwrap()
    }
}
impl Drop for Kernel{
    fn drop(&mut self) {
        let status = unsafe{ clReleaseKernel(self.kernel) };
        Error::result(||(),status,||String::from("Failed to release OpenCL kernel")).unwrap()
    }
}