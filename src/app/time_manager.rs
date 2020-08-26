#![allow(dead_code)]
#![allow(unused_imports)]

use std::time::Instant;

#[derive(Copy, Clone)]
pub enum PrintFPSPeriod {
    No,
    Second,
    FiveSecond,
    DecaSecond,
    Other(u128),
}

#[derive(Copy, Clone)]
pub struct TimeManager {
    pub delta_t: f32,
    pub frame_delta_t: f32,
    pub last_t: Instant,
    pub sec_start: Instant,
    pub frame_start: Instant,
    pub frame_count: u32,
    pub target_fps: u32,

    //Update Settings
    pub print_fps_period: PrintFPSPeriod,
}

impl TimeManager {
    pub fn new(print_fps_period: PrintFPSPeriod) -> TimeManager {
        TimeManager {
            delta_t: 0.0,
            frame_delta_t: 0.0,
            last_t: Instant::now(),
            sec_start: Instant::now(),
            frame_start: Instant::now(),
            frame_count: 0,
            target_fps: 120,

            print_fps_period: print_fps_period,
        }
    }

    //Returns whether a frame should be drawn.
    pub fn update(&mut self) -> bool {
        self.delta_t = Instant::now().duration_since(self.last_t).as_secs_f32();
        self.frame_delta_t = self.frame_delta_t + self.delta_t;
        self.last_t = Instant::now();

        if self.should_draw_frame() {
            match self.print_fps_period {
                PrintFPSPeriod::No => self.update_on_step(1000, true),
                PrintFPSPeriod::Second => self.update_on_step(1000, true),
                PrintFPSPeriod::FiveSecond => self.update_on_step(5000, true),
                PrintFPSPeriod::DecaSecond => self.update_on_step(10000, true),
                PrintFPSPeriod::Other(timespan_ms) => self.update_on_step(timespan_ms, true),
            }

            return true;
        } else {
            return false;
        }
    }

    pub fn update_on_step(&mut self, timespan_ms: u128, print: bool) {
        let time = Instant::now();
        if time.duration_since(self.sec_start).as_millis() >= timespan_ms {
            self.sec_start = time;
            if print {
                println!("FPS: {0}", self.frame_count as f32 / (timespan_ms as f32 / 1000.0));
            }
            self.frame_count = 0;
        }

        self.frame_count += 1;
    }

    pub fn should_draw_frame(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.frame_start).as_secs_f32() >= (1.0 / self.target_fps as f32) {
            self.frame_start = now;
            return true;
        } else {
            return false;
        }
    }
}
