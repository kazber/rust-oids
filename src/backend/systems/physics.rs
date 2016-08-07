use wrapped2d::b2;
use wrapped2d::user_data::*;
use backend::obj;
use backend::obj::Updateable;
use super::*;
use backend::obj::{Solid, Geometry, Transformable};
use backend::world;
use std::collections::HashMap;

struct CreatureData;

impl UserDataTypes for CreatureData {
	type BodyData = world::CreatureRefs;
	type JointData = ();
	type FixtureData = world::CreatureRefs;
}

pub struct PhysicsSystem {
	edge: f32,
	remote: obj::Position,
	world: b2::World<CreatureData>,
	handles: HashMap<world::CreatureRefs, b2::BodyHandle>,
	dropped: Vec<world::CreatureRefs>,
}

use cgmath::Vector;
use cgmath::Vector2;
use cgmath::EuclideanVector;

impl Updateable for PhysicsSystem {
	fn update(&mut self, dt: f32) {
		enum BodyForce {
			Parallel(b2::BodyHandle, b2::Vec2, b2::Vec2),
			Perpendicular(b2::BodyHandle, b2::Vec2),
		}
		let mut v = Vec::new();

		for (h, b) in self.world.bodies() {
			let body = b.borrow();
			let center = (*body).world_center().clone();
			let facing = (*body).world_point(&b2::Vec2 { x: 0., y: 1. }).clone();
			let key = (*body).user_data();
			match key.limb_index {
				// TODO: retrieve properties from userdata
				1 | 2 => v.push(BodyForce::Perpendicular(h, center)),
				3 | 4 => v.push(BodyForce::Parallel(h, center, facing)),
				_ => {}
			}
		}
		for force in v {
			match force {
				BodyForce::Perpendicular(h, center) => {
					let v = self.remote - obj::Position::new(center.x, center.y);
					if v != Vector2::zero() {
						let f = v.normalize_to(10.0);
						self.world.body_mut(h).apply_force(&b2::Vec2 { x: f.x, y: f.y }, &center, true);
					}
				}
				BodyForce::Parallel(h, center, facing) => {
					self.world.body_mut(h).apply_force(&((facing - center) * 3.0), &center, true);
				}
			}
		}

		self.world.step(dt, 8, 3);
	}
}

struct JointRef<'a> {
	refs: world::CreatureRefs,
	handle: b2::BodyHandle,
	mesh: &'a obj::Mesh,
	attachment: Option<world::Attachment>,
}

impl System for PhysicsSystem {
	fn register(&mut self, creature: &world::Creature) {
		// build fixtures
		let joint_refs = PhysicsSystem::build_fixtures(&mut self.world, &creature);
		// and then assemble them with joints
		PhysicsSystem::build_joints(&mut self.world, &joint_refs);
		// record them
		for JointRef { refs, handle, .. } in joint_refs {
			self.handles.insert(refs, handle);
		}
	}

	fn to_world(&self, world: &mut world::World) {
		for key in &self.dropped {
			world.friends.kill(&key.creature_id);
			// println!("Killed object: {}", key.creature_id);
		}
		for (_, b) in self.world.bodies() {
			let body = b.borrow();
			let position = (*body).position();
			let angle = (*body).angle();
			let key = (*body).user_data();

			if let Some(creature) = world.friends.get_mut(key.creature_id) {
				if let Some(object) = creature.limb_mut(key.limb_index) {
					let scale = object.transform().scale;
					object.transform_to(obj::Transform {
						position: obj::Position {
							x: position.x,
							y: position.y,
						},
						angle: angle,
						scale: scale,
					});
				}
			}
		}
	}
}

impl PhysicsSystem {
	pub fn new() -> Self {
		PhysicsSystem {
			world: Self::new_world(),
			edge: 0.,
			remote: obj::Position::new(0., 0.),
			handles: HashMap::new(),
			dropped: Vec::new(),
		}
	}

	fn build_fixtures<'a>(world: &mut b2::World<CreatureData>, creature: &'a world::Creature) -> Vec<JointRef<'a>> {
		let object_id = creature.id();
		let limbs = creature.limbs();
		limbs.enumerate()
			.map(|(limb_index, limb)| {
				let material = limb.material();
				let mut f_def = b2::FixtureDef::new();
				f_def.density = material.density;
				f_def.restitution = material.restitution;
				f_def.friction = material.friction;

				let transform = limb.transform();
				let mut b_def = b2::BodyDef::new();
				b_def.body_type = b2::BodyType::Dynamic;
				b_def.angle = transform.angle;
				b_def.position = b2::Vec2 {
					x: transform.position.x,
					y: transform.position.y,
				};
				let refs = world::CreatureRefs::with_limb(object_id, limb_index as u8);
				let handle = world.create_body_with(&b_def, refs);

				let mesh = limb.mesh();
				let attached_to = limb.attached_to;
				match mesh.shape {
					obj::Shape::Ball { radius } => {
						let mut circle_shape = b2::CircleShape::new();
						circle_shape.set_radius(radius);
						world.body_mut(handle).create_fixture_with(&circle_shape, &mut f_def, refs);
					}
					obj::Shape::Box { radius, ratio } => {
						let mut rect_shape = b2::PolygonShape::new();
						rect_shape.set_as_box(radius * ratio, radius);
						world.body_mut(handle).create_fixture_with(&rect_shape, &mut f_def, refs);
					}
					obj::Shape::Star { radius, n, .. } => {
						let p = &mesh.vertices;
						for i in 0..n {
							let mut quad = b2::PolygonShape::new();
							let i1 = (i * 2 + 1) as usize;
							let i2 = (i * 2) as usize;
							let i3 = ((i * 2 + (n * 2) - 1) % (n * 2)) as usize;
							let (p1, p2, p3) = match mesh.winding {
								obj::Winding::CW => (&p[i1], &p[i2], &p[i3]),
								obj::Winding::CCW => (&p[i1], &p[i3], &p[i2]),
							};
							quad.set(&[b2::Vec2 { x: 0., y: 0. },
							           b2::Vec2 {
								           x: p1.x * radius,
								           y: p1.y * radius,
							           },
							           b2::Vec2 {
								           x: p2.x * radius,
								           y: p2.y * radius,
							           },
							           b2::Vec2 {
								           x: p3.x * radius,
								           y: p3.y * radius,
							           }]);
							let refs = world::CreatureRefs::with_bone(object_id, limb_index as u8, i as u8);
							world.body_mut(handle).create_fixture_with(&quad, &mut f_def, refs);
						}
					}
					obj::Shape::Triangle { radius, .. } => {
						let p = &mesh.vertices;
						let mut tri = b2::PolygonShape::new();
						let (p1, p2, p3) = match mesh.winding {
							obj::Winding::CW => (&p[0], &p[2], &p[1]),
							obj::Winding::CCW => (&p[0], &p[1], &p[2]),
						};
						tri.set(&[b2::Vec2 {
							          x: p1.x * radius,
							          y: p1.y * radius,
						          },
						          b2::Vec2 {
							          x: p2.x * radius,
							          y: p2.y * radius,
						          },
						          b2::Vec2 {
							          x: p3.x * radius,
							          y: p3.y * radius,
						          }]);
						world.body_mut(handle).create_fixture_with(&tri, &mut f_def, refs);
					}
				};
				JointRef {
					refs: refs,
					handle: handle,
					mesh: mesh,
					attachment: attached_to,
				}
			})
			.collect::<Vec<_>>()
	}

	fn build_joints(world: &mut b2::World<CreatureData>, joint_refs: &Vec<JointRef>) {
		for &JointRef { handle: distal, mesh, attachment, .. } in joint_refs {
			if let Some(attachment) = attachment {
				let upstream = &joint_refs[attachment.index as usize];
				let medial = upstream.handle;

				let mut joint = b2::RevoluteJointDef::new(medial, distal);
				joint.collide_connected = true;

				let v0 = upstream.mesh.vertices[attachment.attachment_point as usize] * upstream.mesh.shape.radius();
				joint.local_anchor_a = b2::Vec2 { x: v0.x, y: v0.y };

				let v1 = mesh.vertices[0] * mesh.shape.radius();
				joint.local_anchor_b = b2::Vec2 { x: v1.x, y: v1.y };
				world.create_joint_with(&joint, ());
			}
		}
	}

	pub fn drop_below(&mut self, edge: f32) {
		self.edge = edge;
	}

	pub fn follow_me(&mut self, pos: obj::Position) {
		self.remote = pos;
	}

	fn new_world() -> b2::World<CreatureData> {
		let world = b2::World::new(&b2::Vec2 { x: 0.0, y: 0.0 });

		world
	}
}
