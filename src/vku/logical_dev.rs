use ash::vk;

/// A wrapper around all the necessary state needed to hold a Vulkan logical device.
///
/// A Vulkan logical device is a connection to a physical device which specifies a subeset of
/// the capabilities of that physical device that it needs to perform further operations
pub struct LogicalDev<I: super::InstanceHolder> {
    /// The instance which this logical device belongs to
    instance: I,
    /// The actual Vulkan device handle
    device: ash::Device,
}

impl<I: super::InstanceHolder> LogicalDev<I> {
    pub(super) unsafe fn new(instance: I, device: ash::Device) -> Self {
        Self { instance, device }
    }

    /// Returns an handle to the selected Vulkan queue
    ///
    /// # Safety
    ///
    /// `queue_index` must be smaller or equal to the number of queues created for that
    /// family.
    ///
    /// `queue_family_index` must be one of the indices provided to `new`
    unsafe fn get_queue(&self, queue_family_index: u32, queue_index: u32) -> vk::Queue {
        self.device
            .get_device_queue(queue_family_index, queue_index)
    }
}

impl<I: super::InstanceHolder> Drop for LogicalDev<I> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) }
    }
}

derive_instance_holder!(LogicalDev<I> = instance: I);
derive_surface_holder!(LogicalDev<I> = instance: I);
