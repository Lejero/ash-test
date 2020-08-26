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
use std::io::{stdin, stdout, Read, Write};

const WINDOW_TITLE: &'static str = "Vulkan App";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    app::central::App::start(String::from(WINDOW_TITLE), WINDOW_WIDTH, WINDOW_HEIGHT);
    println!("Going to wait...");
    stdin().read_line(&mut String::new()).unwrap();
    pause(); //TODO: Make app console hang for one input after app ends so debug can be read.
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}
