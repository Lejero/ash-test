#![allow(dead_code)]
#![allow(unused_imports)]

//mod utility;
use ash_test::pipelines;
use ash_test::utility;
use ash_test::vk_assist;
use ash_test::vk_model;

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

use std::f32::consts::PI;
use std::ffi::CString;
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use image::GenericImageView;

use vk_assist::misc_util as misc;
use vk_assist::model_loader as mdl;
use vk_assist::structures::{get_rect_as_intermediate, UniformBufferObject, Vertex};
use vk_assist::types::buffer as bfr;
use vk_assist::types::command as cmd;
use vk_assist::types::image as img;
use vk_assist::types::{buffer, command, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface, vulkan_swap_chain::*};
use vk_model::advanced_model::*;
use vk_model::MeshSize;

//mod pipelines;
use pipelines::current_pipeline_util as pipe;

// Constants
const WINDOW_TITLE: &'static str = "20.Index Buffer";
const TEXTURE_PATH: &'static str = "assets/fighterdiffuse.bmp";
const MODEL_PATH: &'static str = "assets/fighter.obj";

pub struct VulkanApp {
    window: winit::window::Window,

    // vulkan stuff
    _entry: ash::Entry,
    instance: Arc<ash::Instance>,

    vulkan_surface: VulkanSurface,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    device: Arc<VulkanDevice>,

    swap_chain: VulkanSwapChain,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    ubo_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    color_image: img::Image,
    depth_image: img::Image,

    msaa_samples: vk::SampleCountFlags,

    texture_sampler: vk::Sampler,

    model: GFXModel,
    vertex_buffer: bfr::Buffer,
    index_buffer: bfr::Buffer,

    current_ubo: UniformBufferObject,
    uniform_buffers: Vec<bfr::Buffer>,

    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,

    is_framebuffer_resized: bool,
}

impl VulkanApp {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> VulkanApp {
        println!("VulkanApp.new");
        //init window
        let window = utility::window::init_window(event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init instance
        let entry = ash::Entry::new().unwrap();
        let instance = Arc::new(share::create_instance(
            &entry,
            WINDOW_TITLE,
            VALIDATION.is_enable,
            &VALIDATION.required_validation_layers.to_vec(),
        ));

        //init surface
        let vulkan_surface = VulkanSurface::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);

        //init debug
        let (debug_utils_loader, debug_messenger) = setup_debug_utils(VALIDATION.is_enable, &entry, &instance);

        //init device
        let device = Arc::new(VulkanDevice::create_device(
            instance.clone(),
            &vulkan_surface,
            vulkan_device::SWAP_CHAIN_ONLY_EXTENSIONS,
        ));
        let inner_window_size = window.inner_size();
        let msaa_samples = misc::get_max_usable_sample_count(instance.clone(), device.physical_device);

        //init swap chain
        let swap_chain = VulkanSwapChain::new(
            &instance,
            &device,
            &vulkan_surface,
            &ImageSize {
                width: inner_window_size.width,
                height: inner_window_size.height,
            },
        );
        let swapchain_imageviews = share::v1::create_image_views(device.clone(), swap_chain.format, &swap_chain.images);

        //init pipeline
        let render_pass = pipe::create_render_pass(instance.clone(), device.clone(), swap_chain.format, msaa_samples);
        let ubo_layout = pipe::create_descriptor_set_layout(device.clone());
        let (graphics_pipeline, pipeline_layout) = pipe::create_graphics_pipeline(device.clone(), render_pass, swap_chain.extent, ubo_layout, msaa_samples);
        let command_pool = share::v1::create_command_pool(&device.logical_device, &device.queue_family);
        let color_image = misc::create_color_resources(device.clone(), swap_chain.format, swap_chain.extent, msaa_samples);
        let depth_image = misc::create_depth_resources(
            instance.clone(),
            device.clone(),
            device.physical_device,
            command_pool,
            device.graphics_queue,
            swap_chain.extent,
            msaa_samples,
        );
        let swapchain_framebuffers = pipe::create_framebuffers(
            device.clone(),
            render_pass,
            &swapchain_imageviews,
            depth_image.view,
            color_image.view,
            swap_chain.extent,
        );

        //init scene buffers
        img::check_mipmap_support(instance.clone(), device.physical_device, vk::Format::R8G8B8A8_UNORM);
        let texture = VulkanApp::create_texture_image(device.clone(), command_pool, device.graphics_queue, &Path::new(TEXTURE_PATH));
        let texture_sampler = texture.create_sampler();
        //let rectangle = get_rect_as_intermediate(1.0, 1.0);
        let model = mdl::load_model(Path::new(MODEL_PATH), texture);
        let vertex_buffer = VulkanApp::create_vertex_buffer(&instance, device.clone(), device.physical_device, command_pool, device.graphics_queue, &model);
        let index_buffer = VulkanApp::create_index_buffer(&instance, device.clone(), device.physical_device, command_pool, device.graphics_queue, &model);

        let ubo = VulkanApp::create_ubo(swap_chain.extent);
        let uniform_buffers = pipe::create_uniform_buffers(device.clone(), swap_chain.images.len());
        let descriptor_pool = pipe::create_descriptor_pool(device.clone(), swap_chain.images.len());
        let descriptor_sets = pipe::create_descriptor_sets(
            device.clone(),
            descriptor_pool,
            ubo_layout,
            &uniform_buffers,
            &model.diffuse_tex,
            texture_sampler,
            swap_chain.images.len(),
        );
        //init command buffers
        let command_buffers = VulkanApp::create_command_buffers(
            device.clone(),
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            swap_chain.extent,
            &vertex_buffer,
            &index_buffer,
            &model,
            pipeline_layout,
            &descriptor_sets,
        );
        let sync_ojbects = share::v1::create_sync_objects(&device.logical_device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            window,

            _entry: entry,
            instance,
            vulkan_surface,
            debug_utils_loader,
            debug_messenger,

            device,

            swap_chain,
            swapchain_imageviews,
            swapchain_framebuffers,

            pipeline_layout,
            ubo_layout,
            render_pass,
            graphics_pipeline,

            color_image,
            depth_image,

            msaa_samples,

            texture_sampler,

            model,
            vertex_buffer,
            index_buffer,

            current_ubo: ubo,
            uniform_buffers,

            descriptor_pool,
            descriptor_sets,

            command_pool,
            command_buffers,

            image_available_semaphores: sync_ojbects.image_available_semaphores,
            render_finished_semaphores: sync_ojbects.render_finished_semaphores,
            in_flight_fences: sync_ojbects.inflight_fences,
            current_frame: 0,

            is_framebuffer_resized: false,
        }
    }

    fn create_texture_image(device: Arc<VulkanDevice>, command_pool: vk::CommandPool, submit_queue: vk::Queue, image_path: &Path) -> img::Image {
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

        let texture = img::Image::new(
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

        img::transition_image_layout(
            device.clone(),
            command_pool,
            submit_queue,
            texture.image,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            mip_levels,
        );

        cmd::copy_buffer_to_image(
            device.clone(),
            command_pool,
            submit_queue,
            staging_buffer.buffer,
            texture.image,
            image_width,
            image_height,
        );

        img::generate_mipmaps(device.clone(), command_pool, submit_queue, &texture);

        texture
    }

    fn create_vertex_buffer(
        instance: &ash::Instance,
        device: Arc<VulkanDevice>,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        mesh: &GFXModel,
    ) -> bfr::Buffer {
        let buffer_size = mesh.vertices_size();
        let device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let staging_buffer = bfr::create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .logical_device
                .map_memory(staging_buffer.memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(mesh.vertices.as_ptr(), mesh.vertices.len());

            device.logical_device.unmap_memory(staging_buffer.memory);
        }

        let vertex_buffer = bfr::create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_memory_properties,
        );

        bfr::copy_buffer(
            device.clone(),
            submit_queue,
            command_pool,
            staging_buffer.buffer,
            vertex_buffer.buffer,
            buffer_size,
        );

        vertex_buffer
    }

    fn create_index_buffer(
        instance: &ash::Instance,
        device: Arc<VulkanDevice>,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        mesh: &GFXModel,
    ) -> bfr::Buffer {
        // let buffer_size = std::mem::size_of_val(&mesh.indices) as vk::DeviceSize;
        let buffer_size = mesh.indices_size();
        let device_memory_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let staging_buffer = bfr::create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .logical_device
                .map_memory(staging_buffer.memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(mesh.indices.as_ptr(), mesh.indices.len());

            device.logical_device.unmap_memory(staging_buffer.memory);
        }

        let index_buffer = bfr::create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_memory_properties,
        );

        bfr::copy_buffer(
            device.clone(),
            submit_queue,
            command_pool,
            staging_buffer.buffer,
            index_buffer.buffer,
            buffer_size,
        );

        index_buffer
    }

    fn create_ubo(image_size: vk::Extent2D) -> UniformBufferObject {
        let mut ubo = UniformBufferObject {
            model: Mat4::identity(),
            view: look_at(&Vec3::new(0.0, 0.0, 20.0), &Vec3::new(0.0, 0.0, 0.0), &Vec3::new(0.0, 1.0, 0.0)),
            proj: perspective(image_size.width as f32 / image_size.height as f32, PI / 4.0, 0.1, 100.0),
        };

        //ubo.proj[5] *= -1.0; // = ubo.proj[1][1] * -1.0;

        ubo.proj = nalgebra_glm::scale(&ubo.proj, &Vec3::new(1.0, -1.0, 1.0));
        ubo
    }

    fn create_command_buffers(
        device: Arc<VulkanDevice>,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        vertex_buffer: &bfr::Buffer,
        index_buffer: &bfr::Buffer,
        mesh: &GFXModel,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: &Vec<vk::DescriptorSet>,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffers = unsafe {
            device
                .logical_device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        };

        for (i, &command_buffer) in command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                p_inheritance_info: ptr::null(),
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
            };

            unsafe {
                device
                    .logical_device
                    .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                    .expect("Failed to begin recording Command Buffer at beginning!");
            }

            let clear_values = [
                vk::ClearValue {
                    // clear value for color buffer
                    color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
                },
                vk::ClearValue {
                    // clear value for depth buffer
                    depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
                },
            ];

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass,
                framebuffer: framebuffers[i],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: surface_extent,
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            unsafe {
                device
                    .logical_device
                    .cmd_begin_render_pass(command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
                device
                    .logical_device
                    .cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);

                let vertex_buffers = [vertex_buffer.buffer];
                let offsets = [0_u64];
                let descriptor_sets_to_bind = [descriptor_sets[i]];

                device.logical_device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                device
                    .logical_device
                    .cmd_bind_index_buffer(command_buffer, index_buffer.buffer, 0, vk::IndexType::UINT32);
                device.logical_device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &descriptor_sets_to_bind,
                    &[],
                );

                device.logical_device.cmd_draw_indexed(command_buffer, mesh.indices.len() as u32, 1, 0, 0, 0);

                device.logical_device.cmd_end_render_pass(command_buffer);

                device
                    .logical_device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }

    //TODO: use for single buffer update with push constants
    fn write_command_buffer(
        device: Arc<VulkanDevice>,
        command_pool: vk::CommandPool,
        command_buffer: &mut vk::CommandBuffer,
        graphics_pipeline: vk::Pipeline,
        framebuffer: &vk::Framebuffer,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        vertex_buffer: &bfr::Buffer,
        index_buffer: &bfr::Buffer,
        mesh: Arc<GFXModel>,
    ) {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
        };

        unsafe {
            device
                .logical_device
                .begin_command_buffer(*command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
        }];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: ptr::null(),
            render_pass,
            framebuffer: *framebuffer,
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };

        unsafe {
            device
                .logical_device
                .cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE);
            device
                .logical_device
                .cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline);

            let vertex_buffers = [vertex_buffer.buffer];
            let offsets = [0_u64];

            device.logical_device.cmd_bind_vertex_buffers(*command_buffer, 0, &vertex_buffers, &offsets);
            device
                .logical_device
                .cmd_bind_index_buffer(*command_buffer, index_buffer.buffer, 0, vk::IndexType::UINT32);

            device.logical_device.cmd_draw_indexed(*command_buffer, mesh.indices.len() as u32, 1, 0, 0, 0);

            device.logical_device.cmd_end_render_pass(*command_buffer);

            device
                .logical_device
                .end_command_buffer(*command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }
}

// Runtime
impl VulkanApp {
    fn update_uniform_buffer(&mut self, current_image: usize) {
        let ubos = [self.current_ubo];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr = self
                .device
                .logical_device
                .map_memory(self.uniform_buffers[current_image].memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device.logical_device.unmap_memory(self.uniform_buffers[current_image].memory);
        }
    }

    fn draw_frame(&mut self, delta_t: f32) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        unsafe {
            self.device
                .logical_device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = self.swap_chain.swapchain_loader.acquire_next_image(
                self.swap_chain.swapchain,
                std::u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain();
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        self.current_ubo.model = nalgebra_glm::rotate(&self.current_ubo.model, std::f32::consts::PI / 2.0 * delta_t, &Vec3::new(0.0, 1.0, 0.0));
        self.update_uniform_buffer(image_index as usize);

        //Get Semaphores for frame.
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device.logical_device.reset_fences(&wait_fences).expect("Failed to reset Fence!");

            self.device
                .logical_device
                .queue_submit(self.device.graphics_queue, &submit_infos, self.in_flight_fences[self.current_frame])
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swap_chain.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = unsafe { self.swap_chain.swapchain_loader.queue_present(self.device.present_queue, &present_info) };

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn recreate_swapchain(&mut self) {
        unsafe { self.device.logical_device.device_wait_idle().expect("Failed to wait device idle!") };
        self.cleanup_swapchain();

        let inner_window_size = self.window.inner_size();
        let new_swap_chain = VulkanSwapChain::new(
            &self.instance,
            &self.device,
            &self.vulkan_surface,
            &ImageSize {
                width: inner_window_size.width,
                height: inner_window_size.height,
            },
        );
        self.swap_chain = new_swap_chain;

        self.swapchain_imageviews = share::v1::create_image_views(self.device.clone(), self.swap_chain.format, &self.swap_chain.images);
        self.render_pass = pipe::create_render_pass(self.instance.clone(), self.device.clone(), self.swap_chain.format, self.msaa_samples);
        let (graphics_pipeline, pipeline_layout) = pipe::create_graphics_pipeline(
            self.device.clone(),
            self.render_pass,
            self.swap_chain.extent,
            self.ubo_layout,
            self.msaa_samples,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.color_image = misc::create_color_resources(self.device.clone(), self.swap_chain.format, self.swap_chain.extent, self.msaa_samples);

        self.depth_image = misc::create_depth_resources(
            self.instance.clone(),
            self.device.clone(),
            self.device.physical_device,
            self.command_pool,
            self.device.graphics_queue,
            self.swap_chain.extent,
            self.msaa_samples,
        );

        self.swapchain_framebuffers = pipe::create_framebuffers(
            self.device.clone(),
            self.render_pass,
            &self.swapchain_imageviews,
            self.depth_image.view,
            self.color_image.view,
            self.swap_chain.extent,
        );
        self.command_buffers = VulkanApp::create_command_buffers(
            self.device.clone(),
            self.command_pool,
            self.graphics_pipeline,
            &self.swapchain_framebuffers,
            self.render_pass,
            self.swap_chain.extent,
            &self.vertex_buffer,
            &self.index_buffer,
            &self.model,
            self.pipeline_layout,
            &self.descriptor_sets,
        );
    }

    fn cleanup_swapchain(&mut self) {
        unsafe {
            self.depth_image.vk_destroy();
            self.color_image.vk_destroy();

            self.device.logical_device.free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.logical_device.destroy_framebuffer(framebuffer, None);
            }
            self.device.logical_device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.logical_device.destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device.logical_device.destroy_image_view(image_view, None);
            }
            self.swap_chain.vk_destroy();
        }
    }

    fn wait_device_idle(&self) {
        unsafe { self.device.logical_device.device_wait_idle().expect("Failed to wait device idle!") };
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        println!("VulkanApp.drop");

        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device.logical_device.destroy_semaphore(self.image_available_semaphores[i], None);
                self.device.logical_device.destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.logical_device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device.logical_device.destroy_descriptor_pool(self.descriptor_pool, None);

            for i in 0..self.uniform_buffers.len() {
                self.uniform_buffers[i].vk_destroy();
            }

            self.index_buffer.vk_destroy();
            self.vertex_buffer.vk_destroy();

            self.device.logical_device.destroy_sampler(self.texture_sampler, None);
            self.model.vk_destroy();

            self.device.logical_device.destroy_descriptor_set_layout(self.ubo_layout, None);

            self.device.logical_device.destroy_command_pool(self.command_pool, None);

            self.device.logical_device.destroy_device(None);
            self.vulkan_surface.surface_loader.destroy_surface(self.vulkan_surface.surface, None);

            if VALIDATION.is_enable {
                self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanApp {
    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        let mut tick_counter = vk_assist::types::fps_limiter::FPSLimiter::new();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput { virtual_keycode, state, .. } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                let delta_t = tick_counter.delta_time();
                self.draw_frame(delta_t);

                tick_counter.tick_frame();
                if IS_PAINT_FPS_COUNTER {
                    print!("FPS: {}\r", tick_counter.fps());
                }
            }
            Event::LoopDestroyed => {
                unsafe { self.device.logical_device.device_wait_idle().expect("Failed to wait device idle!") };
            }
            _ => (),
        })
    }
}
