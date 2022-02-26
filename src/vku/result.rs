use ash::vk;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Vulkan error: {0:?}")]
    Vulkan(#[from] vk::Result),
}

pub type Result<T> = std::result::Result<T, Error>;
