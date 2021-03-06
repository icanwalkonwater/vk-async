use crate::{
    errors::Result,
    setup::{DebugUtils, VulkanBuilder},
};
use ash::vk;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

pub(crate) const VULKAN_VERSION: u32 = vk::make_api_version(0, 1, 2, 0);

pub struct VulkanInitializer {
    name: Option<CString>,
    ext_instance: Vec<*const c_char>,
}

impl Default for VulkanInitializer {
    fn default() -> Self {
        Self {
            name: None,
            ext_instance: vec![ash::extensions::ext::DebugUtils::name().as_ptr()],
        }
    }
}

impl VulkanInitializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(CString::new(name).unwrap());
        self
    }

    pub fn with_instance_extension(mut self, name: &'static CStr) -> Self {
        self.ext_instance.push(name.as_ptr());
        self
    }
}

impl VulkanInitializer {
    pub fn build(self) -> Result<VulkanBuilder> {
        let entry = unsafe { ash::Entry::new()? };

        let instance = {
            let app_name = CString::new(
                self.name
                    .unwrap_or_else(|| CString::new("App Name").unwrap()),
            )
            .unwrap();
            let engine_name = CString::new(env!("CARGO_PKG_NAME")).unwrap();

            let vk_app_info = vk::ApplicationInfo::builder()
                .application_name(app_name.as_c_str())
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(engine_name.as_c_str())
                .engine_version({
                    let major = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
                    let minor = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
                    let patch = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();
                    vk::make_api_version(0, major, minor, patch)
                })
                .api_version(VULKAN_VERSION);

            unsafe {
                entry.create_instance(
                    &vk::InstanceCreateInfo::builder()
                        .enabled_extension_names(&self.ext_instance)
                        .application_info(&vk_app_info),
                    None,
                )?
            }
        };

        let debug_utils = DebugUtils::new(&entry, &instance)?;

        Ok(VulkanBuilder::new(entry, instance, debug_utils))
    }
}
