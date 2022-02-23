use ash::{prelude::*, vk};

#[derive(Clone, Copy)]
pub struct PhysicalDev<'a> {
    instance: &'a super::Instance<'a>,
    device: vk::PhysicalDevice,
}

impl<'a> PhysicalDev<'a> {
    pub fn instance(&self) -> &super::Instance {
        self.instance
    }

    pub fn list(instance: &'a super::Instance<'a>) -> VkResult<Vec<Self>> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        Ok(devices
            .into_iter()
            .map(|device| Self { instance, device })
            .collect())
    }

    pub fn properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe { self.instance.get_physical_device_properties(self.device) }
    }

    pub fn features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe { self.instance.get_physical_device_features(self.device) }
    }

    pub fn queue_families(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .get_physical_device_queue_family_properties(self.device)
        }
    }
}

impl std::ops::Deref for PhysicalDev<'_> {
    type Target = vk::PhysicalDevice;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
