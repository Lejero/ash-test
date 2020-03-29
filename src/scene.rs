#![allow(dead_code)]
#![allow(unused_imports)]

//mod utility;
use crate::g_model;
use crate::pipelines;
use crate::utility;
use crate::vk_assist;
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
use std::ptr;

use g_model::basic_model::BasicModel;
use g_model::MeshSize;
use vk_assist::structures::{get_rect_as_basic, SimpleVertex, UniformBufferObject};
use vk_assist::types::buffer::{copy_buffer, create_buffer, Buffer};
use vk_assist::types::command::{
    begin_single_time_command, end_single_time_command, find_memory_type,
};
use vk_assist::types::{
    buffer, command, vulkan_device, vulkan_device::VulkanDevice, vulkan_surface::VulkanSurface,
    vulkan_swap_chain::*,
};

//mod pipelines;
use pipelines::basic_ubo_pipeline::create_graphics_pipeline;

// Constants
const WINDOW_TITLE: &'static str = "20.Index Buffer";

pub struct VulkanApp {
    window: winit::window::Window,

    // vulkan stuff
    _entry: ash::Entry,
    instance: Arc<ash::Instance>,

    vulkan_surface: VulkanSurface,

    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,

    device: Arc<VulkanDevice>,

    vulkan_swap_chain: VulkanSwapChain,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: vk::RenderPass,
    ubo_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    rectangle: Arc<BasicModel>,
    // vertex_buffer: vk::Buffer,
    // vertex_buffer_memory: vk::DeviceMemory,
    vertex_buffer: Buffer,
    index_buffer: Buffer,

    current_ubo: UniformBufferObject,
    uniform_buffers: Vec<Buffer>,

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
        //init window
        let window =
            utility::window::init_window(event_loop, WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);

        // init instance
        let entry = ash::Entry::new().unwrap();
        let instance = Arc::new(share::create_instance(
            &entry,
            WINDOW_TITLE,
            VALIDATION.is_enable,
            &VALIDATION.required_validation_layers.to_vec(),
        ));

        //init surface
        let vulkan_surface =
            VulkanSurface::create_surface(&entry, &instance, &window, WINDOW_WIDTH, WINDOW_HEIGHT);

        //init debug
        let (debug_utils_loader, debug_messenger) =
            setup_debug_utils(VALIDATION.is_enable, &entry, &instance);

        //init device
        let device = Arc::new(VulkanDevice::create_device(
            instance.clone(),
            &vulkan_surface,
            vulkan_device::SWAP_CHAIN_ONLY_EXTENSIONS,
        ));
        let inner_window_size = window.inner_size();

        //init swap chain
        let vulkan_swap_chain = VulkanSwapChain::new(
            &instance,
            &device,
            &vulkan_surface,
            &ImageSize {
                width: inner_window_size.width,
                height: inner_window_size.height,
            },
        );
        let swapchain_imageviews = share::v1::create_image_views(
            &device.logical_device,
            vulkan_swap_chain.swapchain_format,
            &vulkan_swap_chain.swapchain_images,
        );

        //init pipeline
        let render_pass = share::v1::create_render_pass(
            &device.logical_device,
            vulkan_swap_chain.swapchain_format,
        );
        let ubo_layout = VulkanApp::create_descriptor_set_layout(device.clone());
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(
            &device.logical_device,
            render_pass,
            vulkan_swap_chain.swapchain_extent,
            ubo_layout,
        );
        let swapchain_framebuffers = share::v1::create_framebuffers(
            &device.logical_device,
            render_pass,
            &swapchain_imageviews,
            vulkan_swap_chain.swapchain_extent,
        );
        let command_pool =
            share::v1::create_command_pool(&device.logical_device, &device.queue_family);
        //init scene buffers
        let rectangle = get_rect_as_basic(1.0, 1.0);
        let vertex_buffer = VulkanApp::create_vertex_buffer(
            &instance,
            device.logical_device.clone(),
            device.physical_device,
            command_pool,
            device.graphics_queue,
            rectangle.clone(),
        );
        let index_buffer = VulkanApp::create_index_buffer(
            &instance,
            device.logical_device.clone(),
            device.physical_device,
            command_pool,
            device.graphics_queue,
            rectangle.clone(),
        );

        let ubo = VulkanApp::create_ubo(vulkan_swap_chain.swapchain_extent);
        let uniform_buffers = VulkanApp::create_uniform_buffers(
            device.clone(),
            vulkan_swap_chain.swapchain_images.len(),
        );
        let descriptor_pool = VulkanApp::create_descriptor_pool(
            device.clone(),
            vulkan_swap_chain.swapchain_images.len(),
        );
        let descriptor_sets = VulkanApp::create_descriptor_sets(
            device.clone(),
            descriptor_pool,
            ubo_layout,
            &uniform_buffers,
            vulkan_swap_chain.swapchain_images.len(),
        );
        //init command buffers
        let command_buffers = VulkanApp::create_command_buffers(
            device.clone(),
            command_pool,
            graphics_pipeline,
            &swapchain_framebuffers,
            render_pass,
            vulkan_swap_chain.swapchain_extent,
            &vertex_buffer,
            &index_buffer,
            rectangle.clone(),
            pipeline_layout,
            &descriptor_sets,
        );
        let sync_ojbects =
            share::v1::create_sync_objects(&device.logical_device, MAX_FRAMES_IN_FLIGHT);

        // cleanup(); the 'drop' function will take care of it.
        VulkanApp {
            // winit stuff
            window,

            // vulkan stuff
            _entry: entry,
            instance,
            vulkan_surface,
            debug_utils_loader,
            debug_messenger,

            device,

            vulkan_swap_chain,
            swapchain_imageviews,
            swapchain_framebuffers,

            pipeline_layout,
            ubo_layout,
            render_pass,
            graphics_pipeline,

            rectangle,
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

    fn create_vertex_buffer(
        instance: &ash::Instance,
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        mesh: Arc<BasicModel>,
    ) -> Buffer {
        let buffer_size = mesh.vertices_size();
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let staging_buffer = create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer.memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut SimpleVertex;

            data_ptr.copy_from_nonoverlapping(mesh.vertices.as_ptr(), mesh.vertices.len());

            device.unmap_memory(staging_buffer.memory);
        }

        let vertex_buffer = create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_memory_properties,
        );

        copy_buffer(
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
        device: Arc<ash::Device>,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        mesh: Arc<BasicModel>,
    ) -> Buffer {
        // let buffer_size = std::mem::size_of_val(&mesh.indices) as vk::DeviceSize;
        let buffer_size = mesh.indices_size();
        let device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let staging_buffer = create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device_memory_properties,
        );

        unsafe {
            let data_ptr = device
                .map_memory(
                    staging_buffer.memory,
                    0,
                    buffer_size,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to Map Memory") as *mut u32;

            data_ptr.copy_from_nonoverlapping(mesh.indices.as_ptr(), mesh.indices.len());

            device.unmap_memory(staging_buffer.memory);
        }

        let index_buffer = create_buffer(
            device.clone(),
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            &device_memory_properties,
        );

        copy_buffer(
            device.clone(),
            submit_queue,
            command_pool,
            staging_buffer.buffer,
            index_buffer.buffer,
            buffer_size,
        );

        index_buffer
    }
    fn create_descriptor_set_layout(device: Arc<VulkanDevice>) -> vk::DescriptorSetLayout {
        let ubo_layout_bindings = [vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: ptr::null(),
        }];

        let ubo_layout_create_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorSetLayoutCreateFlags::empty(),
            binding_count: ubo_layout_bindings.len() as u32,
            p_bindings: ubo_layout_bindings.as_ptr(),
        };

        unsafe {
            device
                .logical_device
                .create_descriptor_set_layout(&ubo_layout_create_info, None)
                .expect("Failed to create Descriptor Set Layout!")
        }
    }

    pub fn create_ubo(image_size: vk::Extent2D) -> UniformBufferObject {
        UniformBufferObject {
            model: Mat4::identity(),
            view: look_at(
                &Vec3::new(2.0, 2.0, 2.0),
                &Vec3::new(0.0, 0.0, 0.0),
                &Vec3::new(0.0, 0.0, 1.0),
            ),
            proj: perspective(
                image_size.width as f32 / image_size.height as f32,
                3.1415929 / 4.0,
                0.1,
                10.0,
            ),
        }
    }

    pub fn create_uniform_buffers(
        device: Arc<VulkanDevice>,
        swapchain_image_count: usize,
    ) -> Vec<Buffer> {
        let buffer_size = std::mem::size_of::<UniformBufferObject>();
        let device_memory_properties = unsafe { device.get_physical_device_memory_properties() };
        let mut uniform_buffers = vec![];

        for _ in 0..swapchain_image_count {
            let uniform_buffer = create_buffer(
                device.logical_device.clone(),
                buffer_size as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                &device_memory_properties,
            );
            uniform_buffers.push(uniform_buffer);
        }

        uniform_buffers
    }

    fn create_descriptor_pool(
        device: Arc<VulkanDevice>,
        swapchain_images_size: usize,
    ) -> vk::DescriptorPool {
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain_images_size as u32,
        }];

        let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DescriptorPoolCreateFlags::empty(),
            max_sets: swapchain_images_size as u32,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
        };

        unsafe {
            device
                .logical_device
                .create_descriptor_pool(&descriptor_pool_create_info, None)
                .expect("Failed to create Descriptor Pool!")
        }
    }

    fn create_descriptor_sets(
        device: Arc<VulkanDevice>,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniforms_buffers: &Vec<Buffer>,
        swapchain_images_size: usize,
    ) -> Vec<vk::DescriptorSet> {
        let mut layouts: Vec<vk::DescriptorSetLayout> = vec![];
        for _ in 0..swapchain_images_size {
            layouts.push(descriptor_set_layout);
        }

        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            p_next: ptr::null(),
            descriptor_pool,
            descriptor_set_count: swapchain_images_size as u32,
            p_set_layouts: layouts.as_ptr(),
        };

        let descriptor_sets = unsafe {
            device
                .logical_device
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
                .expect("Failed to allocate descriptor sets!")
        };

        for (i, &descritptor_set) in descriptor_sets.iter().enumerate() {
            let descriptor_buffer_info = [vk::DescriptorBufferInfo {
                buffer: uniforms_buffers[i].buffer,
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as u64,
            }];

            let descriptor_write_sets = [vk::WriteDescriptorSet {
                s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
                p_next: ptr::null(),
                dst_set: descritptor_set,
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_image_info: ptr::null(),
                p_buffer_info: descriptor_buffer_info.as_ptr(),
                p_texel_buffer_view: ptr::null(),
            }];

            unsafe {
                device
                    .logical_device
                    .update_descriptor_sets(&descriptor_write_sets, &[]);
            }
        }

        descriptor_sets
    }

    fn create_command_buffers(
        device: Arc<VulkanDevice>,
        command_pool: vk::CommandPool,
        graphics_pipeline: vk::Pipeline,
        framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        mesh: Arc<BasicModel>,
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

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

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
                device.logical_device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                device.logical_device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    graphics_pipeline,
                );

                let vertex_buffers = [vertex_buffer.buffer];
                let offsets = [0_u64];
                let descriptor_sets_to_bind = [descriptor_sets[i]];

                device.logical_device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &vertex_buffers,
                    &offsets,
                );
                device.logical_device.cmd_bind_index_buffer(
                    command_buffer,
                    index_buffer.buffer,
                    0,
                    vk::IndexType::UINT32,
                );
                device.logical_device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline_layout,
                    0,
                    &descriptor_sets_to_bind,
                    &[],
                );

                device.logical_device.cmd_draw_indexed(
                    command_buffer,
                    mesh.indices.len() as u32,
                    1,
                    0,
                    0,
                    0,
                );

                device.logical_device.cmd_end_render_pass(command_buffer);

                device
                    .logical_device
                    .end_command_buffer(command_buffer)
                    .expect("Failed to record Command Buffer at Ending!");
            }
        }

        command_buffers
    }

    fn write_command_buffer(
        device: Arc<VulkanDevice>,
        command_pool: vk::CommandPool,
        command_buffer: &mut vk::CommandBuffer,
        graphics_pipeline: vk::Pipeline,
        framebuffer: &vk::Framebuffer,
        render_pass: vk::RenderPass,
        surface_extent: vk::Extent2D,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        mesh: Arc<BasicModel>,
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
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
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
            device.logical_device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
            device.logical_device.cmd_bind_pipeline(
                *command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline,
            );

            let vertex_buffers = [vertex_buffer.buffer];
            let offsets = [0_u64];

            device.logical_device.cmd_bind_vertex_buffers(
                *command_buffer,
                0,
                &vertex_buffers,
                &offsets,
            );
            device.logical_device.cmd_bind_index_buffer(
                *command_buffer,
                index_buffer.buffer,
                0,
                vk::IndexType::UINT32,
            );

            device.logical_device.cmd_draw_indexed(
                *command_buffer,
                mesh.indices.len() as u32,
                1,
                0,
                0,
                0,
            );

            device.logical_device.cmd_end_render_pass(*command_buffer);

            device
                .logical_device
                .end_command_buffer(*command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }
    }
}

// Init
impl VulkanApp {
    fn init_instance() {}
    fn init_surface() {}
    fn init_debug() {}
    fn init_device() {}
    fn init_swap_chain() {}
}

// Fix content -------------------------------------------------------------------------------
impl VulkanApp {
    fn update_uniform_buffer(&mut self, current_image: usize, delta_time: f32) {
        self.current_ubo.model = nalgebra_glm::rotate(
            &self.current_ubo.model,
            std::f32::consts::PI / 2.0 * delta_time,
            &Vec3::new(0.0, 0.0, 1.0),
        );

        let ubos = [self.current_ubo];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            let data_ptr =
                self.device
                    .logical_device
                    .map_memory(
                        self.uniform_buffers[current_image].memory,
                        0,
                        buffer_size,
                        vk::MemoryMapFlags::empty(),
                    )
                    .expect("Failed to Map Memory") as *mut UniformBufferObject;

            data_ptr.copy_from_nonoverlapping(ubos.as_ptr(), ubos.len());

            self.device
                .logical_device
                .unmap_memory(self.uniform_buffers[current_image].memory);
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
            let result = self.vulkan_swap_chain.swapchain_loader.acquire_next_image(
                self.vulkan_swap_chain.swapchain,
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
        self.update_uniform_buffer(image_index as usize, delta_t);

        //Get Semaphores for frame.
        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        //Submit a frame to be written to with the command buffer.
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
            self.device
                .logical_device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .logical_device
                .queue_submit(
                    self.device.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.vulkan_swap_chain.swapchain];

        //Present a frame to the swap chain image when it is done rendering.
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

        let result = unsafe {
            self.vulkan_swap_chain
                .swapchain_loader
                .queue_present(self.device.present_queue, &present_info)
        };
        //Recreate the swap chain if it is in any way indicated (like if the window was resized).
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
        unsafe {
            self.device
                .logical_device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
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
        self.vulkan_swap_chain = new_swap_chain;

        self.swapchain_imageviews = share::v1::create_image_views(
            &self.device.logical_device,
            self.vulkan_swap_chain.swapchain_format,
            &self.vulkan_swap_chain.swapchain_images,
        );
        self.render_pass = share::v1::create_render_pass(
            &self.device.logical_device,
            self.vulkan_swap_chain.swapchain_format,
        );
        let ubo_layout = VulkanApp::create_descriptor_set_layout(self.device.clone());
        let (graphics_pipeline, pipeline_layout) = create_graphics_pipeline(
            &self.device.logical_device,
            self.render_pass,
            self.vulkan_swap_chain.swapchain_extent,
            ubo_layout,
        );
        self.graphics_pipeline = graphics_pipeline;
        self.pipeline_layout = pipeline_layout;

        self.swapchain_framebuffers = share::v1::create_framebuffers(
            &self.device.logical_device,
            self.render_pass,
            &self.swapchain_imageviews,
            self.vulkan_swap_chain.swapchain_extent,
        );
        self.command_buffers = VulkanApp::create_command_buffers(
            self.device.clone(),
            self.command_pool,
            self.graphics_pipeline,
            &self.swapchain_framebuffers,
            self.render_pass,
            self.vulkan_swap_chain.swapchain_extent,
            &self.vertex_buffer,
            &self.index_buffer,
            self.rectangle.clone(),
            self.pipeline_layout,
            &self.descriptor_sets,
        );
    }

    fn cleanup_swapchain(&mut self) {
        unsafe {
            self.device
                .logical_device
                .free_command_buffers(self.command_pool, &self.command_buffers);
            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device
                    .logical_device
                    .destroy_framebuffer(framebuffer, None);
            }
            self.device
                .logical_device
                .destroy_pipeline(self.graphics_pipeline, None);
            self.device
                .logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device
                .logical_device
                .destroy_render_pass(self.render_pass, None);
            for &image_view in self.swapchain_imageviews.iter() {
                self.device
                    .logical_device
                    .destroy_image_view(image_view, None);
            }
            self.vulkan_swap_chain.vk_destroy();
        }
    }

    fn wait_device_idle(&self) {
        unsafe {
            self.device
                .logical_device
                .device_wait_idle()
                .expect("Failed to wait device idle!")
        };
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .logical_device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .logical_device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device
                    .logical_device
                    .destroy_fence(self.in_flight_fences[i], None);
            }

            self.cleanup_swapchain();

            self.device
                .logical_device
                .destroy_descriptor_pool(self.descriptor_pool, None);

            self.device
                .logical_device
                .destroy_descriptor_set_layout(self.ubo_layout, None);

            for i in 0..self.uniform_buffers.len() {
                self.uniform_buffers[i].vk_destroy();
            }

            self.index_buffer.vk_destroy();
            self.vertex_buffer.vk_destroy();

            self.device
                .logical_device
                .destroy_descriptor_set_layout(self.ubo_layout, None);

            self.device
                .logical_device
                .destroy_command_pool(self.command_pool, None);

            self.device.logical_device.destroy_device(None);
            
            self.vulkan_surface
                .surface_loader
                .destroy_surface(self.vulkan_surface.surface, None);

            if VALIDATION.is_enable {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
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
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            *control_flow = ControlFlow::Exit
                        }
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
                unsafe {
                    self.device
                        .logical_device
                        .device_wait_idle()
                        .expect("Failed to wait device idle!")
                };
            }
            _ => (),
        })
    }
}
