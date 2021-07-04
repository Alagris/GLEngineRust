use crate::blocks::world::FaceOrientation;

pub struct BlockProp{
    name:&'static str,
    texture_ids:[u32;6]
}

impl BlockProp{
    const fn regular(name:&'static str, texture_id:u32)->Self{
        Self{name,texture_ids:[texture_id;6]}
    }
    const fn top_sides_bottom(name:&'static str, texture_id_top:u32,texture_id_side:u32,texture_id_bottom:u32)->Self{
        Self{name,texture_ids:[texture_id_top,texture_id_bottom,texture_id_side,texture_id_side,texture_id_side,texture_id_side]}
    }
    pub fn get_texture_id(&self, ort:FaceOrientation)->u32{
        self.texture_ids[ort as usize]
    }
    pub fn name(&self)->&'static str{
        self.name
    }
}

pub const BLOCKS:[BlockProp;12] = [
    BlockProp::regular("air", /*Some dummy value*/256),
    BlockProp::regular("glass", 28),
    // blocks above are transparent. Blocks below are not
    BlockProp::regular("stone", 1),
    BlockProp::regular("dirt", 2),
    BlockProp::top_sides_bottom("grass", 0, 3,2),
    BlockProp::regular("plank", 4),
    BlockProp::top_sides_bottom("slab", 6,5,6),
    BlockProp::regular("brick", 7),
    BlockProp::top_sides_bottom("tnt", 9,8,10),
    BlockProp::regular("cobblestone", 11),
    BlockProp::regular("bedrock", 12),
    BlockProp::regular("sand", 13),
];