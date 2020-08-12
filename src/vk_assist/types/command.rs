#![allow(dead_code)]
//#![allow(unused_imports)]

//mod utility;
use crate::vk_assist;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use memoffset::offset_of;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use std::ffi::CString;
use std::ptr;

use vk_assist::structures::{get_rect_as_basic, get_rectangle, SimpleVertex};
use vk_assist::types::{vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};

pub fn begin_single_time_command(device: Arc<VulkanDevice>, command_pool: vk::CommandPool) -> vk::CommandBuffer {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: ptr::null(),
        command_buffer_count: 1,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
    };

    let command_buffer = unsafe {
        device
            .logical_device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    }[0];

    let command_buffer_begin_info = vk::CommandBufferBeginInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next: ptr::null(),
        p_inheritance_info: ptr::null(),
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
    };

    unsafe {
        device
            .logical_device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    command_buffer
}

pub fn end_single_time_command(device: Arc<VulkanDevice>, command_pool: vk::CommandPool, submit_queue: vk::Queue, command_buffer: vk::CommandBuffer) {
    unsafe {
        device
            .logical_device
            .end_command_buffer(command_buffer)
            .expect("Failed to record Command Buffer at Ending!");
    }

    let buffers_to_submit = [command_buffer];

    let sumbit_infos = [vk::SubmitInfo {
        s_type: vk::StructureType::SUBMIT_INFO,
        p_next: ptr::null(),
        wait_semaphore_count: 0,
        p_wait_semaphores: ptr::null(),
        p_wait_dst_stage_mask: ptr::null(),
        command_buffer_count: 1,
        p_command_buffers: buffers_to_submit.as_ptr(),
        signal_semaphore_count: 0,
        p_signal_semaphores: ptr::null(),
    }];

    unsafe {
        device
            .logical_device
            .queue_submit(submit_queue, &sumbit_infos, vk::Fence::null())
            .expect("Failed to Queue Submit!");
        device.logical_device.queue_wait_idle(submit_queue).expect("Failed to wait Queue idle!");
        device.logical_device.free_command_buffers(command_pool, &buffers_to_submit);
    }
}

pub fn find_memory_type(type_filter: u32, required_properties: vk::MemoryPropertyFlags, mem_properties: &vk::PhysicalDeviceMemoryProperties) -> u32 {
    for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
        if (type_filter & (1 << i)) > 0 && memory_type.property_flags.contains(required_properties) {
            return i as u32;
        }
    }

    panic!("Failed to find suitable memory type!")
}

pub fn copy_buffer_to_image(
    device: Arc<VulkanDevice>,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    buffer: vk::Buffer,
    image: vk::Image,
    width: u32,
    height: u32,
) {
    let command_buffer = begin_single_time_command(device.clone(), command_pool);

    let buffer_image_regions = [vk::BufferImageCopy {
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_extent: vk::Extent3D { width, height, depth: 1 },
        buffer_offset: 0,
        buffer_image_height: 0,
        buffer_row_length: 0,
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
    }];

    unsafe {
        device
            .logical_device
            .cmd_copy_buffer_to_image(command_buffer, buffer, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &buffer_image_regions);
    }

    end_single_time_command(device.clone(), command_pool, submit_queue, command_buffer);
}
