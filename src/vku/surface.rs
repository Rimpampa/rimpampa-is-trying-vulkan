use std::{ffi::CStr, marker::PhantomData};

use ash::{extensions::khr, vk};
use raw_window_handle as rwh;

/// Returns the names of the Vulkan extensions required by the provided window handle
pub fn extensions(window: &dyn rwh::HasRawWindowHandle) -> super::Result<Vec<&'static CStr>> {
    ash_window::enumerate_required_extensions(window)
}

/// A wrapper around all the necessary state needed to hold a Vulkan surface
///
/// A Vulkan surface is a generic interface through which Vulkan interacts with the window system
/// of any OS.
pub struct Surface<'a, I: super::InstanceHolder> {
    /// The Vulkan instance holder that holds this surface
    instance: I,
    /// The actual Vulkan surface handle
    surface: vk::SurfaceKHR,
    /// A set of function pointers to Vulkan functions related to the KHR extension
    fns: khr::Surface,

    /// A marker to the window bound to this surface, the compiler uses this declaration
    /// - more specifically the lifetime bound to it -
    /// to stop the actual window object from being dropped before this value
    /// without requiring any space to store the actual ref
    window: PhantomData<&'a dyn rwh::HasRawWindowHandle>,
}

impl<'a, I: super::InstanceHolder> Surface<'a, I> {
    /// Creates a new Vulkan surface
    ///
    /// The instance should be created with all the necessary extensions,
    /// check the [`extensions`] function to know which are needed.
    pub fn new(instance: I, window: &'a dyn rwh::HasRawWindowHandle) -> super::Result<Self> {
        let surface = unsafe {
            ash_window::create_surface(instance.vk_entry(), instance.vk_instance(), window, None)
        }?;
        Ok(Self {
            surface,
            fns: khr::Surface::new(instance.vk_entry(), instance.vk_instance()),
            window: PhantomData,
            instance,
        })
    }
}

impl<I: super::InstanceHolder> Drop for Surface<'_, I> {
    fn drop(&mut self) {
        unsafe { self.fns.destroy_surface(self.surface, None) };
    }
}

derive_instance_holder!(Surface<'_, I> = instance: I);

/// Private definitions available only to the [vku](super) module
pub(super) mod pvt {
    use super::*;

    /// Private definition of [`vku::SurfaceHolder`](super::SurfaceHolder)
    /// that allows to hide those methods from the public interface.
    ///
    /// Refer to the [`vku::SurfaceHolder`](super::SurfaceHolder) for the trait documentation.
    pub trait SurfaceHolder: super::super::InstanceHolder {
        /// Returns a reference to the underlying [`khr::Surface`]
        fn vk_surface_fns(&self) -> &khr::Surface;

        /// Returns a reference to the underlying [`vk::SurfaceKHR`]
        fn vk_surface(&self) -> &vk::SurfaceKHR;
    }
}
/// A [`vku::SurfaceHolder`](SurfaceHolder) is a type
/// that can access a [`vku::Surface`](Surface) either directly
/// or through another [`vku::SurfaceHolder`](SurfaceHolder)
///
/// It also must have access to an [`vku::Instance`](super::Instance)
/// (the one to which the Vulkan surface belongs to)
pub trait SurfaceHolder: pvt::SurfaceHolder {}
impl<T: pvt::SurfaceHolder> SurfaceHolder for T {}

impl<I: super::InstanceHolder> pvt::SurfaceHolder for Surface<'_, I> {
    fn vk_surface_fns(&self) -> &khr::Surface {
        &self.fns
    }

    fn vk_surface(&self) -> &vk::SurfaceKHR {
        &self.surface
    }
}

/// Implements the [`SurfaceHolder`] in a transitive way by defining the methods
/// using a field of the struct that already implements them
///
/// The `#[generics(...)]` meta-like attribute can be added before everything to declare
/// additional generics (either lifetimes or types).
///
/// # Example
///
/// Derive the trait on a wrapper type
/// ```
/// struct SurfaceWrapper<I: SurfaceHolder>(I);
///
/// derive_surface_holder!(SurfaceWrapper<I> = 0: I);
/// ```
///
/// Derive the trait on a wrapper type that has additional generics
/// ```
/// struct SurfaceRefWrapper<'a, I: SurfaceHolder>(&'a I);
///
/// derive_surface_holder!(
///     #[generics('a)]
///     SurfaceRefWrapper<'a, I> = 0: I
/// );
/// ```
macro_rules! derive_surface_holder {
    ( $( #[generics( $( $generics:tt )* )] )? $self:ty = $field:tt : $generic:ident) => {
        impl<
            // Additional generics, note the comma before closing the optional block
            $( $( $generics )* , )?
            // SurfaceHolder generic
            $generic : $crate::vku::SurfaceHolder
        > $crate::vku::surface::pvt::SurfaceHolder for $self {
            fn vk_surface_fns(&self) -> &ash::extensions::khr::Surface {
                self.$field.vk_surface_fns()
            }

            fn vk_surface(&self) -> &ash::vk::SurfaceKHR {
                self.$field.vk_surface()
            }
        }
    };
}
