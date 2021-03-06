#![allow(dead_code)]
#![allow(unused_imports)]

use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

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

use crate::vk_assist;
use vk_assist::structures::Vertex;
use vk_assist::types::{vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};

use super::MeshSize;

pub struct IntermediateModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub model_matrix: Mat4,
}

impl IntermediateModel {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> IntermediateModel {
        let id_mat = Mat4::identity();

        IntermediateModel {
            vertices,
            indices,

            model_matrix: id_mat,
        }
    }
}

impl MeshSize for IntermediateModel {
    fn vertices_size(&self) -> vk::DeviceSize {
        self.vertices.len() as u64 * std::mem::size_of::<Vertex>() as u64
    }
    fn indices_size(&self) -> vk::DeviceSize {
        self.indices.len() as u64 * std::mem::size_of::<u32>() as u64
    }
}
