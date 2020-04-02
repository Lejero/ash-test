pub mod abstract_model;
pub mod advanced_model;
pub mod basic_model;
pub mod intermediate_model;

use ash::vk;

pub trait MeshSize {
    fn vertices_size(&self) -> vk::DeviceSize;
    fn indices_size(&self) -> vk::DeviceSize;
}
