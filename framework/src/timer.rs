use std::thread;
use std::time::{Instant, Duration};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct Timer<const N_CNT: usize = 50> {
    base_time_point: Instant,
    prev_time_point: Instant,
    curr_time_point: Instant,
    stop_time_point: Option<Instant>,
    
    fps_elapsed_time: f32,
    elapsed_time_in_sec: f32,

    frame_times: [f32; N_CNT],
    sample_count: usize,

    curr_frame_rate: u32,
    frame_per_seconds: u32,

    is_stopped: bool,
}

impl<const N_CNT: usize> Timer<N_CNT> {
    #[inline]
    pub fn new() -> Self {
        let time_point = Instant::now();
        Self {
            base_time_point: time_point,
            prev_time_point: time_point,
            curr_time_point: time_point,
            stop_time_point: None,
            fps_elapsed_time: 0.0,
            elapsed_time_in_sec: 0.0,
            frame_times: [0.0; N_CNT],
            sample_count: 0,
            curr_frame_rate: 0,
            frame_per_seconds: 0,
            is_stopped: false,
        }
    }

    #[inline]
    pub fn tick(&mut self, vsync: Option<u32>) {
        if self.is_stopped {
            return;
        }

        self.curr_time_point = Instant::now();
        let mut elapsed_time_in_sec = self.curr_time_point
            .saturating_duration_since(self.prev_time_point)
            .as_secs_f32();

        if let Some(vsync) = vsync {
            while elapsed_time_in_sec < (1.0 / vsync as f32) {
                if (1.0 / vsync as f32) - elapsed_time_in_sec > Duration::from_millis(64).as_secs_f32() {
                    thread::yield_now();
                }
                self.curr_time_point = Instant::now();
                elapsed_time_in_sec = self.curr_time_point
                    .saturating_duration_since(self.prev_time_point)
                    .as_secs_f32();
            }
        }
        self.prev_time_point = self.curr_time_point;

        if (elapsed_time_in_sec - self.elapsed_time_in_sec).abs() < 1.0 {
            self.frame_times.copy_within(0..(N_CNT - 1), 1);
            self.frame_times[0] = elapsed_time_in_sec;
            self.sample_count = N_CNT.min(self.sample_count + 1);
        }

        self.frame_per_seconds += 1;
        self.fps_elapsed_time += elapsed_time_in_sec;
        if self.fps_elapsed_time > 1.0 {
            self.curr_frame_rate = self.frame_per_seconds;
            self.frame_per_seconds = 0;
            self.fps_elapsed_time = 0.0;
        }

        self.elapsed_time_in_sec = self.frame_times.iter().sum();
        self.elapsed_time_in_sec /= 1_f32.max(self.sample_count as f32);
    }

    #[inline]
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    #[inline]
    pub fn pause(&mut self) {
        let time_point = Instant::now();
        self.stop_time_point = Some(time_point);
        self.prev_time_point = time_point;
        self.curr_time_point = time_point;
        self.fps_elapsed_time = 0.0;
        self.elapsed_time_in_sec = 0.0;
        self.frame_times.fill(0.0);
        self.sample_count = 0;
        self.curr_frame_rate = 0;
        self.frame_per_seconds = 0;
        self.is_stopped = true;
    }

    #[inline]
    pub fn resume(&mut self) -> f32 {
        if let Some(stop_time_point) = self.stop_time_point {
            let time_point = Instant::now();
            let duration_time_in_sec = time_point
                .saturating_duration_since(stop_time_point)
                .as_secs_f32();

            self.stop_time_point = None;
            self.prev_time_point = time_point;
            self.curr_time_point = time_point;
            self.is_stopped = false;

            return duration_time_in_sec;
        }
        return 0.0;
    }

    #[inline]
    pub fn is_stopped(&self) -> bool {
        self.is_stopped
    }

    #[inline]
    pub fn get_frame_rate(&self) -> u32 {
        self.curr_frame_rate
    }

    #[inline]
    pub fn get_elapsed_time_in_sec(&self) -> f32 {
        self.elapsed_time_in_sec
    }

    #[inline]
    pub fn get_total_time_in_sec(&self) -> f32 {
        self.curr_time_point
            .saturating_duration_since(self.base_time_point)
            .as_secs_f32()
    }
}