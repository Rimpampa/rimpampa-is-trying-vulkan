use std::ffi;

use cstr::cstr;
use winit::window as win;

mod vku;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = win::WindowBuilder::new()
        .with_title("Vulkan Test")
        .with_inner_size(winit::dpi::LogicalSize::new(200, 200))
        .build(&event_loop)
        .unwrap();

    let entry = unsafe { ash::Entry::load().unwrap() };

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
            &entry,
            &validation_layers,
            &extensions,
            cstr!("Vulkan Tutorial"),
        )
        .unwrap()
    };

    let _debug_utils = vku::DebugUtils::new(&instance).unwrap();

    let _phy_dev = vku::PhysicalDev::list(&instance).unwrap();

    let _surface = vku::Surface::new(&instance, &window).unwrap();
}
