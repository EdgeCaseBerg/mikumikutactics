use crate::constants::{MusicId, SfxId, TextureId};
use std::collections::HashSet;
use std::error::Error;

/* Scripts like
 * Use CHARACTER BACKGROUND PLAYSFX PLAYMUSIC ops to determine what to load for each scene
 *
 * BACKGROUND battlefield
 * ENTER miku FROM left
 * CHARACTER miku IS idle
 * SAY blablablabl blabla bla blabl bla
 * CHOICE leading | optional, but parse empty choice ignored for ease of script writing.
 *   | a You're my bestie!
 *   | b Sure it neat to have a friend like you rin!
 *   | c ...
 * LABEL a
 * ENTER rin FROM right
 * FOCUS right
 * CHARACTER rin IS happy
 * SAY bla bla bla bla
 * GOTO d
 * LABEL b
 * SAY bla bla bla bla
 * GOTO d
 * LABEL c
 * SAY bla bla bla bla
 * GOTO d
 * LABEL d
 * CHARACTER miku IS mad
 * SAY bla bla bla bla
 * HIDE left
 * HIDE right
 *
 */

type VnResult<T> = Result<T, Box<dyn Error>>;

// Unresolved novel ops are loaded from the file, but have not been validated against
// the existing textures, sfx, and music assets to be turned into ids that can be loaded
// by the rest of the game.
#[derive(Debug, Clone, PartialEq)]
pub enum UnvalidatedNovelOps {
    Say(String),
    Choices(Vec<Choice>),
    Label(String),
    Goto(String),
    Show(String, String),
    Hide(String),
    Focus(String),
    NewCharacterState(String, String),
    Background(String),
    PlaySfx(String),
    PlayMusic(String),
    // later, triggers? like SET FLAG x, IF FLAG x ?
    // LOAD_SCRIPT another_scene to start?
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub label: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Character {
    name: String,
    state: CharacterState,
    texture_id: TextureId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharacterState {
    Idle, // This potentially just becomes Idle and Tagged("state") maybe? maybe some convention like sprite sheet tagged with emotional state to load?
    Happy,
    Mad,
}

impl CharacterState {
    fn lift(s: &str) -> Option<CharacterState> {
        let pp = match &s.to_lowercase()[..] {
            "idle" => CharacterState::Idle,
            "happy" => CharacterState::Happy,
            "mad" => CharacterState::Mad,
            _ => return None,
        };
        Some(pp)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortraitPosition {
    Left,
    Center,
    Right,
}

impl PortraitPosition {
    fn lift(s: &str) -> Option<PortraitPosition> {
        let pp = match &s.to_lowercase()[..] {
            "left" => PortraitPosition::Left,
            "center" => PortraitPosition::Center,
            "right" => PortraitPosition::Right,
            _ => return None,
        };
        Some(pp)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NovelOps {
    Say(String),
    Choices(Vec<Choice>),
    Label(String),
    Goto(String),
    Show(Character, PortraitPosition),
    Hide(PortraitPosition),
    Focus(PortraitPosition),
    NewCharacterState(Character),
    Background(TextureId),
    PlaySfx(SfxId),
    PlayMusic(MusicId),
    // later, triggers? like SET FLAG x, IF FLAG x ?
    // LOAD_SCRIPT another_scene to start?
}

/// Trait for a scene or backend to implement in order to convert stuff like 'miku idle' into a proper sprite texture
pub trait SceneLoadingContext {
    fn name_to_texture(&self, name: &str) -> Option<TextureId>;
    fn name_to_sfx(&self, name: &str) -> Option<SfxId>;
    fn name_to_music(&self, name: &str) -> Option<MusicId>;
}
#[derive(Debug)]
pub struct VisualNovelMachine {
    pub program_counter: usize,
    pub program: Vec<NovelOps>,
    pub music: Option<MusicId>,
    pub sfx: Option<SfxId>,
    pub focused: Option<PortraitPosition>,
    pub background: Option<TextureId>,
    pub slot_left: Option<Character>,
    pub slot_center: Option<Character>,
    pub slot_right: Option<Character>,
}

impl Default for VisualNovelMachine {
    fn default() -> Self {
        VisualNovelMachine {
            program_counter: 0,
            program: vec![],
            music: None,
            sfx: None,
            focused: None,
            background: None,
            slot_left: None,
            slot_center: None,
            slot_right: None,
        }
    }
}

impl From<Vec<NovelOps>> for VisualNovelMachine {
    fn from(ops: Vec<NovelOps>) -> VisualNovelMachine {
        let mut vn = VisualNovelMachine::default();
        vn.program = ops;
        vn
    }
}

fn validate_program(
    raw: Vec<UnvalidatedNovelOps>,
    context: &impl SceneLoadingContext,
) -> VnResult<Vec<NovelOps>> {
    // once we're done with toy programs, this will probably need to work with
    // a proper struct getting the game context passed in to resolve textures
    // and that sort of thing. For now... just hardcode so we can keep prototyping.
    let set_of_all_labels: HashSet<String> = raw
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
                let mut all_valid = true;
                for choice in &choices {
                    if !set_of_all_labels.contains(&choice.label) {
                        all_valid = false;
                        errors.push(format!(
                            "cannot create choice because label {} does not exist",
                            &choice.label
                        ));
                    }
                }
                if all_valid {
                    valid_ops.push(NovelOps::Choices(choices));
                }
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
            UnvalidatedNovelOps::Show(character_name, pp) => {
                if let Some(texture_id) = context.name_to_texture(&character_name) {
                    let character = Character {
                        name: character_name,
                        state: CharacterState::Idle,
                        texture_id,
                    };
                    if let Some(pp) = PortraitPosition::lift(&pp) {
                        valid_ops.push(NovelOps::Show(character, pp));
                    } else {
                        errors.push(format!(
                            "cannot show character at position {:?} as position is not valid",
                            pp
                        ));
                    }
                } else {
                    errors.push(format!(
                        "cannot load texture for character {:?}",
                        character_name
                    ));
                }
            }
            UnvalidatedNovelOps::Hide(pp) => {
                if let Some(pp) = PortraitPosition::lift(&pp) {
                    valid_ops.push(NovelOps::Hide(pp));
                } else {
                    errors.push(format!(
                        "cannot hide character at position {:?} as position is not valid",
                        pp
                    ));
                }
            }
            UnvalidatedNovelOps::Focus(pp) => {
                if let Some(pp) = PortraitPosition::lift(&pp) {
                    valid_ops.push(NovelOps::Focus(pp));
                } else {
                    errors.push(format!(
                        "cannot focus position {:?} as position is not valid",
                        pp
                    ));
                }
            }
            UnvalidatedNovelOps::NewCharacterState(character_name, state) => {
                if let Some(texture_id) = context.name_to_texture(&character_name) {
                    if let Some(state) = CharacterState::lift(&state) {
                        let character = Character {
                            name: character_name,
                            state,
                            texture_id,
                        };
                        valid_ops.push(NovelOps::NewCharacterState(character));
                    } else {
                        errors.push(format!(
                            "unknown character state {} for character {:?}",
                            state, character_name
                        ));
                    }
                } else {
                    errors.push(format!(
                        "cannot load texture for character {:?}",
                        character_name
                    ));
                }
            }
            UnvalidatedNovelOps::Background(name) => {
                if let Some(texture_id) = context.name_to_texture(&name) {
                    valid_ops.push(NovelOps::Background(texture_id));
                } else {
                    errors.push(format!("cannot load texture for background {:?}", name));
                }
            }
            UnvalidatedNovelOps::PlaySfx(name) => {
                if let Some(sfx_id) = context.name_to_sfx(&name) {
                    valid_ops.push(NovelOps::PlaySfx(sfx_id));
                } else {
                    errors.push(format!("cannot load sound effect for name {:?}", name));
                }
            }
            UnvalidatedNovelOps::PlayMusic(name) => {
                if let Some(music_id) = context.name_to_music(&name) {
                    valid_ops.push(NovelOps::PlayMusic(music_id));
                } else {
                    errors.push(format!("cannot load music for name {:?}", name));
                }
            }
        }
    }
    if errors.len() > 0 {
        return Err(errors.join(", ").into());
    }
    Ok(valid_ops)
}


pub fn parse_scene(
    raw_scene: &str,
    context: &impl SceneLoadingContext,
) -> VnResult<VisualNovelMachine> {
    let ops = parse_scene_to_operations(raw_scene)?;
    let ops = validate_program(ops, context)?;
    let vn = ops.into();
    Ok(vn)
}

fn parse_scene_to_operations(raw_scene: &str) -> VnResult<Vec<UnvalidatedNovelOps>> {
    let mut operations = Vec::new();
    let mut line_iter = raw_scene.lines();
    let mut errors = Vec::new();
    let mut line_number = 0;
    while let Some(line) = line_iter.next() {
        line_number += 1;
        if let Some(op) = parse_background(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_label(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_hide(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_goto(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_new_character_state(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_show_character(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_focus(line, &mut errors, line_number) {
            operations.push(op);
        }
    }
    if errors.len() > 0 {
        return Err(errors.join(", ").into());
    }
    Ok(operations)
}

fn parse_background(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("BACKGROUND") {
        return None;
    }
    let Some((_, bg_name)) = line.split_once("BACKGROUND ") else {
        return None;
    };
    if bg_name.trim().is_empty() {
        errors.push(
            format!("line {} could not parse background name from empty string, add a background name to scene", line_number)
        );
        return None;
    }
    Some(UnvalidatedNovelOps::Background(bg_name.to_owned()))
}

fn parse_label(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("LABEL") {
        return None;
    }
    let Some((_, name)) = line.split_once("LABEL ") else {
        errors.push(format!(
            "line {} could not parse label name from {}",
            line_number, line
        ));
        return None;
    };
    if name.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse label name from empty string, add a label name to scene",
            line_number
        ));
        return None;
    }
    Some(UnvalidatedNovelOps::Label(name.to_owned()))
}

fn parse_hide(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("HIDE") {
        return None;
    }
    let Some((_, position)) = line.split_once("HIDE ") else {
        errors.push(format!(
            "line {} could not parse hide position from {}",
            line_number, line
        ));
        return None;
    };
    if position.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse hide position from empty string, add a position to scene",
            line_number
        ));
        return None;
    }
    Some(UnvalidatedNovelOps::Hide(position.to_owned()))
}

fn parse_focus(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("FOCUS") {
        return None;
    }
    let Some((_, position)) = line.split_once("FOCUS ") else {
        errors.push(format!(
            "line {} could not parse focus position from {}",
            line_number, line
        ));
        return None;
    };
    if position.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse focus position from empty string, add a position to scene",
            line_number
        ));
        return None;
    }
    Some(UnvalidatedNovelOps::Focus(position.to_owned()))
}

fn parse_goto(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("GOTO") {
        return None;
    }
    let Some((_, target)) = line.split_once("GOTO ") else {
        errors.push(format!(
            "line {} could not parse goto from {}",
            line_number, line
        ));
        return None;
    };
    if target.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse goto target from empty string, add a goto target to scene",
            line_number
        ));
        return None;
    }
    Some(UnvalidatedNovelOps::Goto(target.to_owned()))
}

fn parse_new_character_state(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("CHARACTER") {
        return None;
    }
    let Some((_, name_and_state)) = line.split_once("CHARACTER ") else {
        errors.push(format!(
            "line {} could not parse character from {}",
            line_number, line
        ));
        return None;
    };
    if name_and_state.trim().is_empty() {
        errors.push(
            format!("line {} could not parse character from empty string, add a character and state to scene", line_number),
        );
        return None;
    }
    let Some((name, state)) = name_and_state.split_once(" IS ") else {
        errors.push(format!(
            "line {} could not parse character from {}, missing IS between character and state",
            line_number, line
        ));
        return None;
    };

    if name.trim().is_empty() {
        errors.push(
            format!("line {} could not parse character from empty string, add a character and state to scene", line_number),
        );
        return None;
    }
    if state.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse state from empty string, add a character and state to scene",
            line_number
        ));
        return None;
    }

    Some(UnvalidatedNovelOps::NewCharacterState(
        name.to_owned(),
        state.to_owned(),
    ))
}

fn parse_show_character(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("ENTER") {
        return None;
    }
    let Some((_, name_and_position)) = line.split_once("ENTER ") else {
        errors.push(format!(
            "line {} could not parse show character from {}",
            line_number, line
        ));
        return None;
    };
    if name_and_position.trim().is_empty() {
        errors.push(
            format!("line {} could not parse show character from empty string, add a character and position to scene", line_number),
        );
        return None;
    }
    let Some((name, position)) = name_and_position.split_once(" FROM ") else {
        errors.push(format!(
            "line {} could not parse show character from {}, missing FROM between character and position",
            line_number, line
        ));
        return None;
    };

    if name.trim().is_empty() {
        errors.push(
            format!("line {} could not parse character from empty string, add a character and position to scene", line_number),
        );
        return None;
    }
    if position.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse position from empty string, add a character and position to scene",
            line_number
        ));
        return None;
    }

    Some(UnvalidatedNovelOps::Show(
        name.to_owned(),
        position.to_owned(),
    ))
}

#[cfg(test)]
mod visual_novel_tests {
    use super::*;
    use crate::constants::{
        MUSIC_ID_PACHEBAL, MusicId, SFX_ID_BLIP, SfxId, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_STORY_BG,
        TextureId,
    };
    use std::collections::HashMap;

    struct TmpSceneLoadingContext {
        textures: HashMap<String, TextureId>,
        sounds: HashMap<String, SfxId>,
        music: HashMap<String, MusicId>,
    }

    impl Default for TmpSceneLoadingContext {
        fn default() -> TmpSceneLoadingContext {
            TmpSceneLoadingContext {
                textures: HashMap::from([
                    ("battlefield".to_owned(), TEXTURE_ID_STORY_BG),
                    ("miku".to_owned(), TEXTURE_ID_LEEKSHEET),
                    ("rin".to_owned(), TEXTURE_ID_LEEKSHEET),
                ]),
                sounds: HashMap::from([("blip".to_owned(), SFX_ID_BLIP)]),
                music: HashMap::from([("pachebal".to_owned(), MUSIC_ID_PACHEBAL)]),
            }
        }
    }

    impl SceneLoadingContext for TmpSceneLoadingContext {
        fn name_to_texture(&self, name: &str) -> Option<TextureId> {
            self.textures.get(name).copied()
        }
        fn name_to_sfx(&self, name: &str) -> Option<SfxId> {
            self.sounds.get(name).copied()
        }
        fn name_to_music(&self, name: &str) -> Option<MusicId> {
            self.music.get(name).copied()
        }
    }

    #[test]
    fn parse_label_scene_op_successfully() {
        let ops = parse_scene_to_operations("LABEL foo");
        if let Ok(ops) = ops {
            assert_eq!(ops, vec![UnvalidatedNovelOps::Label("foo".to_owned()),]);
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_label_scene_op_failure() {
        let ops = parse_scene_to_operations("LABEL");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse label name from LABEL"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_hide_scene_op_is_failure() {
        let ops = parse_scene_to_operations("HIDE ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse hide position from empty string, add a position to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_hide_scene_op_successfully() {
        let ops = parse_scene_to_operations("HIDE foo");
        if let Ok(ops) = ops {
            assert_eq!(ops, vec![UnvalidatedNovelOps::Hide("foo".to_owned()),]);
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_hide_scene_op_failure() {
        let ops = parse_scene_to_operations("HIDE");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse hide position from HIDE"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_label_scene_op_is_failure() {
        let ops = parse_scene_to_operations("LABEL ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse label name from empty string, add a label name to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_goto_scene_op_successfully() {
        let ops = parse_scene_to_operations("GOTO foo");
        if let Ok(ops) = ops {
            assert_eq!(ops, vec![UnvalidatedNovelOps::Goto("foo".to_owned()),]);
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_goto_scene_op_failure() {
        let ops = parse_scene_to_operations("GOTO");
        if let Err(problem) = ops {
            assert_eq!(problem.to_string(), "line 1 could not parse goto from GOTO");
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_goto_scene_op_is_failure() {
        let ops = parse_scene_to_operations("GOTO ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse goto target from empty string, add a goto target to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_character_state_scene_op_successfully() {
        let ops = parse_scene_to_operations("CHARACTER miku IS idle");
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![UnvalidatedNovelOps::NewCharacterState(
                    "miku".to_owned(),
                    "idle".to_owned()
                )]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_character_state_scene_op_failure() {
        let ops = parse_scene_to_operations("CHARACTER");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse character from CHARACTER"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_character_state_scene_op_is_failure() {
        let ops = parse_scene_to_operations("CHARACTER ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse character from empty string, add a character and state to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_state_character_state_scene_op_is_failure() {
        let ops = parse_scene_to_operations("CHARACTER miku IS ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse state from empty string, add a character and state to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_missing_state_character_state_scene_op_is_failure() {
        let ops = parse_scene_to_operations("CHARACTER miku IS");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse character from CHARACTER miku IS, missing IS between character and state"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_character_position_scene_op_successfully() {
        let ops = parse_scene_to_operations("ENTER miku FROM center");
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![UnvalidatedNovelOps::Show(
                    "miku".to_owned(),
                    "center".to_owned()
                )]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_character_show_scene_op_failure() {
        let ops = parse_scene_to_operations("ENTER");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse show character from ENTER"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_character_show_scene_op_is_failure() {
        let ops = parse_scene_to_operations("ENTER ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse show character from empty string, add a character and position to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_position_character_show_scene_op_is_failure() {
        let ops = parse_scene_to_operations("ENTER miku FROM ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse position from empty string, add a character and position to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_missing_position_character_show_scene_op_is_failure() {
        let ops = parse_scene_to_operations("ENTER miku FROM");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse show character from ENTER miku FROM, missing FROM between character and position"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_missing_seperator_token_character_show_scene_op_is_failure() {
        let ops = parse_scene_to_operations("ENTER miku left");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse show character from ENTER miku left, missing FROM between character and position"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_focus_scene_op_is_failure() {
        let ops = parse_scene_to_operations("FOCUS ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse focus position from empty string, add a position to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_focus_scene_op_successfully() {
        let ops = parse_scene_to_operations("FOCUS left");
        if let Ok(ops) = ops {
            assert_eq!(ops, vec![UnvalidatedNovelOps::Focus("left".to_owned()),]);
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_focus_scene_op_failure() {
        let ops = parse_scene_to_operations("FOCUS");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse focus position from FOCUS"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    /*
    SAY
    CHOICE
    */

    #[test]
    fn parse_full_scene_ops_successfully() {
        let ops = parse_scene_to_operations(concat!(
            "\nGOTO foo",
            "\nLABEL foo",
            "\nBACKGROUND battlefield",
            "\nHIDE left",
            "\nCHARACTER miku IS idle",
            "\nENTER miku FROM left",
            "\nCHOICE",
            "\n| label1 text for label one",
            "\n| label2 choice ends once no more pipes on left",
            "\n", // empty line is a no-op and means nothing
            "\nFOCUS left",
            "\nCHOICE",
            "\n| label3 text for label one of choice 2",
            "\n| label4 choice ends when there's no more stuff"
        ));
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![
                    UnvalidatedNovelOps::Goto("foo".to_owned()),
                    UnvalidatedNovelOps::Label("foo".to_owned()),
                    UnvalidatedNovelOps::Background("battlefield".to_owned()),
                    UnvalidatedNovelOps::Hide("left".to_owned()),
                    UnvalidatedNovelOps::NewCharacterState("miku".to_owned(), "idle".to_owned()),
                    UnvalidatedNovelOps::Show("miku".to_owned(), "left".to_owned()),
                    UnvalidatedNovelOps::Choices(vec![
                        Choice {
                            label: "label1".to_owned(),
                            text: "text for label one".to_owned(),
                        },
                        Choice {
                            label: "label2".to_owned(),
                            text: "choice ends once no more pipes on left".to_owned(),
                        },
                    ]),
                    UnvalidatedNovelOps::Focus("left".to_owned()),
                    UnvalidatedNovelOps::Choices(vec![
                        Choice {
                            label: "label3".to_owned(),
                            text: "text for label one of choice 2".to_owned(),
                        },
                        Choice {
                            label: "label4".to_owned(),
                            text: "choice ends when there's no more stuff".to_owned(),
                        },
                    ]),
                ]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn goto_a_label_that_doesnt_exist_is_invalid() {
        let result = validate_program(
            vec![
                UnvalidatedNovelOps::Say(
                    "a label that doesnt match will be an error on compilation".to_string(),
                ),
                UnvalidatedNovelOps::Label("304".to_string()),
                UnvalidatedNovelOps::Goto("404".to_string()),
            ],
            &TmpSceneLoadingContext::default(),
        );
        if let Err(problem) = result {
            assert_eq!(
                problem.to_string(),
                "cannot create GOTO (404) because label 404 does not exist"
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }

    #[test]
    fn goto_a_label_that_exists_is_valid() {
        let result = validate_program(
            vec![
                UnvalidatedNovelOps::Say("a label that does match will compile".to_string()),
                UnvalidatedNovelOps::Label("304".to_string()),
                UnvalidatedNovelOps::Goto("304".to_string()),
            ],
            &TmpSceneLoadingContext::default(),
        );
        if let Ok(operations) = result {
            assert_eq!(
                operations,
                vec![
                    NovelOps::Say("a label that does match will compile".to_string()),
                    NovelOps::Label("304".to_string()),
                    NovelOps::Goto("304".to_string()),
                ]
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }

    #[test]
    fn goto_a_choice_that_does_not_exist_raises_errors() {
        let result = validate_program(
            vec![
                UnvalidatedNovelOps::Say(
                    "a label that doesnt match will be an error on compilation".to_string(),
                ),
                UnvalidatedNovelOps::Label("304".to_string()),
                UnvalidatedNovelOps::Choices(vec![
                    Choice {
                        label: "404".to_owned(),
                        text: "Missing target".to_owned(),
                    },
                    Choice {
                        label: "304".to_owned(),
                        text: "Hits target, but choice will still fail".to_owned(),
                    },
                ]),
            ],
            &TmpSceneLoadingContext::default(),
        );
        if let Err(problem) = result {
            assert_eq!(
                problem.to_string(),
                "cannot create choice because label 404 does not exist"
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }

    #[test]
    fn goto_a_choice_that_does_exist_compiles() {
        let result = validate_program(
            vec![
                UnvalidatedNovelOps::Say(
                    "a label that doesnt match will be an error on compilation".to_string(),
                ),
                UnvalidatedNovelOps::Label("304".to_string()),
                UnvalidatedNovelOps::Choices(vec![
                    Choice {
                        label: "200".to_owned(),
                        text: "Target exists".to_owned(),
                    },
                    Choice {
                        label: "304".to_owned(),
                        text: "Hits target, but choice will still fail".to_owned(),
                    },
                ]),
                UnvalidatedNovelOps::Label("200".to_string()),
            ],
            &TmpSceneLoadingContext::default(),
        );
        if let Ok(operations) = result {
            assert_eq!(
                operations,
                vec![
                    NovelOps::Say(
                        "a label that doesnt match will be an error on compilation".to_string(),
                    ),
                    NovelOps::Label("304".to_string()),
                    NovelOps::Choices(vec![
                        Choice {
                            label: "200".to_owned(),
                            text: "Target exists".to_owned(),
                        },
                        Choice {
                            label: "304".to_owned(),
                            text: "Hits target, but choice will still fail".to_owned(),
                        },
                    ]),
                    NovelOps::Label("200".to_string()),
                ]
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }

    #[test]
    fn an_unknown_portrait_position_is_invalid() {
        let result = validate_program(
            vec![UnvalidatedNovelOps::Hide(
                "not a valid position".to_string(),
            )],
            &TmpSceneLoadingContext::default(),
        );
        if let Err(problem) = result {
            assert_eq!(
                problem.to_string(),
                "cannot hide character at position \"not a valid position\" as position is not valid"
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }

    #[test]
    fn hide_a_known_position_is_valid() {
        let result = validate_program(
            vec![
                UnvalidatedNovelOps::Say("a label that does match will compile".to_string()),
                UnvalidatedNovelOps::Hide("left".to_string()),
            ],
            &TmpSceneLoadingContext::default(),
        );
        if let Ok(operations) = result {
            assert_eq!(
                operations,
                vec![
                    NovelOps::Say("a label that does match will compile".to_string()),
                    NovelOps::Hide(PortraitPosition::Left),
                ]
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }
}
