#![allow(dead_code)]
//#![allow(unused_imports)]

use crate::app;
use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use std::sync::Arc;

pub struct VulkanSurface {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,

    pub screen_width: u32,
    pub screen_height: u32,
}

impl VulkanSurface {
    pub fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: Arc<winit::window::Window>,
        screen_width: u32,
        screen_height: u32,
    ) -> VulkanSurface {
        let surface = unsafe { app::platforms::create_surface(entry, instance, window.as_ref()).expect("Failed to create surface.") };
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        VulkanSurface {
            surface_loader,
            surface,
            screen_width,
            screen_height,
        }
    }
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        // unsafe {
        //     //self.surface_loader.destroy_surface(self.surface, None);
        // }
    }
}
