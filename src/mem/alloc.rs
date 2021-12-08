use crate::{errors::Result, mem::vma_ensure_mapped};
use ash::vk;
use std::sync::Arc;

pub(crate) struct RawAllocation {
    pub(crate) allocation: vk_mem::Allocation,
    pub(crate) info: vk_mem::AllocationInfo,
    pub(crate) size: vk::DeviceSize,
    pub(crate) vma: Arc<vk_mem::Allocator>,
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
        debug_assert!(
            ((std::mem::size_of::<D>() * (offset + out.len())) as vk::DeviceSize) <= self.size
        );

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
