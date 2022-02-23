use ash::{prelude::*, vk};
use cstr::cstr;
use std::{ffi, ops, os::raw};

pub struct Instance<'a> {
    instance: ash::Instance,
    entry: &'a ash::Entry,
}

impl<'a> Instance<'a> {
    /// Initializes a new Vulkan instance
    ///
    /// # Safety
    ///
    /// `validation_layers_names` and `extensions_names` must contain pointers
    /// to null-terminated strings, they should be considered as `[&CStr]`
    pub unsafe fn new(
        entry: &'a ash::Entry,
        validation_layers_names: &[*const raw::c_char],
        extensions_names: &[*const raw::c_char],
        app_name: &ffi::CStr,
    ) -> VkResult<Self> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(cstr!("No Engine"))
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_0)
            .build();

        #[cfg(debug_assertions)]
        let mut dbg_utils_info = super::DebugUtils::create_info();

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(extensions_names)
            .enabled_layer_names(validation_layers_names);

        #[cfg(debug_assertions)]
        let instance_info = instance_info.push_next(&mut dbg_utils_info);

        let instance = entry.create_instance(&instance_info.build(), None)?;

        Ok(Self { instance, entry })
    }

    pub fn entry(&self) -> &ash::Entry {
        self.entry
    }
}

impl ops::Deref for Instance<'_> {
    type Target = ash::Instance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Drop for Instance<'_> {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
