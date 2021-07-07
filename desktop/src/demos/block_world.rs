use crate::render_gl;

use crate::render_gl::data::{VertexTexNorTan, f32_f32_f32, VertexAlphaClr, VertexSizeAlphaClr};
use crate::render_gl::model::Model;
use crate::resources::Resources;
use failure::err_msg;
use sdl2::video::Window;
use sdl2::{Sdl, TimerSubsystem};
use crate::blocks::{World, Block};
use crate::render_gl::instanced_model::InstancedModel;
use crate::render_gl::instanced_array_model::InstancedArrayModel;
use crate::render_gl::array_model::{ArrayModel, Primitive};
use crate::render_gl::instanced_logical_model::InstancedLogicalModel;
use crate::render_gl::buffer::{DynamicBuffer, AnyBuffer, ShaderStorageArrayBuffer};
use crate::render_gl::texture::Filter::Nearest;
use crate::blocks::block_properties::{STONE, GRASS, GLASS, CRAFTING, SLAB, ICE, LEAVES, TNT, BLOCKS, BEDROCK, DIRT, PLANK};
use crate::render_gl::uniform_buffer::UniformBuffer;
use crate::blocks::{Entities, Entity, ZombieVariant};
use crate::blocks::WorldSize;

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
    let mobs_program = render_gl::Program::from_res(&gl, &res, "shaders/mobs")?;
    let orb_program = render_gl::Program::from_res(&gl, &res, "shaders/orb")?;
    let texture = render_gl::texture::Texture::from_res_with_filter("img/blocks.png", &res, Nearest, &gl)?;
    let zombie_texture = render_gl::texture::Texture::from_res_with_filter("img/mobs.jpeg", &res, Nearest,&gl)?;

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
    let chunk_location_uniform = warn_ok(shader_program.get_uniform_vec3fv("chunk_location").map_err(err_msg)).unwrap();
    shader_program.set_uniform_buffer(matrices_uniform,&matrices);
    let orb_matrices_uniform = warn_ok(orb_program.get_uniform_std140("Matrices").map_err(err_msg)).unwrap();
    orb_program.set_uniform_buffer(orb_matrices_uniform,&matrices);
    let mobs_texture_uniform = warn_ok(mobs_program.get_uniform_texture("myTextureSampler").map_err(err_msg)).unwrap();
    let mobs_matrices_uniform = warn_ok(mobs_program.get_uniform_std140::<Matrices,2>("Matrices").map_err(err_msg)).unwrap();
    mobs_program.set_uniform_buffer(mobs_matrices_uniform,&matrices);
    let mut entities = Entities::new();
    entities.push(Entity::Zombie(ZombieVariant::Zombie), &glm::vec3(4.,0.,0.), &glm::quat_angle_axis(0f32, &glm::vec3(0., 1., 0.)));
    entities.push(Entity::Zombie(ZombieVariant::Steve), &glm::vec3(5.,0.,0.),&glm::quat_angle_axis(2f32, &glm::vec3(0., 1., 0.)));

    let mut world = World::new(2,2, &gl);
    world.blocks_mut().no_update_fill_level(0,1,BEDROCK);
    world.blocks_mut().no_update_fill_level(1,1,DIRT);
    world.blocks_mut().no_update_fill_level(2,1,GRASS);
    world.blocks_mut().no_update_outline(5,2,5,5,5,5,PLANK);
    world.compute_faces();
    world.gl_update_all_chunks();

    let mut model_mobs = InstancedLogicalModel::new(DynamicBuffer::new(entities.bone_slice(),&gl),&gl);
    let mut points = vec![VertexSizeAlphaClr::new((0.,0.,0.),64.,(0.,0.,0.,1.));64];
    let mut model_orbs = ArrayModel::new(DynamicBuffer::new(&points,&gl),&gl);
    let model_matrix = glm::identity::<f32, 4>();
    let mut rotation = glm::quat_identity();
    let mut location = glm::vec4(0f32, 2f32, 2f32, 0f32);
    let mut block_in_hand = 2u32;
    let movement_speed = 0.005f32;
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
            world.gl_update_all_chunks();
        }

        // draw triangle
        color_buffer.clear(&gl);

        let location3 = &glm::vec4_to_vec3(&location);
        let v = glm::quat_to_mat4(&rotation) * glm::translation(&-location3);

        let m = model_matrix;
        matrices.mv = &v * m;
        matrices.mvp = projection_matrix * &matrices.mv;
        matrices.update();

        entities.update(0,&glm::vec3((fps_counter.ticks() as f32/1000.).sin(),0.,0.), &glm::quat_angle_axis(fps_counter.ticks() as f32/1000., &glm::vec3(0., 1., 0.)));
        model_mobs.ibo_mut().update(entities.bone_slice());
        mobs_program.set_used();
        mobs_program.set_uniform_texture(mobs_texture_uniform, &zombie_texture, 0);
        model_mobs.draw_instanced_triangles(0,/*1 cube=6 quads=12 triangles=36 vertices*/36, model_mobs.ibo().len());

        shader_program.set_used();
        shader_program.set_uniform_texture(texture_uniform, &texture, 0);
        world.gl_draw(chunk_location_uniform,&shader_program);

        orb_program.set_used();
        model_orbs.draw_vertices(Primitive::Points, 64);


        window.gl_swap_window();
    }
    Ok(())
}
