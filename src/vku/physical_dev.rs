use ash::vk;

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
    device: vk::PhysicalDevice,
}

// Cannot derive Clone + Copy due to the unwanted additional trait bound constrains
// that it adds to the generics, so implement them manually

impl<I: super::InstanceHolder> Clone for PhysicalDevRef<'_, I> {
    fn clone(&self) -> Self {
        let Self { instance, device } = *self;
        Self { instance, device }
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
            device: *self.devices.get(index)?,
        })
    }

    /// Returns an iterator over all the physical device handles
    pub fn iter(&self) -> impl Iterator<Item = PhysicalDevRef<'_, I>> {
        self.devices.iter().map(|&device| PhysicalDevRef {
            instance: &self.instance,
            device,
        })
    }

    /// Selects the physical device at `index` and a list of queue family indices
    /// and uses them to construct a Vulkan logical device
    pub fn select(
        self,
        index: usize,
        queue_families: &[u32],
    ) -> super::Result<super::LogicalDev<I>> {
        super::LogicalDev::new(self.instance, self.devices[index], queue_families)
    }
}

impl<I: super::InstanceHolder> PhysicalDevRef<'_, I> {
    /// Returns the properties of this physical device
    pub fn properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_properties(self.device)
        }
    }

    /// Returns the features of this physical device
    pub fn features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_features(self.device)
        }
    }

    /// Returns the list of queue families supported
    pub fn queue_families(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_queue_family_properties(self.device)
        }
    }
}

impl<I: super::SurfaceHolder> PhysicalDevRef<'_, I> {
    /// Returns whether or not the [`vku::Surface`](super::Surface) bound to the
    /// current instance is supported by this physical device and queue family
    pub fn supports_surface(&self, queue_family: u32) -> super::Result<bool> {
        Ok(unsafe {
            self.instance
                .vk_surface_fns()
                .get_physical_device_surface_support(
                    self.device,
                    queue_family,
                    *self.instance.vk_surface(),
                )?
        })
    }
}
