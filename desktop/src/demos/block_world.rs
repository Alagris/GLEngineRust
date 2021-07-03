use crate::render_gl;

use crate::render_gl::data::VertexTexNorTan;
use crate::render_gl::model::Model;
use crate::resources::Resources;
use failure::err_msg;
use sdl2::video::Window;
use sdl2::{Sdl, TimerSubsystem};
use crate::blocks::world::{World, WorldFaces, WorldChunks, Block};
use crate::render_gl::instanced_model::InstancedModel;
use crate::render_gl::instanced_array_model::InstancedArrayModel;
use crate::render_gl::array_model::ArrayModel;
use crate::render_gl::instanced_logical_model::InstancedLogicalModel;
use crate::render_gl::buffer::{DynamicBuffer, AnyBuffer};

pub fn run(
    gl: gl::Gl,
    res: Resources,
    sdl: Sdl,
    window: Window,
    timer: TimerSubsystem,
) -> Result<(), failure::Error> {
    unsafe{
        gl.Enable(gl::CULL_FACE);
    }
    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/block")?;
    let texture = render_gl::texture::Texture::from_res("img/bricks2.jpg", &res, &gl)?;

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

    let texture_uniform = warn_ok(shader_program.get_uniform_texture("myTextureSampler").map_err(err_msg));
    let mvp_uniform = warn_ok(shader_program.get_uniform_matrix4fv("MVP").map_err(err_msg));
    let mut world = World::<1,1>::new();
    world.set_block(1,1,1,Block::new(12));
    world.set_block(1,2,1,Block::new(12));
    world.set_block(1,1,2,Block::new(12));
    world.set_block(15,0,15,Block::new(12));
    world.set_block(14,0,15,Block::new(12));
    world.set_block(13,0,15,Block::new(12));
    world.set_block(2,0,0,Block::new(12));
    world.set_block(3,0,0,Block::new(12));
    world.set_block(4,0,0,Block::new(12));
    world.set_block(3,0,0,Block::new(0));
    // world.compute_faces();
    let mut model = InstancedLogicalModel::new(DynamicBuffer::new(world.get_chunk_faces(0,0).as_slice(),&gl),&gl);
    let model_matrix = glm::identity::<f32, 4>();
    let mut rotation = glm::quat_identity();
    let mut location = glm::vec4(0f32, 2f32, 2f32, 0f32);
    let movement_speed = 0.001f32;
    let player_reach = 3f32;
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
        let movement_vector = input.get_direction_unit_vector() * movement_speed * fps_counter.delta_f32();
        let inverse_rotation = glm::quat_inverse(&rotation);
        let movement_vector = glm::quat_rotate_vec(&inverse_rotation, &movement_vector);
        location += movement_vector;
        if input.has_mouse_left_click()||input.has_mouse_right_click() {
            let ray_trace_vector = glm::vec4(0f32,0.,-player_reach, 0.);
            let ray_trace_vector = glm::quat_rotate_vec(&inverse_rotation, &ray_trace_vector);
            if input.has_mouse_left_click() {
                world.ray_cast_remove_block(location.as_slice(), ray_trace_vector.as_slice());
            }else{
                world.ray_cast_place_block(location.as_slice(), ray_trace_vector.as_slice(), Block::new(12));
            }
            model.ibo_mut().update(world.get_chunk_faces(0,0).as_slice())
        }

        // draw triangle
        color_buffer.clear(&gl);
        shader_program.set_used();
        let location3 = &glm::vec4_to_vec3(&location);
        let v = glm::quat_to_mat4(&rotation) * glm::translation(&-location3);

        let m = model_matrix;
        let mv = &v * m;
        let mvp = projection_matrix * &mv;
        mvp_uniform.map(|u| shader_program.set_uniform_matrix4fv(u, mvp.as_slice()));
        texture_uniform.map(|u| shader_program.set_uniform_texture(u, &texture, 0));
        model.draw_instanced_triangles(0,/*one quad=2 triangles=6 vertices*/6, model.ibo().len());


        window.gl_swap_window();
    }
    Ok(())
}
