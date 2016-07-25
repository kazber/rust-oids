use app::obj;
use app::obj::*;
use rand;
use rand::Rng;
use std::collections::HashMap;
use std::slice;

#[derive(Clone)]
pub struct State {
	age_seconds: f32,
	age_frames: usize,
	charge: f32,
	target_charge: f32,
	tau: f32,
}

impl Default for State {
	fn default() -> Self {
		State {
			age_seconds: 0.,
			age_frames: 0,
			charge: 1.,
			target_charge: 0.,
			tau: 2.0,
		}
	}
}

impl State {
	pub fn update(&mut self, dt: f32) {
		self.age_seconds += dt;
		self.age_frames += 1;
		let alpha = 1. - f32::exp(-dt / self.tau);
		self.charge = self.target_charge * alpha + self.charge * (1. - alpha);
	}

	pub fn with_charge(initial: f32, target: f32) -> Self {
		State {
			charge: initial,
			target_charge: target,
			..Self::default()
		}
	}

	pub fn charge(&self) -> f32 {
		self.charge
	}
}

pub struct Limb {
	transform: Transform,
	mesh: Mesh,
	material: Material,
	pub state: State,
}

pub struct Creature {
	id: Id,
	limbs: Vec<Limb>,
}

impl GameObject for Creature {
	fn id(&self) -> Id {
		self.id
	}
}

impl Transformable for Creature {
	fn transform(&self) -> Transform {
		self.limbs.first().unwrap().transform()
	}
	fn transform_to(&mut self, t: Transform) {
		self.limbs.first_mut().unwrap().transform_to(t);
	}
}

impl Transformable for Limb {
	fn transform(&self) -> Transform {
		self.transform
	}
	fn transform_to(&mut self, t: Transform) {
		self.transform = t;
	}
}

impl obj::Geometry for Limb {
	fn mesh(&self) -> &Mesh {
		&self.mesh
	}
}

impl obj::Solid for Limb {
	fn material(&self) -> Material {
		self.material
	}
}

impl obj::Drawable for Limb {
	fn color(&self) -> Rgba {
		// let lightness = 1. - self.material.density * 0.5;
		// [0., 10. * lightness, 0., 1.]
		[9. * self.state.charge + 0.1, 4. * self.state.charge, 0., 1.]
	}
}

impl Creature {
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn limbs(&self) -> slice::Iter<Limb> {
		self.limbs.iter()
	}

	pub fn limbs_mut(&mut self) -> slice::IterMut<Limb> {
		self.limbs.iter_mut()
	}

	pub fn limb_mut(&mut self, index: LimbIndex) -> Option<&mut Limb> {
		self.limbs.get_mut(index as usize)
	}
}

pub struct Flock {
	last_id: Id,
	creatures: HashMap<Id, Creature>,
}

impl Flock {
	pub fn new() -> Flock {
		Flock {
			last_id: 0,
			creatures: HashMap::new(),
		}
	}

	pub fn get(&self, id: Id) -> Option<&Creature> {
		self.creatures.get(&id)
	}

	pub fn get_mut(&mut self, id: Id) -> Option<&mut Creature> {
		self.creatures.get_mut(&id)
	}

	pub fn next_id(&mut self) -> Id {
		self.last_id = self.last_id + 1;
		self.last_id
	}

	pub fn new_ball(&mut self, pos: Position) -> Id {
		let mut rng = rand::thread_rng();
		let radius: f32 = (rng.gen::<f32>() * 1.0) + 1.0;
		self.new_creature(Shape::new_ball(radius), pos, 0.)
	}

	pub fn new_star(&mut self, pos: Position) -> Id {
		let mut rng = rand::thread_rng();
		let radius = (rng.gen::<f32>() * 1.0) + 1.0;
		let n = rng.gen::<u8>() % 3 + 5;
		let ratio = (rng.gen::<f32>() * 0.2) + 0.1;
		self.new_creature(Shape::new_star(radius, ratio, n), pos, 0.3)
	}

	pub fn new_creature(&mut self, shape: Shape, initial_pos: Position, final_charge: f32) -> Id {
		let mut rng = rand::thread_rng();

		let id = self.next_id();

		let material = Material { density: (rng.gen::<f32>() * 1.0) + 1.0, ..Default::default() };
		let state = State::with_charge(rng.gen::<f32>(), final_charge);

		let arm1 = Limb {
			transform: obj::Transform::with_position(initial_pos + Position::new(1., 0.)),
			mesh: Mesh::from_shape(shape.clone()),
			material: material.clone(),
			state: state.clone(),
		};

		let arm2 = Limb {
			transform: obj::Transform::with_position(initial_pos - Position::new(1., 0.)),
			mesh: Mesh::from_shape(shape.clone()),
			material: material.clone(),
			state: state.clone(),
		};

		let body = Limb {
			transform: obj::Transform::with_position(initial_pos),
			mesh: Mesh::from_shape(shape),
			material: material,
			state: state,
		};

		let creature = Creature {
			id: id,
			limbs: vec![body, arm1, arm2],
		};

		self.creatures.insert(id, creature);

		id
	}

	pub fn kill(&mut self, id: &Id) {
		self.creatures.remove(id);
	}

	pub fn creatures(&self) -> &HashMap<Id, Creature> {
		&self.creatures
	}
}

#[repr(packed)]
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct CreatureRefs {
	pub creature_id: obj::Id,
	pub limb_index: obj::LimbIndex,
	pub bone_index: obj::BoneIndex,
}

impl Default for CreatureRefs {
	fn default() -> CreatureRefs {
		CreatureRefs {
			creature_id: 0xdeadbeef,
			limb_index: 0xff,
			bone_index: 0xff,
		}
	}
}

impl CreatureRefs {
	pub fn with_id(id: obj::Id) -> CreatureRefs {
		CreatureRefs { creature_id: id, ..Default::default() }
	}

	pub fn with_limb(id: obj::Id, limb_index: obj::LimbIndex) -> CreatureRefs {
		CreatureRefs {
			creature_id: id,
			limb_index: limb_index,
			..Default::default()
		}
	}

	pub fn with_bone(id: obj::Id, limb_index: obj::LimbIndex, bone_index: obj::BoneIndex) -> CreatureRefs {
		CreatureRefs {
			creature_id: id,
			limb_index: limb_index,
			bone_index: bone_index,
		}
	}
}

pub struct World {
	pub friends: Flock,
}

impl World {
	pub fn new() -> Self {
		World { friends: Flock::new() }
	}

	pub fn new_ball(&mut self, pos: obj::Position) -> obj::Id {
		self.friends.new_ball(pos)
	}

	pub fn new_star(&mut self, pos: obj::Position) -> obj::Id {
		self.friends.new_star(pos)
	}

	pub fn friend(&self, id: obj::Id) -> Option<&Creature> {
		self.friends.get(id)
	}

	pub fn friend_mut(&mut self, id: obj::Id) -> Option<&mut Creature> {
		self.friends.get_mut(id)
	}
}
