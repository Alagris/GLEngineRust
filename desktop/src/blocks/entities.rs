use crate::render_gl::data::{VertexAttrib, InstancePosId, f32_f32_f32, InstancePosBytes};

const ZOMBIE_LEFT_LEG: u8 = 0;
const ZOMBIE_RIGHT_LEG: u8 = 1;
const ZOMBIE_TORSO: u8 = 2;
const ZOMBIE_HEAD: u8 = 3;
const ZOMBIE_LEFT_HAND: u8 = 4;
const ZOMBIE_RIGHT_HAND: u8 = 5;
#[derive(Clone,Copy,PartialEq,Eq)]
pub enum ZombieVariant{
    Zombie = 0,
    Steve = 1,
}

pub enum Entity {
    Zombie(ZombieVariant)
}
impl Entity{
    pub fn len(&self)->usize{
        match self{
            Entity::Zombie(_) => 6
        }
    }
}

pub struct Entities {
    body_parts: Vec<InstancePosBytes>,
    // entities: Vec<(Entity,usize)>,
}

impl Entities {
    pub fn new() -> Self {
        Self { body_parts: vec![] }//, entities: vec![]
    }
    pub fn as_slice(&self) -> &[InstancePosBytes] {
        &self.body_parts
    }
    pub fn push(&mut self, ent: Entity, pos:&[f32]) {
        let pos = f32_f32_f32::from((pos[0],pos[1],pos[2]));
        match ent {
            Entity::Zombie(variant) => {
                // self.entities.push((ent,self.body_parts.len()));
                self.body_parts.push(InstancePosBytes::new(pos, (ZOMBIE_LEFT_LEG,variant as u8,0,0 )));
                self.body_parts.push(InstancePosBytes::new(pos, (ZOMBIE_RIGHT_LEG,variant as u8,0,0 )));
                self.body_parts.push(InstancePosBytes::new(pos, (ZOMBIE_TORSO,variant as u8,0,0 )));
                self.body_parts.push(InstancePosBytes::new(pos, (ZOMBIE_HEAD,variant as u8,0,0 )));
                self.body_parts.push(InstancePosBytes::new(pos, (ZOMBIE_LEFT_HAND,variant as u8,0,0 )));
                self.body_parts.push(InstancePosBytes::new(pos, (ZOMBIE_RIGHT_HAND,variant as u8,0,0 )));
            }
        }
    }
}
