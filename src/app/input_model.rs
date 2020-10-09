use winit::event::{ElementState as ES, Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode as VKC, WindowEvent};

pub struct InputKey {
    pub key_down: bool,
    pub key: VKC,
}

impl InputKey {
    pub fn new(key: VKC) -> InputKey {
        InputKey { key_down: false, key }
    }
}

pub struct InputModel {
    pub forward_key: InputKey,
    pub back_key: InputKey,
    pub left_key: InputKey,
    pub right_key: InputKey,
    pub up_key: InputKey,
    pub down_key: InputKey,
}

impl InputModel {
    pub fn default() -> InputModel {
        InputModel {
            forward_key: InputKey::new(VKC::W),
            back_key: InputKey::new(VKC::S),
            left_key: InputKey::new(VKC::A),
            right_key: InputKey::new(VKC::D),
            up_key: InputKey::new(VKC::Q),
            down_key: InputKey::new(VKC::Z),
        }
    }
}
