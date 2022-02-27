use std::{ffi::CStr, marker};

use ash::{extensions::khr, vk};
use raw_window_handle as rwh;

pub struct Surface<'a, I: super::InstanceHolder> {
    instance: I,
    surface: vk::SurfaceKHR,
    fns: khr::Surface,

    window: marker::PhantomData<&'a dyn rwh::HasRawWindowHandle>,
}

impl<'a, I: super::InstanceHolder> Surface<'a, I> {
    pub fn new(instance: I, window: &'a dyn rwh::HasRawWindowHandle) -> super::Result<Self> {
        let surface = unsafe {
            ash_window::create_surface(instance.vk_entry(), instance.vk_instance(), window, None)
        }?;
        Ok(Self {
            surface,
            fns: khr::Surface::new(instance.vk_entry(), instance.vk_instance()),
            window: marker::PhantomData,
            instance,
        })
    }

    pub fn extensions(
        window: &'a dyn rwh::HasRawWindowHandle,
    ) -> super::Result<Vec<&'static CStr>> {
        Ok(ash_window::enumerate_required_extensions(window)?)
    }
}

impl<I: super::InstanceHolder> Drop for Surface<'_, I> {
    fn drop(&mut self) {
        unsafe { self.fns.destroy_surface(self.surface, None) };
    }
}

derive_instance_holder!(Surface<'_, I> = instance: I);

pub trait SurfaceHolder: super::InstanceHolder {
    fn vk_surface_fns(&self) -> &khr::Surface;
    fn vk_surface(&self) -> &vk::SurfaceKHR;
}

impl<I: super::InstanceHolder> SurfaceHolder for Surface<'_, I> {
    fn vk_surface_fns(&self) -> &khr::Surface {
        &self.fns
    }

    fn vk_surface(&self) -> &vk::SurfaceKHR {
        &self.surface
    }
}

macro_rules! derive_surface_holder {
    ($self:ty = $field:ident : $generic:ident) => {
        impl<$generic: $crate::vku::SurfaceHolder> $crate::vku::SurfaceHolder for $self {
            fn vk_surface_fns(&self) -> &ash::extensions::khr::Surface {
                self.$field.vk_surface_fns()
            }

            fn vk_surface(&self) -> &ash::vk::SurfaceKHR {
                self.$field.vk_surface()
            }
        }
    };
}
