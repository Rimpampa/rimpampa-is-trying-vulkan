use ash::vk;

/// Information abount a queue family
///
/// # Validity
///
/// The validity depends on the physical device it refers to and consists of this checks:
/// - `index` must be lower than the lenght of [`queue_families`] of [`vku::PhysicalDevRef`](super::PhysicalDevRef)
/// - the length of `priorities` must be lower than the `queue_count` for the queue at `index`
/// - the values in `priorities` must sum up to `1.0`
#[derive(Clone, Copy)]
pub struct QueueFamilyInfo<'a> {
    pub index: u32,
    pub priorities: &'a [f32],
}

impl<'a> QueueFamilyInfo<'a> {
    /// Get the Vulkan struct that describes of to create a queue with those properties
    ///
    /// # Safety
    ///
    /// The return value contains a pointer to the `priorities` slice, this means that `'a` must
    /// live unitl the last use of the return value is made.
    pub fn create_info(self) -> vk::DeviceQueueCreateInfo {
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(self.index)
            .queue_priorities(self.priorities)
            .build()
    }
}
