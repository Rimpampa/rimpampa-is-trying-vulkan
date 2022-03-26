use std::os::raw::c_char;

use ash::{extensions::khr, vk};

/// A list of Vulkan physical device handles
///
/// A physical device in Vulkan is a reference to a physical GPU
///
/// This struct contains the list of all physical devices and
/// is used to select the most suitable one by iterating though this list
///
/// # Examples
///
/// ```
/// let list = PhysicalDevList::list(instance)?;
/// let index = most_suitable(&list);
/// let logical_device = list.select(index, queue_family_indices)?;
/// ```
pub struct PhysicalDevList<I: super::InstanceHolder> {
    /// The instance from which those devices
    instance: I,
    /// The list of physical device handles that are available for this `instance`
    devices: Vec<vk::PhysicalDevice>,
}

/// A reference to a Vulkan physical device handle
pub struct PhysicalDevRef<'a, I: super::InstanceHolder> {
    /// Instance to which the devices belongs
    instance: &'a I,
    /// Device handle
    pub handle: vk::PhysicalDevice,
}

// Cannot derive Clone + Copy due to the unwanted additional trait bound constrains
// that it adds to the generics, so implement them manually

impl<I: super::InstanceHolder> Clone for PhysicalDevRef<'_, I> {
    fn clone(&self) -> Self {
        let Self { instance, handle } = *self;
        Self { instance, handle }
    }
}

impl<I: super::InstanceHolder> Copy for PhysicalDevRef<'_, I> {}

impl<I: super::InstanceHolder> PhysicalDevList<I> {
    /// List all the available physical devices for the provided instance
    pub fn list(instance: I) -> super::Result<Self> {
        let devices = unsafe { instance.vk_instance().enumerate_physical_devices()? };
        Ok(Self { instance, devices })
    }

    /// Returns a reference to the physical device handle at `index`
    pub fn get(&self, index: usize) -> Option<PhysicalDevRef<'_, I>> {
        Some(PhysicalDevRef {
            instance: &self.instance,
            handle: *self.devices.get(index)?,
        })
    }

    /// Returns an iterator over all the physical device handles
    pub fn iter(&self) -> impl Iterator<Item = PhysicalDevRef<'_, I>> {
        self.devices.iter().map(|&device| PhysicalDevRef {
            instance: &self.instance,
            handle: device,
        })
    }

    /// Selects the physical device at `index` and a list of queue family indices
    /// and uses them to construct a Vulkan logical device
    ///
    /// # Parameters
    ///
    /// - `index`: the index of the physical device
    /// - `queue_family_infos`: queue family info and queues count
    ///
    /// # Panics
    ///
    /// If `index` points outside the list of available physical devices
    ///
    /// ## Debug Only
    ///
    /// If the same queue index is specified twice, or if `queue_family_infos` is empty
    ///
    /// # Safety
    ///
    /// `queue_family_infos` must be valid for the selected physical device.
    ///
    /// Check the documentation of [`vku::QueueFamilyInfo`](super::QueueFamilyInfo)
    /// to know what valid means.
    pub unsafe fn select<'a, Arr: AsRef<[super::QueueFamilyInfo<'a>]>>(
        self,
        index: usize,
        queue_family_infos: Arr,
        extensions: &[*const c_char],
    ) -> super::Result<super::LogicalDev<I>> {
        super::LogicalDev::new(self, index, queue_family_infos, extensions)
    }

    // NOTE: this can be done because this struct doesn't have a Drop impl
    pub fn unwrap(self) -> I {
        self.instance
    }
}

impl<I: super::InstanceHolder> PhysicalDevRef<'_, I> {
    /// Returns the properties of this physical device
    pub fn properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_properties(self.handle)
        }
    }

    /// Returns the features of this physical device
    pub fn features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_features(self.handle)
        }
    }

    /// Returns the list of queue families supported
    pub fn queue_families(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_queue_family_properties(self.handle)
        }
    }

    /// Returns the list of queue families supported
    pub fn extension_properties(&self) -> super::Result<Vec<vk::ExtensionProperties>> {
        unsafe {
            self.instance
                .vk_instance()
                .enumerate_device_extension_properties(self.handle)
        }
    }
}

impl<I: super::SurfaceHolder> PhysicalDevRef<'_, I> {
    fn vk_surface(&self) -> (&khr::Surface, &vk::SurfaceKHR) {
        (self.instance.vk_surface_fns(), self.instance.vk_surface())
    }

    /// Returns whether or not the [`vku::Surface`](super::Surface) bound to the
    /// current instance is supported by this physical device and queue family
    ///
    /// # Safety
    ///
    /// `queue_family_index` must be a valid queue family index for this physical device
    pub unsafe fn supports_surface(&self, queue_family_index: u32) -> super::Result<bool> {
        let (fns, surface) = self.vk_surface();
        fns.get_physical_device_surface_support(self.handle, queue_family_index, *surface)
    }

    /// Returns the capabilities that this devices has for the surface
    ///
    /// # Safety
    ///
    /// The device must support the `VK_KHR_swapchain` device extension
    pub unsafe fn surface_capabilities(&self) -> super::Result<vk::SurfaceCapabilitiesKHR> {
        let (fns, surface) = self.vk_surface();
        fns.get_physical_device_surface_capabilities(self.handle, *surface)
    }

    /// Returns the supported color formats by this devices for the surface
    ///
    /// # Safety
    ///
    /// The device must support the `VK_KHR_swapchain` device extension
    pub unsafe fn surface_formats(&self) -> super::Result<Vec<vk::SurfaceFormatKHR>> {
        let (fns, surface) = self.vk_surface();
        fns.get_physical_device_surface_formats(self.handle, *surface)
    }

    /// Returns the supported present modes by this devices for the surface
    ///
    /// # Safety
    ///
    /// The device must support the `VK_KHR_swapchain` device extension
    pub unsafe fn surface_present_modes(&self) -> super::Result<Vec<vk::PresentModeKHR>> {
        let (fns, surface) = self.vk_surface();
        fns.get_physical_device_surface_present_modes(self.handle, *surface)
    }
}
