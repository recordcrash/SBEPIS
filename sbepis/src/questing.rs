use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_rapier3d::prelude::*;
use itertools::Itertools;
use leafwing_input_manager::prelude::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use uuid::Uuid;

use crate::camera::PlayerCamera;
use crate::input::{
	button_just_pressed, input_manager_bundle, input_managers_where_button_just_pressed,
};
use crate::iter_system::IteratorSystemTrait;
use crate::menus::*;
use crate::player_controller::PlayerAction;
use crate::some_or_return;

pub struct QuestingPlugin;
impl Plugin for QuestingPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Quests>()
			.register_type::<QuestScreen>()
			.register_type::<QuestGiver>()
			.register_type::<QuestId>()
			.register_type::<Quest>()
			.init_resource::<Quests>()
			.add_plugins(InputManagerMenuPlugin::<QuestProposalAction>::default())
			.add_systems(Startup, spawn_quest_screen)
			.add_systems(
				Update,
				(
					// FIXME: Sometimes pressing interact will open and then immediately close the quest screen
					interact_with_quest_giver
						.pipe(propose_quest)
						.run_if(button_just_pressed(PlayerAction::Interact)),
					update_quest_screen_nodes,
					input_managers_where_button_just_pressed(QuestProposalAction::Decline)
						.iter_map(get_proposed_quest)
						.iter_map(finish_quest)
						.map(|_| ()),
					close_menu_on(QuestProposalAction::Accept),
					close_menu_on(QuestProposalAction::Decline),
					show_menu::<QuestScreen>
						.run_if(button_just_pressed(PlayerAction::OpenQuestScreen)),
				),
			);

		#[cfg(feature = "inspector")]
		app.register_type_data::<QuestId, bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl>();
	}
}

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct Quests(pub HashMap<QuestId, Quest>);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Reflect)]
pub struct QuestId(Uuid);
impl QuestId {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Uuid::new_v4())
	}
}
impl Display for QuestId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
#[cfg(feature = "inspector")]
impl bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive for QuestId {
	fn ui(
		&mut self,
		ui: &mut bevy_inspector_egui::egui::Ui,
		_options: &dyn std::any::Any,
		_id: bevy_inspector_egui::egui::Id,
		_env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
	) -> bool {
		ui.add_enabled_ui(false, |ui| {
			ui.text_edit_singleline(&mut self.0.to_string());
		});
		false
	}

	fn ui_readonly(
		&self,
		ui: &mut bevy_inspector_egui::egui::Ui,
		_options: &dyn std::any::Any,
		_id: bevy_inspector_egui::egui::Id,
		_env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
	) {
		ui.add_enabled_ui(false, |ui| {
			ui.text_edit_singleline(&mut self.0.to_string());
		});
	}
}

#[derive(Debug, Reflect)]
pub enum QuestType {
	Fetch,
	Kill(u32),
}
impl Distribution<QuestType> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuestType {
		match rng.gen_range(0..=1) {
			0 => QuestType::Fetch,
			_ => QuestType::Kill(rng.gen_range(1..=5)),
		}
	}
}

#[derive(Debug, Reflect)]
pub struct Quest {
	pub id: QuestId,
	pub quest_type: QuestType,
	pub name: String,
	pub description: String,
	pub completed: bool,
}
impl Distribution<Quest> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Quest {
		let quest_type: QuestType = rng.gen();
		match quest_type {
			QuestType::Kill(amount) => Quest {
				id: QuestId::new(),
				name: "Awesome Kill Quest".to_string(),
				quest_type,
				description: format!("imps killed my grandma... pwease go take revenge on those darn imps for me... kill {amount}!!"),
				completed: false,
			},
			QuestType::Fetch => Quest {
				id: QuestId::new(),
				name: "Awesome Fetch Quest".to_string(),
				quest_type,
				description: "imps stole my orange cube... pwease go get it back!!".to_string(),
				completed: false,
			},
		}
	}
}

#[derive(Component, Default, Reflect)]
pub struct QuestGiver {
	pub given_quest: Option<QuestId>,
}

#[derive(Component, Default, Reflect)]
pub struct QuestScreen {
	pub quests: HashSet<QuestId>,
	pub quest_nodes: HashMap<QuestId, Entity>,
}

#[derive(Component)]
pub struct QuestProposal {
	pub quest_id: QuestId,
}

fn spawn_quest_screen(mut commands: Commands) {
	commands
		.spawn((
			NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					flex_direction: FlexDirection::Column,
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					..default()
				},
				background_color: bevy::color::palettes::css::GRAY.with_alpha(0.5).into(),
				visibility: Visibility::Hidden,
				..default()
			},
			input_manager_bundle(
				InputMap::default().with(MenuAction::CloseMenu, KeyCode::KeyJ),
				false,
			),
			Menu,
			MenuWithMouse,
			MenuWithInputManager,
			MenuHidesWhenClosed,
			QuestScreen::default(),
		))
		.insert(Name::new("Quest Screen"));
}

fn interact_with_quest_giver(
	rapier_context: Res<RapierContext>,
	player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
	quest_givers: Query<Entity, With<QuestGiver>>,
) -> Option<Entity> {
	let player_camera = player_camera.get_single().expect("Player camera missing");
	let mut quest_giver = None;
	rapier_context.intersections_with_ray(
		player_camera.translation(),
		player_camera.forward().into(),
		3.0,
		false,
		QueryFilter::default(),
		|entity, _intersection| {
			if quest_givers.get(entity).is_ok() {
				quest_giver = Some(entity);
				false
			} else {
				true
			}
		},
	);
	quest_giver
}

fn propose_quest(
	In(quest_giver): In<Option<Entity>>,
	mut commands: Commands,
	mut quests: ResMut<Quests>,
	mut quest_screen: Query<&mut QuestScreen>,
	mut quest_givers: Query<&mut QuestGiver>,
	mut menu_stack: ResMut<MenuStack>,
) {
	let quest_giver = some_or_return!(quest_giver);
	let mut quest_giver = quest_givers
		.get_mut(quest_giver)
		.expect("Quest giver missing");
	if quest_giver.given_quest.is_some() {
		return;
	}

	let quest: Quest = rand::random();
	let quest_id = quest.id;
	quests.0.insert(quest_id, quest);
	let quest = quests
		.0
		.get(&quest_id)
		.expect("Unknown quest even though we just inserted it");

	let mut quest_screen = quest_screen.single_mut();
	quest_screen.quests.insert(quest_id);

	quest_giver.given_quest = Some(quest_id);

	let proposal = commands
		.spawn((
			TextBundle {
				text: Text::from_section(
					format!(
						"{}\n{}\n[E] to accept or [Space] to decline",
						quest.name, quest.description
					),
					TextStyle {
						font_size: 20.0,
						color: Color::WHITE,
						..default()
					},
				),
				style: Style { ..default() },
				..default()
			},
			input_manager_bundle(
				InputMap::default()
					.with(QuestProposalAction::Accept, KeyCode::KeyE)
					.with(QuestProposalAction::Decline, KeyCode::Space),
				false,
			),
			Menu,
			MenuWithMouse,
			MenuWithInputManager,
			MenuDespawnsWhenClosed,
			QuestProposal { quest_id },
		))
		.insert(Name::new(format!("Quest Proposal for {quest_id}")))
		.id();

	menu_stack.push(proposal);
}

fn get_proposed_quest(
	In(input): In<Entity>,
	quest_proposals: Query<&QuestProposal>,
) -> Option<QuestId> {
	quest_proposals.get(input).map(|qp| qp.quest_id).ok()
}

fn finish_quest(
	In(quest_id): In<Option<QuestId>>,
	mut quests: ResMut<Quests>,
	mut quest_screen: Query<&mut QuestScreen>,
	mut quest_givers: Query<&mut QuestGiver>,
) {
	let quest_id = some_or_return!(quest_id);

	quests.0.remove(&quest_id);

	let mut quest_screen = quest_screen.single_mut();
	quest_screen.quests.remove(&quest_id);

	let mut quest_giver = quest_givers
		.iter_mut()
		.find(|qg| qg.given_quest == Some(quest_id))
		.expect("Quest giver missing");
	quest_giver.given_quest = None;
}

fn update_quest_screen_nodes(
	mut commands: Commands,
	quests: Res<Quests>,
	mut quest_screen: Query<(Entity, &mut QuestScreen)>,
) {
	let (quest_screen_entity, mut quest_screen) = quest_screen.single_mut();

	// Add new quests
	let new_quests = quest_screen
		.quests
		.iter()
		.filter(|q| !quest_screen.quest_nodes.keys().contains(q))
		.copied()
		.collect_vec();
	for quest_id in new_quests {
		let quest = quests.0.get(&quest_id).expect("Unknown quest");
		let quest_node = commands
			.spawn((
				Name::new(format!("Quest Node for {quest_id}")),
				TextBundle {
					text: Text::from_section(
						quest.name.clone(),
						TextStyle {
							font_size: 20.0,
							color: Color::WHITE,
							..default()
						},
					),
					..default()
				},
			))
			.set_parent(quest_screen_entity)
			.id();
		quest_screen.quest_nodes.insert(quest_id, quest_node);
	}

	// Remove old quests
	let old_quests = quest_screen
		.quest_nodes
		.keys()
		.filter(|q| !quest_screen.quests.contains(*q))
		.copied()
		.collect_vec();
	for quest_id in old_quests {
		if let Some(quest_node) = quest_screen.quest_nodes.remove(&quest_id) {
			commands.entity(quest_node).despawn_recursive();
		}
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum QuestProposalAction {
	Accept,
	Decline,
}
impl Actionlike for QuestProposalAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			QuestProposalAction::Accept => InputControlKind::Button,
			QuestProposalAction::Decline => InputControlKind::Button,
		}
	}
}
