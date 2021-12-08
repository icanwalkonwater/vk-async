use crate::{errors::Result, VulkanApp};
use ash::vk;
use std::sync::Arc;

mod buffer;
pub use buffer::*;

pub(crate) fn vma_ensure_mapped(
    vma: &vk_mem::Allocator,
    allocation: &vk_mem::Allocation,
    info: &vk_mem::AllocationInfo,
) -> Result<(bool, *mut u8)> {
    if info.get_mapped_data().is_null() {
        Ok((true, vma.map_memory(allocation)?))
    } else {
        Ok((false, info.get_mapped_data()))
    }
}

impl VulkanApp {
    /// Create a new CPU side buffer and immediately write some data in it.
    pub fn new_cpu_buffer<D: Sized + Copy>(
        &self,
        data: &[D],
        usage: vk::BufferUsageFlags,
    ) -> Result<CpuToGpuBufferHandle<D>> {
        let mut buffer = CpuToGpuBufferHandle::new(
            Arc::clone(&self.vma),
            (std::mem::size_of::<D>() * data.len()) as _,
            usage,
        )?;
        buffer.write_to(data)?;
        Ok(buffer)
    }
}
