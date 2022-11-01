extern crate bevy;
extern crate serde;

mod defs;
mod plugins;

use bevy::prelude::{App, Handle, Res, AssetServer, AudioSource, Commands, info};

use bevy::DefaultPlugins;
use bevy::window::WindowDescriptor;
use bevy::asset::AssetServerError;
use indextree::NodeEdge;
use plugins::GamePlugin;

use core::fmt;
use std::path::Path;
use std::str::Chars;
use indextree::{Arena, NodeId};

use std::collections::HashMap;

struct SyllableAtom {
    audio_handle: Option<Handle<AudioSource>>,
    c: char
}

impl SyllableAtom {
    pub fn new(c: char) -> SyllableAtom {
        SyllableAtom {
            audio_handle: None,
            c
        }
    }

    pub fn char(&self) -> char {
        return self.c;
    }

    pub fn set_handle(&mut self, handle: Handle<AudioSource>)
    {
        self.audio_handle = Some(handle);
    }
}

#[derive(Default)]
struct SyllablesTree {
    arena: Arena<SyllableAtom>,
    root: Option<NodeId>
}

impl SyllablesTree {
    pub fn new() -> SyllablesTree {
        SyllablesTree {
            arena: Arena::new(),
            root: None
        }
    }

    fn find_child(&self, node_id: NodeId, c: char) -> Option<NodeId> {
        for child_id in node_id.children(&self.arena) {
            if let Some(child_node) = self.arena.get(child_id) {
                let child_char = child_node.get().char();
                if child_char == c {
                    return Some(child_id);
                }
            }
        }
        return None;
    }

    pub fn get(&self, syllable: String) -> Option<(Handle<AudioSource>, bool)>
    {
        let mut chars = syllable.chars();

        if let Some(mut last_node_id) = self.root {
            info!("root found!");
            loop {
                match chars.next() {
                    Some(c) => {
                        info!("current char = '{}'", c);
                        match self.find_child(last_node_id, c) {
                            Some(child_id) => {
                                info!("find_child='{}'", c);
                                last_node_id = child_id;
                            },
                            None => { return None; }
                        }
                    },
                    None => {
                        // All syllable's letters consumed, return audio handle for the leaf node
                        return match self.arena.get(last_node_id) {
                            Some(node) => {
                                if let Some(audio_handle) = &node.get().audio_handle {
                                    return Some((audio_handle.clone(), last_node_id.children(&self.arena).next().is_none()));
                                }

                                return self.get(node.get().char().to_string()).map(|r| (r.0, false));
                            },
                            None => None
                        };
                    }
                }
            }
        }

        return None;
    }

    fn add_syllable_recursive(&mut self, chars: &mut Chars, root_id: NodeId, handle: Handle<AudioSource>)
    {
        let mut new_root_id_opt: Option<NodeId> = None;
        if let Some(c) = chars.next() {
            for child_id in root_id.children(&self.arena) {
                if let Some(child_node) = self.arena.get(child_id) {
                    let child_char = child_node.get().char();
                    if child_char == c {
                        new_root_id_opt = Some(child_id.clone());
                        break;
                    }
                    else if child_char > c {
                        let new_root_id = self.arena.new_node(SyllableAtom::new(c));
                        new_root_id_opt = Some(new_root_id);
                        child_id.insert_before(new_root_id, &mut self.arena);
                        break;
                    }
                }
            }


            if let Some(child_id) = new_root_id_opt {
                self.add_syllable_recursive(chars, child_id.clone(), handle);
            }
            else {
                let new_root_id = self.arena.new_node(SyllableAtom::new(c));
                root_id.append(new_root_id, &mut self.arena);
                self.add_syllable_recursive(chars, new_root_id, handle);
            }
        }
        else {
            if let Some(root_node) = self.arena.get_mut(root_id) {
                let syllab_atom = root_node.get_mut();
                syllab_atom.set_handle(handle);
            }
        }
    }

    pub fn add_syllable(&mut self, syllable: &str, handle: Handle<AudioSource>) {
        let mut chars = syllable.chars();

        if self.arena.is_empty() {
            if let Some(c) = chars.next() {
                let root_id = self.arena.new_node(SyllableAtom::new(c));
                self.root = Some(root_id);
                self.add_syllable_recursive(&mut chars, root_id, handle);
            }
        }
        else if let Some(root_id) = self.root {
            self.add_syllable_recursive(&mut chars, root_id, handle);
        }
    }
}

impl fmt::Display for SyllablesTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut indent_level = 0;
        if let Some(root_node_id) = self.root {
            for node_edge in root_node_id.traverse(&self.arena) {
                match node_edge {
                    NodeEdge::Start(node_id) => {
                        if indent_level > 0 {
                            write!(f, "{}|---", " ".repeat(4 * (indent_level - 1)))?;
                        }
                        if let Some(node) = self.arena.get(node_id) {
                            write!(f, "{}{}", node.get().char(), if node.get().audio_handle.is_some() { "[x]" } else { "" })?;
                        }
                        write!(f, "\n")?;
                        indent_level += 1;
                    },
                    NodeEdge::End(_) => {
                        indent_level -= 1;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn load_folder<P: AsRef<Path>>(
    path: P,
    server: &Res<AssetServer>,
) -> Result<HashMap<String, Handle<AudioSource>>, AssetServerError> {
    let path = path.as_ref();
    if !server.asset_io().is_dir(path) {
        return Err(AssetServerError::AssetFolderNotADirectory(
            path.to_str().unwrap().to_string(),
        ));
    }

    let mut handles = HashMap::new();
    for child_path in server.asset_io().read_directory(path)? {
        if server.asset_io().is_dir(&child_path) {
            handles.extend(load_folder(&child_path, server)?);
        } else {
            let ext = child_path.extension();
            if ext.map_or(true, |ostr| ostr.to_str().map_or(true, |s| s.ne("wav"))) {
                continue;
            }

            if let Some(ostr) = child_path.file_stem() {
                if let Some(file_stem) = ostr.to_str() {
                    let lowercase = file_stem.to_lowercase();
                    if !handles.contains_key(&lowercase) {
                        let handle: Handle<AudioSource> =
                            server.load(child_path.to_str().expect("Path should be a valid string."));
                    
                        handles.insert(lowercase, handle);
                    }
                }
            }
       }
    }

    return Ok(handles);
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>) {
    if let Ok(audio_handles) = load_folder("audio/fr", &asset_server) {
        let mut syllabs_tree = SyllablesTree::new();

        for (file_stem, audio_handle) in audio_handles.into_iter() {
            syllabs_tree.add_syllable(&file_stem, audio_handle);
        }

        info!("{}", syllabs_tree);

        commands.insert_resource(syllabs_tree)
    }
}

fn main() {
    // let args: Vec<String> = env::args().collect();

    // let mapping_data: String = if !args.is_empty() {
    //     load_asset_mapping(&args[0]).expect("Mapping file is missing!")
    // }
    // else {
    //     load_asset_mapping("audio-fr").expect("Mapping file is missing!")
    // };

    // let mapping_config: Config = toml::from_str(&mapping_data).expect("Mapping file is not a valid toml file!");

    App::new()
        .insert_resource(WindowDescriptor {
            title: "Syllabs".to_string(),
            width: 1280.,
            height: 720.,
            ..Default::default()
        })
        .add_startup_system(setup)
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}
