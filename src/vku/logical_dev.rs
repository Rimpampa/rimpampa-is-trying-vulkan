use std::os::raw::c_char;

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
    /// Creates a new Vulkan device by chosing a list of queue families from a physical device
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
    pub(super) unsafe fn new<'a>(
        dev_list: super::PhysicalDevList<I>,
        selected_dev: usize,
        queue_family_infos: impl AsRef<[super::QueueFamilyInfo<'a>]>,
        extensions: &[*const c_char],
    ) -> super::Result<Self> {
        // Can't have a device with zero queues enabled
        debug_assert!(!queue_family_infos.as_ref().is_empty());
        // Can't create two separate queues of the same family
        debug_assert!(std::iter::successors(
            queue_family_infos.as_ref().split_first(),
            |(_, s)| s.split_first()
        )
        .all(|(f, r)| !r.iter().any(|r| r.index == f.index)));

        let queue_create_infos: Vec<_> = queue_family_infos
            .as_ref()
            .iter()
            .map(|&info| info.create_info())
            .collect();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&extensions)
            .build();

        let phydev = dev_list.get(selected_dev).unwrap().handle;
        let instance = dev_list.unwrap();
        let device = instance
            .vk_instance()
            .create_device(phydev, &create_info, None)?;

        Ok(Self { instance, device })
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
