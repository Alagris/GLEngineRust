use crate::render_gl;
use crate::render_gl::terrain::{iterate, Graph};
use crate::render_gl::texture::Cubemap;

use crate::render_gl::data::VertexTexNorTan;
use crate::render_gl::model::Model;
use crate::render_gl::model_data::ModelData;
use crate::resources::Resources;
use failure::err_msg;
use sdl2::video::Window;
use sdl2::{Sdl, TimerSubsystem};
use crate::render_gl::buffer::{BufferStaticDraw, BufferDynamicFixedLen};

pub fn run(
    gl: gl::Gl,
    res: Resources,
    sdl: Sdl,
    window: Window,
    timer: TimerSubsystem,
) -> Result<(), failure::Error> {
    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/procedural")?;
    let sky_box_program = render_gl::Program::from_res(&gl, &res, "shaders/skybox")?;

    let graph_w = 30;
    let graph_h = 30;
    let mut g = Graph::regular(graph_w, graph_h, 1f32);

    let mut model_terrain =
        ModelData::<_,BufferDynamicFixedLen>::new_from_tex_nor(g.to_ver_nor_tex().as_slice(), g.to_indices(), &gl)?; //Model::from_file("assets/model/susanne.obj", &gl)?;

    let model_cube = Model::<VertexTexNorTan,BufferStaticDraw>::from_res("model/cube.obj", &res, &gl)?;

    let texture = render_gl::texture::Texture::from_res("img/bricks2.jpg", &res, &gl)?;
    let texture_normal =
        render_gl::texture::Texture::from_res("img/bricks2_normal.jpg", &res, &gl)?;
    let texture_depth = render_gl::texture::Texture::from_res("img/bricks2_disp.jpg", &res, &gl)?;

    let sky_box_texture = Cubemap::from_res(
        [
            "img/right.jpg",
            "img/left.jpg",
            "img/top.jpg",
            "img/bottom.jpg",
            "img/front.jpg",
            "img/back.jpg",
        ],
        &res,
        &gl,
    )?;
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
    let sky_box_texture_uniform = warn_ok(
        sky_box_program
            .get_uniform_cube_texture("skybox")
            .map_err(err_msg),
    );


    let mv3x3_uniform = warn_ok(shader_program.get_uniform_matrix3fv("MV3x3").map_err(err_msg));
    let texture_uniform = warn_ok(
        shader_program
            .get_uniform_texture("myTextureSampler")
            .map_err(err_msg),
    );
    let texture_normal_uniform = warn_ok(
        shader_program
            .get_uniform_texture("normalMap")
            .map_err(err_msg),
    );
    let texture_depth_uniform = warn_ok(
        shader_program
            .get_uniform_texture("depthMap")
            .map_err(err_msg),
    );
    let mvp_uniform = warn_ok(shader_program.get_uniform_matrix4fv("MVP").map_err(err_msg));
    let light_source_uniform = warn_ok(
        shader_program
            .get_uniform_vec3fv("lightSource")
            .map_err(err_msg),
    );
    let light_color_uniform = warn_ok(
        shader_program
            .get_uniform_vec4fv("lightColor")
            .map_err(err_msg),
    );
    let m_uniform = warn_ok(shader_program.get_uniform_matrix4fv("M").map_err(err_msg));
    let v_uniform = warn_ok(shader_program.get_uniform_matrix4fv("V").map_err(err_msg));

    let susanne_model_matrix = glm::translation(&glm::vec3(2f32, 0f32, 2f32));

    let mut rotation = glm::quat_identity();
    let mut location = glm::vec4(0f32, 2f32, 2f32, 0f32);
    let mut light_location = glm::vec3(0f32, 2f32, 0f32);
    let mut light_strength = 20f32;
    let mut movement_speed = 0.01f32;
    let rotation_speed = 1f32;
    let mut fps_counter = render_gl::fps::FpsCounter::new(timer,60);
    let fov = 60f32 / 360f32 * std::f32::consts::PI * 2f32;
    let mut projection_matrix = glm::perspective(
        (viewport.w as f32) / (viewport.h as f32),
        fov,
        0.1f32,
        100f32,
    );
    let event_pump = sdl.event_pump().map_err(err_msg)?;
    let mut input = render_gl::input::Input::new(event_pump);

    fn rand(multiplier: f32) -> f32 {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        (time.subsec_nanos() as f32).cos() * multiplier
    }
    'main: loop {
        fps_counter.update();
        input.poll();

        if input.quit() {
            break;
        }
        if input.escape() {
            input.reset_escape();
            sdl.mouse()
                .set_relative_mouse_mode(!sdl.mouse().relative_mouse_mode());
        }
        if input.has_mouse_move() {
            let normalized_x = (input.mouse_move_xrel() as f32) / (viewport.w as f32)
                * fps_counter.delta_f32()
                * rotation_speed;
            let normalized_y = (input.mouse_move_yrel() as f32) / (viewport.h as f32)
                * fps_counter.delta_f32()
                * rotation_speed;
            rotation = glm::quat_angle_axis(normalized_y, &glm::vec3(1f32, 0f32, 0f32))
                * rotation
                * glm::quat_angle_axis(normalized_x, &glm::vec3(0f32, 1f32, 0f32));
        }
        if input.has_resize() {
            viewport.update_size(input.resize_w(), input.resize_h());
            viewport.set_used(&gl);
            projection_matrix = glm::perspective(
                (viewport.w as f32) / (viewport.h as f32),
                fov,
                0.1f32,
                20f32,
            );
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
            g = iterate(
                g,
                graph_w,
                graph_h,
                &|x, y| {
                    1. / ((x - 5.) * (x - 5.) + (y - 10.) * (y - 10.)).sqrt()
                        + 1. / ((x - 10.) * (x - 10.) + (y - 1.) * (y - 1.)).sqrt()
                        + 1. / ((x - 20.) * (x - 20.) + (y - 20.) * (y - 20.)).sqrt()
                },
                1f32,
                1f32,
            );
            model_terrain.update_from_tex_nor(g.to_ver_nor_tex().as_slice())?;
        }
        if input.is_2() {
            movement_speed += 0.00001f32;
        }
        if input.is_3() {
            movement_speed -= 0.00001f32;
        }
        let movement_vector =
            input.get_direction_unit_vector() * (movement_speed * fps_counter.delta_f32());
        let movement_vector = glm::quat_rotate_vec(&glm::quat_inverse(&rotation), &movement_vector);
        location += movement_vector;
        let location3 = &glm::vec4_to_vec3(&location);
        let v = glm::quat_to_mat4(&rotation) * glm::translation(&-location3);

        color_buffer.clear(&gl);

        unsafe {
            gl.DepthMask(gl::FALSE);
        }
        sky_box_program.set_used();
        sky_box_texture_uniform
            .map(|u| sky_box_program.set_uniform_cube_texture(u, &sky_box_texture, 0));
        sky_box_vp_uniform.map(|u| {
            sky_box_program.set_uniform_matrix4fv(
                u,
                (projection_matrix * &glm::mat3_to_mat4(&glm::mat4_to_mat3(&v))).as_slice(),
            )
        });
        model_cube.draw_triangles();
        unsafe {
            gl.DepthMask(gl::TRUE);
        }

        shader_program.set_used();

        let m = susanne_model_matrix;
        let mv = v * m;
        let mvp = projection_matrix * &mv;
        mvp_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, mvp.as_slice()));
        v_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, v.as_slice()));
        m_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, m.as_slice()));
        light_source_uniform
            .map(|u| shader_program.set_uniform_vec3fv(u, light_location.as_slice()));
        light_color_uniform
            .map(|u| shader_program.set_uniform_vec4fv(u, &[1f32, 1f32, 1f32, light_strength]));
        texture_uniform.map(|u| shader_program.set_uniform_texture(u, &texture, 0));
        texture_normal_uniform.map(|u| shader_program.set_uniform_texture(u, &texture_normal, 1));
        texture_depth_uniform.map(|u| shader_program.set_uniform_texture(u, &texture_depth, 2));
        mv3x3_uniform
            .map(|u| shader_program.set_uniform_matrix3fv(u, glm::mat4_to_mat3(&mv).as_slice()));
        model_terrain.draw_triangles();


        window.gl_swap_window();
    }

    Ok(())
}
