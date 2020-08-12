use super::scene;
use crate::vk_assist::types::frame_manager::FrameManager;

use std::sync::Arc;
use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

pub struct App {
    //event_loop: EventLoop<()>,
    window: Arc<Window>,
    vulkan_app: scene::VulkanApp,
    delta_t: f32,
    frame_delta_t: f32,
    last_t: Instant,
    frame_manager: FrameManager,
}

impl App {
    pub fn start(title: String, width: u32, height: u32) {
        let event_loop = EventLoop::new();
        let window = Arc::new(init_window(&event_loop, &title, width, height));
        let vulkan_app = scene::VulkanApp::new(window.clone());

        let app = App {
            //event_loop: event_loop,
            window: window,
            vulkan_app: vulkan_app,

            delta_t: 0.0,
            frame_delta_t: 0.0,
            last_t: Instant::now(),
            frame_manager: FrameManager::new(),
        };

        app.main_loop(event_loop);
    }

    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
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
                self.delta_t = Instant::now().duration_since(self.last_t).as_secs_f32();
                self.frame_delta_t = self.frame_delta_t + self.delta_t;
                self.last_t = Instant::now();
                if self.frame_manager.should_draw_frame() {
                    self.frame_manager.update_step_on_decasec(true);
                    self.vulkan_app.draw_frame(self.frame_delta_t);
                    self.frame_delta_t = 0.0;
                }
            }
            Event::LoopDestroyed => {
                self.vulkan_app.wait_device_idle();
            }
            _ => (),
        });
    }

    pub fn redraw(mut self) {
        self.delta_t = Instant::now().duration_since(self.last_t).as_secs_f32();
        self.frame_delta_t = self.frame_delta_t + self.delta_t;
        self.last_t = Instant::now();
        if self.frame_manager.should_draw_frame() {
            self.frame_manager.update_step_on_decasec(true);
            self.vulkan_app.draw_frame(self.frame_delta_t);
            self.frame_delta_t = 0.0;
        }
    }
}

pub fn init_window(event_loop: &EventLoop<()>, title: &str, width: u32, height: u32) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize::new(width, height))
        .build(event_loop)
        .expect("Failed to create window.")
}
