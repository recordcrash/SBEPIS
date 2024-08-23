use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;

use crate::entity::health::CanDealDamage;
use crate::entity::GelViscosity;
use crate::fray::FrayMusic;
use crate::util::MapRange;

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer {
	pub damage: f32,
	pub pivot: Entity,
}

#[derive(Component, Default)]
pub struct InAnimation {
	pub time: Duration,
}

#[derive(Event)]
pub struct DamageEvent {
	pub victim: Entity,
	pub damage: f32,
}

pub fn attack(
	mut commands: Commands,
	hammers: Query<Entity, (With<HammerPivot>, Without<InAnimation>)>,
) {
	for hammer in hammers.iter() {
		commands.entity(hammer).insert(InAnimation::default());
	}
}

pub fn animate_hammer(
	mut commands: Commands,
	hammer_heads: Query<(Entity, &Hammer, Option<&CanDealDamage>)>,
	mut hammer_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<HammerPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<DamageEvent>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (hammer_head_entity, hammer_head, dealer) in hammer_heads.iter() {
		let Ok((hammer_pivot_entity, mut transform, mut animation)) =
			hammer_pivots.get_mut(hammer_head.pivot)
		else {
			continue;
		};
		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let time = fray.time_to_bpm_beat(animation.time);

		if (prev_time..time).contains(&0.0) {
			commands
				.entity(hammer_head_entity)
				.insert(CanDealDamage::default());
		}
		if (prev_time..time).contains(&0.5) {
			commands
				.entity(hammer_head_entity)
				.remove::<CanDealDamage>();

			if let Some(dealer) = dealer {
				for entity in dealer.hit_entities.iter() {
					ev_hit.send(DamageEvent {
						victim: *entity,
						damage: fray.modify_fray_damage(hammer_head.damage),
					});
				}
			}
		}
		if (prev_time..time).contains(&3.5) {
			commands.entity(hammer_pivot_entity).remove::<InAnimation>();
		}

		let angle = match time {
			0.0..0.5 => time
				.map_range(0.0..0.5, 0.0..(PI * 0.5))
				.cos()
				.map_range(0.0..1.0, (-PI * 0.5)..0.0),
			0.5..3.5 => time
				.map_range(0.5..3.5, 0.0..PI)
				.cos()
				.map_range(-1.0..1.0, 0.0..(-PI * 0.5)),
			_ => 0.0,
		};
		transform.rotation = Quat::from_rotation_x(angle);
	}
}

pub fn collide_hammer(
	mut ev_collision: EventReader<CollisionEvent>,
	mut dealers: Query<&mut CanDealDamage>,
	healths: Query<Entity, With<GelViscosity>>,
) {
	for event in ev_collision.read() {
		if let CollisionEvent::Started(a, b, _flags) = event {
			if let (Ok(mut dealer), Ok(entity)) = (dealers.get_mut(*a), healths.get(*b)) {
				dealer.hit_entities.push(entity);
			}
			if let (Ok(mut dealer), Ok(entity)) = (dealers.get_mut(*b), healths.get(*a)) {
				dealer.hit_entities.push(entity);
			}
		}
	}
}

pub fn deal_all_damage(
	mut ev_hit: EventReader<DamageEvent>,
	mut commands: Commands,
	mut healths: Query<&mut GelViscosity>,
) {
	for event in ev_hit.read() {
		let damage = event.damage;
		let mut health = healths.get_mut(event.victim).unwrap();

		if damage > 0.0 && health.value <= 0.0 {
			commands.entity(event.victim).despawn_recursive();
			return;
		}

		health.value -= damage;
	}
}