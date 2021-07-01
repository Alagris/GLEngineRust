use sdl2;

pub struct FpsCounter {
    timer: sdl2::TimerSubsystem,
    previous: u32,
    delta: u32,
}

impl FpsCounter {
    pub fn new(timer: sdl2::TimerSubsystem) -> Self {
        let previous = timer.ticks();
        Self {
            timer,
            previous,
            delta: 0,
        }
    }
    pub fn update(&mut self) {
        let current = self.timer.ticks();
        self.delta = current - self.previous;
        self.previous = current;
    }
    pub fn delta(&self) -> u32 {
        self.delta
    }
    pub fn delta_f32(&self) -> f32 {
        self.delta as f32
    }
}
