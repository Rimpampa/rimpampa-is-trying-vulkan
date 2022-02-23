use std::ffi::CStr;

use ash::{extensions::khr, prelude::*, vk};
use raw_window_handle as rwh;

pub struct Surface<'a, 'b: 'a> {
    surface: vk::SurfaceKHR,
    fns: khr::Surface,

    _instance: &'a super::Instance<'a>,
    _window: &'b dyn rwh::HasRawWindowHandle,
}

impl<'a, 'b: 'a> Surface<'a, 'b> {
    pub fn new(
        instance: &'a super::Instance<'a>,
        window: &'b dyn rwh::HasRawWindowHandle,
    ) -> VkResult<Self> {
        let surface =
            unsafe { ash_window::create_surface(instance.entry(), instance, window, None) }?;
        Ok(Self {
            surface,
            fns: khr::Surface::new(instance.entry(), instance),
            _instance: instance,
            _window: window,
        })
    }

    pub fn extensions(window: &'a dyn rwh::HasRawWindowHandle) -> VkResult<Vec<&'static CStr>> {
        ash_window::enumerate_required_extensions(window)
    }

    pub fn has_support(&self, dev: super::PhysicalDev<'_>, queue_family: u32) -> VkResult<bool> {
        unsafe {
            self.fns
                .get_physical_device_surface_support(*dev, queue_family, self.surface)
        }
    }
}

impl Drop for Surface<'_, '_> {
    fn drop(&mut self) {
        unsafe { self.fns.destroy_surface(self.surface, None) };
    }
}
