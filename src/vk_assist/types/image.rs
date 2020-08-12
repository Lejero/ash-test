#![allow(dead_code)]
//#![allow(unused_imports)]

//mod utility;
use crate::vk_assist;
use std::cmp::max;
use std::sync::Arc;

use ash::version::DeviceV1_0;
use ash::version::InstanceV1_0;
use ash::vk;
use memoffset::offset_of;
use nalgebra_glm::{Mat4, Vec2, Vec3, Vec4};

use std::f32::consts::PI;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

use image::GenericImageView;

use vk_assist::structures::{get_rect_as_basic, get_rectangle, SimpleVertex};
use vk_assist::types::buffer as bfr;
use vk_assist::types::command as cmd;
use vk_assist::types::command::*;
use vk_assist::types::{vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};

use super::command::{begin_single_time_command, end_single_time_command, find_memory_type};

pub struct Image {
    device: Arc<VulkanDevice>,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
    size: ImageSize,
    mip_levels: u32,
    layer_count: u32,
}

impl Image {
    // pub fn get_image(&self) -> vk::Image {
    //     self.image
    // }
    // pub fn get_memory(&self) -> vk::DeviceMemory {
    //     self.memory
    // }
    // pub fn get_view(&self) -> vk::ImageView {
    //     self.view
    // }
    pub fn set_view(&mut self, format: vk::Format, aspect_flags: vk::ImageAspectFlags, mip_levels: u32) {
        self.view = create_image_view(self.device.clone(), self.image, format, aspect_flags, mip_levels)
    }
    pub fn get_size(&self) -> &ImageSize {
        &self.size
    }
    pub fn get_mip_levels(&self) -> u32 {
        self.mip_levels
    }
    pub fn get_layer_count(&self) -> u32 {
        self.layer_count
    }

    pub fn vk_destroy(&mut self) {
        unsafe {
            self.device.logical_device.destroy_image(self.image, None);
            self.device.logical_device.destroy_image_view(self.view, None);
            self.device.logical_device.free_memory(self.memory, None);
        }
    }

    pub fn new(
        device: Arc<VulkanDevice>,
        width: u32,
        height: u32,
        mip_levels: u32,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        required_mem_properties: vk::MemoryPropertyFlags,
    ) -> Image {
        let (image, memory) = create_img(
            device.clone(),
            width,
            height,
            mip_levels,
            num_samples,
            format,
            tiling,
            usage,
            required_mem_properties,
        );
        let view = create_image_view(device.clone(), image, format, vk::ImageAspectFlags::COLOR, mip_levels);

        Image {
            device,
            image,
            memory,
            view,
            size: ImageSize { width, height },
            mip_levels,
            layer_count: 1,
        }
    }
    pub fn new_depth_map(
        device: Arc<VulkanDevice>,
        width: u32,
        height: u32,
        mip_levels: u32,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        required_mem_properties: vk::MemoryPropertyFlags,
    ) -> Image {
        let (image, memory) = create_img(
            device.clone(),
            width,
            height,
            mip_levels,
            num_samples,
            format,
            tiling,
            usage,
            required_mem_properties,
        );

        let view = create_image_view(device.clone(), image, format, vk::ImageAspectFlags::DEPTH, mip_levels);

        Image {
            device,
            image,
            memory,
            view,
            size: ImageSize { width, height },
            mip_levels,
            layer_count: 1,
        }
    }

    pub fn create_sampler(&self) -> vk::Sampler {
        let sampler_create_info = vk::SamplerCreateInfo {
            s_type: vk::StructureType::SAMPLER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SamplerCreateFlags::empty(),
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: 16.0,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            min_lod: 0.0,
            max_lod: self.mip_levels as f32,
            mip_lod_bias: 0.0,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
        };
        unsafe {
            self.device
                .logical_device
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        self.vk_destroy();
    }
}

pub fn create_img(
    device: Arc<VulkanDevice>,
    width: u32,
    height: u32,
    mip_levels: u32,
    num_samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    required_memory_properties: vk::MemoryPropertyFlags,
) -> (vk::Image, vk::DeviceMemory) {
    let image_create_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ImageCreateFlags::empty(),
        image_type: vk::ImageType::TYPE_2D,
        format,
        mip_levels,
        array_layers: 1,
        samples: num_samples,
        tiling,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
        initial_layout: vk::ImageLayout::UNDEFINED,
        extent: vk::Extent3D { width, height, depth: 1 },
    };

    let texture_image = unsafe {
        device
            .logical_device
            .create_image(&image_create_info, None)
            .expect("Failed to create Texture Image!")
    };

    let image_memory_requirement = unsafe { device.logical_device.get_image_memory_requirements(texture_image) };
    let memory_allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: ptr::null(),
        allocation_size: image_memory_requirement.size,
        memory_type_index: find_memory_type(
            image_memory_requirement.memory_type_bits,
            required_memory_properties,
            &device.get_physical_device_memory_properties(),
        ),
    };

    let texture_image_memory = unsafe {
        device
            .logical_device
            .allocate_memory(&memory_allocate_info, None)
            .expect("Failed to allocate Texture Image memory!")
    };

    unsafe {
        device
            .logical_device
            .bind_image_memory(texture_image, texture_image_memory, 0)
            .expect("Failed to bind Image Memmory!");
    }

    (texture_image, texture_image_memory)
}

pub fn create_image_view(
    device: Arc<VulkanDevice>,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
    mip_levels: u32,
) -> vk::ImageView {
    let imageview_create_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ImageViewCreateFlags::empty(),
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
        image: image,
    };

    unsafe {
        device
            .logical_device
            .create_image_view(&imageview_create_info, None)
            .expect("Failed to create Image View!")
    }
}

pub fn create_image(
    device: Arc<VulkanDevice>,
    width: u32,
    height: u32,
    mip_levels: u32,
    num_samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    required_memory_properties: vk::MemoryPropertyFlags,
    device_memory_properties: &vk::PhysicalDeviceMemoryProperties,
) -> (vk::Image, vk::DeviceMemory) {
    let image_create_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ImageCreateFlags::empty(),
        image_type: vk::ImageType::TYPE_2D,
        format,
        mip_levels,
        array_layers: 1,
        samples: num_samples,
        tiling,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        queue_family_index_count: 0,
        p_queue_family_indices: ptr::null(),
        initial_layout: vk::ImageLayout::UNDEFINED,
        extent: vk::Extent3D { width, height, depth: 1 },
    };

    let texture_image = unsafe {
        device
            .logical_device
            .create_image(&image_create_info, None)
            .expect("Failed to create Texture Image!")
    };

    let image_memory_requirement = unsafe { device.logical_device.get_image_memory_requirements(texture_image) };
    let memory_allocate_info = vk::MemoryAllocateInfo {
        s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
        p_next: ptr::null(),
        allocation_size: image_memory_requirement.size,
        memory_type_index: find_memory_type(image_memory_requirement.memory_type_bits, required_memory_properties, device_memory_properties),
    };

    let texture_image_memory = unsafe {
        device
            .logical_device
            .allocate_memory(&memory_allocate_info, None)
            .expect("Failed to allocate Texture Image memory!")
    };

    unsafe {
        device
            .logical_device
            .bind_image_memory(texture_image, texture_image_memory, 0)
            .expect("Failed to bind Image Memmory!");
    }

    (texture_image, texture_image_memory)
}

pub fn transition_image_layout(
    device: Arc<VulkanDevice>,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    image: vk::Image,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    mip_levels: u32,
) {
    let command_buffer = begin_single_time_command(device.clone(), command_pool);

    let src_access_mask;
    let dst_access_mask;
    let source_stage;
    let destination_stage;

    if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
        src_access_mask = vk::AccessFlags::empty();
        dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        destination_stage = vk::PipelineStageFlags::TRANSFER;
    } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
        src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        dst_access_mask = vk::AccessFlags::SHADER_READ;
        source_stage = vk::PipelineStageFlags::TRANSFER;
        destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    } else if old_layout == vk::ImageLayout::UNDEFINED && new_layout == vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL {
        src_access_mask = vk::AccessFlags::empty();
        dst_access_mask = vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
        source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        destination_stage = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
    } else {
        panic!("Unsupported layout transition!")
    }

    let image_barriers = [vk::ImageMemoryBarrier {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        p_next: ptr::null(),
        src_access_mask,
        dst_access_mask,
        old_layout,
        new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
    }];

    unsafe {
        device.logical_device.cmd_pipeline_barrier(
            command_buffer,
            source_stage,
            destination_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_barriers,
        );
    }

    end_single_time_command(device.clone(), command_pool, submit_queue, command_buffer);
}

pub fn check_mipmap_support(instance: Arc<ash::Instance>, physcial_device: vk::PhysicalDevice, image_format: vk::Format) {
    let format_properties = unsafe { instance.get_physical_device_format_properties(physcial_device, image_format) };

    let is_sample_image_filter_linear_support = format_properties
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR);

    if is_sample_image_filter_linear_support == false {
        panic!("Texture Image format does not support linear blitting!")
    }
}

pub fn generate_mipmaps(device: Arc<VulkanDevice>, command_pool: vk::CommandPool, submit_queue: vk::Queue, image: &Image) {
    let command_buffer = begin_single_time_command(device.clone(), command_pool);

    let mut image_barrier = vk::ImageMemoryBarrier {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        p_next: ptr::null(),
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::empty(),
        old_layout: vk::ImageLayout::UNDEFINED,
        new_layout: vk::ImageLayout::UNDEFINED,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: image.image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
    };

    let mut mip_width = image.size.width as i32;
    let mut mip_height = image.size.height as i32;

    for i in 1..image.mip_levels {
        image_barrier.subresource_range.base_mip_level = i - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

        unsafe {
            device.logical_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }

        let blits = [vk::ImageBlit {
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: i - 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: mip_width,
                    y: mip_height,
                    z: 1,
                },
            ],
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: i,
                base_array_layer: 0,
                layer_count: 1,
            },
            dst_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: max(mip_width / 2, 1),
                    y: max(mip_height / 2, 1),
                    z: 1,
                },
            ],
        }];

        unsafe {
            device.logical_device.cmd_blit_image(
                command_buffer,
                image.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &blits,
                vk::Filter::LINEAR,
            );
        }

        image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.logical_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier.clone()],
            );
        }

        mip_width = max(mip_width / 2, 1);
        mip_height = max(mip_height / 2, 1);
    }

    image_barrier.subresource_range.base_mip_level = image.mip_levels - 1;
    image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    unsafe {
        device.logical_device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[image_barrier.clone()],
        );
    }

    end_single_time_command(device.clone(), command_pool, submit_queue, command_buffer);
}

pub fn create_texture_image(device: Arc<VulkanDevice>, command_pool: vk::CommandPool, image_path: &Path) -> Image {
    let mut image_object = image::open(image_path).unwrap(); // this function is slow in debug mode.
    image_object = image_object.flipv();
    let (image_width, image_height) = (image_object.width(), image_object.height());
    let image_data = match &image_object {
        image::DynamicImage::ImageBgr8(_) | image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba().into_raw(),
        image::DynamicImage::ImageBgra8(_) | image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageRgba8(_) => image_object.raw_pixels(),
    };

    let image_size = (::std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
    let mip_levels = ((::std::cmp::max(image_width, image_height) as f32).log2().floor() as u32) + 1;

    if image_size <= 0 {
        panic!("Failed to load texture image!")
    }

    //Get CPU visible buffer for staging
    let staging_buffer = bfr::create_buffer(
        device.clone(),
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        &device.get_physical_device_memory_properties(),
    );

    //map enough memory to hold the image in the staging buffer. Copy the image into the buffer.
    unsafe {
        let data_ptr = device
            .logical_device
            .map_memory(staging_buffer.memory, 0, image_size, vk::MemoryMapFlags::empty())
            .expect("Failed to Map Memory") as *mut u8;

        data_ptr.copy_from_nonoverlapping(image_data.as_ptr(), image_data.len());

        device.logical_device.unmap_memory(staging_buffer.memory);
    }

    let texture = Image::new(
        device.clone(),
        image_width,
        image_height,
        mip_levels,
        vk::SampleCountFlags::TYPE_1,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    transition_image_layout(
        device.clone(),
        command_pool,
        device.graphics_queue,
        texture.image,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        mip_levels,
    );

    copy_buffer_to_image(
        device.clone(),
        command_pool,
        device.graphics_queue,
        staging_buffer.buffer,
        texture.image,
        image_width,
        image_height,
    );

    generate_mipmaps(device.clone(), command_pool, device.graphics_queue, &texture);

    texture
}
