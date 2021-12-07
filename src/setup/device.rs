use crate::{
    errors::{Result, VulkanError},
    setup::{VulkanBuilder, VULKAN_VERSION},
};
use ash::{vk, vk::QueueFamilyProperties2};
use log::warn;
use std::ffi::CStr;

#[derive(Debug)]
pub struct PhysicalDeviceInfo {
    pub(crate) handle: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties2,
    pub extensions: Vec<vk::ExtensionProperties>,
    pub features: vk::PhysicalDeviceFeatures2,
    pub queue_families: Vec<QueueFamilyProperties2>,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties2,
}

impl PhysicalDeviceInfo {
    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_ptr(self.properties.properties.device_name.as_ptr())
                .to_str()
                .unwrap()
        }
    }

    pub fn is_discrete(&self) -> bool {
        self.properties.properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
    }

    pub fn vram_size(&self) -> vk::DeviceSize {
        self.memory_properties
            .memory_properties
            .memory_heaps
            .iter()
            .filter(|heap| heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|heap| heap.size)
            .sum()
    }
}

impl VulkanBuilder {
    pub fn list_available_physical_devices(&mut self) -> Result<Vec<PhysicalDeviceInfo>> {
        let devices = unsafe { self.instance.enumerate_physical_devices()? };

        let devices = devices
            .into_iter()
            .map(|d| unsafe {
                let mut properties = vk::PhysicalDeviceProperties2::default();
                self.instance
                    .get_physical_device_properties2(d, &mut properties);

                let extensions = self
                    .instance
                    .enumerate_device_extension_properties(d)
                    .expect("Failed to enumerate device extensions");

                let mut features = vk::PhysicalDeviceFeatures2::default();
                self.instance
                    .get_physical_device_features2(d, &mut features);

                let mut queue_families = Vec::new();
                queue_families.resize_with(
                    self.instance
                        .get_physical_device_queue_family_properties2_len(d),
                    || vk::QueueFamilyProperties2::default(),
                );
                self.instance
                    .get_physical_device_queue_family_properties2(d, &mut queue_families);

                let mut memory_properties = vk::PhysicalDeviceMemoryProperties2::default();
                self.instance
                    .get_physical_device_memory_properties2(d, &mut memory_properties);

                PhysicalDeviceInfo {
                    handle: d,
                    properties,
                    extensions,
                    features,
                    queue_families,
                    memory_properties,
                }
            })
            .filter(|info| {
                if !self.is_device_suitable(info) {
                    let name =
                        unsafe { CStr::from_ptr(info.properties.properties.device_name.as_ptr()) }
                            .to_str()
                            .unwrap();
                    warn!("Found unsuitable device: {}", name);
                    false
                } else {
                    true
                }
            })
            .collect();

        Ok(devices)
    }

    fn is_device_suitable(&self, info: &PhysicalDeviceInfo) -> bool {
        // Check Vulkan version
        if info.properties.properties.api_version < VULKAN_VERSION {
            return false;
        }

        // Check required extensions
        // TODO: no extensions yet

        // Check queues
        if let Err(_) = DeviceQueues::from_device(info) {
            return false;
        }

        true
    }
}

/// Represents the queue indices to use for graphics, compute and transfer.
///
/// When created, it tries to find a queue family that isn't shared with graphics
/// but will fallback on whatever is available.
///
/// So these indices can overlap.
pub(crate) struct DeviceQueues {
    pub(crate) graphics: u32,
    pub(crate) compute: u32,
    pub(crate) transfer: u32,
}

const QUEUE_PRIORITIES_ONE: [f32; 1] = [1.0];

impl DeviceQueues {
    pub(crate) fn from_device(info: &PhysicalDeviceInfo) -> Result<Self> {
        Ok(Self {
            graphics: Self::find_graphics_queue(info),
            compute: Self::find_compute_queue(info)?,
            transfer: Self::find_transfer_queue(info)?,
        })
    }

    pub(crate) fn as_queue_create_info(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut create_info = vec![
            // Graphics
            vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(self.graphics)
                .queue_priorities(&QUEUE_PRIORITIES_ONE)
                .build(),
        ];

        // Compute
        if self.compute != self.graphics {
            create_info.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(self.compute)
                    .queue_priorities(&QUEUE_PRIORITIES_ONE)
                    .build(),
            );
        }

        // Transfer
        if self.transfer != self.compute && self.transfer != self.graphics {
            create_info.push(
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(self.transfer)
                    .queue_priorities(&QUEUE_PRIORITIES_ONE)
                    .build(),
            );
        }

        create_info
    }

    fn find_graphics_queue(info: &PhysicalDeviceInfo) -> u32 {
        info.queue_families
            .iter()
            .enumerate()
            .find(|(_, queue)| {
                queue
                    .queue_family_properties
                    .queue_flags
                    .contains(vk::QueueFlags::GRAPHICS)
            })
            .map(|(i, _)| i)
            .unwrap() as _
    }

    fn find_compute_queue(info: &PhysicalDeviceInfo) -> Result<u32> {
        Self::try_find_queue_not_shared_with(
            info,
            vk::QueueFlags::COMPUTE,
            vk::QueueFlags::GRAPHICS,
        )
        .ok_or(VulkanError::NoComputeQueue)
    }

    fn find_transfer_queue(info: &PhysicalDeviceInfo) -> Result<u32> {
        Self::try_find_queue_not_shared_with(
            info,
            vk::QueueFlags::TRANSFER,
            vk::QueueFlags::GRAPHICS,
        )
        .ok_or(VulkanError::NoTransferQueue)
    }

    /// Helper
    fn try_find_queue_not_shared_with(
        info: &PhysicalDeviceInfo,
        to_find: vk::QueueFlags,
        to_avoid: vk::QueueFlags,
    ) -> Option<u32> {
        info.queue_families
            .iter()
            .enumerate()
            .filter(|(_, queue)| queue.queue_family_properties.queue_flags.contains(to_find))
            .fold(None, |acc, (i, queue)| {
                // Try to get one that isn't also used for graphics
                if let Some((prev_i, prev_queue)) = acc {
                    if !queue.queue_family_properties.queue_flags.contains(to_avoid) {
                        Some((i, queue))
                    } else {
                        Some((prev_i, prev_queue))
                    }
                } else {
                    Some((i, queue))
                }
            })
            .map(|(i, _)| i as _)
    }
}
