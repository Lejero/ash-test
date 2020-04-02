#![allow(dead_code)]

//mod utility;
use crate::pipelines;
use crate::utility;
use crate::vk_assist;
use crate::vk_model;
use std::f32::consts::PI;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use memoffset::offset_of;
use nalgebra_glm::{identity, look_at, perspective};
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use utility::{constants::*, debug::*, share};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use std::ffi::CString;
use std::path::Path;
use std::ptr;

use image::GenericImageView;

use vk_assist::misc_util as misc;
use vk_assist::structures::{get_rect_as_intermediate, UniformBufferObject, Vertex};
use vk_assist::types::buffer as bfr;
use vk_assist::types::command as cmd;
use vk_assist::types::image as img;
use vk_assist::types::{buffer, command, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};
use vk_model::advanced_model::*;
use vk_model::MeshSize;

pub fn load_model(model_path: &Path, diffuse_tex: img::Image) -> GFXModel {
    let model_obj = tobj::load_obj(model_path).expect("Failed to load model object!");

    let mut vertices = vec![];
    let mut indices = vec![];

    let (models, _) = model_obj;
    for m in models.iter() {
        let mesh = &m.mesh;

        if mesh.texcoords.len() == 0 {
            panic!("Missing texture coordinate for the model.")
        }

        let total_vertices_count = mesh.positions.len() / 3;
        for i in 0..total_vertices_count {
            let vertex = Vertex {
                pos: Vec3::new(mesh.positions[i * 3], mesh.positions[i * 3 + 1], mesh.positions[i * 3 + 2]),
                color: Vec3::new(1.0, 1.0, 1.0),
                uv: Vec2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]),
            };
            vertices.push(vertex);
        }

        indices = mesh.indices.clone();
    }

    GFXModel::new(vertices, indices, diffuse_tex)
}
