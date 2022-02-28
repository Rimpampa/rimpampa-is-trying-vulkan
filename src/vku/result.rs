use ash::vk;

/// Error related to a Vulkan operation
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error directly returned by a Vulkan function
    #[error("Vulkan error: {0:?}")]
    Vulkan(#[from] vk::Result),
}

pub type Result<T> = std::result::Result<T, Error>;
