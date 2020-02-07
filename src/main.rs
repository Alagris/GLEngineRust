extern crate gl;
extern crate sdl2;
extern crate nalgebra_glm as glm;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;
extern crate image;
extern crate genmesh;
extern crate rand;

pub mod render_gl;
pub mod resources;

use failure::err_msg;
use crate::resources::Resources;
use std::path::Path;
use render_gl::terrain::*;
use render_gl::data::{self, Vertex, VertexTex, VertexTexCol, VertexTexNor};
use render_gl::buffer::{ArrayBuffer, ElementArrayBuffer, VertexArray};
use render_gl::{color_buffer, Viewport, physics_model::{BallPhysicsModel, PhysicsModel, CollisionVector}};
use std::ffi::{CString, CStr};

use std::fs::File;
use std::io::BufReader;
use core::ptr;
use collada::document::ColladaDocument;
use collada::{Polylist, PrimitiveElement, Shape, Triangles};

use render_gl::model::Model;

use genmesh::{MapToVertices, Polygon};
use glm::U4;
use std::char::decode_utf16;
use crate::render_gl::texture::Cubemap;

fn main() {
    if let Err(e) = run() {
        println!("Error! {}", failure_to_string(e));
    }
}

extern "system" fn message_callback(source: gl::types::GLenum,
                                    type_of_error: gl::types::GLenum,
                                    id: gl::types::GLuint,
                                    severity: gl::types::GLenum,
                                    length: gl::types::GLsizei,
                                    message: *const gl::types::GLchar,
                                    user_param: *mut gl::types::GLvoid) {
    let type_of_error = match type_of_error {
        gl::NO_ERROR => String::from("GL_NO_ERROR"),
        gl::INVALID_VALUE => String::from("GL_INVALID_VALUE"),
        gl::INVALID_OPERATION => String::from("GL_INVALID_OPERATION"),
        gl::STACK_OVERFLOW => String::from("GL_STACK_OVERFLOW"),
        gl::STACK_UNDERFLOW => String::from("GL_STACK_UNDERFLOW"),
        gl::OUT_OF_MEMORY => String::from("GL_OUT_OF_MEMORY"),
        gl::INVALID_FRAMEBUFFER_OPERATION => String::from("GL_INVALID_FRAMEBUFFER_OPERATION"),
        gl::CONTEXT_LOST => String::from("GL_CONTEXT_LOST"),
        gl::DEBUG_SOURCE_API => String::from("DEBUG_SOURCE_API"),
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => String::from("DEBUG_SOURCE_WINDOW_SYSTEM"),
        gl::DEBUG_SOURCE_SHADER_COMPILER => String::from("DEBUG_SOURCE_SHADER_COMPILER"),
        gl::DEBUG_SOURCE_THIRD_PARTY => String::from("DEBUG_SOURCE_THIRD_PARTY"),
        gl::DEBUG_SOURCE_APPLICATION => String::from("DEBUG_SOURCE_APPLICATION"),
        gl::DEBUG_SOURCE_OTHER => String::from("DEBUG_SOURCE_OTHER"),
        gl::DEBUG_TYPE_ERROR => String::from("DEBUG_TYPE_ERROR"),
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => String::from("DEBUG_TYPE_DEPRECATED_BEHAVIOR"),
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => String::from("DEBUG_TYPE_UNDEFINED_BEHAVIOR"),
        gl::DEBUG_TYPE_PORTABILITY => String::from("DEBUG_TYPE_PORTABILITY"),
        gl::DEBUG_TYPE_PERFORMANCE => String::from("DEBUG_TYPE_PERFORMANCE"),
        gl::DEBUG_TYPE_MARKER => String::from("DEBUG_TYPE_MARKER"),
        gl::DEBUG_TYPE_PUSH_GROUP => String::from("DEBUG_TYPE_PUSH_GROUP"),
        gl::DEBUG_TYPE_POP_GROUP => String::from("DEBUG_TYPE_POP_GROUP"),
        gl::DEBUG_TYPE_OTHER => String::from("DEBUG_TYPE_OTHER"),
        x => x.to_string()
    };
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => String::from("HIGH"),
        gl::DEBUG_SEVERITY_MEDIUM => String::from("MEDIUM"),
        gl::DEBUG_SEVERITY_LOW => String::from("LOW"),
        gl::DEBUG_SEVERITY_NOTIFICATION => String::from("NOTIFICATION"),
        x => x.to_string()
    };
    let msg = if let Ok(s) = unsafe { CStr::from_ptr(message) }.to_str() { s } else { "invalid c string!" };
    println!("GL CALLBACK: type = {}, severity = {}, message = {}\n", type_of_error, severity, msg);
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;
    let mut timer = sdl.timer().map_err(err_msg)?;
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("Game", 900, 700)
        .opengl()
        .resizable()
        .build()?;
    sdl.mouse().set_relative_mouse_mode(true);
    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });
    unsafe {
        gl.DepthFunc(gl::LESS);
        gl.Enable(gl::DEPTH_TEST);
//        gl.Enable(gl::CULL_FACE);
        gl.Enable(gl::DEBUG_OUTPUT);
        gl.DebugMessageCallback(message_callback, 0 as *const gl::types::GLvoid);
    }
    // set up shader program

    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/procedural")?;
    let sky_box_program = render_gl::Program::from_res(&gl, &res, "shaders/skybox")?;
    let normal_mapping_program = render_gl::Program::from_res(&gl, &res, "shaders/normal_mapping")?;
    let debug_shader_program = render_gl::Program::from_res(&gl, &res, "shaders/debug")?;

    let graph_w = 30;
    let graph_h = 30;
    let mut g = Graph::regular(graph_w, graph_h, 1f32);
    let model = Model::from_file("assets/model/wall.obj", &gl)?;

    let mut model_susanne = Model::new_from_ver_nor_tex((g.to_ver_nor_tex(), g.to_indices()), &gl)?; //Model::from_file("assets/model/susanne.obj", &gl)?;

    let model_ball = Model::from_file("assets/model/ball.obj", &gl)?;

    let model_cube = Model::from_file("assets/model/cube.obj", &gl)?;

    let texture = render_gl::texture::Texture::new(&Path::new("assets/img/bricks2.jpg"), &gl)?;
    let texture_normal = render_gl::texture::Texture::new(&Path::new("assets/img/bricks2_normal.jpg"), &gl)?;
    let texture_depth = render_gl::texture::Texture::new(&Path::new("assets/img/bricks2_disp.jpg"), &gl)?;

    let sky_box_texture = Cubemap:: new([
                                           &Path::new("assets/img/right.jpg"),
                                           &Path::new("assets/img/left.jpg"),
                                           &Path::new("assets/img/top.jpg"),
                                           &Path::new("assets/img/bottom.jpg"),
                                           &Path::new("assets/img/front.jpg"),
                                           &Path::new("assets/img/back.jpg")
                                       ], &gl)?;
// set up shared state for window
    let mut viewport = render_gl::Viewport::for_window(900, 700);
    viewport.set_used(&gl);

    let color_buffer: render_gl::color_buffer::ColorBuffer = (0.3, 0.3, 0.5, 1.0).into();
    color_buffer.set_used(&gl);

    fn warn_ok<T>(result: Result<T, failure::Error>) -> Option<T> {
        match result {
            Ok(t) => Some(t),
            Err(err) => {
                println!("{}", err);
                None
            }
        }
    }
    let sky_box_vp_uniform = warn_ok(sky_box_program.get_uniform_matrix4fv("VP").map_err(err_msg));
    let sky_box_texture_uniform = warn_ok(sky_box_program.get_uniform_cube_texture("skybox").map_err(err_msg));

    let debug_mvp_uniform = warn_ok(debug_shader_program.get_uniform_matrix4fv("MVP").map_err(err_msg));
    let debug_v_uniform = warn_ok(debug_shader_program.get_uniform_matrix4fv("V").map_err(err_msg));
    let debug_p_uniform = warn_ok(debug_shader_program.get_uniform_matrix4fv("P").map_err(err_msg));
    let debug_m_uniform = warn_ok(debug_shader_program.get_uniform_matrix4fv("M").map_err(err_msg));
    let debug_normal_length = warn_ok(debug_shader_program.get_uniform_1f("normalLength").map_err(err_msg));

    let mv3x3_uniform = warn_ok(shader_program.get_uniform_matrix3fv("MV3x3").map_err(err_msg));
    let texture_uniform = warn_ok(shader_program.get_uniform_texture("myTextureSampler").map_err(err_msg));
    let texture_normal_uniform = warn_ok(shader_program.get_uniform_texture("normalMap").map_err(err_msg));
    let texture_depth_uniform = warn_ok(shader_program.get_uniform_texture("depthMap").map_err(err_msg));
    let mvp_uniform = warn_ok(shader_program.get_uniform_matrix4fv("MVP").map_err(err_msg));
    let light_source_uniform = warn_ok(shader_program.get_uniform_vec3fv("lightSource").map_err(err_msg));
    let light_color_uniform = warn_ok(shader_program.get_uniform_vec4fv("lightColor").map_err(err_msg));
//    let view_pos_uniform = shader_program.get_uniform_vec3fv("viewPos").map_err(err_msg)?;
    let m_uniform = warn_ok(shader_program.get_uniform_matrix4fv("M").map_err(err_msg));
//    let mv_uniform = shader_program.get_uniform_matrix4fv("MV").map_err(err_msg)?;
    let v_uniform = warn_ok(shader_program.get_uniform_matrix4fv("V").map_err(err_msg));


    let mv3x3_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_matrix3fv("MV3x3").map_err(err_msg));
    let texture_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_texture("myTextureSampler").map_err(err_msg));
    let texture_normal_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_texture("normalMap").map_err(err_msg));
    let texture_depth_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_texture("depthMap").map_err(err_msg));
    let mvp_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_matrix4fv("MVP").map_err(err_msg));
    let light_source_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_vec3fv("lightSource").map_err(err_msg));
    let light_color_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_vec4fv("lightColor").map_err(err_msg));
//    let view_pos_uniform = shader_program.get_uniform_vec3fv("viewPos").map_err(err_msg)?;
    let m_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_matrix4fv("M").map_err(err_msg));
//    let mv_uniform = shader_program.get_uniform_matrix4fv("MV").map_err(err_msg)?;
    let v_parallax_uniform = warn_ok(normal_mapping_program.get_uniform_matrix4fv("V").map_err(err_msg));


    let mut susanne_model_matrix = glm::translation(&glm::vec3(2f32, 0f32, 2f32));


    let mut model_matrix = glm::identity::<f32, U4>();
    let mut rotation = glm::quat_identity();
    let mut location = glm::vec4(0f32, 2f32, 2f32, 0f32);
    let mut light_location = glm::vec3(0f32, 2f32, 0f32);
    let mut light_strength = 20f32;
    let mut movement_speed = 0.01f32;
    let mut normal_length = 1f32;
    let rotation_speed = 1f32;
    let mut fps_counter = render_gl::fps::FpsCounter::new(timer);
    let fov = 60f32 / 360f32 * std::f32::consts::PI * 2f32;
    let mut projection_matrix = glm::perspective((viewport.w as f32) / (viewport.h as f32), fov, 0.1f32, 100f32);
    let event_pump = sdl.event_pump().map_err(err_msg)?;
    let mut input = render_gl::input::Input::new(event_pump);

    fn rand(multiplier: f32) -> f32 {
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
        (time.subsec_nanos() as f32).cos() * multiplier
    }
//    let ball_scale = 1f32;
//    let ball_speed = movement_speed*1f32;
//    let mut balls = vec![
//        BallPhysicsModel::new(PhysicsModel::new(rand(4f32), rand(4f32), rand(4f32), ball_scale, ball_scale, ball_scale, rand(ball_speed) , rand(ball_speed), rand(ball_speed), 0f32, 0f32, 0f32, rand(4f32).abs().min(1f32)), ball_scale),
//        BallPhysicsModel::new(PhysicsModel::new(rand(4f32), rand(4f32), rand(4f32), ball_scale, ball_scale, ball_scale, rand(ball_speed) , rand(ball_speed), rand(ball_speed), 0f32, 0f32, 0f32, rand(4f32).abs().min(1f32)), ball_scale),
//        BallPhysicsModel::new(PhysicsModel::new(rand(4f32), rand(4f32), rand(4f32), ball_scale, ball_scale, ball_scale, rand(ball_speed) , rand(ball_speed), rand(ball_speed), 0f32, 0f32, 0f32, rand(4f32).abs().min(1f32)), ball_scale),
//        BallPhysicsModel::new(PhysicsModel::new(rand(4f32), rand(4f32), rand(4f32), ball_scale, ball_scale, ball_scale, rand(ball_speed) , rand(ball_speed), rand(ball_speed), 0f32, 0f32, 0f32, rand(4f32).abs().min(1f32)), ball_scale),
//    ];
    //let balls_bounding_box = (glm::vec3(-4f32, -4f32, -4f32), glm::vec3(4f32, 4f32, 4f32));
    'main: loop {
        fps_counter.update();
        input.poll();

        if input.quit() {
            break;
        }
        if input.escape() {
            input.reset_escape();
            sdl.mouse().set_relative_mouse_mode(!sdl.mouse().relative_mouse_mode());
        }
        if input.has_mouse_move() {
            let normalized_x = (input.mouse_move_xrel() as f32) / (viewport.w as f32) * fps_counter.delta_f32() * rotation_speed;
            let normalized_y = (input.mouse_move_yrel() as f32) / (viewport.h as f32) * fps_counter.delta_f32() * rotation_speed;
            rotation = glm::quat_angle_axis(normalized_y, &glm::vec3(1f32, 0f32, 0f32)) * rotation * glm::quat_angle_axis(normalized_x, &glm::vec3(0f32, 1f32, 0f32));
        }
        if input.has_resize() {
            viewport.update_size(input.resize_w(), input.resize_h());
            viewport.set_used(&gl);
            projection_matrix = glm::perspective((viewport.w as f32) / (viewport.h as f32), fov, 0.1f32, 20f32);
        }
        if input.is_q() {
            light_strength += 0.01f32;
        }
        if input.is_e() {
            light_strength -= 0.01f32;
        }
        if input.is_r() {
            light_location.x = location.x;
            light_location.y = location.y;
            light_location.z = location.z;
        }
        if input.is_1() {
            // &|x, y| 1f32 / (x + y).ln()
            // &|x, y|  (x + y).ln()
            // &|x, y| ((x -5.)*(x-5.)+ (y-10.)*(y-10.)).sqrt()
            // &|x, y| 1./((x -5.)*(x-5.)+ (y-10.)*(y-10.)).sqrt() + 1./((x -10.)*(x-10.)+ (y-1.)*(y-1.)).sqrt() +  1./((x -20.)*(x-20.)+ (y-20.)*(y-20.)).sqrt()
            g = iterate(g, graph_w, graph_h,  &|x, y| 1./((x -5.)*(x-5.)+ (y-10.)*(y-10.)).sqrt() + 1./((x -10.)*(x-10.)+ (y-1.)*(y-1.)).sqrt() +  1./((x -20.)*(x-20.)+ (y-20.)*(y-20.)).sqrt(), 1f32, 1f32);
            model_susanne.update_vbo(g.to_ver_nor_tex());
        }
        if input.is_2() {
            movement_speed += 0.00001f32;
        }
        if input.is_3() {
            movement_speed -= 0.00001f32;
        }
//        for ball in balls.iter_mut() {
//
//            let ball = ball.model_mut();
//            let x = ball.location().x;
//            let y = ball.location().y;
//            let z = ball.location().z;
//            println!("{} {} {} / {} {} {}",x,y,z,ball.velocity().x,ball.velocity().y,ball.velocity().z);
//            let bottom_left_back_corner = balls_bounding_box.0;
//            let top_right_front_corner = balls_bounding_box.1;
//            let x_wall = glm::vec3(1f32,0f32,0f32);
//            let y_wall = glm::vec3(0f32,1f32,0f32);
//            let z_wall = glm::vec3(0f32,0f32,1f32);
//            if x < bottom_left_back_corner.x{
//                ball.bounce(&CollisionVector::new(x_wall));
//            }
//            if y < bottom_left_back_corner.y{
//                ball.bounce(&CollisionVector::new(y_wall));
//            }
//            if z < bottom_left_back_corner.z{
//                ball.bounce(&CollisionVector::new(z_wall));
//            }
//            if x > top_right_front_corner.x{
//                ball.bounce(&CollisionVector::new(x_wall));
//            }
//            if y > top_right_front_corner.y{
//                ball.bounce(&CollisionVector::new(y_wall));
//            }
//            if z > top_right_front_corner.z{
//                ball.bounce(&CollisionVector::new(z_wall));
//            }
//
//            ball.update(fps_counter.delta_f32());
//
//        }
//        for ball_a in 0..(balls.len() - 1) {
//            for ball_b in (ball_a + 1)..balls.len() {
//                let slice = &mut balls[ball_a..(ball_b+1)];
//                let (a,rest) = slice.split_first_mut().unwrap();
//                let b = rest.last_mut().unwrap();
//                if let Some(collision_vector) = a.collision_vector(b) {
//                    a.model_mut().collide(b.model_mut(), &collision_vector);
//                }
//            }
//        }
        let movement_vector = input.get_direction_unit_vector() * (movement_speed * fps_counter.delta_f32());
        let movement_vector = glm::quat_rotate_vec(&glm::quat_inverse(&rotation), &movement_vector);
        location += movement_vector;
        let location3 = &glm::vec4_to_vec3(&location);
        let v = glm::quat_to_mat4(&rotation) * glm::translation(&-location3);

        color_buffer.clear(&gl);

        unsafe {
            gl.DepthMask(gl::FALSE);
        }
        sky_box_program.set_used();
        sky_box_texture_uniform.map(|u| sky_box_program.set_uniform_cube_texture(u,&sky_box_texture,0));
        sky_box_vp_uniform.map(|u| sky_box_program.set_uniform_matrix4fv(u,(projection_matrix * &glm::mat3_to_mat4(&glm::mat4_to_mat3(&v))).as_slice()));
        model_cube.bind();
        model_cube.draw_triangles();
        unsafe {
            gl.DepthMask(gl::TRUE);
        }


        shader_program.set_used();

        let m = susanne_model_matrix;
        let mv = v * m;
        let mvp = projection_matrix * &mv;
        mvp_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, mvp.as_slice()));
//        shader_program.set_uniform_matrix4fv(mv_uniform, mv.as_slice());
        v_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, v.as_slice()));
        m_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, m.as_slice()));
//        shader_program.set_uniform_vec3fv(view_pos_uniform, location3.as_slice());
        light_source_uniform.map(|u| shader_program.set_uniform_vec3fv(u, light_location.as_slice()));
        light_color_uniform.map(|u| shader_program.set_uniform_vec4fv(u, &[1f32, 1f32, 1f32, light_strength]));
        texture_uniform.map(|u| shader_program.set_uniform_texture(u, &texture, 0));
        texture_normal_uniform.map(|u| shader_program.set_uniform_texture(u, &texture_normal, 1));
        texture_depth_uniform.map(|u| shader_program.set_uniform_texture(u, &texture_depth, 2));
        mv3x3_uniform.map(|u| shader_program.set_uniform_matrix3fv(u, glm::mat4_to_mat3(&mv).as_slice()));
        model_susanne.bind();
        model_susanne.draw_triangles();

//        for ball in &balls {
//            let m = ball.model().to_mat4();
//            let mv = v * m;
//            let mvp = projection_matrix * &mv;
//            mvp_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, mvp.as_slice()));
////        shader_program.set_uniform_matrix4fv(mv_uniform, mv.as_slice());
//            m_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, m.as_slice()));
//            mv3x3_uniform.map(|u| shader_program.set_uniform_matrix3fv(u, glm::mat4_to_mat3(&mv).as_slice()));
//            model_ball.bind();
//            model_ball.draw_triangles();
//        }
/*
        debug_shader_program.set_used();
        debug_mvp_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, mvp.as_slice()));
        debug_m_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, model_matrix.as_slice()));
        debug_v_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, v.as_slice()));
        debug_p_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, projection_matrix.as_slice()));
        debug_normal_length.map(|u| debug_shader_program.set_uniform_1f(u, normal_length));
        model_susanne.bind();
        model_susanne.draw_triangles();
*/



        window.gl_swap_window();
    }

    Ok(())
}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e
        .iter_chain()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .enumerate()
        {
            if i > 0 {
                let _ = writeln!(&mut result, "   Which caused the following issue:");
            }
            let _ = write!(&mut result, "{}", cause);
            if let Some(backtrace) = cause.backtrace() {
                let backtrace_str = format!("{}", backtrace);
                if backtrace_str.len() > 0 {
                    let _ = writeln!(&mut result, " This happened at {}", backtrace);
                } else {
                    let _ = writeln!(&mut result);
                }
            } else {
                let _ = writeln!(&mut result);
            }
        }

    result
}