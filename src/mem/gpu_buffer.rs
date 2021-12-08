use ash::vk;
use std::{marker::PhantomData, sync::Arc};

use crate::{
    errors::Result,
    mem::{create_buffer, RawAllocation},
};

use crate::VulkanApp;

pub struct GpuBufferHandle<D> {
    handle: vk::Buffer,
    raw: RawAllocation,
    _marker: PhantomData<D>,
}

impl<D> Drop for GpuBufferHandle<D> {
    fn drop(&mut self) {
        self.raw
            .vma
            .destroy_buffer(self.handle, &self.raw.allocation);
    }
}

impl<D: Sized + Copy> GpuBufferHandle<D> {
    pub(crate) fn new(
        vma: Arc<vk_mem::Allocator>,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
    ) -> Result<Self> {
        let (handle, raw) = create_buffer(
            vma,
            size,
            // To read and write with staging buffers
            usage | vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
            vk_mem::MemoryUsage::GpuOnly,
        )?;

        Ok(Self {
            handle,
            raw,
            _marker: Default::default(),
        })
    }

    pub async fn write_to(&mut self, app: &VulkanApp, data: &[D]) -> Result<()> {
        let (staging_handle, mut staging_raw) = create_buffer(
            Arc::clone(&self.raw.vma),
            self.raw.size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuOnly,
        )?;

        staging_raw.write_to(data)?;
        unsafe {
            app.create_copy_buffer_cmd((staging_handle, &staging_raw), (self.handle, &self.raw))
        }?
        .await?;

        // Destroy staging
        self.raw
            .vma
            .destroy_buffer(staging_handle, &staging_raw.allocation);
        Ok(())
    }

    pub async fn read(&self, app: &VulkanApp, out: &mut [D], offset: usize) -> Result<()> {
        let (staging_handle, staging_raw) = create_buffer(
            Arc::clone(&self.raw.vma),
            self.raw.size,
            vk::BufferUsageFlags::TRANSFER_DST,
            vk_mem::MemoryUsage::CpuOnly,
        )?;

        unsafe {
            app.create_copy_buffer_cmd((self.handle, &self.raw), (staging_handle, &staging_raw))
        }?
        .await?;

        staging_raw.read(out, offset)?;

        // Destroy staging
        self.raw
            .vma
            .destroy_buffer(staging_handle, &staging_raw.allocation);
        Ok(())
    }
}
