use cstr::cstr;

mod vku;

fn main() {
    let entry = unsafe { ash::Entry::load().unwrap() };

    let validation_layers = vec![
        cstr!(VK_LAYER_KHRONOS_validation).as_ptr(),
        // ...
    ];

    let extensions = vec![
        cstr!(VK_EXT_debug_utils).as_ptr(),
        cstr!(VK_KHR_surface).as_ptr(),
        // ...
    ];

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
}
