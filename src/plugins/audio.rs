use bevy::{
    input::keyboard::KeyboardInput,
    prelude::*, audio::AudioSink,
};

use crate::SyllablesTree;
use crate::defs::str_from_key;
use crate::plugins::GameContext;

use std::time::SystemTime;

pub struct InternalAudioPlugin;

impl Plugin for InternalAudioPlugin  {
    fn build(&self, app: &mut App) {
        app.add_system(keyboard_input_system);
    }
}

fn keyboard_input_system(syllab_tree: Res<SyllablesTree>, mut game_ctx: ResMut<GameContext>, audio: Res<Audio>, mut keyboard_input_events: EventReader<KeyboardInput>) {
    use bevy::input::ButtonState;

    let sys_time = SystemTime::now();

    for event in keyboard_input_events.iter() {
        match event.state {
            ButtonState::Pressed => {
                info!("Key press: {:?} ({})", event.key_code, event.scan_code);

                if let Ok(duration) = sys_time.duration_since(game_ctx.last_keypress) {
                    const KEYPRESS_LIMIT_MS: u128 = 150;
                    if duration.as_millis() < KEYPRESS_LIMIT_MS {
                        println!("too soon, waiting for {}ms...", KEYPRESS_LIMIT_MS - duration.as_millis());
                        continue;
                    }
                }

                if let Some(key_code) = event.key_code {
                    if let Some(c) = str_from_key(key_code).map(|str| str.to_lowercase()).and_then(|str| str.chars().next()) {
                        game_ctx.syllable.push(c);
                        game_ctx.is_syllable = false;
                        info!("push key code syllable='{}'", game_ctx.syllable);                     
                        if let Some((audio_handle, is_syllable)) = syllab_tree.get(game_ctx.syllable.clone()) {
                            audio.play(audio_handle);
                            game_ctx.is_syllable = is_syllable;
                        }
                        else {
                            game_ctx.syllable.clear();
                            game_ctx.syllable.push(c);
                            if let Some((audio_handle, is_syllable)) = syllab_tree.get(game_ctx.syllable.clone()) {
                                audio.play(audio_handle);
                                game_ctx.is_syllable = is_syllable;
                            }
                        }
                        info!("syllable={}", game_ctx.syllable);
                    }
                    else {
                        game_ctx.syllable.clear();
                    }
                }

                game_ctx.last_keypress = SystemTime::now();
            }
            ButtonState::Released => {
                info!("Key release: {:?} ({})", event.key_code, event.scan_code);
            }
        }
    }
}
