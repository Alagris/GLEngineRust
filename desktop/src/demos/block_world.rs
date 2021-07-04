use crate::render_gl;

use crate::render_gl::data::{VertexTexNorTan, f32_f32_f32, VertexAlphaClr, VertexSizeAlphaClr};
use crate::render_gl::model::Model;
use crate::resources::Resources;
use failure::err_msg;
use sdl2::video::Window;
use sdl2::{Sdl, TimerSubsystem};
use crate::blocks::world::{World, WorldFaces, WorldChunks, Block};
use crate::render_gl::instanced_model::InstancedModel;
use crate::render_gl::instanced_array_model::InstancedArrayModel;
use crate::render_gl::array_model::{ArrayModel, Primitive};
use crate::render_gl::instanced_logical_model::InstancedLogicalModel;
use crate::render_gl::buffer::{DynamicBuffer, AnyBuffer};
use crate::render_gl::texture::Filter::Nearest;
use crate::blocks::block_properties::{STONE, GRASS, GLASS, CRAFTING, SLAB, ICE, LEAVES, TNT};
use crate::render_gl::uniform_buffer::UniformBuffer;

pub fn run(
    gl: gl::Gl,
    res: Resources,
    sdl: Sdl,
    window: Window,
    timer: TimerSubsystem,
) -> Result<(), failure::Error> {
    unsafe{
        gl.Enable(gl::CULL_FACE);
        gl.Enable(gl::BLEND);
        gl.Enable(gl::PROGRAM_POINT_SIZE);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    let shader_program = render_gl::Program::from_res(&gl, &res, "shaders/block")?;
    let orb_program = render_gl::Program::from_res(&gl, &res, "shaders/orb")?;
    let texture = render_gl::texture::Texture::from_res_with_filter("img/blocks.png", &res, Nearest, &gl)?;

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
    struct Matrices{
        mvp:glm::Mat4,
        mv:glm::Mat4
    }
    let mut matrices = UniformBuffer::new(Matrices{ mvp: glm::identity(), mv: glm::identity() },&gl);
    let texture_uniform = warn_ok(shader_program.get_uniform_texture("myTextureSampler").map_err(err_msg)).unwrap();
    let matrices_uniform = warn_ok(shader_program.get_uniform_std140::<Matrices,2>("Matrices").map_err(err_msg)).unwrap();
    shader_program.set_uniform_buffer(matrices_uniform,&matrices);
    let orb_matrices_uniform = warn_ok(orb_program.get_uniform_std140("Matrices").map_err(err_msg)).unwrap();
    orb_program.set_uniform_buffer(orb_matrices_uniform,&matrices);

    let mut world = World::<1,1>::new();
    world.set_block(1,1,1,STONE);
    world.set_block(1,2,1,STONE);
    world.set_block(1,1,2,STONE);
    world.set_block(15,0,15,GRASS);
    world.set_block(14,0,15,GRASS);
    world.set_block(13,0,15,GRASS);
    world.set_block(2,0,0, SLAB);
    world.set_block(3,0,0,ICE);
    world.set_block(4,0,0,LEAVES);
    world.set_block(3,0,0,TNT);
    world.set_block(3,1,3,CRAFTING);
    world.set_block(3,2,3,GLASS);
    world.set_block(3,3,3,GLASS);
    world.set_block(3,2,4,GLASS);
    world.set_block(3,3,4,GLASS);
    // world.compute_faces();
    let mut model_transparent = InstancedLogicalModel::new(DynamicBuffer::new(world.get_chunk_faces(0,0).transparent_as_slice(),&gl),&gl);
    let mut model_opaque = InstancedLogicalModel::new(DynamicBuffer::new(world.get_chunk_faces(0,0).opaque_as_slice(),&gl),&gl);
    let mut points = vec![VertexSizeAlphaClr::new((0.,0.,0.),64.,(0.,0.,0.,1.));64];
    let mut model_orbs = ArrayModel::new(DynamicBuffer::new(&points,&gl),&gl);
    let model_matrix = glm::identity::<f32, 4>();
    let mut rotation = glm::quat_identity();
    let mut location = glm::vec4(0f32, 2f32, 2f32, 0f32);
    let mut block_in_hand = 2u32;
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
        if input.number() > -1{
            block_in_hand = (input.number()+1) as u32
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
                world.ray_cast_place_block(location.as_slice(), ray_trace_vector.as_slice(), Block::new(block_in_hand));
            }
            model_transparent.ibo_mut().update(world.get_chunk_faces(0,0).transparent_as_slice());
            model_opaque.ibo_mut().update(world.get_chunk_faces(0,0).opaque_as_slice())
        }

        // draw triangle
        color_buffer.clear(&gl);
        shader_program.set_used();
        let location3 = &glm::vec4_to_vec3(&location);
        let v = glm::quat_to_mat4(&rotation) * glm::translation(&-location3);

        let m = model_matrix;
        matrices.mv = &v * m;
        matrices.mvp = projection_matrix * &matrices.mv;
        matrices.update();
        // shader_program.set_uniform_matrix4fv(mvp_uniform, mvp.as_slice());
        shader_program.set_uniform_texture(texture_uniform, &texture, 0);
        model_opaque.draw_instanced_triangles(0,/*one quad=2 triangles=6 vertices*/6, model_opaque.ibo().len());
        model_transparent.draw_instanced_triangles(0,/*one quad=2 triangles=6 vertices*/6, model_transparent.ibo().len());

        orb_program.set_used();
        model_orbs.draw_vertices(Primitive::Points, 64);

        window.gl_swap_window();
    }
    Ok(())
}
