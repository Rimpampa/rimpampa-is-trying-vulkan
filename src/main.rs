use std::ffi::CStr;

use ash::extensions::{ext, khr};
use ash::vk;
use cstr::cstr;
use winit::window as win;

#[derive(Clone, Copy, Debug, thiserror::Error)]
enum AppError {
    /// An error directly returned by a Vulkan function
    #[error("Vulkan error: {0:?}")]
    Vku(#[from] vku::Error),

    #[error("There are no suitable physical devices")]
    NoSuitablePhyDev,
}

type AppResult<T> = Result<T, AppError>;

struct VulkanState<'a>(vku::LogicalDev<vku::Surface<'a, vku::DebugUtils<vku::Instance<'a>>>>);

impl<'a> VulkanState<'a> {
    fn create(entry: &'a ash::Entry, window: &'a win::Window) -> AppResult<Self> {
        let validation_layers = vec![
            cstr!(VK_LAYER_KHRONOS_validation).as_ptr(),
            // ...
        ];

        let mut extensions = vec![
            ext::DebugUtils::name().as_ptr(),
            khr::Surface::name().as_ptr(),
            // ...
        ];

        extensions.extend(
            vku::surface::extensions(&window)
                .unwrap()
                .into_iter()
                .map(CStr::as_ptr),
        );

        let device_extensions = vec![
            khr::Swapchain::name(),
            // ...
        ];

        let win_size = window.inner_size();
        let win_size = vk::Extent2D {
            width: win_size.width,
            height: win_size.height,
        };

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

        let (dev_idx, create_info) = phy_devs
            .iter()
            .enumerate()
            .filter_map(|(i, dev)| Some((i, VkCreateInfo::new(dev, &device_extensions, win_size)?)))
            .next()
            .ok_or(AppError::NoSuitablePhyDev)?;

        let queue_create_info = create_info.queue_family_creation_infos();
        let dev_exts_ptr: Vec<_> = device_extensions.iter().map(|s| s.as_ptr()).collect();
        let logic_dev = unsafe { phy_devs.select(dev_idx, queue_create_info, &dev_exts_ptr)? };

        Ok(Self(logic_dev))
    }
}

#[derive(Clone, Copy, Default)]
struct VkCreateInfo {
    /// The graphics queue family queue index
    graphics_queue_id: u32,
    /// The present queue family queue index
    present_queue_id: u32,
    /// The chosen swapchain format
    swapchain_fmt: vk::SurfaceFormatKHR,
    /// The chosen swapchain presentation mode
    swapchain_pmode: vk::PresentModeKHR,
    /// The chosen swapchain area
    swapchain_extent: vk::Extent2D,
    /// The chosen swapchain image count
    swapchain_imgs: u32,
}

impl VkCreateInfo {
    /// Checks if the physical device has the right properties for the application
    ///
    /// # Return
    ///
    /// The queues families indices of the needed queue families if all present
    fn new<I: vku::SurfaceHolder>(
        dev: vku::PhysicalDevRef<I>,
        dev_exts: &[&CStr],
        win_size: vk::Extent2D,
    ) -> Option<VkCreateInfo> {
        let create_info = Self::default();

        let prop = dev.properties();
        let feat = dev.features();
        let exts: Vec<_> = dev
            .extension_properties()
            .ok()?
            .iter()
            // SAFETY: This pointer was generated by the Vulkan driver
            .map(|prop| unsafe { CStr::from_ptr(prop.extension_name.as_ptr()) })
            .collect();

        let dev_types = {
            use vk::PhysicalDeviceType as devtype;
            [devtype::DISCRETE_GPU, devtype::INTEGRATED_GPU]
        };
        if feat.tessellation_shader == 0 || !dev_types.contains(&prop.device_type) {
            return None;
        }

        if !dev_exts.iter().all(|ext_name| exts.contains(ext_name)) {
            return None;
        }

        let create_info = match dev_exts.contains(&khr::Swapchain::name()) {
            // SAFETY: just checked if the extension is supported
            true => unsafe { create_info.get_swapchain_properties(dev, win_size)? },
            false => create_info,
        };
        create_info.get_queue_family_indices(dev)
    }

    /// # Safety
    ///
    /// This method expects the `VK_KHR_swapchain` extension to be supported
    /// by the device
    unsafe fn get_swapchain_properties<I: vku::SurfaceHolder>(
        self,
        dev: vku::PhysicalDevRef<I>,
        win_size: vk::Extent2D,
    ) -> Option<Self> {
        let (caps, fmts, pmods) = (
            dev.surface_capabilities().ok()?,
            dev.surface_formats().ok()?,
            dev.surface_present_modes().ok()?,
        );

        let format = *fmts
            .iter()
            .filter(|fmt| fmt.format == vk::Format::R8G8B8A8_SRGB)
            .find(|fmt| fmt.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .or_else(|| fmts.first())?;

        if pmods.is_empty() {
            return None;
        }
        let pmode = pmods
            .contains(&vk::PresentModeKHR::MAILBOX)
            .then(|| vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let vk::Extent2D {
            height: max_height,
            width: max_width,
        } = caps.max_image_extent;
        let vk::Extent2D {
            height: min_height,
            width: min_width,
        } = caps.min_image_extent;

        let extent = match caps.current_extent {
            vk::Extent2D {
                height: u32::MAX,
                width: u32::MAX,
            } => vk::Extent2D {
                height: win_size.height.clamp(min_height, max_height),
                width: win_size.width.clamp(min_width, max_width),
            },
            ext => ext,
        };

        let imgs = match caps.max_image_count {
            0 => caps.min_image_count + 1,
            n => n.min(caps.min_image_count + 1),
        };

        Some(Self {
            swapchain_fmt: format,
            swapchain_pmode: pmode,
            swapchain_extent: extent,
            swapchain_imgs: imgs,
            ..self
        })
    }

    /// Returns the queue families indices needed by the application,
    /// or [None] if they are not supported
    fn get_queue_family_indices<I: vku::SurfaceHolder>(
        self,
        dev: vku::PhysicalDevRef<I>,
    ) -> Option<Self> {
        let queue_families = dev.queue_families();
        let graphics_queue_id = queue_families
            .iter()
            .position(|fam| fam.queue_flags.contains(vk::QueueFlags::GRAPHICS))?
            as u32;
        let present_queue_id = (0..queue_families.len())
            // SAFETY:
            // The range is based on the length of the Vec returned by `queue_families`
            // and the same device is being used
            .find(|&fam| unsafe { dev.supports_surface(fam as u32).unwrap_or(false) })?
            as u32;
        Some(Self {
            graphics_queue_id,
            present_queue_id,
            ..self
        })
    }

    /// Returns the info needed for creating the queues
    fn queue_family_creation_infos(self) -> Vec<vku::QueueFamilyInfo<'static>> {
        let arr = [self.graphics_queue_id, self.present_queue_id];
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
        vec
    }
}

impl VkCreateInfo {}

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
