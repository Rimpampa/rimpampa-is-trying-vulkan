use ash::vk;

pub struct LogicalDev<'a> {
    _physical_dev: super::PhysicalDev<'a>,
    device: ash::Device,
    queues: Vec<vk::Queue>,
}

impl<'a> LogicalDev<'a> {
    pub fn new(
        physical_dev: super::PhysicalDev<'a>,
        queue_family_indices: &[u32],
    ) -> super::Result<Self> {
        // Can't have a device with zero queues enabled
        debug_assert!(!queue_family_indices.is_empty());
        // Can't create two separate queues of the same family
        debug_assert!(queue_family_indices.iter().all(|a| queue_family_indices
            .iter()
            .filter(|&b| a == b)
            .count()
            == 1));

        let queue_create_infos: Vec<_> = queue_family_indices
            .iter()
            .map(|&idx| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(idx)
                    .queue_priorities(&[1.0])
                    .build()
            })
            .collect();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .build();

        let device = unsafe {
            physical_dev
                .instance()
                .create_device(*physical_dev, &create_info, None)?
        };
        let queues = queue_family_indices
            .iter()
            .map(|&i| unsafe { device.get_device_queue(i, 0) })
            .collect();

        Ok(Self {
            _physical_dev: physical_dev,
            device,
            queues,
        })
    }
}

impl Drop for LogicalDev<'_> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) }
    }
}
