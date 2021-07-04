use crate::render_gl::data::{VertexAttrib, InstancePosId, f32_f32_f32};

const ZOMBIE_LEFT_LEG: u32 = 0;
const ZOMBIE_RIGHT_LEG: u32 = 1;
const ZOMBIE_TORSO: u32 = 2;
const ZOMBIE_HEAD: u32 = 3;
const ZOMBIE_LEFT_HAND: u32 = 4;
const ZOMBIE_RIGHT_HAND: u32 = 5;

pub enum Entity {
    Zombie
}
impl Entity{
    pub fn len(&self)->usize{
        match self{
            Entity::Zombie => 6
        }
    }
}

pub struct Entities {
    body_parts: Vec<InstancePosId>,
    entities: Vec<(Entity,usize)>,
}

impl Entities {
    pub fn new() -> Self {
        Self { body_parts: vec![], entities: vec![] }
    }
    pub fn as_slice(&self) -> &[InstancePosId] {
        &self.body_parts
    }
    pub fn push(&mut self, ent: Entity, pos:&[f32]) {
        let pos = f32_f32_f32::from((pos[0],pos[1],pos[2]));
        match ent {
            Entity::Zombie => {
                self.entities.push((ent,self.body_parts.len()));
                self.body_parts.push(InstancePosId::new(pos, ZOMBIE_LEFT_LEG));
                self.body_parts.push(InstancePosId::new(pos, ZOMBIE_RIGHT_LEG));
                self.body_parts.push(InstancePosId::new(pos, ZOMBIE_TORSO));
                self.body_parts.push(InstancePosId::new(pos, ZOMBIE_HEAD));
                self.body_parts.push(InstancePosId::new(pos, ZOMBIE_LEFT_HAND));
                self.body_parts.push(InstancePosId::new(pos, ZOMBIE_RIGHT_HAND));
            }
        }
    }
}
