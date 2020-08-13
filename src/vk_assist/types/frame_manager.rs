#![allow(dead_code)]
#![allow(unused_imports)]

use std::time::Instant;

#[derive(Copy, Clone)]
pub struct FrameManager {
    pub sec_start: Instant,
    pub frame_start: Instant,
    pub frame_count: u32,
    pub target_fps: u32,
}

impl FrameManager {
    pub fn new() -> FrameManager {
        FrameManager {
            sec_start: Instant::now(),
            frame_start: Instant::now(),
            frame_count: 0,
            target_fps: 120,
        }
    }
    pub fn update_step_on_sec(&mut self, print: bool) {
        let time = Instant::now();
        if time.duration_since(self.sec_start).as_secs() > 0 {
            self.sec_start = time;
            if print {
                println!("FPS: {}", self.frame_count);
            }
            self.frame_count = 0;
        }

        self.frame_count += 1;
    }

    pub fn update_step_on_decasec(&mut self, print: bool) {
        let time = Instant::now();
        if time.duration_since(self.sec_start).as_secs() > 9 {
            self.sec_start = time;
            if print {
                println!("FPS: {}", self.frame_count / 10);
            }
            self.frame_count = 0;
        }

        self.frame_count += 1;
    }

    pub fn should_draw_frame(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.frame_start).as_secs_f64() >= (1.0 / self.target_fps as f64) {
            self.frame_start = now;
            return true;
        } else {
            return false;
        }
    }
}
