use ash::vk;

pub struct PhysicalDevList<I: super::InstanceHolder> {
    instance: I,
    devices: Vec<vk::PhysicalDevice>,
}

pub struct PhysicalDevRef<'a, I: super::InstanceHolder> {
    instance: &'a I,
    device: vk::PhysicalDevice,
}

impl<I: super::InstanceHolder> Clone for PhysicalDevRef<'_, I> {
    fn clone(&self) -> Self {
        let Self { instance, device } = *self;
        Self { instance, device }
    }
}

impl<I: super::InstanceHolder> Copy for PhysicalDevRef<'_, I> {}

impl<I: super::InstanceHolder> PhysicalDevList<I> {
    pub fn list(instance: I) -> super::Result<Self> {
        let devices = unsafe { instance.vk_instance().enumerate_physical_devices()? };
        Ok(Self { instance, devices })
    }

    pub fn get(&self, index: usize) -> Option<PhysicalDevRef<'_, I>> {
        Some(PhysicalDevRef {
            instance: &self.instance,
            device: *self.devices.get(index)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = PhysicalDevRef<'_, I>> {
        self.devices.iter().map(|&device| PhysicalDevRef {
            instance: &self.instance,
            device,
        })
    }

    pub fn select(
        self,
        index: usize,
        queue_families: &[u32],
    ) -> super::Result<super::LogicalDev<I>> {
        super::LogicalDev::new(self.instance, self.devices[index], queue_families)
    }
}

impl<I: super::InstanceHolder> PhysicalDevRef<'_, I> {
    pub fn properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_properties(self.device)
        }
    }

    pub fn features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_features(self.device)
        }
    }

    pub fn queue_families(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .vk_instance()
                .get_physical_device_queue_family_properties(self.device)
        }
    }
}

impl<I: super::SurfaceHolder> PhysicalDevRef<'_, I> {
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
