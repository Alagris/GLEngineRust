use crate::render_gl;
use crate::render_gl::terrain::{Graph, iterate};
use crate::render_gl::texture::Cubemap;
use std::path::Path;
use failure::err_msg;
use crate::render_gl::model::Model;
use crate::resources::Resources;
use sdl2::{Sdl, TimerSubsystem};
use sdl2::video::Window;

pub fn run(gl:gl::Gl, res:Resources,sdl:Sdl,window:Window,timer:TimerSubsystem) -> Result<(), failure::Error> {
    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/shader")?;
    let normal_mapping_program = render_gl::Program::from_res(&gl, &res, "shaders/normal_mapping")?;
    let debug_shader_program = render_gl::Program::from_res(&gl, &res, "shaders/debug")?;

    let model = Model::from_res("model/wall.obj", &res,&gl)?;

    let model_susanne = Model::from_res("model/susanne.obj", &res,&gl)?;

    let texture = render_gl::texture::Texture::from_res("img/bricks2.jpg", &res, &gl)?;
    let texture_normal = render_gl::texture::Texture::from_res("img/bricks2_normal.jpg", &res, &gl)?;
    let texture_depth = render_gl::texture::Texture::from_res("img/bricks2_disp.jpg", &res, &gl)?;

// set up shared state for window
    let mut viewport = render_gl::Viewport::for_window(900, 700);
    viewport.set_used(&gl);

    let color_buffer: render_gl::color_buffer::ColorBuffer = (0.3, 0.3, 0.5, 1.0).into();
    color_buffer.set_used(&gl);

    fn warn_ok<T>(result:Result<T, failure::Error>) -> Option<T>{
        match result{
            Ok(t) => Some(t),
            Err(err) => {println!("{}",err); None}
        }
    }
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


    let mut susanne_model_matrix = glm::translation(&glm::vec3(2f32,0f32,2f32));

    let mut model_matrix = glm::identity::<f32, 4>();
    let mut rotation = glm::quat_identity();
    let mut location = glm::vec4(0f32, 2f32, 2f32, 0f32);
    let mut light_location = glm::vec3(0f32, 2f32, 0f32);
    let mut light_strength = 1f32;
    let movement_speed = 0.001f32;
    let mut normal_length = 1f32;
    let rotation_speed = 1f32;
    let mut fps_counter = render_gl::fps::FpsCounter::new(timer);
    let fov = 60f32 / 360f32 * std::f32::consts::PI * 2f32;
    let mut projection_matrix = glm::perspective((viewport.w as f32) / (viewport.h as f32), fov, 0.1f32, 100f32);
    let mut velocity = glm::vec3(0f32, 0f32, 0f32);
    let event_pump = sdl.event_pump().map_err(err_msg)?;
    let mut input = render_gl::input::Input::new(event_pump);
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
        let movement_vector = input.get_direction_unit_vector() * (movement_speed * fps_counter.delta_f32());
        let movement_vector = glm::quat_rotate_vec(&glm::quat_inverse(&rotation), &movement_vector);
        location += movement_vector;
// draw triangle
        color_buffer.clear(&gl);
        shader_program.set_used();
        let location3 = &glm::vec4_to_vec3(&location);
        let v = glm::quat_to_mat4(&rotation) * glm::translation(&-location3);

        let m = susanne_model_matrix;
        let mv = &v * m;
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

        debug_shader_program.set_used();
        debug_mvp_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, mvp.as_slice()));
        debug_m_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, model_matrix.as_slice()));
        debug_v_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, v.as_slice()));
        debug_p_uniform.map(|u| debug_shader_program.set_uniform_matrix4fv(u, projection_matrix.as_slice()));
        debug_normal_length.map(|u| debug_shader_program.set_uniform_1f(u, normal_length));
        model_susanne.bind();
        model_susanne.draw_triangles();


        let m = model_matrix;
        let mv = v * m;
        let mvp = projection_matrix * &mv;
        normal_mapping_program.set_used();
        mvp_parallax_uniform.map(|u| normal_mapping_program.set_uniform_matrix4fv(u, mvp.as_slice()));
//        shader_program.set_uniform_matrix4fv(mv_uniform, mv.as_slice());
        v_parallax_uniform.map(|u| normal_mapping_program.set_uniform_matrix4fv(u, v.as_slice()));
        m_parallax_uniform.map(|u| normal_mapping_program.set_uniform_matrix4fv(u, m.as_slice()));
//        shader_program.set_uniform_vec3fv(view_pos_uniform, location3.as_slice());
        light_source_parallax_uniform.map(|u| normal_mapping_program.set_uniform_vec3fv(u, light_location.as_slice()));
        light_color_parallax_uniform.map(|u| normal_mapping_program.set_uniform_vec4fv(u, &[1f32, 1f32, 1f32, light_strength]));
        texture_parallax_uniform.map(|u| normal_mapping_program.set_uniform_texture(u, &texture, 0));
        texture_normal_parallax_uniform.map(|u| normal_mapping_program.set_uniform_texture(u, &texture_normal, 1));
        texture_depth_parallax_uniform.map(|u| normal_mapping_program.set_uniform_texture(u, &texture_depth, 2));
        mv3x3_parallax_uniform.map(|u| normal_mapping_program.set_uniform_matrix3fv(u, glm::mat4_to_mat3(&mv).as_slice()));
        model.bind();
        model.draw_triangles();






        window.gl_swap_window();
    }
    Ok(())
}