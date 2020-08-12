//#![allow(dead_code)]
//#![allow(unused_imports)]

use crate::vk_assist;
use crate::vk_model;
use std::ops::DerefMut;
use std::sync::RwLock;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use memoffset::offset_of;
use nalgebra_glm::{identity, look_at, perspective};
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

use std::f32::consts::PI;
use std::ffi::CString;
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use image::GenericImageView;

use vk_assist::misc_util as misc;
use vk_assist::model_loader as mdl;
use vk_assist::structures::{get_rect_as_intermediate, UniformBufferObject, Vertex};
use vk_assist::types::{buffer as bfr, command as cmd, image as img};
use vk_assist::types::{buffer, command, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};
use vk_model::advanced_model::*;
use vk_model::MeshSize;

const TEXTURE_PATH: &'static str = "assets/fighterdiffuse.bmp";
const MODEL_PATH: &'static str = "assets/fighter.obj";

pub struct Assets {
    pub fighter: Arc<GFXModel>,
}

impl Assets {
    pub fn init(device: Arc<VulkanDevice>, command_pool: vk::CommandPool) -> Assets {
        let texture = img::create_texture_image(device.clone(), command_pool, &Path::new(TEXTURE_PATH));
        let model = mdl::load_model(Path::new(MODEL_PATH), texture);

        Assets { fighter: Arc::new(model) }
    }

    // pub fn vk_destroy(&mut self) {
    //     self.fighter.diffuse_tex.vk_destroy();
    // }
}
