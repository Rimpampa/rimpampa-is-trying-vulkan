#[allow(unused_imports)]
use crate as vku; // <--- Used in docs

use ash::{extensions::khr, vk};

/// How the image is to be shared between all the queue families
pub enum ImageSharing {
    /// The image is owned by one queue family at a time, changing the ownership
    /// must be done explicitly
    Exclusive,
    /// The image is shared between the queue families contained in this value
    Concurrent(Vec<u32>),
}

impl ImageSharing {
    /// Convert the enum into the values expected by the Vulkan API
    fn vk_convert(&self) -> (vk::SharingMode, &[u32]) {
        match self {
            ImageSharing::Exclusive => (vk::SharingMode::EXCLUSIVE, &[]),
            ImageSharing::Concurrent(v) => (vk::SharingMode::CONCURRENT, v),
        }
    }
}

/// Swapchain image details
pub struct ImageDetails {
    /// Number of buffered images
    pub count: u32,
    /// Format for storing the images
    pub format: vk::Format,
    /// Color space for storing the images
    pub color_space: vk::ColorSpaceKHR,
    /// Size of the images
    pub extent: vk::Extent2D,
    /// How the images will be shared between the different queue families
    pub sharing: ImageSharing,
    /// TODO
    pub transform: vk::SurfaceTransformFlagsKHR,
    /// TODO
    pub present_mode: vk::PresentModeKHR,
}

/// A wrapper around all the necessary state needed to hold a Vulkan swapchain
///
/// A Vulkan swapchain handles how the rendered images are stored and buffered
pub struct Swapchain<I: super::SurfaceHolder + super::DeviceHolder> {
    /// The instance which this swapchain belongs to
    instance: I,
    /// Function pointers for the KHR swapchain extension
    fns: khr::Swapchain,
    /// The Vulkan swapchain handle
    swapchain: vk::SwapchainKHR,
}

impl<I: super::SurfaceHolder + super::DeviceHolder> Swapchain<I> {
    /// Creates a new Vulkan swapchain
    ///
    /// # Safety
    ///
    /// Regarding the values of the fields in `details`:
    ///
    /// - `surface` must be a surface that is supported by the device as determined using
    ///   [`vku::PhysicalDevRef::supports_surface`]
    ///
    /// - `count` must be less than or equal to the value returned in the `max_image_count` member of the
    ///   [`vk::SurfaceCapabilitiesKHR`] structure returned by
    ///   [`vku::PhysicalDevRef::surface_capabilities`]
    ///   for the surface if the returned `max_image_count` is not zero
    ///
    /// - if `present_mode` is not
    ///   [`vk::PresentModeKHR::SHARED_DEMAND_REFRESH`] nor
    ///   [`vk::PresentModeKHR::SHARED_CONTINUOUS_REFRESH`],
    ///   then `count` must be greater than or equal to the value returned in the `min_image_count` member of the
    ///   [`vk::SurfaceCapabilitiesKHR`] structure returned by
    ///   [`vku::PhysicalDevRef::surface_capabilities`] for the surface
    ///
    /// - if `present_mode` is not
    ///   [`vk::PresentModeKHR::SHARED_DEMAND_REFRESH`] nor
    ///   [`vk::PresentModeKHR::SHARED_CONTINUOUS_REFRESH`],
    ///   then `count` must be `1`
    ///
    /// - `format` and `color_space` must match the `format` and `color_cpace` members, respectively,
    ///   of one of the [`vk::SurfaceFormatKHR`] structures returned by
    ///   [`vku::PhysicalDevRef::surface_formats`] for the surface
    ///
    /// - `extent` must be between `min_image_extent` and `max_image_extent`, inclusive,
    ///   of the [`vk::SurfaceCapabilitiesKHR`] structure returned by
    ///   [`vku::PhysicalDevRef::surface_capabilities`] for the surface
    ///
    /// - `extent` members `width` and `height` must both be non-zero
    ///
    /// - if `sharing` is [`ImageSharing::Concurrent`], the length of the [`Vec`] must be greater than `1`
    ///
    /// - if `sharing` is [`ImageSharing::Concurrent`], each element of the [`Vec`] must be unique
    ///   and must be less than the number of elemenets returned by
    ///   [`vku::PhysicalDevRef::queue_families`] of the selected device
    ///
    /// - `transform` must be one of the bits present in the `supported_transforms` member of the
    ///    [`vk::SurfaceCapabilitiesKHR`] structure returned by
    ///    [`vku::PhysicalDevRef::surface_capabilities`] for the surface
    ///
    /// - `present_mode` must be one of the [`vk::PresentModeKHR`] values returned by
    ///   [`vku::PhysicalDevRef::surface_present_modes`] for the surface
    pub unsafe fn new(instance: I, details: ImageDetails) -> super::Result<Self> {
        let fns = khr::Swapchain::new(instance.vk_instance(), instance.vk_device());

        let (sharing_mode, queue_indices) = details.sharing.vk_convert();
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*instance.vk_surface())
            .min_image_count(details.count)
            .image_format(details.format)
            .image_color_space(details.color_space)
            .image_extent(details.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(sharing_mode)
            .queue_family_indices(queue_indices)
            .pre_transform(details.transform)
            // NOTE: must be one of the bits present in the supportedCompositeAlpha
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(details.present_mode)
            .clipped(true)
            .build();

        let swapchain = fns.create_swapchain(&create_info, None)?;
        Ok(Self {
            instance,
            fns,
            swapchain,
        })
    }

    /// Gets the swapchain images
    pub fn images(&self) -> super::Result<Vec<vk::Image>> {
        unsafe { self.fns.get_swapchain_images(self.swapchain) }
    }
}

impl<I: super::SurfaceHolder + super::DeviceHolder> Drop for Swapchain<I> {
    fn drop(&mut self) {
        unsafe { self.fns.destroy_swapchain(self.swapchain, None) }
    }
}
