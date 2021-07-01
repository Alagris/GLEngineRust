use nalgebra_glm as glm;
use sdl2;

pub struct Input {
    event_pump: sdl2::EventPump,
    quit: bool,
    escape: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    forward: bool,
    backward: bool,
    resize_w: i32,
    resize_h: i32,
    has_resize: bool,
    mouse_move_x: i32,
    mouse_move_y: i32,
    mouse_move_xrel: i32,
    mouse_move_yrel: i32,
    has_mouse_move: bool,
    q: bool,
    e: bool,
    r: bool,
    no1: bool,
    no2: bool,
    no3: bool,
    no4: bool,
    no5: bool,
}

impl Input {
    pub fn new(event_pump: sdl2::EventPump) -> Input {
        Input {
            event_pump,
            quit: false,
            escape: false,
            left: false,
            right: false,
            up: false,
            down: false,
            forward: false,
            backward: false,
            resize_w: 0,
            resize_h: 0,
            has_resize: false,
            mouse_move_x: 0,
            mouse_move_y: 0,
            mouse_move_xrel: 0,
            mouse_move_yrel: 0,
            has_mouse_move: false,
            q: false,
            e: false,
            r: false,
            no1: false,
            no2: false,
            no3: false,
            no4: false,
            no5: false,
        }
    }
    pub fn poll(&mut self) {
        self.has_resize = false;
        self.has_mouse_move = false;
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => self.quit = true,
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    self.resize_w = w;
                    self.resize_h = h;
                    self.has_resize = true;
                }
                //                    viewport.update_size(w, h);
                //                    viewport.set_used(&gl);
                //                    projection_matrix = glm::perspective((viewport.w as f32) / (viewport.h as f32), fov, 0.1f32, 20f32);
                sdl2::event::Event::KeyDown { keycode, .. } => {
                    if let Some(k) = keycode {
                        match k {
                            sdl2::keyboard::Keycode::Num1 => {
                                self.no1 = true;
                            }
                            sdl2::keyboard::Keycode::Num2 => {
                                self.no2 = true;
                            }
                            sdl2::keyboard::Keycode::Num3 => {
                                self.no3 = true;
                            }
                            sdl2::keyboard::Keycode::Num4 => {
                                self.no4 = true;
                            }
                            sdl2::keyboard::Keycode::Num5 => {
                                self.no5 = true;
                            }
                            sdl2::keyboard::Keycode::R => {
                                self.r = true;
                            }
                            sdl2::keyboard::Keycode::E => {
                                self.e = true;
                            }
                            sdl2::keyboard::Keycode::Q => {
                                self.q = true;
                            }
                            sdl2::keyboard::Keycode::D => {
                                self.right = true;
                            }
                            sdl2::keyboard::Keycode::A => {
                                self.left = true;
                            }
                            sdl2::keyboard::Keycode::W => {
                                self.forward = true;
                            }
                            sdl2::keyboard::Keycode::S => {
                                self.backward = true;
                            }
                            sdl2::keyboard::Keycode::Space => {
                                self.up = true;
                            }
                            sdl2::keyboard::Keycode::LShift => {
                                self.down = true;
                            }
                            sdl2::keyboard::Keycode::Escape => {
                                self.escape = true;
                            }
                            _ => (),
                        }
                    }
                }
                sdl2::event::Event::KeyUp { keycode, .. } => {
                    if let Some(k) = keycode {
                        match k {
                            sdl2::keyboard::Keycode::Num1 => {
                                self.no1 = false;
                            }
                            sdl2::keyboard::Keycode::Num2 => {
                                self.no2 = false;
                            }
                            sdl2::keyboard::Keycode::Num3 => {
                                self.no3 = false;
                            }
                            sdl2::keyboard::Keycode::Num4 => {
                                self.no4 = false;
                            }
                            sdl2::keyboard::Keycode::Num5 => {
                                self.no5 = false;
                            }
                            sdl2::keyboard::Keycode::R => {
                                self.r = false;
                            }
                            sdl2::keyboard::Keycode::E => {
                                self.e = false;
                            }
                            sdl2::keyboard::Keycode::Q => {
                                self.q = false;
                            }
                            sdl2::keyboard::Keycode::D => {
                                self.right = false;
                            }
                            sdl2::keyboard::Keycode::A => {
                                self.left = false;
                            }
                            sdl2::keyboard::Keycode::W => {
                                self.forward = false;
                            }
                            sdl2::keyboard::Keycode::S => {
                                self.backward = false;
                            }
                            sdl2::keyboard::Keycode::Space => {
                                self.up = false;
                            }
                            sdl2::keyboard::Keycode::LShift => {
                                self.down = false;
                            }
                            sdl2::keyboard::Keycode::Escape => {
                                self.escape = false;
                            }
                            _ => (),
                        }
                    }
                }
                sdl2::event::Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    self.mouse_move_x = x;
                    self.mouse_move_y = y;
                    self.mouse_move_xrel = xrel;
                    self.mouse_move_yrel = yrel;
                    self.has_mouse_move = true;
                }
                _ => {}
            }
        }
    }
    pub fn has_resize(&self) -> bool {
        self.has_resize
    }
    pub fn resize_w(&self) -> i32 {
        self.resize_w
    }
    pub fn resize_h(&self) -> i32 {
        self.resize_h
    }
    pub fn has_mouse_move(&self) -> bool {
        self.has_mouse_move
    }
    pub fn mouse_move_x(&self) -> i32 {
        self.mouse_move_x
    }
    pub fn mouse_move_y(&self) -> i32 {
        self.mouse_move_y
    }
    pub fn mouse_move_xrel(&self) -> i32 {
        self.mouse_move_xrel
    }
    pub fn mouse_move_yrel(&self) -> i32 {
        self.mouse_move_yrel
    }
    pub fn quit(&self) -> bool {
        self.quit
    }
    pub fn get_direction_unit_vector(&self) -> glm::TVec4<f32> {
        let x_axis = -(self.left as i32) + (self.right as i32);
        let y_axis = -(self.down as i32) + (self.up as i32);
        let z_axis = -(self.forward as i32) + (self.backward as i32);
        let length = ((x_axis * x_axis + y_axis * y_axis + z_axis * z_axis) as f32).sqrt();
        if length == 0f32 {
            return glm::vec4(0f32, 0f32, 0f32, 0f32);
        }
        //normalized values:
        let x_axis = x_axis as f32 / length;
        let y_axis = y_axis as f32 / length;
        let z_axis = z_axis as f32 / length;
        glm::vec4(x_axis, y_axis, z_axis, 0f32)
    }
    pub fn escape(&self) -> bool {
        self.escape
    }
    pub fn reset_escape(&mut self) {
        self.escape = false;
    }

    pub fn is_q(&self) -> bool {
        self.q
    }
    pub fn is_e(&self) -> bool {
        self.e
    }
    pub fn is_r(&self) -> bool {
        self.r
    }
    pub fn is_1(&self) -> bool {
        self.no1
    }
    pub fn is_2(&self) -> bool {
        self.no2
    }
    pub fn is_3(&self) -> bool {
        self.no3
    }
    pub fn is_4(&self) -> bool {
        self.no4
    }
    pub fn is_5(&self) -> bool {
        self.no5
    }
}
