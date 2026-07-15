use crate::constants::{MusicId, SfxId, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_STORY_BG, TextureId};
use std::collections::HashSet;
use std::error::Error;

/* Scripts like
 * Use CHARACTER BACKGROUND PLAYSFX PLAYMUSIC ops to determine what to load for each scene
 *
 * BACKGROUND battlefield
 * SHOW miku left
 * CHARACTER miku idle
 * SAY blablablabl blabla bla blabl bla
 * CHOICE leading | optional, but parse empty choice ignored for ease of script writing.
 *   | a
 *   | b
 *   | c
 * LABEL a
 * SHOW rin right
 * CHARACTER rin happy
 * SAY bla bla bla bla
 * GOTO d
 * LABEL b
 * SAY bla bla bla bla
 * GOTO d
 * LABEL c
 * SAY bla bla bla bla
 * GOTO d
 * LABEL d
 * CHARACTER miku mad
 * SAY bla bla bla bla
 * HIDE left
 * HIDE right
 *
 */

// Unresolved novel ops are loaded from the file, but have not been validated against
// the existing textures, sfx, and music assets to be turned into ids that can be loaded
// by the rest of the game.
#[derive(Debug, Clone)]
pub enum UnvalidatedNovelOps {
    Say(String),
    Choices(Vec<Choice>),
    Label(String),
    Goto(String),
    Show(Character, PortraitPosition),
    Hide(PortraitPosition),
    SetCharacterState(Character, String),
    Background(String),
    PlaySfx(String),
    PlayMusic(String),
    // later, triggers? like SET FLAG x, IF FLAG x ?
    // LOAD_SCRIPT another_scene to start?
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Character {
    name: String,
    state: CharacterState,
    texture_id: Option<TextureId>,
}

#[derive(Debug, Clone)]
pub enum CharacterState {
    Idle, // This potentially just becomes Idle and Tagged("state") maybe? maybe some convention like sprite sheet tagged with emotional state to load?
    Happy,
    Mad,
}

#[derive(Debug, Clone)]
pub enum PortraitPosition {
    Left,
    Center,
    Right,
}
#[derive(Debug, Clone)]
pub enum NovelOps {
    Say(String),
    Choices(Vec<Choice>),
    Label(String),
    Goto(String),
    Show(Character, PortraitPosition),
    Hide(PortraitPosition),
    SetCharacterState(Character, CharacterState),
    Background(TextureId),
    PlaySfx(SfxId),
    PlayMusic(MusicId),
    // later, triggers? like SET FLAG x, IF FLAG x ?
    // LOAD_SCRIPT another_scene to start?
}

// This and the compile program are just placeholders mostly.
// we'd have a struct or some sort of atlas layer to resolve
// things when we 'compile' which we'd do when we load the data
// up into the VN for use in a scene
fn tmp_name_to_texture(name: &str) -> Option<TextureId> {
    let t = match name {
        "battlefield" => TEXTURE_ID_STORY_BG,
        _ => TEXTURE_ID_LEEKSHEET,
    };
    Some(t)
}

fn compile_program(raw: Vec<UnvalidatedNovelOps>) -> Result<Vec<NovelOps>, Box<dyn Error>> {
    // once we're done with toy programs, this will probably need to work with
    // a proper struct getting the game context passed in to resolve textures
    // and that sort of thing. For now... just hardcode so we can keep prototyping.
    let mut set_of_all_labels: HashSet<String> = raw
        .iter()
        .filter_map(|r| {
            if let UnvalidatedNovelOps::Label(s) = r {
                Some(s.clone())
            } else {
                None
            }
        })
        .collect();

    // let mut set_of_all_goto_targets = HashSet::new();
    let mut valid_ops = Vec::with_capacity(raw.len());
    let mut errors = vec![];
    for unvalidated in raw {
        match unvalidated {
            UnvalidatedNovelOps::Say(s) => {
                valid_ops.push(NovelOps::Say(s));
            }
            UnvalidatedNovelOps::Choices(choices) => {
                valid_ops.push(NovelOps::Choices(choices));
            }
            UnvalidatedNovelOps::Label(s) => {
                valid_ops.push(NovelOps::Label(s));
            }
            UnvalidatedNovelOps::Goto(s) => {
                if !set_of_all_labels.contains(&s) {
                    errors.push(format!(
                        "cannot create GOTO ({0}) because label {0} does not exist",
                        s
                    ));
                } else {
                    valid_ops.push(NovelOps::Goto(s));
                }
            }
            UnvalidatedNovelOps::Show(character, pp) => {
                if let Some(texture_id) = tmp_name_to_texture(&character.name) {
                    let mut character = character.clone();
                    character.texture_id = Some(texture_id);
                    valid_ops.push(NovelOps::Show(character, pp));
                } else {
                    errors.push(format!("cannot load texture for character {:?}", character));
                }
            }
            UnvalidatedNovelOps::Hide(pp) => {
                valid_ops.push(NovelOps::Hide(pp));
            }
            UnvalidatedNovelOps::SetCharacterState(c, state) => {
                if state.contains("happy") {
                    let state = CharacterState::Happy;
                    valid_ops.push(NovelOps::SetCharacterState(c, state));
                } else if state.contains("angry") {
                    let state = CharacterState::Mad;
                    valid_ops.push(NovelOps::SetCharacterState(c, state));
                } else if state.contains("idle") {
                    let state = CharacterState::Idle;
                    valid_ops.push(NovelOps::SetCharacterState(c, state));
                } else {
                    errors.push(format!(
                        "unknown character state {} for character {:?}",
                        state, c
                    ));
                }
            }
            UnvalidatedNovelOps::Background(name) => {
                if let Some(texture_id) = tmp_name_to_texture(&name) {
                    valid_ops.push(NovelOps::Background(texture_id));
                } else {
                    errors.push(format!("cannot load texture for background {:?}", name));
                }
            }
            UnvalidatedNovelOps::PlaySfx(String) => {
                // TODO: load proper sfx and music
                valid_ops.push(NovelOps::PlaySfx(SfxId(0)));
            }
            UnvalidatedNovelOps::PlayMusic(String) => {
                // TODO: load proper sfx and music
                valid_ops.push(NovelOps::PlayMusic(MusicId(0)));
            }
        }
    }
    if errors.len() > 0 {
        return Err(errors.join(", ").into());
    }
    Ok(valid_ops)
}

#[derive(Debug)]
struct VisualNovelMachine {
    program_counter: usize,
    program: Vec<NovelOps>,
    music: Option<MusicId>,
    sfx: Option<SfxId>,
    focused: Option<PortraitPosition>,
    background: Option<TextureId>,
    slot_left: Option<Character>,
    slot_center: Option<Character>,
    slot_right: Option<Character>,
}
