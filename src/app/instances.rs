//#![allow(dead_code)]

use crate::app::assets::*;
use crate::vk_assist;
use crate::vk_model;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};
use std::f32::consts::PI;
use std::sync::Arc;
use vk_model::advanced_model::*;

const TEXTURE_PATH: &'static str = "assets/fighterdiffuse.bmp";
const MODEL_PATH: &'static str = "assets/fighter.obj";

pub struct GInstance {
    pub asset: Arc<GFXModel>,

    pub model_matrix: Mat4,
}

impl GInstance {
    pub fn new(asset: Arc<GFXModel>, model_matrix: Mat4) -> GInstance {
        GInstance { asset, model_matrix }
    }
}

pub struct Instances {
    pub g_instances: Vec<GInstance>,
}

impl Instances {
    // pub fn new() -> Instances {

    // }
}
