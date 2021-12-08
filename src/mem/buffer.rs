use crate::{errors::Result, mem::vma_ensure_mapped};
use ash::vk;
use std::{
    ops::{Range, RangeInclusive},
    sync::Arc,
};
use std::marker::PhantomData;

pub(crate) struct RawAllocation {
    allocation: vk_mem::Allocation,
    info: vk_mem::AllocationInfo,
    #[cfg(debug_assertions)]
    size: vk::DeviceSize,
    vma: Arc<vk_mem::Allocator>,
}

impl RawAllocation {
    pub fn write_to<D: Sized + Copy>(&mut self, data: &[D]) -> Result<()> {
        debug_assert!((std::mem::size_of::<D>() * data.len()) as vk::DeviceSize <= self.size);

        let (need_to_unmap, mapped_ptr) =
            vma_ensure_mapped(&self.vma, &self.allocation, &self.info)?;

        let size = (std::mem::size_of::<D>() * data.len()) as _;
        let mut mapped_slice = unsafe {
            ash::util::Align::new(
                mapped_ptr as *mut std::ffi::c_void,
                std::mem::align_of::<D>() as _,
                size,
            )
        };

        mapped_slice.copy_from_slice(data);
        self.vma.flush_allocation(&self.allocation, 0, size as _);

        if need_to_unmap {
            self.vma.unmap_memory(&self.allocation);
        }

        Ok(())
    }

    pub fn read<D: Sized + Copy>(&self, out: &mut [D], offset: usize) -> Result<()> {
        debug_assert!(((std::mem::size_of::<D>() * offset) as vk::DeviceSize) <= self.size);
        debug_assert!(((std::mem::size_of::<D>() * (offset + out.len())) as vk::DeviceSize) <= self.size);

        let (need_to_unmap, mapped_ptr) =
            vma_ensure_mapped(&self.vma, &self.allocation, &self.info)?;

        let mapped_ptr = unsafe { mapped_ptr.offset((offset * std::mem::size_of::<D>()) as _) };
        let mapped_data = unsafe { std::slice::from_raw_parts(mapped_ptr as *const D, out.len()) };

        out.copy_from_slice(mapped_data);

        if need_to_unmap {
            self.vma.unmap_memory(&self.allocation);
        }

        Ok(())
    }
}

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
        let (handle, allocation, info) = vma.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(size)
                .usage(usage)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::CpuToGpu,
                ..Default::default()
            },
        )?;

        Ok(Self {
            handle,
            raw: RawAllocation {
                allocation,
                info,
                size,
                vma,
            },
            _marker: Default::default()
        })
    }

    pub fn write_to(&mut self, data: &[D]) -> Result<()> {
        self.raw.write_to(data)
    }

    pub fn read(&self, out: &mut [D], offset: usize) -> Result<()> {
        self.raw.read(out, offset)
    }
}
