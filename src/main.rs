use std::{convert, ffi};

use ash::vk;
use cstr::cstr;
use winit::window as win;

mod vku;

struct VulkanState<'a>(vku::LogicalDev<vku::Surface<'a, vku::DebugUtils<vku::Instance<'a>>>>);

impl<'a> VulkanState<'a> {
    fn create(entry: &'a ash::Entry, window: &'a win::Window) -> vku::Result<Self> {
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
            vku::surface::extensions(&window)
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
            )?
        };

        let debug_utils = vku::DebugUtils::new(instance)?;

        let surface = vku::Surface::new(debug_utils, window)?;

        let phy_devs = vku::PhysicalDevList::list(surface)?;

        let (idx, queues): (usize, Vec<vku::QueueFamilyInfo<'_>>) = phy_devs
            .iter()
            .enumerate()
            .filter(|(_, dev)| is_physical_device_suitable(*dev))
            .map(|(i, dev)| (i, QueueFamiliesIndices::get(dev)))
            .find_map(|(i, result)| match result {
                Ok(indices) => indices.zip().map(|indices| Ok((i, indices))),
                Err(e) => Some(Err(e)),
            })
            .expect("no suitable physical device found")?;

        let logic_dev = unsafe { phy_devs.select(idx, queues)? };

        Ok(Self(logic_dev))
    }
}

fn is_physical_device_suitable<I: vku::InstanceHolder>(dev: vku::PhysicalDevRef<I>) -> bool {
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
    fn get<I: vku::SurfaceHolder>(dev: vku::PhysicalDevRef<I>) -> vku::Result<Self> {
        let queue_families = dev.queue_families();
        let graphics = queue_families
            .iter()
            .position(|fam| fam.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|v| v as u32);
        let present = (0..queue_families.len())
            .map(|fam| dev.supports_surface(fam as u32).unwrap())
            .position(convert::identity)
            .map(|v| v as u32);
        Ok(Self { graphics, present })
    }

    fn zip(self) -> Option<Vec<vku::QueueFamilyInfo<'static>>> {
        let arr = [self.graphics?, self.present?];
        let mut vec = Vec::<vku::QueueFamilyInfo>::with_capacity(arr.len());
        arr.into_iter().for_each(|n| {
            if vec.iter().any(|i| i.index == n) {
                return;
            }
            vec.push(vku::QueueFamilyInfo {
                index: n,
                priorities: &[1.0],
            })
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
