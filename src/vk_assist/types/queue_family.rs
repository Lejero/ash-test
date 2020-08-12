#![allow(dead_code)]
#![allow(unused_imports)]

extern crate ash;

use crate::vk_assist::types::vulkan_surface::VulkanSurface;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use std::sync::Arc;

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub fn find_queue_family(instance: &ash::Instance, physical_device: vk::PhysicalDevice, surface_stuff: &VulkanSurface) -> QueueFamilyIndices {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut queue_family_indices = QueueFamilyIndices::new();

    let mut index = 0;
    for queue_family in queue_families.iter() {
        if queue_family.queue_count > 0 && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            queue_family_indices.graphics_family = Some(index);
        }

        let is_present_support = unsafe {
            surface_stuff
                .surface_loader
                .get_physical_device_surface_support(physical_device, index as u32, surface_stuff.surface)
        };
        if queue_family.queue_count > 0 && is_present_support.expect("No Queue Present Support Result!") {
            queue_family_indices.present_family = Some(index);
        }

        if queue_family_indices.is_complete() {
            break;
        }

        index += 1;
    }

    queue_family_indices
}
