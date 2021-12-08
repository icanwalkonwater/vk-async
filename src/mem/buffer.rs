use crate::{errors::Result, mem, mem::RawAllocation};
use ash::vk;
use std::{marker::PhantomData, sync::Arc};

/// Long-lived CPU buffer to store dynamic data later read from the GPU.
/// Refer to [`vk_mem::MemoryUsage::CpuToGpu`]
pub struct CpuToGpuBufferHandle<D> {
    handle: vk::Buffer,
    raw: RawAllocation,
    _marker: PhantomData<D>,
}

impl<D> Drop for CpuToGpuBufferHandle<D> {
    fn drop(&mut self) {
        self.raw
            .vma
            .destroy_buffer(self.handle, &self.raw.allocation);
    }
}

impl<D: Sized + Copy> CpuToGpuBufferHandle<D> {
    pub(crate) fn new(
        vma: Arc<vk_mem::Allocator>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        let (handle, raw) = mem::create_buffer(vma, size, usage, vk_mem::MemoryUsage::CpuToGpu)?;

        Ok(Self {
            handle,
            raw,
            _marker: Default::default(),
        })
    }

    pub fn write_to(&mut self, data: &[D]) -> Result<()> {
        self.raw.write_to(data)
    }

    pub fn read(&self, out: &mut [D], offset: usize) -> Result<()> {
        self.raw.read(out, offset)
    }
}
