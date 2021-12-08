use crate::{errors::Result, VulkanApp};
use ash::vk;
use std::sync::Arc;

mod alloc;
pub use alloc::*;

mod buffer;
pub use buffer::*;

mod gpu_buffer;
pub use gpu_buffer::*;

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

pub(crate) fn create_buffer(
    vma: Arc<vk_mem::Allocator>,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    location: vk_mem::MemoryUsage,
) -> Result<(vk::Buffer, RawAllocation)> {
    let (handle, allocation, info) = vma.create_buffer(
        &vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE),
        &vk_mem::AllocationCreateInfo {
            usage: location,
            ..Default::default()
        },
    )?;

    Ok((
        handle,
        RawAllocation {
            allocation,
            info,
            size,
            vma,
        },
    ))
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

    pub async fn upload_to_gpu_buffer<D: Sized + Copy>(
        &self,
        data: &[D],
        usage: vk::BufferUsageFlags,
    ) -> Result<GpuBufferHandle<D>> {
        let mut buffer = GpuBufferHandle::new(
            Arc::clone(&self.vma),
            (std::mem::size_of::<D>() * data.len()) as _,
            usage,
        )?;
        buffer.write_to(self, data).await?;
        Ok(buffer)
    }
}
