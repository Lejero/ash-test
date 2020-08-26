use super::camera::Camera;
use super::scene;
use super::time_manager::{PrintFPSPeriod, TimeManager};
use std::cell::RefCell;
use std::rc::Rc;
use winit::event::MouseButton::Other;

use std::sync::Arc;
use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

// #[derive(Copy, Clone)]
pub struct App {
    //event_loop: EventLoop<()>,
    window: Arc<Window>,
    renderer: RefCell<scene::VulkanApp>,
    camera: Camera,
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
            time_manager: TimeManager::new(PrintFPSPeriod::Other(2200)),
        };

        app.main_loop(event_loop);
    }

    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(
            move |event, _, control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => App::key_handler(input, control_flow), // match input {
                    WindowEvent::MouseInput { .. } | WindowEvent::MouseWheel { .. } | WindowEvent::AxisMotion { .. } => App::mouse_handler(event, control_flow),
                    _ => {}
                },
                Event::MainEventsCleared => {
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
        // self.delta_t = Instant::now().duration_since(self.last_t).as_secs_f32();
        // self.frame_delta_t = self.frame_delta_t + self.delta_t;
        // self.last_t = Instant::now();
        // if self.frame_manager.should_draw_frame() {
        //     self.frame_manager.update_step_on_decasec(true);
        //     self.renderer.borrow_mut().draw_frame(self.frame_delta_t);
        //     self.frame_delta_t = 0.0;
        // }
        if self.time_manager.update() {
            self.renderer.borrow_mut().draw_frame_with_cam(self.time_manager.frame_delta_t, &self.camera);
            self.time_manager.frame_delta_t = 0.0;
        }
    }

    pub fn key_handler(input: KeyboardInput, control_flow: &mut winit::event_loop::ControlFlow) {
        match input {
            KeyboardInput { virtual_keycode, state, .. } => match (virtual_keycode, state) {
                (Some(VirtualKeyCode::Escape), ElementState::Pressed) => *control_flow = ControlFlow::Exit,
                (Some(VirtualKeyCode::W), ElementState::Pressed) => println!("Forward"),
                (Some(VirtualKeyCode::S), ElementState::Pressed) => println!("Back"),
                (Some(VirtualKeyCode::A), ElementState::Pressed) => println!("Left"),
                (Some(VirtualKeyCode::D), ElementState::Pressed) => println!("Right"),
                (Some(VirtualKeyCode::Q), ElementState::Pressed) => println!("Up"),
                (Some(VirtualKeyCode::Z), ElementState::Pressed) => println!("Down"),
                _ => {}
            },
        }
    }

    #[allow(unused_variables)]
    pub fn mouse_handler(event: WindowEvent, control_flow: &mut winit::event_loop::ControlFlow) {
        match event {
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
                modifiers,
            } => {
                // println!(
                //     "Button: {0}",
                //     match button {
                //         MouseButton::Left => "Left",
                //         MouseButton::Right => "Right",
                //         MouseButton::Middle => "Middle",
                //         MouseButton::Other(_val) => {
                //             //format!("OtherButton({0})", _val).as_str()
                //             "Other"
                //         }
                //     }
                // )
                match button {
                    MouseButton::Left => println!("Left Button"),
                    MouseButton::Right => println!("Right Button"),
                    MouseButton::Middle => println!("Middle Button"),
                    MouseButton::Other(val) => println!("OtherButton({0})", val),
                }
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
                modifiers,
            } => println!("Delta:{0},{1}", "", ""),
            WindowEvent::AxisMotion { device_id, axis, value } => println!("Axis:{0}, Value:{1}", axis, value),
            _ => {}
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
