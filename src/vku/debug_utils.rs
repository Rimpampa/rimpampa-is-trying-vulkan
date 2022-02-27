use std::ffi;

use ash::{extensions::ext, vk};

unsafe extern "system" fn vk_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    if (message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
        || message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::INFO)
        && message_type == vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
    {
        return vk::FALSE;
    }

    eprintln!("[ {:?} ] [ {:?} ]", message_severity, message_type);
    if !p_callback_data.is_null() {
        let msg = ffi::CStr::from_ptr((*p_callback_data).p_message);
        match msg.to_str() {
            Ok(str) => eprintln!("{}", str),
            Err(_) => eprintln!("{:?}", msg),
        }
    }
    vk::FALSE
}

pub fn create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity({
            use vk::DebugUtilsMessageSeverityFlagsEXT as flag;
            flag::WARNING | flag::INFO | flag::VERBOSE | flag::ERROR
        })
        .message_type({
            use vk::DebugUtilsMessageTypeFlagsEXT as flag;
            flag::VALIDATION | flag::PERFORMANCE | flag::GENERAL
        })
        .pfn_user_callback(Some(vk_debug_callback))
        .build()
}

pub struct DebugUtils<I: super::InstanceHolder> {
    instance: I,
    context: ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

impl<I: super::InstanceHolder> DebugUtils<I> {
    pub fn new(instance: I) -> super::Result<Self> {
        let context = ext::DebugUtils::new(instance.vk_entry(), instance.vk_instance());
        let messenger_create_info = create_info();
        let messenger =
            unsafe { context.create_debug_utils_messenger(&messenger_create_info, None)? };

        Ok(Self {
            instance,
            context,
            messenger,
        })
    }
}

impl<I: super::InstanceHolder> Drop for DebugUtils<I> {
    fn drop(&mut self) {
        unsafe {
            self.context
                .destroy_debug_utils_messenger(self.messenger, None);
        }
    }
}

derive_instance_holder!(DebugUtils<I> = instance: I);
derive_surface_holder!(DebugUtils<I> = instance: I);
