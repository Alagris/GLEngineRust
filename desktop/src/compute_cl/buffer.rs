//! Interfaces with a buffer.

use std;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Range};
use std::any::TypeId;
use crate::render_gl::buffer::{BufferType, BufferUsage};
use cl_sys::*;
use crate::compute_cl::error::Error;


/// A chunk of memory physically located on a device, such as a GPU.
///
/// Data is stored remotely in a memory buffer on the device associated with
/// `queue`.
///
#[derive(Debug)]
pub struct Buffer<T> {
    mem: cl_mem,
    len: usize,
    offset: Option<usize>,
    _data: PhantomData<T>,
}

impl<T> Buffer<T> {
    pub fn new(mem: cl_mem, len: usize, offset: Option<usize>) -> Self {
        Self {
            mem,
            len,
            offset,
            _data: PhantomData,
        }
    }
    pub fn mem(&self) -> cl_mem {
        self.mem
    }
    // pub fn fill(&mut self, queue:&CommandQueue, fill_val:T) -> Result<(), Error> {
    //     unsafe{
    //         ocl::core::enqueue_fill_buffer(queue, &self.obj_core, fill_val,0, self.len(), None::<core::Event>, None::<&mut core::Event>, None)
    //     }
    // }
    //
    // pub fn read(&self, queue:&CommandQueue, offset:usize, dst:&mut[T]) -> Result<(),Error>{
    //     if offset+dst.len() > self.len(){
    //         return Err(Error::from(format!("Buffer has length {} is less that destination length {} plus offset {}",self.len(),dst.len(),offset)));
    //     }
    //     unsafe {
    //         ocl::core::enqueue_read_buffer(&queue, &self.obj_core, true, offset, dst, None::<core::Event>, None::<&mut core::Event>)
    //     }
    // }
    //
    // pub fn to_vec(&self, queue:&CommandQueue) -> Result<Vec<T>,Error>{
    //     let mut vec = Vec::with_capacity(self.len());
    //     unsafe{vec.set_len(vec.capacity())};
    //     self.read(queue,0,vec.as_mut_slice())?;
    //     Ok(vec)
    // }

    /// Returns the offset of the sub-buffer within its buffer if this is a
    /// sub-buffer.
    #[inline]
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    /// Returns the length of the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    // /// Returns true if this is a sub-buffer.
    // #[inline]
    // pub fn is_sub_buffer(&self) -> bool {
    //     debug_assert!({
    //         let is_sub_buffer = match self.mem_info(MemInfo::AssociatedMemobject).unwrap() {
    //             MemInfoResult::AssociatedMemobject(Some(_)) => true,
    //             MemInfoResult::AssociatedMemobject(None) => panic!("Buffer::is_sub_buffer"),
    //             _ => unreachable!(),
    //         };
    //         self.offset.is_some() == is_sub_buffer
    //     });
    //     self.offset.is_some()
    // }
    //
    // /// Returns info about the underlying memory object.
    // #[inline]
    // pub fn mem_info(&self, info_kind: MemInfo) -> OclCoreResult<MemInfoResult> {
    //     core::get_mem_object_info(&self.obj_core, info_kind)
    // }

    // /// Returns the memory flags used during the creation of this buffer.
    // ///
    // #[inline]
    // pub fn flags(&self) -> OclResult<MemFlags> {
    //     match self.mem_info(MemInfo::Flags)? {
    //         MemInfoResult::Flags(flags) => Ok(flags),
    //         _ => unreachable!(),
    //     }
    // }

    // /// Creates a new sub-buffer from a region of this buffer.
    // ///
    // /// ### Flags (adapted from [SDK])
    // ///
    // /// [NOTE]: Flags described below can be found in the [`oclocl::flags`] module
    // /// or within the [`MemFlags`][mem_flags] type (example:
    // /// [`MemFlags::new().read_write()`]).
    // ///
    // /// `flags`: A bit-field that is used to specify allocation and usage
    // /// information about the sub-buffer memory object being created and is
    // /// described in the table below. If the `MEM_READ_WRITE`, `MEM_READ_ONLY`
    // /// or `MEM_WRITE_ONLY` values are not specified in flags, they are
    // /// inherited from the corresponding memory access qualifers associated
    // /// with buffer. The `MEM_USE_HOST_PTR`, `MEM_ALLOC_HOST_PTR` and
    // /// `MEM_COPY_HOST_PTR` values cannot be specified in flags but are
    // /// inherited from the corresponding memory access qualifiers associated
    // /// with buffer. If `MEM_COPY_HOST_PTR` is specified in the memory access
    // /// qualifier values associated with buffer it does not imply any
    // /// additional copies when the sub-buffer is created from buffer. If the
    // /// `MEM_HOST_WRITE_ONLY`, `MEM_HOST_READ_ONLY` or `MEM_HOST_NO_ACCESS`
    // /// values are not specified in flags, they are inherited from the
    // /// corresponding memory access qualifiers associated with buffer.
    // ///
    // /// ### Offset and Dimensions
    // ///
    // /// `offset` and `len` set up the region of the sub-buffer within the
    // ///  original buffer and must not fall beyond the boundaries of it.
    // ///
    // /// `offset` must be a multiple of the `DeviceInfo::MemBaseAddrAlign`
    // /// otherwise you will get a `CL_MISALIGNED_SUB_BUFFER_OFFSET` error. To
    // /// determine, use `Device::mem_base_addr_align` for the device associated
    // /// with the queue which will be use with this sub-buffer.
    // ///
    // /// [SDK]: https://www.khronos.org/registry/cl/sdk/1.2/docs/man/xhtml/clCreateSubBuffer.html
    // /// [`oclocl::flags`]: flags/index.html
    // /// [mem_flags]: flags/struct.MemFlags.html
    // /// [`MemFlags::new().read_write()`] flags/struct.MemFlags.html#method.read_write
    // ///
    // pub fn create_sub_buffer(&self, flags_opt: Option<MemFlags>, offset: usize, len: usize) -> OclResult<Buffer<T>> {
    //     let flags = flags_opt.unwrap_or(ocl::flags::MEM_READ_WRITE);
    //
    //     // Check flags here to preempt a somewhat vague OpenCL runtime error message:
    //     assert!(!flags.contains(ocl::flags::MEM_USE_HOST_PTR) &&
    //                 !flags.contains(ocl::flags::MEM_ALLOC_HOST_PTR) &&
    //                 !flags.contains(ocl::flags::MEM_COPY_HOST_PTR),
    //             "'MEM_USE_HOST_PTR', 'MEM_ALLOC_HOST_PTR', or 'MEM_COPY_HOST_PTR' flags may \
    //         not be specified when creating a sub-buffer. They will be inherited from \
    //         the containing buffer.");
    //
    //     let buffer_len = self.len();
    //
    //     if offset > buffer_len {
    //         return Err(format!("Buffer::create_sub_buffer: Origin ({:?}) is outside of the \
    //             dimensions of the source buffer ({:?}).", offset, buffer_len).into());
    //     }
    //
    //     if offset + len > buffer_len {
    //         return Err(format!("Buffer::create_sub_buffer: Sub-buffer region (origin: '{:?}', \
    //             len: '{:?}') exceeds the dimensions of the source buffer ({:?}).",
    //                            offset, len, buffer_len).into());
    //     }
    //     let (offset,obj_core) = if let Some(parent_offset) =  self.offset(){
    //         let offset = parent_offset+offset;
    //         let reg = BufferRegion::<T>::new(offset, len);
    //         (offset,match self.mem_info(MemInfo::AssociatedMemobject).unwrap() {
    //             MemInfoResult::AssociatedMemobject(Some(parent)) => core::create_sub_buffer::<T>(&parent, flags,&reg)?,
    //             _ => unreachable!(),
    //         })
    //     }else{
    //         let reg = BufferRegion::new(offset, len);
    //         (offset,core::create_sub_buffer::<T>(self, flags,&reg)?)
    //     };
    //
    //
    //     Ok(Buffer {
    //         obj_core,
    //         len,
    //         // Share mapped status with super-buffer:
    //         // is_mapped: self.is_mapped.clone(),
    //         offset: Some(offset),
    //         _data: PhantomData,
    //     })
    // }
    //
    //
    // /// Formats memory info.
    // #[inline]
    // fn fmt_mem_info(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    //     f.debug_struct("Buffer Mem")
    //         .field("Type", &self.mem_info(MemInfo::Type))
    //         .field("Flags", &self.mem_info(MemInfo::Flags))
    //         .field("Size", &self.mem_info(MemInfo::Size))
    //         .field("HostPtr", &self.mem_info(MemInfo::HostPtr))
    //         .field("MapCount", &self.mem_info(MemInfo::MapCount))
    //         .field("ReferenceCount", &self.mem_info(MemInfo::ReferenceCount))
    //         .field("Context", &self.mem_info(MemInfo::Context))
    //         .field("AssociatedMemobject", &self.mem_info(MemInfo::AssociatedMemobject))
    //         .field("Offset", &self.mem_info(MemInfo::Offset))
    //         .finish()
    // }
}

impl<T> Clone for Buffer<T> {
    fn clone(&self) -> Self {
        let mut status = unsafe { clRetainMemObject(self.mem) };
        Error::result(||Self::new(self.mem, self.len, self.offset),status,||String::from("Failed to retain OpenCL buffer")).unwrap()
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        let mut status = unsafe { clReleaseMemObject(self.mem) };
        if status != CL_SUCCESS {
            eprintln!("Attempted to drop invalid OpenCL buffer! ({})", status);
        }
    }
}


