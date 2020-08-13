#![allow(dead_code)]
#![allow(unused_imports)]

mod app;
mod pipelines;
//mod utility;
mod vk_assist;
mod vk_model;

use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

use app::scene::VulkanApp;

const WINDOW_TITLE: &'static str = "Vulkan App";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    app::central::App::start(String::from(WINDOW_TITLE), WINDOW_WIDTH, WINDOW_HEIGHT);
}
