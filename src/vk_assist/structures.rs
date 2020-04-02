#![allow(dead_code)]
#![allow(unused_imports)]

use crate::vk_model::basic_model::*;
use crate::vk_model::intermediate_model::*;
use std::sync::Arc;

use ash::vk;
use memoffset::offset_of;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

pub struct SyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub inflight_fences: Vec<vk::Fence>,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ViewProjUBO {
    pub view: Mat4,
    pub proj: Mat4,
}
#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct ModelUBO {
    pub model: Mat4,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct SimpleVertex {
    pub pos: Vec3,
    pub color: Vec3,
}
#[allow(dead_code)]
impl SimpleVertex {
    pub fn get_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: ::std::mem::size_of::<SimpleVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(SimpleVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(SimpleVertex, color) as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Vec3,
    pub uv: Vec2,
}
#[allow(dead_code)]
impl Vertex {
    pub fn get_binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: ::std::mem::size_of::<Self>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Self, color) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Self, uv) as u32,
            },
        ]
    }
}
#[allow(dead_code)]
pub fn get_rectangle(x_dim: f32, y_dim: f32) -> [SimpleVertex; 4] {
    let half_x = x_dim / 2.0;
    let half_y = y_dim / 2.0;
    [
        SimpleVertex {
            pos: Vec3::new(-half_x, -half_y, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
        },
        SimpleVertex {
            pos: Vec3::new(half_x, -half_y, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
        },
        SimpleVertex {
            pos: Vec3::new(half_x, half_y, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
        },
        SimpleVertex {
            pos: Vec3::new(-half_x, half_y, 0.0),
            color: Vec3::new(1.0, 1.0, 1.0),
        },
    ]
}
#[allow(dead_code)]
pub fn get_rect_as_basic(x_dim: f32, y_dim: f32) -> Arc<BasicModel> {
    let half_x = x_dim / 2.0;
    let half_y = y_dim / 2.0;

    let vertices: [SimpleVertex; 4] = [
        SimpleVertex {
            pos: Vec3::new(-half_x, -half_y, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
        },
        SimpleVertex {
            pos: Vec3::new(half_x, -half_y, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
        },
        SimpleVertex {
            pos: Vec3::new(half_x, half_y, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
        },
        SimpleVertex {
            pos: Vec3::new(-half_x, half_y, 0.0),
            color: Vec3::new(1.0, 1.0, 1.0),
        },
    ];

    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    Arc::new(BasicModel::new(vertices.to_vec(), indices.to_vec()))
}
#[allow(dead_code)]
pub fn get_rect_as_intermediate(x_dim: f32, y_dim: f32) -> Arc<IntermediateModel> {
    let half_x = x_dim / 2.0;
    let half_y = y_dim / 2.0;

    let vertices: [Vertex; 4] = [
        Vertex {
            pos: Vec3::new(-half_x, -half_y, 0.0),
            color: Vec3::new(1.0, 0.0, 0.0),
            uv: Vec2::new(1.0, 0.0),
        },
        Vertex {
            pos: Vec3::new(half_x, -half_y, 0.0),
            color: Vec3::new(0.0, 1.0, 0.0),
            uv: Vec2::new(0.0, 0.0),
        },
        Vertex {
            pos: Vec3::new(half_x, half_y, 0.0),
            color: Vec3::new(0.0, 0.0, 1.0),
            uv: Vec2::new(0.0, 1.0),
        },
        Vertex {
            pos: Vec3::new(-half_x, half_y, 0.0),
            color: Vec3::new(1.0, 1.0, 1.0),
            uv: Vec2::new(1.0, 1.0),
        },
    ];

    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    Arc::new(IntermediateModel::new(vertices.to_vec(), indices.to_vec()))
}
