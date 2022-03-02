use std::{iter, marker::PhantomData};

use ash::vk;

/// A wrapper around all the necessary state needed to hold a Vulkan logical device.
///
/// A Vulkan logical device is a connection to a physical device which specifies a subeset of
/// the capabilities of that physical device that it needs to perform further operations
pub struct LogicalDev<'a, I, Arr>
where
    I: super::InstanceHolder,
    Arr: AsRef<[super::QueueFamilyInfo<'a>]> + 'a,
{
    /// The instance which this logical device belongs to
    instance: I,
    /// The actual Vulkan device handle
    device: ash::Device,
    /// The pairs of queue family index and queues count selected when creating this device
    /// relative to the underlying physical devices
    queue_family_infos: Arr,

    _m: std::marker::PhantomData<&'a ()>,
}

impl<'a, I, Arr> LogicalDev<'a, I, Arr>
where
    I: super::InstanceHolder,
    Arr: AsRef<[super::QueueFamilyInfo<'a>]>,
{
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
    pub(super) unsafe fn new(
        dev_list: super::PhysicalDevList<I>,
        selected_dev: usize,
        queue_family_infos: Arr,
    ) -> super::Result<Self> {
        // Can't have a device with zero queues enabled
        debug_assert!(!queue_family_infos.as_ref().is_empty());
        // Can't create two separate queues of the same family
        debug_assert!(
            iter::successors(queue_family_infos.as_ref().split_first(), |(_, s)| s
                .split_first())
            .all(|(f, r)| !r.iter().any(|r| r.index == f.index))
        );

        let queue_create_infos: Vec<_> = queue_family_infos
            .as_ref()
            .iter()
            .map(|&info| info.create_info())
            .collect();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .build();

        let phydev = dev_list.get(selected_dev).unwrap().handle;
        let instance = dev_list.unwrap();
        let device = instance
            .vk_instance()
            .create_device(phydev, &create_info, None)?;

        Ok(Self {
            instance,
            device,
            queue_family_infos,
            _m: PhantomData,
        })
    }

    /// Returns an handle to the selected Vulkan queue
    ///
    /// # Parameters
    ///
    /// - `queue_family_index_index`:
    /// is the index relative to the queue family indices selected on device creation
    /// - `queue_index`:
    /// the index of the actual queue
    ///
    /// # Return
    ///
    /// This function returns [`None`] when `queue_family_index_index` is bigger than the
    /// number of selected queue family indices, otherwise [`Some(vk::Queue)`](vk::Queue).
    ///
    /// # Safety
    ///
    /// `queue_index` must be smaller or equal to the number of queues created for that
    /// family.
    unsafe fn get_queue(
        &self,
        queue_family_index_index: usize,
        queue_index: u32,
    ) -> Option<vk::Queue> {
        Some(
            self.device.get_device_queue(
                self.queue_family_infos
                    .as_ref()
                    .get(queue_family_index_index)?
                    .index,
                queue_index,
            ),
        )
    }
}

impl<'a, I, Arr> Drop for LogicalDev<'a, I, Arr>
where
    I: super::InstanceHolder,
    Arr: AsRef<[super::QueueFamilyInfo<'a>]>,
{
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) }
    }
}

derive_instance_holder!(
    #[generics('a, Arr: AsRef<[super::QueueFamilyInfo<'a>]>)]
    LogicalDev<'a, I, Arr> = instance: I
);
derive_surface_holder!(
    #[generics('a, Arr: AsRef<[super::QueueFamilyInfo<'a>]>)]
    LogicalDev<'a, I, Arr> = instance: I
);
