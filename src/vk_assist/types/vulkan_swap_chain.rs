use crate::utility::{constants::*, debug, share, structures, tools};
use crate::vk_assist::types::{
    queue_family, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface,
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

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

pub struct VulkanSwapChain {
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    // pub swapchain_imageviews: Vec<vk::ImageView>,
    // pub swapchain_framebuffers: Vec<vk::Framebuffer>,
}

impl VulkanSwapChain {
    pub fn new(
        instance: &ash::Instance,
        device: &VulkanDevice,
        surface: &VulkanSurface,
        image_size: &ImageSize,
    ) -> VulkanSwapChain {
        create_swapchain(instance, device, surface, image_size)
    }
}

pub fn create_swapchain(
    instance: &ash::Instance,
    device: &VulkanDevice,
    surface: &VulkanSurface,
    image_size: &ImageSize,
) -> VulkanSwapChain {
    let swapchain_support = query_swapchain_support(device.physical_device, surface);

    let surface_format = choose_swapchain_format(&swapchain_support.formats);
    let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
    let extent = choose_swapchain_extent(&swapchain_support.capabilities, image_size);

    let image_count = swapchain_support.capabilities.min_image_count + 1;
    let image_count = if swapchain_support.capabilities.max_image_count > 0 {
        image_count.min(swapchain_support.capabilities.max_image_count)
    } else {
        image_count
    };

    //Try to pick Concurrent SharingMode if possible, that is, if the graphics and present family are the same.
    let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
        if device.queue_family.graphics_family != device.queue_family.present_family {
            (
                vk::SharingMode::CONCURRENT,
                2,
                vec![
                    device.queue_family.graphics_family.unwrap(),
                    device.queue_family.present_family.unwrap(),
                ],
            )
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, vec![])
        };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: vk::SwapchainCreateFlagsKHR::empty(),
        surface: surface.surface,
        min_image_count: image_count,
        image_color_space: surface_format.color_space,
        image_format: surface_format.format,
        image_extent: extent,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode,
        p_queue_family_indices: queue_family_indices.as_ptr(),
        queue_family_index_count,
        pre_transform: swapchain_support.capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        image_array_layers: 1,
    };

    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, &device.logical_device);
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create Swapchain!")
    };

    let swapchain_images = unsafe {
        swapchain_loader
            .get_swapchain_images(swapchain)
            .expect("Failed to get Swapchain Images.")
    };

    VulkanSwapChain {
        swapchain_loader,
        swapchain,
        swapchain_images,
        swapchain_format: surface_format.format,
        swapchain_extent: extent,
    }
}

pub fn query_swapchain_support(
    physical_device: vk::PhysicalDevice,
    surface: &VulkanSurface,
) -> SwapChainSupportDetail {
    unsafe {
        let capabilities = surface
            .surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface.surface)
            .expect("Failed to query for surface capabilities.");
        let formats = surface
            .surface_loader
            .get_physical_device_surface_formats(physical_device, surface.surface)
            .expect("Failed to query for surface formats.");
        let present_modes = surface
            .surface_loader
            .get_physical_device_surface_present_modes(physical_device, surface.surface)
            .expect("Failed to query for surface present mode.");

        SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        }
    }
}

pub fn choose_swapchain_format(
    available_formats: &Vec<vk::SurfaceFormatKHR>,
) -> vk::SurfaceFormatKHR {
    for available_format in available_formats {
        if available_format.format == vk::Format::B8G8R8A8_SRGB
            && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return available_format.clone();
        }
    }

    return available_formats.first().unwrap().clone();
}

pub fn choose_swapchain_present_mode(
    available_present_modes: &Vec<vk::PresentModeKHR>,
) -> vk::PresentModeKHR {
    for &available_present_mode in available_present_modes.iter() {
        if available_present_mode == vk::PresentModeKHR::MAILBOX {
            return available_present_mode;
        }
    }

    vk::PresentModeKHR::FIFO
}

pub fn choose_swapchain_extent(
    capabilities: &vk::SurfaceCapabilitiesKHR,
    image_size: &ImageSize,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::max_value() {
        capabilities.current_extent
    } else {
        use num::clamp;

        vk::Extent2D {
            width: clamp(
                image_size.width,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: clamp(
                image_size.height,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}
