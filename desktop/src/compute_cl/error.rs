use std::fmt::{Display, Formatter, Debug};
use std::num::NonZeroI32;



pub const ERR_CODES_1_30:[&'static str;19] = ["CL_DEVICE_NOT_FOUND",
"CL_DEVICE_NOT_AVAILABLE",
"CL_COMPILER_NOT_AVAILABLE",
"CL_MEM_OBJECT_ALLOCATION_FAILURE",
"CL_OUT_OF_RESOURCES",
"CL_OUT_OF_HOST_MEMORY",
"CL_PROFILING_INFO_NOT_AVAILABLE",
"CL_MEM_COPY_OVERLAP",
"CL_IMAGE_FORMAT_MISMATCH",
"CL_IMAGE_FORMAT_NOT_SUPPORTED",
"CL_BUILD_PROGRAM_FAILURE",
"CL_MAP_FAILURE",
"CL_MISALIGNED_SUB_BUFFER_OFFSET",
"CL_EXEC_STATUS_ERROR_FOR_EVENTS_IN_WAIT_LIST",
"CL_COMPILE_PROGRAM_FAILURE",
"CL_LINKER_NOT_AVAILABLE",
"CL_LINK_PROGRAM_FAILURE",
"CL_DEVICE_PARTITION_FAILED",
"CL_KERNEL_ARG_INFO_NOT_AVAILABLE",];

pub const ERR_CODES_30_1000:[&'static str;70-30+1] = ["CL_INVALID_VALUE",
"CL_INVALID_DEVICE_TYPE",
"CL_INVALID_PLATFORM",
"CL_INVALID_DEVICE",
"CL_INVALID_CONTEXT",
"CL_INVALID_QUEUE_PROPERTIES",
"CL_INVALID_COMMAND_QUEUE",
"CL_INVALID_HOST_PTR",
"CL_INVALID_MEM_OBJECT",
"CL_INVALID_IMAGE_FORMAT_DESCRIPTOR",
"CL_INVALID_IMAGE_SIZE",
"CL_INVALID_SAMPLER",
"CL_INVALID_BINARY",
"CL_INVALID_BUILD_OPTIONS",
"CL_INVALID_PROGRAM",
"CL_INVALID_PROGRAM_EXECUTABLE",
"CL_INVALID_KERNEL_NAME",
"CL_INVALID_KERNEL_DEFINITION",
"CL_INVALID_KERNEL",
"CL_INVALID_ARG_INDEX",
"CL_INVALID_ARG_VALUE",
"CL_INVALID_ARG_SIZE",
"CL_INVALID_KERNEL_ARGS",
"CL_INVALID_WORK_DIMENSION",
"CL_INVALID_WORK_GROUP_SIZE",
"CL_INVALID_WORK_ITEM_SIZE",
"CL_INVALID_GLOBAL_OFFSET",
"CL_INVALID_EVENT_WAIT_LIST",
"CL_INVALID_EVENT",
"CL_INVALID_OPERATION",
"CL_INVALID_GL_OBJECT",
"CL_INVALID_BUFFER_SIZE",
"CL_INVALID_MIP_LEVEL",
"CL_INVALID_GLOBAL_WORK_SIZE",
"CL_INVALID_PROPERTY",
"CL_INVALID_IMAGE_DESCRIPTOR",
"CL_INVALID_COMPILER_OPTIONS",
"CL_INVALID_LINKER_OPTIONS",
"CL_INVALID_DEVICE_PARTITION_COUNT",
"CL_INVALID_PIPE_SIZE",
"CL_INVALID_DEVICE_QUEUE"];

pub const ERR_CODES_1001:&'static str = "CL_PLATFORM_NOT_FOUND_KHR";

#[derive(Debug, Fail)]
pub struct ClGlError{
    err:NonZeroI32
}
impl From<i32> for ClGlError{
    fn from(err: i32) -> Self {
        Self{err:NonZeroI32::new(err).unwrap()}
    }
}
impl ClGlError{
    pub fn result<T>(value:T, err:i32)->Result<T,ClGlError>{
        assert_eq!(cl_sys::CL_SUCCESS,0);
        match NonZeroI32::new(err){
            None => Ok(value),
            Some(err) => Err(Self{err})
        }
    }
    pub fn name(&self)->Option<&'static str>{
        let err = self.err.get();
        if err>-30{
            ERR_CODES_1_30.get(-(err-1) as usize).cloned()
        }else if err>-1000{
            ERR_CODES_30_1000.get(-(err-30) as usize).cloned()
        }else if err==-1001{
            Some(ERR_CODES_1001)
        }else{
            None
        }
    }
}
impl Display for ClGlError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.name() {
            Some(s) => f.write_str(s),
            None => write!(f,"UNKNOWN CODE({})",self.err),
        }
    }
}