use std::{convert, ffi, ops::Not};

use ash::{prelude::*, vk};
use cstr::cstr;
use winit::window as win;

mod vku;

#[ouroboros::self_referencing]
struct VulkanState<'a> {
    instance: vku::Instance<'a>,

    #[cfg(debug_assertions)]
    #[covariant]
    #[borrows(instance)]
    debug: vku::DebugUtils<'this>,

    #[covariant]
    #[borrows(instance)]
    surface: vku::Surface<'this, 'a>,

    #[covariant]
    #[borrows(instance)]
    phy_devs: Vec<vku::PhysicalDev<'this>>,

    #[covariant]
    #[borrows(phy_devs, surface)]
    logic_dev: vku::LogicalDev<'this>,
}

impl<'a> VulkanState<'a> {
    fn create(entry: &'a ash::Entry, window: &'a win::Window) -> Self {
        let validation_layers = vec![
            cstr!(VK_LAYER_KHRONOS_validation).as_ptr(),
            // ...
        ];

        let mut extensions = vec![
            cstr!(VK_EXT_debug_utils).as_ptr(),
            cstr!(VK_KHR_surface).as_ptr(),
            // ...
        ];

        extensions.extend(
            vku::Surface::extensions(&window)
                .unwrap()
                .into_iter()
                .map(ffi::CStr::as_ptr),
        );

        let instance = unsafe {
            vku::Instance::new(
                entry,
                &validation_layers,
                &extensions,
                cstr!("Vulkan Tutorial"),
            )
            .unwrap()
        };

        VulkanStateBuilder {
            instance,
            #[cfg(debug_assertions)]
            debug_builder: |i| vku::DebugUtils::new(i).unwrap(),
            surface_builder: |i| vku::Surface::new(i, window).unwrap(),
            phy_devs_builder: |i| vku::PhysicalDev::list(i).unwrap(),
            logic_dev_builder: |devs, s| {
                let (idx, queues) = devs
                    .iter()
                    .copied()
                    .filter(|&dev| is_physical_device_suitable(dev))
                    .map(|dev| QueueFamiliesIndices::get(s, dev).unwrap())
                    .enumerate()
                    .flat_map(|(dev_idx, idxs)| Some(dev_idx).zip(idxs.zip()))
                    .next()
                    .expect("no suitable physical device found");
                vku::LogicalDev::new(devs[idx], &queues).unwrap()
            },
        }
        .build()
    }
}

fn is_physical_device_suitable(dev: vku::PhysicalDev) -> bool {
    let prop = dev.properties();
    let feat = dev.features();
    feat.tessellation_shader > 0
        && (prop.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            || prop.device_type == vk::PhysicalDeviceType::INTEGRATED_GPU)
}

#[derive(Clone, Copy)]
struct QueueFamiliesIndices {
    graphics: Option<u32>,
    present: Option<u32>,
}

impl QueueFamiliesIndices {
    fn get(surface: &vku::Surface, dev: vku::PhysicalDev) -> VkResult<Self> {
        let queue_families = dev.queue_families();
        let graphics = queue_families
            .iter()
            .position(|fam| fam.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|v| v as u32);
        let present = (0..queue_families.len())
            .map(|fam| surface.has_support(dev, fam as u32).unwrap())
            .position(convert::identity)
            .map(|v| v as u32);
        Ok(Self { graphics, present })
    }

    fn zip(self) -> Option<Vec<u32>> {
        let arr = [self.graphics?, self.present?];
        let mut vec = Vec::with_capacity(arr.len());
        arr.into_iter().for_each(|n| {
            vec.contains(&n).not().then(|| vec.push(n));
        });
        Some(vec)
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = win::WindowBuilder::new()
        .with_title("Vulkan Test")
        .with_inner_size(winit::dpi::LogicalSize::new(200, 200))
        .build(&event_loop)
        .unwrap();

    let entry = unsafe { ash::Entry::load().unwrap() };

    let vk_state = VulkanState::create(&entry, &window);
}
