#![allow(dead_code)]
#![allow(unused_imports)]

use crate::utility::{constants::*, debug, share, tools};
use crate::vk_assist::types::{
    queue_family, vulkan_surface::VulkanSurface, vulkan_swap_chain,
    vulkan_swap_chain::SwapChainSupportDetail,
};

use std::ffi::CString;
use std::os::raw::c_char;
use std::os::raw::c_void;
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

pub struct DeviceExtensions {
    pub names: [&'static str; 1],
    //    pub raw_names: [*const i8; 1],
}

pub const SWAP_CHAIN_ONLY_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    names: ["VK_KHR_swapchain"],
};

pub struct VulkanDevice {
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: Arc<ash::Device>,

    pub queue_family: queue_family::QueueFamilyIndices,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
}

impl VulkanDevice {
    pub fn create_device(
        instance: &ash::Instance,
        surface: &VulkanSurface,
        required_extensions: DeviceExtensions,
    ) -> VulkanDevice {
        let physical_device = pick_physical_device(&instance, &surface, &required_extensions);
        let (logical_device, queue_family) = create_logical_device(
            &instance,
            physical_device,
            &VALIDATION,
            &required_extensions,
            &surface,
        );
        let graphics_queue =
            unsafe { logical_device.get_device_queue(queue_family.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(queue_family.present_family.unwrap(), 0) };

        VulkanDevice {
            physical_device,
            logical_device: Arc::new(logical_device),
            queue_family,
            graphics_queue,
            present_queue,
        }
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.destroy_device(None);
        }
    }
}

pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_stuff: &VulkanSurface,
    required_device_extensions: &DeviceExtensions,
) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate Physical Devices!")
    };

    let result = physical_devices.iter().find(|physical_device| {
        let is_suitable = is_physical_device_suitable(
            instance,
            **physical_device,
            surface_stuff,
            required_device_extensions,
        );

        // if is_suitable {
        //     let device_properties = instance.get_physical_device_properties(**physical_device);
        //     let device_name = super::tools::vk_to_string(&device_properties.device_name);
        //     println!("Using GPU: {}", device_name);
        // }

        is_suitable
    });

    match result {
        Some(p_physical_device) => *p_physical_device,
        None => panic!("Failed to find a suitable GPU!"),
    }
}

pub fn is_physical_device_suitable(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface: &VulkanSurface,
    required_device_extensions: &DeviceExtensions,
) -> bool {
    let device_features = unsafe { instance.get_physical_device_features(physical_device) };

    let indices = queue_family::find_queue_family(instance, physical_device, surface);

    let is_queue_family_supported = indices.is_complete();
    let is_device_extension_supported =
        check_device_extension_support(instance, physical_device, required_device_extensions);
    let is_swapchain_supported = if is_device_extension_supported {
        let swapchain_support =
            vulkan_swap_chain::query_swapchain_support(physical_device, surface);
        !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
    } else {
        false
    };
    let is_support_sampler_anisotropy = device_features.sampler_anisotropy == 1;

    return is_queue_family_supported
        && is_device_extension_supported
        && is_swapchain_supported
        && is_support_sampler_anisotropy;
}

pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    validation: &debug::ValidationInfo,
    device_extensions: &DeviceExtensions,
    surface_stuff: &VulkanSurface,
) -> (ash::Device, queue_family::QueueFamilyIndices) {
    let indices = queue_family::find_queue_family(instance, physical_device, surface_stuff);

    use std::collections::HashSet;
    let mut unique_queue_families = HashSet::new();
    unique_queue_families.insert(indices.graphics_family.unwrap());
    unique_queue_families.insert(indices.present_family.unwrap());

    let queue_priorities = [1.0_f32];
    let mut queue_create_infos = vec![];
    for &queue_family in unique_queue_families.iter() {
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceQueueCreateFlags::empty(),
            queue_family_index: queue_family,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        queue_create_infos.push(queue_create_info);
    }

    let physical_device_features = vk::PhysicalDeviceFeatures {
        sampler_anisotropy: vk::TRUE, // enable anisotropy device feature from Chapter-24.
        ..Default::default()
    };

    let requred_validation_layer_raw_names: Vec<CString> = validation
        .required_validation_layers
        .iter()
        .map(|layer_name| CString::new(*layer_name).unwrap())
        .collect();
    let enable_layer_names: Vec<*const c_char> = requred_validation_layer_raw_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let enable_extension_raw_names: Vec<CString> = device_extensions
        .names
        .iter()
        .map(|name| CString::new(*name).unwrap())
        .collect();
    // let enable_extension_names = device_extensions.names.get_extensions_raw_names();
    let enable_extension_names: Vec<*const c_char> = enable_extension_raw_names
        .iter()
        .map(|name| name.as_ptr())
        .collect();

    let device_create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: queue_create_infos.len() as u32,
        p_queue_create_infos: queue_create_infos.as_ptr(),
        enabled_layer_count: if validation.is_enable {
            enable_layer_names.len()
        } else {
            0
        } as u32,
        pp_enabled_layer_names: if validation.is_enable {
            enable_layer_names.as_ptr()
        } else {
            ptr::null()
        },
        enabled_extension_count: enable_extension_names.len() as u32,
        pp_enabled_extension_names: enable_extension_names.as_ptr(),
        p_enabled_features: &physical_device_features,
    };

    let device: ash::Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .expect("Failed to create logical Device!")
    };

    (device, indices)
}

pub fn check_device_extension_support(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device_extensions: &DeviceExtensions,
) -> bool {
    let available_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .expect("Failed to get device extension properties.")
    };

    let mut available_extension_names = vec![];

    for extension in available_extensions.iter() {
        let extension_name = tools::vk_to_string(&extension.extension_name);

        available_extension_names.push(extension_name);
    }

    use std::collections::HashSet;
    let mut required_extensions = HashSet::new();
    for extension in device_extensions.names.iter() {
        required_extensions.insert(extension.to_string());
    }

    for extension_name in available_extension_names.iter() {
        required_extensions.remove(extension_name);
    }

    return required_extensions.is_empty();
}
