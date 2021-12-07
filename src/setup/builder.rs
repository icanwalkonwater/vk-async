use crate::{
    errors::{Result, VulkanError},
    setup::{DebugUtils, DeviceQueues, PhysicalDeviceInfo, VulkanInitializer},
    VulkanApp,
};
use ash::vk;
use std::{ffi::CStr, mem::ManuallyDrop, os::raw::c_char};

type DeviceAdapter = (vk::PhysicalDevice, DeviceQueues);

pub struct VulkanBuilder {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) debug_utils: ManuallyDrop<DebugUtils>,
    physical_device: Option<DeviceAdapter>,
    device_extensions: Vec<*const c_char>,
}

impl VulkanBuilder {
    pub fn builder() -> VulkanInitializer {
        VulkanInitializer::default()
    }

    pub(crate) fn new(entry: ash::Entry, instance: ash::Instance, debug_utils: DebugUtils) -> Self {
        Self {
            entry,
            instance,
            debug_utils: ManuallyDrop::new(debug_utils),
            physical_device: None,
            device_extensions: Vec::new(),
        }
    }

    pub fn set_physical_device(mut self, device: PhysicalDeviceInfo) -> Self {
        let queues = DeviceQueues::from_device(&device).unwrap();
        self.physical_device = Some((device.handle, queues));
        self
    }

    pub fn with_device_extension(mut self, name: &'static CStr) -> Self {
        self.device_extensions.push(name.as_ptr());
        self
    }
}

impl VulkanBuilder {
    pub fn build(self) -> Result<VulkanApp> {
        let device = {
            let physical = self
                .physical_device
                .as_ref()
                .ok_or(VulkanError::NoPhysicalDevicePicked)?;
            let queue_create_info = physical.1.as_queue_create_info();

            unsafe {
                self.instance.create_device(
                    physical.0,
                    &vk::DeviceCreateInfo::builder()
                        .enabled_extension_names(&self.device_extensions)
                        .queue_create_infos(&queue_create_info),
                    None,
                )?
            }
        };

        let app = VulkanApp {
            _entry: self.entry,
            instance: self.instance,
            debug_utils: self.debug_utils,
            device,
        };

        Ok(app)
    }
}