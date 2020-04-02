#![allow(dead_code)]

use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use std::sync::Arc;

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

use crate::vk_assist;
use vk_assist::structures::Vertex;
use vk_assist::types::buffer as bfr;
use vk_assist::types::command as cmd;
use vk_assist::types::image as img;
use vk_assist::types::{vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};

use super::MeshSize;

pub struct GFXModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub diffuse_tex: img::Image,

    pub model_matrix: Mat4,
}

impl GFXModel {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, diffuse_tex: img::Image) -> GFXModel {
        let id_mat = Mat4::identity();

        GFXModel {
            vertices,
            indices,
            diffuse_tex,

            model_matrix: id_mat,
        }
    }

    pub fn vk_destroy(&mut self) {
        self.diffuse_tex.vk_destroy();
    }
}

impl MeshSize for GFXModel {
    fn vertices_size(&self) -> vk::DeviceSize {
        self.vertices.len() as u64 * std::mem::size_of::<Vertex>() as u64
    }
    fn indices_size(&self) -> vk::DeviceSize {
        self.indices.len() as u64 * std::mem::size_of::<u32>() as u64
    }
}
