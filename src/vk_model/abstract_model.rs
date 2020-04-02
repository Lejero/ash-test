#![allow(dead_code)]
#![allow(unused_imports)]

use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

//mod utility;
use crate::utility;
use utility::{constants::*, debug::*, share};

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use memoffset::offset_of;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use std::ffi::CString;
use std::ptr;

use super::MeshSize;
use crate::vk_assist;
use vk_assist::structures::SimpleVertex;
use vk_assist::types::{
    vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*,
};

//TODO: use type for vertices. define trait for vertices requiring get_binding_description, get_attribute_descriptions, and maybe a sizeof shorthand. Is there a trait to ensure the type can be consistently sized easily already?
pub struct AbstractModel {
    pub vertices: Vec<SimpleVertex>,
    pub indices: Vec<u32>,

    pub vertices_size: vk::DeviceSize,
    pub indices_size: vk::DeviceSize,

    pub model_matrix: Mat4,
}

impl AbstractModel {
    pub fn new(vertices: Vec<SimpleVertex>, indices: Vec<u32>) -> AbstractModel {
        let id_mat = Mat4::identity();

        AbstractModel {
            vertices_size: vertices.len() as u64 * std::mem::size_of::<SimpleVertex>() as u64,
            indices_size: indices.len() as u64 * std::mem::size_of::<u32>() as u64,

            vertices,
            indices,

            model_matrix: id_mat,
        }
    }
}

impl MeshSize for AbstractModel {
    fn vertices_size(&self) -> vk::DeviceSize {
        self.vertices.len() as u64 * std::mem::size_of::<SimpleVertex>() as u64
    }
    fn indices_size(&self) -> vk::DeviceSize {
        self.indices.len() as u64 * std::mem::size_of::<u32>() as u64
    }
}
