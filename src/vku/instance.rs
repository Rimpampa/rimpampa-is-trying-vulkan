use ash::vk;
use cstr::cstr;
use std::{ffi, os::raw};

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
    ) -> super::Result<Self> {
        let app_info = vk::ApplicationInfo::builder()
            .application_name(app_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(cstr!("No Engine"))
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_0)
            .build();

        #[cfg(debug_assertions)]
        let mut dbg_utils_info = super::DebugUtils::<Self>::create_info();

        let instance_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(extensions_names)
            .enabled_layer_names(validation_layers_names);

        #[cfg(debug_assertions)]
        let instance_info = instance_info.push_next(&mut dbg_utils_info);

        let instance = entry.create_instance(&instance_info.build(), None)?;

        Ok(Self { instance, entry })
    }
}

impl Drop for Instance<'_> {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

pub trait InstanceHolder {
    fn vk_instance(&self) -> &ash::Instance;
    fn vk_entry(&self) -> &ash::Entry;
}

impl InstanceHolder for Instance<'_> {
    fn vk_instance(&self) -> &ash::Instance {
        &self.instance
    }

    fn vk_entry(&self) -> &ash::Entry {
        self.entry
    }
}

macro_rules! derive_instance_holder {
    ($self:ty = $field:ident : $generic:ident) => {
        impl<$generic: $crate::vku::InstanceHolder> $crate::vku::InstanceHolder for $self {
            fn vk_instance(&self) -> &ash::Instance {
                self.$field.vk_instance()
            }

            fn vk_entry(&self) -> &ash::Entry {
                self.$field.vk_entry()
            }
        }
    };
}
