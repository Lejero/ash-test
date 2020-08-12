#![allow(dead_code)]
//#![allow(unused_imports)]

//mod utility;
use crate::app;
use crate::vk_assist;
use crate::vk_assist::types::queue_family::QueueFamilyIndices;
use std::f32::consts::PI;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

use vk_assist::structures::SyncObjects;

use std::ffi::c_void;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

use crate::vk_assist::types::buffer as bfr;
use crate::vk_assist::types::command as cmd;
use crate::vk_assist::types::image as img;
use crate::vk_assist::types::{buffer, command, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};
use image::GenericImageView;

use app::debug::ValidationInfo;
use ash::vk::make_version;
pub const APPLICATION_VERSION: u32 = make_version(1, 0, 0);
pub const ENGINE_VERSION: u32 = make_version(1, 0, 0);
pub const API_VERSION: u32 = make_version(1, 0, 92);
pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub fn create_shader_module(device: &ash::Device, code: Vec<u8>) -> vk::ShaderModule {
    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
    };

    unsafe {
        device
            .create_shader_module(&shader_module_create_info, None)
            .expect("Failed to create Shader Module!")
    }
}

pub fn find_depth_format(instance: Arc<ash::Instance>, physical_device: vk::PhysicalDevice) -> vk::Format {
    find_supported_format(
        instance.clone(),
        physical_device,
        &[vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

pub fn find_supported_format(
    instance: Arc<ash::Instance>,
    physical_device: vk::PhysicalDevice,
    candidate_formats: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for &format in candidate_formats.iter() {
        let format_properties = unsafe { instance.get_physical_device_format_properties(physical_device, format) };
        if tiling == vk::ImageTiling::LINEAR && format_properties.linear_tiling_features.contains(features) {
            return format.clone();
        } else if tiling == vk::ImageTiling::OPTIMAL && format_properties.optimal_tiling_features.contains(features) {
            return format.clone();
        }
    }

    panic!("Failed to find supported format!")
}

pub fn has_stencil_component(format: vk::Format) -> bool {
    format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
}

pub fn get_max_usable_sample_count(instance: Arc<ash::Instance>, physical_device: vk::PhysicalDevice) -> vk::SampleCountFlags {
    let physical_device_properties = unsafe { instance.get_physical_device_properties(physical_device) };

    let count = std::cmp::min(
        physical_device_properties.limits.framebuffer_color_sample_counts,
        physical_device_properties.limits.framebuffer_depth_sample_counts,
    );

    if count.contains(vk::SampleCountFlags::TYPE_64) {
        return vk::SampleCountFlags::TYPE_64;
    }
    if count.contains(vk::SampleCountFlags::TYPE_32) {
        return vk::SampleCountFlags::TYPE_32;
    }
    if count.contains(vk::SampleCountFlags::TYPE_16) {
        return vk::SampleCountFlags::TYPE_16;
    }
    if count.contains(vk::SampleCountFlags::TYPE_8) {
        return vk::SampleCountFlags::TYPE_8;
    }
    if count.contains(vk::SampleCountFlags::TYPE_4) {
        return vk::SampleCountFlags::TYPE_4;
    }
    if count.contains(vk::SampleCountFlags::TYPE_2) {
        return vk::SampleCountFlags::TYPE_2;
    }

    vk::SampleCountFlags::TYPE_1
}

pub fn create_color_resources(
    device: Arc<VulkanDevice>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
) -> img::Image {
    let color_format = swapchain_format;

    let color_image = img::Image::new(
        device.clone(),
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        msaa_samples,
        color_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    color_image
}

pub fn create_depth_resources(
    instance: Arc<ash::Instance>,
    device: Arc<VulkanDevice>,
    physical_device: vk::PhysicalDevice,
    _command_pool: vk::CommandPool,
    _submit_queue: vk::Queue,
    swapchain_extent: vk::Extent2D,
    msaa_samples: vk::SampleCountFlags,
) -> img::Image {
    let depth_format = find_depth_format(instance.clone(), physical_device);
    let depth_image = img::Image::new_depth_map(
        device.clone(),
        swapchain_extent.width,
        swapchain_extent.height,
        1,
        msaa_samples,
        depth_format,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    depth_image
}

pub fn reallocate_command_buffer(device: Arc<VulkanDevice>, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    //Allocate
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: 1,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffer_vec = unsafe {
        device
            .logical_device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    };

    command_buffer_vec[0]
}

pub fn create_instance(entry: &ash::Entry, window_title: &str, is_enable_debug: bool, required_validation_layers: &Vec<&str>) -> ash::Instance {
    if is_enable_debug && app::debug::check_validation_layer_support(entry, required_validation_layers) == false {
        panic!("Validation layers requested, but not available!");
    }

    let app_name = CString::new(window_title).unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfo {
        p_application_name: app_name.as_ptr(),
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        application_version: APPLICATION_VERSION,
        p_engine_name: engine_name.as_ptr(),
        engine_version: ENGINE_VERSION,
        api_version: API_VERSION,
    };

    // This create info used to debug issues in vk::createInstance and vk::destroyInstance.
    let debug_utils_create_info = app::debug::populate_debug_messenger_create_info();

    // VK_EXT debug report has been requested here.
    let extension_names = app::platforms::required_extension_names();

    let requred_validation_layer_raw_names: Vec<CString> = required_validation_layers.iter().map(|layer_name| CString::new(*layer_name).unwrap()).collect();
    let layer_names: Vec<*const i8> = requred_validation_layer_raw_names.iter().map(|layer_name| layer_name.as_ptr()).collect();

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: if VALIDATION.is_enable {
            &debug_utils_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void
        } else {
            ptr::null()
        },
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &app_info,
        pp_enabled_layer_names: if is_enable_debug { layer_names.as_ptr() } else { ptr::null() },
        enabled_layer_count: if is_enable_debug { layer_names.len() } else { 0 } as u32,
        pp_enabled_extension_names: extension_names.as_ptr(),
        enabled_extension_count: extension_names.len() as u32,
    };

    let instance: ash::Instance = unsafe { entry.create_instance(&create_info, None).expect("Failed to create instance!") };

    instance
}

pub fn create_image_views(device: Arc<VulkanDevice>, surface_format: vk::Format, images: &Vec<vk::Image>) -> Vec<vk::ImageView> {
    let swapchain_imageviews: Vec<vk::ImageView> = images
        .iter()
        .map(|&image| create_image_view(device.clone(), image, surface_format, vk::ImageAspectFlags::COLOR, 1))
        .collect();

    swapchain_imageviews
}

pub fn create_image_view(
    device: Arc<VulkanDevice>,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let imageview_create_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ImageViewCreateFlags::empty(),
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
        image,
    };

    unsafe {
        device
            .logical_device
            .create_image_view(&imageview_create_info, None)
            .expect("Failed to create Image View!")
    }
}

pub fn create_command_pool(device: &ash::Device, queue_families: &QueueFamilyIndices) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::CommandPoolCreateFlags::empty(),
        queue_family_index: queue_families.graphics_family.unwrap(),
    };

    unsafe {
        device
            .create_command_pool(&command_pool_create_info, None)
            .expect("Failed to create Command Pool!")
    }
}

pub fn create_sync_objects(device: &ash::Device, max_frame_in_flight: usize) -> SyncObjects {
    let mut sync_objects = SyncObjects {
        image_available_semaphores: vec![],
        render_finished_semaphores: vec![],
        inflight_fences: vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::SemaphoreCreateFlags::empty(),
    };

    let fence_create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FenceCreateFlags::SIGNALED,
    };

    for _ in 0..max_frame_in_flight {
        unsafe {
            let image_available_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let render_finished_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let inflight_fence = device.create_fence(&fence_create_info, None).expect("Failed to create Fence Object!");

            sync_objects.image_available_semaphores.push(image_available_semaphore);
            sync_objects.render_finished_semaphores.push(render_finished_semaphore);
            sync_objects.inflight_fences.push(inflight_fence);
        }
    }

    sync_objects
}
