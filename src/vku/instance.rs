use ash::vk;
use cstr::cstr;
use std::{ffi, os::raw};

/// A wrapper around all the necessary state needed to hold a Vulkan instance.
///
/// A Vulkan instance is a the connection between the application and the Vulkan library.
/// It's a reference to all the Vulkan objects created through it.
pub struct Instance<'a> {
    /// The acutal Vulkan instance handle
    instance: ash::Instance,
    /// The Vulkan entry point: a set of function pointers to Vulkan functions
    // TODO: this can probably be cloned
    entry: &'a ash::Entry,
}

impl<'a> Instance<'a> {
    /// Initializes a new Vulkan instance
    ///
    /// In the [`vk::ApplicationInfo`] sets the `application_name` to the value of the parameter `app_name`
    ///
    /// In debug mode adds a [`vk::DebugUtilsMessengerCreateInfoEXT`] struct to the [`vk::InstanceCreateInfo`],
    /// which enables the debuf utils extension for the instance related calls.
    ///
    /// # Safety
    ///
    /// `validation_layers_names` and `extensions_names` must contain pointers to null-terminated strings,
    /// they should be considered as [slice](std::slice)s of [`&CStr`](ffi::CStr)
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
        let mut dbg_utils_info = super::debug_utils::create_info();

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

/// Private definitions available only to the [vku](super) module
pub(super) mod pvt {
    /// Private definition of [`vku::InstanceHolder`](super::InstanceHolder)
    /// that allows to hide those methods from the public interface.
    ///
    /// Refer to the [`vku::InstanceHolder`](super::InstanceHolder) for the trait documentation.
    pub trait InstanceHolder {
        /// Returns a reference to the underlying [`ash::Instance`]
        fn vk_instance(&self) -> &ash::Instance;

        /// Returns a reference to the underlying [`ash::Entry`]
        fn vk_entry(&self) -> &ash::Entry;
    }
}

/// An [`vku::InstanceHolder`](InstanceHolder) is a type
/// that can access an [`vku::Instance`](Instance) either directly or
/// through another [`vku::InstanceHolder`](InstanceHolder)
pub trait InstanceHolder: pvt::InstanceHolder {}
impl<T: pvt::InstanceHolder> InstanceHolder for T {}

impl pvt::InstanceHolder for Instance<'_> {
    fn vk_instance(&self) -> &ash::Instance {
        &self.instance
    }

    fn vk_entry(&self) -> &ash::Entry {
        self.entry
    }
}

/// Implements the [`InstanceHolder`] in a transitive way by defining the methods
/// using a field of the struct that already implements them
///
/// # Example
///
/// ```
/// struct InstanceWrapper<I: InstanceHolder>(I);
///
/// derive_instance_holder!(InstanceWrapper<I> = 0: I);
/// ```
macro_rules! derive_instance_holder {
    ($self:ty = $field:ident : $generic:ident) => {
        impl<$generic: $crate::vku::InstanceHolder> $crate::vku::instance::pvt::InstanceHolder
            for $self
        {
            fn vk_instance(&self) -> &ash::Instance {
                self.$field.vk_instance()
            }

            fn vk_entry(&self) -> &ash::Entry {
                self.$field.vk_entry()
            }
        }
    };
}
