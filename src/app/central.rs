use super::camera::Camera;
use super::input_model::{InputKey, InputModel};
use super::scene;
use super::time_manager::{PrintFPSPeriod, TimeManager};
use std::cell::RefCell;
use std::rc::Rc;
use winit::event::MouseButton::Other;

use std::sync::Arc;
use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::event::{ButtonId, DeviceEvent, ElementState as ES, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode as VKC, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

// #[derive(Copy, Clone)]
pub struct App {
    //event_loop: EventLoop<()>,
    window: Arc<Window>,
    renderer: RefCell<scene::VulkanApp>,
    camera: Camera,
    input_model: InputModel,
    //Time
    time_manager: TimeManager,
}

impl App {
    pub fn start(title: String, width: u32, height: u32) {
        let event_loop = EventLoop::new();
        let window = Arc::new(init_window(&event_loop, &title, width, height));
        let renderer = RefCell::new(scene::VulkanApp::new(window.clone()));

        let cam = Camera::new((width / height) as f32, 0.1, 100.0, 4.0);
        println!("look_vec:{0}", cam.look_vec());
        println!("up_vec:{0}", cam.up_vec());
        println!("eye_pos_vec:{0}", cam.eye_pos_vec());
        println!("view_mat:{0}", cam.view_mat);
        println!("perspective_mat:{0}", cam.perspective_mat);

        let app = App {
            //event_loop: event_loop,
            window: window,
            renderer: renderer,
            camera: cam,
            input_model: InputModel::default(),
            time_manager: TimeManager::new(PrintFPSPeriod::Other(2200)),
        };

        app.main_loop(event_loop);
    }

    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(
            move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {}
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => App::key_handler(input, &mut self.input_model, &mut self.camera, control_flow), // match input {
                    WindowEvent::MouseInput { button, state, .. } => self.mouse_button_handler(button, state),
                    _ => {}
                },
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => self.mouse_movement_handler(delta),
                    //DeviceEvent::Button { button, state } => self.mouse_button_handler(button, state), //This only detects the left, middle, and right mouse buttons and only by numbers. The window event proves a more useful measure.
                    DeviceEvent::Key(input) => App::key_handler(input, &mut self.input_model, &mut self.camera, control_flow),
                    DeviceEvent::Motion { axis, value } => {}
                    _ => {}
                },
                Event::MainEventsCleared => {
                    self.update();
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_window_id) => {
                    self.redraw();
                }
                Event::LoopDestroyed => {
                    self.renderer.borrow_mut().wait_device_idle();
                }
                _ => (),
            }, //end clojure
        );
    }

    pub fn redraw(&mut self) {
        if self.time_manager.update() {
            self.renderer.borrow_mut().draw_frame_with_cam(self.time_manager.frame_delta_t, &self.camera);
            self.time_manager.frame_delta_t = 0.0;
        }
    }

    pub fn key_handler(input: KeyboardInput, input_model: &mut InputModel, camera: &mut Camera, control_flow: &mut winit::event_loop::ControlFlow) {
        match input {
            KeyboardInput { virtual_keycode, state, .. } => match (virtual_keycode, state) {
                (Some(VKC::Escape), ES::Pressed) => *control_flow = ControlFlow::Exit,
                (Some(VKC::W), es) => input_model.forward_key.key_down = es == ES::Pressed,
                (Some(VKC::S), es) => input_model.back_key.key_down = es == ES::Pressed,
                (Some(VKC::A), es) => input_model.left_key.key_down = es == ES::Pressed,
                (Some(VKC::D), es) => input_model.right_key.key_down = es == ES::Pressed,
                (Some(VKC::Q), es) => input_model.up_key.key_down = es == ES::Pressed,
                (Some(VKC::Z), es) => input_model.down_key.key_down = es == ES::Pressed,
                _ => {}
            },
        }
    }

    #[allow(unused_variables)]
    pub fn mouse_button_handler(&mut self, button: MouseButton, state: ES) {
        match button {
            MouseButton::Left => println!("Button: {0}", "Left"),
            MouseButton::Middle => println!("Button: {0}", "Middle"),
            MouseButton::Right => println!("Button: {0}", "Right"),
            MouseButton::Other(id) => println!("Button: Other({0})", id),
        }
    }

    #[allow(unused_variables)]
    pub fn mouse_movement_handler(&mut self, delta: (f64, f64)) {
        //println!("MouseDelta:({0}, {1})", delta.0, delta.1);

        //self.camera.rotate(0.1 * delta.0 as f32, &self.camera.up_vec());
        self.camera.rotate(0.1 * delta.1 as f32, &self.camera.look_cross_up_vec());
    }

    fn update(&mut self) {
        let scalar = 1.0 * self.time_manager.delta_t;

        if self.input_model.forward_key.key_down {
            self.camera.translate(&(-1.0 * scalar * self.camera.look_vec()).clone());
        }
        if self.input_model.back_key.key_down {
            self.camera.translate(&(scalar * self.camera.look_vec()).clone());
        }
        if self.input_model.left_key.key_down {
            self.camera.translate(&(-1.0 * scalar * self.camera.look_cross_up_vec()).clone());
        }
        if self.input_model.right_key.key_down {
            self.camera.translate(&(scalar * self.camera.look_cross_up_vec()).clone());
        }
        if self.input_model.up_key.key_down {
            self.camera.translate(&(scalar * self.camera.up_vec()).clone());
        }
        if self.input_model.down_key.key_down {
            self.camera.translate(&(-1.0 * scalar * self.camera.up_vec()).clone());
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
