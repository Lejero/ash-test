#![allow(dead_code)]
#![allow(unused_imports)]

//mod utility;
use crate::utility;
use crate::vk_assist;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use memoffset::offset_of;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use utility::{constants::*, debug::*, share};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use std::ffi::CString;
use std::ptr;

use vk_assist::structures::{get_rect_as_basic, get_rectangle, SimpleVertex};
use vk_assist::types::{vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};

use super::command::{begin_single_time_command, end_single_time_command, find_memory_type};

pub struct Buffer {
    device: Arc<ash::Device>,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    alignment: vk::DeviceSize,
    usage_flags: vk::BufferUsageFlags,
    mem_prop_flags: vk::MemoryPropertyFlags,
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

impl Buffer {
    pub fn get_device(&self) -> Arc<ash::Device> {
        self.device.clone()
    }
    pub fn get_size(&self) -> &vk::DeviceSize {
        &self.size
    }
    pub fn get_alighment(&self) -> &vk::DeviceSize {
        &self.alignment
    }
    pub fn get_usage_flags(&self) -> &vk::BufferUsageFlags {
        &self.usage_flags
    }
    pub fn get_mem_prop_flags(&self) -> &vk::MemoryPropertyFlags {
        &self.mem_prop_flags
    }

    pub fn vk_destroy(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
    // pub fn new(
    //     device: Arc<VulkanDevice>,
    //     size: vk::DeviceSize,
    //     alignment: vk::DeviceSize,
    // ) -> Buffer {
    // }
}

pub fn create_buffer(
    device: Arc<VulkanDevice>,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    required_memory_properties: vk::MemoryPropertyFlags,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> Buffer {
    let buffer_create_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };

    let buffer = unsafe {
        device
            .logical_device
            .create_buffer(&buffer_create_info, None)
            .expect("Failed to create Vertex Buffer")
    };

    let mem_requirements = unsafe { device.logical_device.get_buffer_memory_requirements(buffer) };
    let memory_type = find_memory_type(mem_requirements.memory_type_bits, required_memory_properties, device_memory_properties);

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
    };

    let buffer_memory = unsafe {
        device
            .logical_device
            .allocate_memory(&allocate_info, None)
            .expect("Failed to allocate vertex buffer memory!")
    };

    unsafe {
        device
            .logical_device
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .expect("Failed to bind Buffer");
    }

    Buffer {
        device: device.logical_device.clone(),
        buffer: buffer,
        memory: buffer_memory,
        size: size,
        alignment: 0,
        usage_flags: usage,
        mem_prop_flags: required_memory_properties,
    }
}

pub fn create_buffer_2(
    device: Arc<ash::Device>,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    required_memory_properties: vk::MemoryPropertyFlags,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Buffer, vk::DeviceMemory) {
    let buffer_create_info = vk::BufferCreateInfo {
        s_type: vk::StructureType::BUFFER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::BufferCreateFlags::empty(),
        size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
    };

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None).expect("Failed to create Vertex Buffer") };

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memory_type = find_memory_type(mem_requirements.memory_type_bits, required_memory_properties, device_memory_properties);

    let allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: ptr::null(),
        allocation_size: mem_requirements.size,
        memory_type_index: memory_type,
    };

    let buffer_memory = unsafe { device.allocate_memory(&allocate_info, None).expect("Failed to allocate vertex buffer memory!") };

    unsafe {
        device.bind_buffer_memory(buffer, buffer_memory, 0).expect("Failed to bind Buffer");
    }

    (buffer, buffer_memory)
}

pub fn copy_buffer(
    device: Arc<VulkanDevice>,
    submit_queue: vk::Queue,
    command_pool: vk::CommandPool,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
) {
    let command_buffer = begin_single_time_command(device.clone(), command_pool);

    let copy_regions = [vk::BufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size,
    }];

    unsafe {
        device.logical_device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
    }

    end_single_time_command(device.clone(), command_pool, submit_queue, command_buffer);
}
