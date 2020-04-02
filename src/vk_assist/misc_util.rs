#![allow(dead_code)]
//#![allow(unused_imports)]

//mod utility;
use std::f32::consts::PI;
use std::sync::Arc;

use ash::version::InstanceV1_0;
use ash::vk;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

use std::ffi::CString;
use std::path::Path;
use std::ptr;

use crate::vk_assist::types::buffer as bfr;
use crate::vk_assist::types::command as cmd;
use crate::vk_assist::types::image as img;
use crate::vk_assist::types::{buffer, command, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};
use image::GenericImageView;

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
