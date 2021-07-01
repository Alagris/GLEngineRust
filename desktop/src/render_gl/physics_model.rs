use glm;

pub trait Scalar: glm::RealField + Copy {
    fn cast(x: i32) -> Self;
}

pub struct CollisionVector<N: Scalar> {
    val: glm::TVec<N, 3>,
}

impl<N: Scalar> CollisionVector<N> {
    pub fn new(val: glm::TVec<N, 3>) -> Self {
        Self { val }
    }
}

impl Scalar for f32 {
    fn cast(i: i32) -> Self {
        i as f32
    }
}

impl Scalar for f64 {
    fn cast(i: i32) -> Self {
        i as f64
    }
}

pub struct PhysicsModel<N: Scalar> {
    rotation: glm::Qua<N>,
    location: glm::TVec<N, 3>,
    scale: glm::TVec<N, 3>,
    velocity: glm::TVec<N, 3>,
    acceleration: glm::TVec<N, 3>,
    mass: N,
}

impl<N: Scalar> PhysicsModel<N> {
    pub fn update(&mut self, delta: N) {
        self.location += self.velocity * delta;
        self.velocity += self.acceleration * delta;
    }

    pub fn to_mat4(&self) -> glm::TMat4<N> {
        glm::translate(
            &glm::scale(&glm::quat_to_mat4(&self.rotation), &self.scale),
            &self.location,
        )
    }
    pub fn new_from_vectors(
        rotation: glm::Qua<N>,
        location: glm::TVec<N, 3>,
        scale: glm::TVec<N, 3>,
        velocity: glm::TVec<N, 3>,
        acceleration: glm::TVec<N, 3>,
        mass: N,
    ) -> PhysicsModel<N> {
        PhysicsModel {
            rotation,
            location,
            scale,
            velocity,
            acceleration,
            mass,
        }
    }
    pub fn new(
        x: N,
        y: N,
        z: N,
        w: N,
        h: N,
        d: N,
        vx: N,
        vy: N,
        vz: N,
        ax: N,
        ay: N,
        az: N,
        mass: N,
    ) -> PhysicsModel<N> {
        PhysicsModel {
            rotation: glm::quat_identity(),
            location: glm::vec3(x, y, z),
            scale: glm::vec3(w, h, d),
            velocity: glm::vec3(vx, vy, vz),
            acceleration: glm::vec3(ax, ay, az),
            mass,
        }
    }

    pub fn location(&self) -> &glm::TVec<N, 3> {
        &self.location
    }

    pub fn velocity(&self) -> &glm::TVec<N, 3> {
        &self.velocity
    }

    pub fn momentum(&self) -> N {
        glm::length(&self.velocity) * self.mass
    }

    pub fn adjust_velocity_to_momentum(&mut self, momentum: N) {
        let new_velocity_modulo = momentum / self.mass;
        if new_velocity_modulo > N::cast(0) {
            self.velocity = glm::normalize(&self.velocity) * new_velocity_modulo;
        }
    }

    pub fn collide(
        &mut self,
        other: &mut PhysicsModel<N>,
        collision_vector_from_self_to_other: &CollisionVector<N>,
    ) {
        self.bounce(collision_vector_from_self_to_other);
        other.bounce(collision_vector_from_self_to_other);
        let momentum0 = self.momentum();
        let momentum1 = other.momentum();
        self.velocity -= collision_vector_from_self_to_other.val;
        other.velocity += collision_vector_from_self_to_other.val;
        let total = momentum0 + momentum1;
        let new_momentum = total / N::cast(2);
        self.adjust_velocity_to_momentum(new_momentum);
        other.adjust_velocity_to_momentum(new_momentum);
        assert!(
            (self.momentum() + other.momentum() - total).abs() < N::cast(1) / N::cast(100000),
            "{} + {} = {} < {}, {} : {}",
            self.momentum(),
            other.momentum(),
            (self.momentum() + other.momentum()),
            total,
            self.mass,
            other.mass
        );
    }

    pub fn bounce(&mut self, collision_vector: &CollisionVector<N>) {
        self.velocity = glm::reflect_vec(&self.velocity, &collision_vector.val);
    }
}

pub struct BallPhysicsModel<N: Scalar> {
    physics_model: PhysicsModel<N>,
    radius: N,
}

impl<N: Scalar> BallPhysicsModel<N> {
    pub fn new(model: PhysicsModel<N>, radius: N) -> BallPhysicsModel<N> {
        BallPhysicsModel {
            physics_model: model,
            radius,
        }
    }
    pub fn model(&self) -> &PhysicsModel<N> {
        &self.physics_model
    }
    pub fn model_mut(&mut self) -> &mut PhysicsModel<N> {
        &mut self.physics_model
    }
    pub fn collision_vector(&self, other: &BallPhysicsModel<N>) -> Option<CollisionVector<N>> {
        let dist = glm::distance(&self.physics_model.location, &other.physics_model.location);
        let collision_depth = (self.radius + other.radius - dist) / N::cast(2);
        if collision_depth >= N::cast(0) {
            let direction =
                glm::normalize(&(&other.physics_model.location - &self.physics_model.location))
                    * collision_depth;
            Some(CollisionVector { val: direction })
        } else {
            None
        }
    }
}
