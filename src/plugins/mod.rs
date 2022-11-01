use bevy::prelude::*;

mod audio;
mod text;

use audio::InternalAudioPlugin;
use text::TextPlugin;
use std::time::SystemTime;

pub struct GameContext {
    syllable: String,
    is_syllable: bool,
    last_keypress: SystemTime
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
           .insert_resource(GameContext {
                syllable: String::new(), is_syllable: false, last_keypress: SystemTime::now() })
           .add_plugin(InternalAudioPlugin)
           .add_plugin(TextPlugin);
    }
}
