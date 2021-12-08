use crate::setup::DebugUtils;
use std::{mem::ManuallyDrop, sync::Arc};

pub mod mem;
pub mod setup;

pub mod errors {
    use thiserror::Error;

    pub type Result<T> = std::result::Result<T, VulkanError>;

    #[derive(Error, Debug)]
    pub enum VulkanError {
        #[error("{0}")]
        LoadingError(#[from] ash::LoadingError),
        #[error("{0}")]
        InstanceError(#[from] ash::InstanceError),
        #[error("{0}")]
        VkError(#[from] ash::vk::Result),
        #[error("{0}")]
        VmaError(#[from] vk_mem::Error),
        #[error("No compute queue found")]
        NoComputeQueue,
        #[error("No transfer queue found")]
        NoTransferQueue,
        #[error("No physical device picked")]
        NoPhysicalDevicePicked,
    }
}

pub struct VulkanApp {
    pub(crate) _entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) debug_utils: ManuallyDrop<DebugUtils>,
    pub(crate) device: ash::Device,
    pub(crate) vma: Arc<vk_mem::Allocator>,
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            // TODO: add destroys here

            Arc::get_mut(&mut self.vma).expect("There still are buffers around referencing VMA !").destroy();

            self.device.destroy_device(None);
            ManuallyDrop::drop(&mut self.debug_utils);
            self.instance.destroy_instance(None);
        }
    }
}
