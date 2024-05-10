use bevy::{a11y::accesskit::TextSelection, prelude::*};

pub struct MenuPlugin<S: States> {
    pub state: S,
}

impl<S: States> Plugin for MenuPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.state.clone()), spawn_menu);
        app.add_systems(OnExit(self.state.clone()), despawn_screen::<PauseUI>);
    }
}

#[derive(Component)]
pub struct PauseUI;

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseUI,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|commands| {
            commands.spawn(TextBundle::from_section(
                "Game Paused",
                TextStyle {
                    color: Color::rgb(1., 0.0, 0.0),
                    font_size: 200.,
                    ..default()
                },
            ));
        });
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
