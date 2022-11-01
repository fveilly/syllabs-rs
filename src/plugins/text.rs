
use bevy::prelude::*;

pub struct TextPlugin;

use crate::plugins::GameContext;

impl Plugin for TextPlugin  {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
        .add_system(text_update_system);
    }
}

#[derive(Component)]
struct SyllableText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>)
{
    commands.spawn_bundle(Camera2dBundle::default());

    let text = TextBundle::from_section(
        "Hello World",
        TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 300.0,
            color: Color::WHITE
        }
    )
    .with_text_alignment(TextAlignment::CENTER)
    .with_style(Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            left: Val::Px(580.0),
            top: Val::Px(230.0),
            ..default()
        },
        align_content: AlignContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });

    commands.spawn()
        .insert_bundle(text)
        .insert(SyllableText);
}

fn text_update_system(game_ctx: Res <GameContext>, mut query: Query<&mut Text, With<SyllableText>>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{}", game_ctx.syllable);
        text.sections[0].style.color = if game_ctx.is_syllable { Color::RED } else { Color::WHITE };
    }
}
