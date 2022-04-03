use ash::vk;

/// A wrapper around all the necessary state needed to hold a Vulkan logical device.
///
/// A Vulkan logical device is a connection to a physical device which specifies a subeset of
/// the capabilities of that physical device that it needs to perform further operations
pub struct LogicalDev<I: super::InstanceHolder> {
    /// The instance which this logical device belongs to
    instance: I,
    /// The actual Vulkan device handle
    device: ash::Device,
}

impl<I: super::InstanceHolder> LogicalDev<I> {
    pub(super) unsafe fn new(instance: I, device: ash::Device) -> Self {
        Self { instance, device }
    }

    /// Returns an handle to the selected Vulkan queue
    ///
    /// # Safety
    ///
    /// `queue_index` must be smaller or equal to the number of queues created for that
    /// family.
    ///
    /// `queue_family_index` must be one of the indices provided to `new`
    unsafe fn get_queue(&self, queue_family_index: u32, queue_index: u32) -> vk::Queue {
        self.device
            .get_device_queue(queue_family_index, queue_index)
    }
}

impl<I: super::InstanceHolder> Drop for LogicalDev<I> {
    fn drop(&mut self) {
        unsafe { self.device.destroy_device(None) }
    }
}

derive_instance_holder!(LogicalDev<I> = instance: I);
derive_surface_holder!(LogicalDev<I> = instance: I);

/// Private definitions available only to the [vku](super) module
pub(super) mod pvt {
    /// Private definition of [`vku::DeviceHolder`](super::DeviceHolder)
    /// that allows to hide those methods from the public interface.
    ///
    /// Refer to the [`vku::DeviceHolder`](super::DeviceHolder) for the trait documentation.
    pub trait DeviceHolder {
        /// Returns a reference to the underlying [`vk::Device`](ash::vk::Device)
        fn vk_device(&self) -> &ash::Device;
    }
}

/// An [`vku::DeviceHolder`](DeviceHolder) is a type
/// that can access an [`vku::LogicalDev`](LogicalDev) either directly or
/// through another [`vku::DeviceHolder`](DeviceHolder)
pub trait DeviceHolder: pvt::DeviceHolder {}
impl<T: pvt::DeviceHolder> DeviceHolder for T {}

impl<I: super::InstanceHolder> pvt::DeviceHolder for LogicalDev<I> {
    fn vk_device(&self) -> &ash::Device {
        &self.device
    }
}

/// Implements the [`DeviceHolder`] in a transitive way by defining the methods
/// using a field of the struct that already implements them
///
/// The `#[generics(...)]` meta-like attribute can be added before everything to declare
/// additional generics (either lifetimes or types).
///
/// # Example
///
/// Derive the trait on a wrapper type
/// ```
/// struct DeviceWrapper<I: DeviceHolder>(I);
///
/// derive_instance_holder!(DeviceWrapper<I> = 0: I);
/// ```
///
/// Derive the trait on a wrapper type that has additional generics
/// ```
/// struct DeviceWrapper<'a, I: DeviceHolder>(&'a I);
///
/// derive_device_holder!(
///     #[generics('a)]
///     DeviceWrapper<'a, I> = 0: I
/// );
/// ```
macro_rules! derive_device_holder {
    ( $( #[generics( $( $generics:tt )* )] )? $self:ty = $field:tt : $generic:ident) => {
        impl<
            // Additional generics, note the comma before closing the optional block
            $( $( $generics )* , )?
            // InstanceHodler generic
            $generic : $crate::DeviceHolder
        > $crate::instance::pvt::DeviceHolder for $self {
            fn vk_device(&self) -> &ash::Device {
                self.$field.vk_device()
            }
        }
    };
}
