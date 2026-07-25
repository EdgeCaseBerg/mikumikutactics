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
 * CHOICE | optional OPTION newlines are not needed for the first option if you hate legibility
 *   | happy OPTION You're my bestie!
 *   | friend points gained OPTION Sure is neat to have a friend like you rin!
 *   | angry rival gained OPTION you're a dumb meany doo doo head!
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
 * SAY blablabla
 * say text can go on multiple lines and shouldn't cause any problems
 * for other operations unless someone were to randomly put a keyword in
 * but as long as thigns like LABEL, ENTER, CHARACTER, SAY, CHOICE, FOCUS, HIDE
 * and all that other business like GOTO or similar aren't the first word it should
 * be fiiiine.
 */

type VnResult<T> = Result<T, Box<dyn Error>>;

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

impl VisualNovelMachine {
    pub fn advance(&mut self) {
        self.do_current_op()
        self.program_counter += 1;
    }
    pub fn do_current_op(&mut self) {
        if self.program_counter >= self.program.len() {
            return;
        }
        match self.program[self.program_counter] {
            NovelOps::Background(texture_id) => {
                self.background = Some(texture_id);
            }
            _ => {}
        }
    }
    pub fn jump(&mut self, label: &str) {
        if self.program.is_empty() {
            return;
        }

        // find the first instance of this label. (both directions)
        let mut i = self.program_counter;
        let mut j = self.program_counter;

        if self.program[self.program_counter].is_label(label) {
            // Do nothing. We're already at the label.
            return;
        }

        // Since a label may be backwards, search in both directions at once.
        let max_idx = self.program.len() - 1;
        loop {
            i = i.saturating_sub(1).min(0);
            j = j.saturating_add(1).max(max_idx);

            if self.program[i].is_label(label) {
                self.program_counter = i;
                return;
            }

            if i == 0 && j > max_idx {
                break;
            }

            if self.program[j].is_label(label) {
                self.program_counter = j;
                return;
            }
        }
    }
}

// clippy is all like "wah wah you match rust defaults use OUR Derive"
// but its better for us to have this spelled out explicitly here because
// it makes it more obvious that we're choosing these values on purpose.
// clippy's default lints channeling Microsoft Office Clippy's spirit.
// I'd [allow(clippy::derivable_impls)] add, but I added it to the toml instead
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
        VisualNovelMachine {
            program: ops,
            ..Default::default()
        }
    }
}

/// Trait for a scene or backend to implement in order to convert stuff like 'miku idle' into a proper sprite texture
pub trait SceneLoadingContext {
    fn name_to_texture(&self, name: &str) -> Option<TextureId>;
    fn name_to_sfx(&self, name: &str) -> Option<SfxId>;
    fn name_to_music(&self, name: &str) -> Option<MusicId>;
}

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
    pub name: String,
    pub state: CharacterState,
    pub texture_id: TextureId,
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

impl NovelOps {
    pub fn is_label(&self, label: &str) -> bool {
        if let NovelOps::Label(s) = self {
            label == s
        } else {
            false
        }
    }
}

fn validate_program(
    raw: Vec<UnvalidatedNovelOps>,
    context: &impl SceneLoadingContext,
) -> VnResult<Vec<NovelOps>> {
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

    let mut valid_ops = Vec::with_capacity(raw.len());
    let mut errors = vec![];
    for unvalidated in raw {
        match unvalidated {
            UnvalidatedNovelOps::Say(_) => {
                if let Some(op) = validate_say(unvalidated) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Label(_) => {
                if let Some(op) = validate_label(unvalidated) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Goto(_) => {
                if let Some(op) = validate_goto(unvalidated, &set_of_all_labels, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Hide(_) => {
                if let Some(op) = validate_hide(unvalidated, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Focus(_) => {
                if let Some(op) = validate_focus(unvalidated, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Background(_) => {
                if let Some(op) = validate_background(unvalidated, context, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::PlaySfx(_) => {
                if let Some(op) = validate_play_sfx(unvalidated, context, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::PlayMusic(_) => {
                if let Some(op) = validate_play_music(unvalidated, context, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Choices(_) => {
                if let Some(op) = validate_choices(unvalidated, &set_of_all_labels, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::Show(..) => {
                if let Some(op) = validate_show(unvalidated, context, &mut errors) {
                    valid_ops.push(op);
                }
            }
            UnvalidatedNovelOps::NewCharacterState(..) => {
                if let Some(op) = validate_new_character_state(unvalidated, context, &mut errors) {
                    valid_ops.push(op);
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join(", ").into());
    }
    Ok(valid_ops)
}

fn validate_say(unvalidated: UnvalidatedNovelOps) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Say(s) = unvalidated else {
        return None;
    };
    Some(NovelOps::Say(s))
}

fn validate_label(unvalidated: UnvalidatedNovelOps) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Label(s) = unvalidated else {
        return None;
    };
    Some(NovelOps::Label(s))
}

fn validate_goto(
    unvalidated: UnvalidatedNovelOps,
    set_of_all_labels: &HashSet<String>,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Goto(s) = unvalidated else {
        return None;
    };
    if !set_of_all_labels.contains(&s) {
        errors.push(format!(
            "cannot create GOTO ({0}) because label {0} does not exist",
            s
        ));
        None
    } else {
        Some(NovelOps::Goto(s))
    }
}
fn validate_hide(unvalidated: UnvalidatedNovelOps, errors: &mut Vec<String>) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Hide(pp) = unvalidated else {
        return None;
    };
    if let Some(pp) = PortraitPosition::lift(&pp) {
        Some(NovelOps::Hide(pp))
    } else {
        errors.push(format!(
            "cannot hide character at position {:?} as position is not valid",
            pp
        ));
        None
    }
}

fn validate_focus(unvalidated: UnvalidatedNovelOps, errors: &mut Vec<String>) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Focus(pp) = unvalidated else {
        return None;
    };
    if let Some(pp) = PortraitPosition::lift(&pp) {
        Some(NovelOps::Focus(pp))
    } else {
        errors.push(format!(
            "cannot focus position {:?} as position is not valid",
            pp
        ));
        None
    }
}

fn validate_background(
    unvalidated: UnvalidatedNovelOps,
    context: &impl SceneLoadingContext,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Background(name) = unvalidated else {
        return None;
    };
    if let Some(texture_id) = context.name_to_texture(&name) {
        Some(NovelOps::Background(texture_id))
    } else {
        errors.push(format!("cannot load texture for background {:?}", name));
        None
    }
}

fn validate_play_sfx(
    unvalidated: UnvalidatedNovelOps,
    context: &impl SceneLoadingContext,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::PlaySfx(name) = unvalidated else {
        return None;
    };
    if let Some(sfx_id) = context.name_to_sfx(&name) {
        Some(NovelOps::PlaySfx(sfx_id))
    } else {
        errors.push(format!("cannot load sound effect for name {:?}", name));
        None
    }
}

fn validate_play_music(
    unvalidated: UnvalidatedNovelOps,
    context: &impl SceneLoadingContext,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::PlayMusic(name) = unvalidated else {
        return None;
    };
    if let Some(music_id) = context.name_to_music(&name) {
        Some(NovelOps::PlayMusic(music_id))
    } else {
        errors.push(format!("cannot load music for name {:?}", name));
        None
    }
}

fn validate_choices(
    unvalidated: UnvalidatedNovelOps,
    set_of_all_labels: &HashSet<String>,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Choices(choices) = unvalidated else {
        return None;
    };
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
        return Some(NovelOps::Choices(choices));
    }
    None
}

fn validate_show(
    unvalidated: UnvalidatedNovelOps,
    context: &impl SceneLoadingContext,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::Show(character_name, pp) = unvalidated else {
        return None;
    };

    let maybe_texture_id = context.name_to_texture(&character_name);
    let maybe_pp = PortraitPosition::lift(&pp);

    if maybe_texture_id.is_none() {
        errors.push(format!(
            "cannot load texture for character {:?}",
            character_name
        ));
    }
    if maybe_pp.is_none() {
        errors.push(format!(
            "cannot show character at position {:?} as position is not valid",
            pp
        ));
    }

    let texture_id = maybe_texture_id?;
    let pp = maybe_pp?;

    let character = Character {
        name: character_name,
        state: CharacterState::Idle,
        texture_id,
    };
    Some(NovelOps::Show(character, pp))
}

fn validate_new_character_state(
    unvalidated: UnvalidatedNovelOps,
    context: &impl SceneLoadingContext,
    errors: &mut Vec<String>,
) -> Option<NovelOps> {
    let UnvalidatedNovelOps::NewCharacterState(character_name, state) = unvalidated else {
        return None;
    };
    let maybe_texture_id = context.name_to_texture(&character_name);
    let maybe_state = CharacterState::lift(&state);
    if maybe_texture_id.is_none() {
        errors.push(format!(
            "cannot load texture for character {:?}",
            character_name
        ));
    }
    if maybe_state.is_none() {
        errors.push(format!(
            "unknown character state {} for character {:?}",
            state, character_name
        ));
    }
    let texture_id = maybe_texture_id?;
    let state = maybe_state?;
    let character = Character {
        name: character_name,
        state,
        texture_id,
    };
    Some(NovelOps::NewCharacterState(character))
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
    let mut line_iter = raw_scene.lines().peekable();
    let mut errors = Vec::new();
    let mut line_number = 0;
    while let Some(line) = line_iter.next() {
        line_number += 1;
        if let Some(op) = parse_single_line_ops(line, &mut errors, line_number) {
            operations.push(op);
        }
        if let Some(op) = parse_choices(line, &mut errors, &mut line_number, &mut line_iter) {
            operations.push(op);
        }
        // we parse say last since its the only op that spans multiple lines
        // in a way that has no delimiter beyond the SAY itself, so we are
        // in "SAY" mode for a while until we hit a fully blank line.
        let mut ops = parse_say(line, &mut line_iter, &mut line_number);
        if !ops.is_empty() {
            operations.append(&mut ops);
        }
    }
    if !errors.is_empty() {
        return Err(errors.join(", ").into());
    }
    Ok(operations)
}

fn parse_single_line_ops(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if let Some(op) = parse_background(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_label(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_hide(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_goto(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_new_character_state(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_show_character(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_focus(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_play(line, errors, line_number) {
        return Some(op);
    }
    if let Some(op) = parse_music(line, errors, line_number) {
        return Some(op);
    }
    if parse_choice_option(line, errors, line_number).is_some() {
        errors.push(format!(
            "line {} invalid state, trying to add choice option when no choice has been started",
            line_number
        ));
    }
    None
}

fn parse_choices<'a>(
    line: &str,
    errors: &mut Vec<String>,
    line_number: &mut usize,
    line_iter: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
) -> Option<UnvalidatedNovelOps> {
    let (_, potential_inline_choice) = line.split_once("CHOICE")?;

    let mut choices = if let Some(c) =
        parse_choice_option(potential_inline_choice.trim(), errors, *line_number)
    {
        vec![c]
    } else if potential_inline_choice.trim().is_empty() {
        vec![]
    } else {
        errors.push(
            format!("invalid inline choice after start of CHOICE on line {}, ensure you begin the option with |", line_number)
        );
        vec![]
    };

    // then advance the iter
    let mut option_line_number = *line_number;
    while let Some(next_line) = line_iter.peek() {
        if let Some(choice) = parse_choice_option(next_line, errors, option_line_number) {
            option_line_number += 1;
            choices.push(choice);
            line_iter.next();
            continue;
        } else {
            break;
        }
    }

    *line_number += option_line_number - *line_number;

    if line_iter.peek().is_none() && choices.is_empty() {
        errors.push(
            format!("script ended before choice options were included, your script likely needs options for your CHOICE on line {}", line_number)
        );
    } else if choices.is_empty() {
        errors.push(
            format!("no choice options found for CHOICE, your script likely needs options for your CHOICE on line {}", line_number)
        );
    }

    Some(UnvalidatedNovelOps::Choices(choices))
}

fn parse_choice_option(line: &str, errors: &mut Vec<String>, line_number: usize) -> Option<Choice> {
    if !line.starts_with('|') {
        return None;
    }
    let (label, text) = line.split_once(" OPTION")?;
    let c = if let Some(label) = label.strip_prefix('|') {
        Choice {
            label: label.trim().to_owned(),
            text: text.trim().to_owned(),
        }
    } else {
        Choice {
            label: label.trim().to_owned(),
            text: text.trim().to_owned(),
        }
    };
    if c.label.is_empty() {
        errors.push(format!(
            "line {} option is missing label in CHOICE",
            line_number
        ));
    }
    if c.text.is_empty() {
        errors.push(format!(
            "line {} option is missing text in CHOICE, use ... for an empty choice if needed",
            line_number
        ));
    }
    Some(c)
}

fn parse_say<'a>(
    line: &str,
    line_iter: &mut std::iter::Peekable<impl Iterator<Item = &'a str>>,
    line_number: &mut usize,
) -> Vec<UnvalidatedNovelOps> {
    let Some((_, text)) = line.split_once("SAY") else {
        return vec![];
    };

    let mut words = vec![];
    // We are now in SAY mode. Start chewing lines until we hit either a
    // fresh newline with nothing on it, the end of the stream, OR another
    // known operation (which we can check via \nCHOICE and parse_simple )
    if !text.is_empty() {
        words.push(text.trim());
    }

    if line_iter.peek().is_none() {
        return vec![UnvalidatedNovelOps::Say(words.join(" "))];
    }

    let mut ops = vec![];
    while let Some(next_line) = line_iter.peek() {
        if next_line.trim().is_empty() {
            // DONE! We found a boundary for SAY
            ops.push(UnvalidatedNovelOps::Say(words.join(" ")));
            words = vec![];
            line_iter.next();
            *line_number += 1;
            continue;
        }

        let upcoming_choice = next_line.starts_with("CHOICE");
        let has_single_line_op = parse_single_line_ops(next_line, &mut vec![], 0).is_some();
        if has_single_line_op || upcoming_choice {
            // There is a new op, so SAY is ending. Break out.
            let w = words.join(" ");
            if !w.is_empty() {
                ops.push(UnvalidatedNovelOps::Say(w));
            }
            return ops;
        }

        // otherwise, we're still in the say, so add it to the words
        // and consume the iter.
        words.push(next_line);
        line_iter.next();
        *line_number += 1;
    }

    let w = words.join(" ");
    if !w.is_empty() {
        ops.push(UnvalidatedNovelOps::Say(w));
    }

    ops
}

fn parse_play(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("PLAY") {
        return None;
    }
    let Some((_, name)) = line.split_once("PLAY ") else {
        errors.push(format!(
            "line {} could not parse sound effect name from {}",
            line_number, line
        ));
        return None;
    };
    if name.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse PLAY sound effect name from empty string",
            line_number
        ));
        return None;
    }
    Some(UnvalidatedNovelOps::PlaySfx(name.to_owned()))
}

fn parse_music(
    line: &str,
    errors: &mut Vec<String>,
    line_number: usize,
) -> Option<UnvalidatedNovelOps> {
    if !line.starts_with("MUSIC") {
        return None;
    }
    let Some((_, name)) = line.split_once("MUSIC ") else {
        errors.push(format!(
            "line {} could not parse music name from {}",
            line_number, line
        ));
        return None;
    };
    if name.trim().is_empty() {
        errors.push(format!(
            "line {} could not parse music name from empty string",
            line_number
        ));
        return None;
    }
    Some(UnvalidatedNovelOps::PlayMusic(name.to_owned()))
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
        errors.push(format!(
            "line {} could not parse background name from {}",
            line_number, line
        ));
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
    fn visual_novel_jump_to_forward_label() {
        let mut vn: VisualNovelMachine = vec![
            NovelOps::Label("start".to_owned()),
            NovelOps::Label("1".to_owned()),
            NovelOps::Label("middle".to_owned()),
            NovelOps::Label("3".to_owned()),
            NovelOps::Label("end".to_owned()),
        ]
        .into();
        vn.program_counter = 2; // start at middle.
        vn.jump("end");
        assert_eq!(4, vn.program_counter);
    }

    #[test]
    fn visual_novel_jump_to_backwards_label() {
        let mut vn: VisualNovelMachine = vec![
            NovelOps::Label("start".to_owned()),
            NovelOps::Label("1".to_owned()),
            NovelOps::Label("middle".to_owned()),
            NovelOps::Label("3".to_owned()),
            NovelOps::Label("end".to_owned()),
        ]
        .into();
        vn.program_counter = 2; // start at middle.
        vn.jump("start");
        assert_eq!(0, vn.program_counter);
    }
    #[test]
    fn visual_novel_jump_to_nonexisting_label_does_move_counter() {
        let mut vn: VisualNovelMachine = vec![
            NovelOps::Label("start".to_owned()),
            NovelOps::Label("1".to_owned()),
            NovelOps::Label("middle".to_owned()),
            NovelOps::Label("3".to_owned()),
            NovelOps::Label("end".to_owned()),
        ]
        .into();
        vn.program_counter = 4;
        vn.jump("I DONT EXIST");
        assert_eq!(4, vn.program_counter);
    }
    #[test]
    fn visual_novel_jump_to_label_already_on_does_nothing() {
        let mut vn: VisualNovelMachine = vec![
            NovelOps::Label("start".to_owned()),
            NovelOps::Label("1".to_owned()),
            NovelOps::Label("middle".to_owned()),
            NovelOps::Label("3".to_owned()),
            NovelOps::Label("end".to_owned()),
        ]
        .into();
        vn.program_counter = 1;
        vn.jump("1");
        assert_eq!(1, vn.program_counter);
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

    #[test]
    fn parse_empty_choice_scene_op_failure() {
        let ops = parse_scene_to_operations("CHOICE");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "script ended before choice options were included, your script likely needs options for your CHOICE on line 1"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
        let ops = parse_scene_to_operations("CHOICE ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "script ended before choice options were included, your script likely needs options for your CHOICE on line 1"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_single_option_choice_scene_op_successfully() {
        let ops = parse_scene_to_operations("CHOICE | label OPTION using pipe as expected");
        if let Ok(ref choices) = ops {
            assert_eq!(choices.len(), 1, "expected choices to be non-empty");
            let UnvalidatedNovelOps::Choices(choices) = choices.iter().next().unwrap() else {
                panic!("expected successful parse, got {:?}", ops);
            };
            assert_eq!(
                choices,
                &vec![Choice {
                    label: "label".to_owned(),
                    text: "using pipe as expected".to_owned()
                }]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_single_option_choice_scene_op_failed_on_empty_label() {
        let ops = parse_scene_to_operations("CHOICE | OPTION using pipe as expected");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 option is missing label in CHOICE",
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_invalid_inline_option_choice_scene_op_failure() {
        let ops = parse_scene_to_operations(
            "CHOICE OPTION using pipe as expected\n|banana OPTION two\nSAY hi",
        );
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "invalid inline choice after start of CHOICE on line 1, ensure you begin the option with |"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_single_option_choice_scene_op_failed_on_empty_text() {
        let ops = parse_scene_to_operations("CHOICE |label OPTION ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 option is missing text in CHOICE, use ... for an empty choice if needed"
                    .to_owned(),
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_typical_option_choice_in_scene_op_successfully() {
        let ops = parse_scene_to_operations(concat!(
            "CHOICE\n",
            "| label 1 OPTION text1\n",
            "| label 2 OPTION text2\n",
        ));
        if let Ok(ref choices) = ops {
            assert_eq!(choices.len(), 1, "expected choices to be non-empty");
            let UnvalidatedNovelOps::Choices(choices) = choices.iter().next().unwrap() else {
                panic!("expected successful parse, got {:?}", ops);
            };
            assert_eq!(
                choices,
                &vec![
                    Choice {
                        label: "label 1".to_owned(),
                        text: "text1".to_owned()
                    },
                    Choice {
                        label: "label 2".to_owned(),
                        text: "text2".to_owned()
                    }
                ]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_two_inline_choices_successfully() {
        let ops = parse_scene_to_operations(concat!(
            "CHOICE | label 1 OPTION text1\n",
            "CHOICE | label 2 OPTION text2\n",
        ));
        if let Ok(ref choices) = ops {
            assert_eq!(choices.len(), 2, "expected choices to be non-empty");
            let mut choices_iter = choices.iter();
            let UnvalidatedNovelOps::Choices(choices1) = choices_iter.next().unwrap() else {
                panic!("expected successful parse, got {:?}", ops);
            };
            let UnvalidatedNovelOps::Choices(choices2) = choices_iter.next().unwrap() else {
                panic!("expected successful parse, got {:?}", ops);
            };
            assert_eq!(
                choices1,
                &vec![Choice {
                    label: "label 1".to_owned(),
                    text: "text1".to_owned()
                },]
            );
            assert_eq!(
                choices2,
                &vec![Choice {
                    label: "label 2".to_owned(),
                    text: "text2".to_owned()
                }]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_floating_option_choice_scene_op_fail_even_after_open_choice() {
        let ops = parse_scene_to_operations(concat!(
            "CHOICE\n",
            "| label 1 OPTION text1\n",
            "| label 2 OPTION text2\n",
            "\n",
            "| label 2 OPTION floating text\n",
        ));
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 5 invalid state, trying to add choice option when no choice has been started"
                    .to_owned(),
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_floatinge_option_choice_scene_op_fail() {
        let ops = parse_scene_to_operations("|label OPTION text");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 invalid state, trying to add choice option when no choice has been started"
                    .to_owned(),
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_say_text_from_a_single_line_successfully() {
        let ops = parse_scene_to_operations(concat!("SAY some text on a line"));
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![UnvalidatedNovelOps::Say("some text on a line".to_owned()),]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_say_text_from_multiple_lines_successfully() {
        let ops = parse_scene_to_operations(concat!(
            "SAY\n",
            "say text can go on multiple lines and shouldn't cause any problems\n",
            "for other operations unless someone were to randomly put a keyword in\n",
            "but as long as thigns like LABEL, ENTER, CHARACTER, SAY, CHOICE, FOCUS, HIDE\n",
            "and all that other business like GOTO or similar aren't the first word it should\n",
            "be fiiiine.\n",
        ));
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![UnvalidatedNovelOps::Say(concat!(
                    "say text can go on multiple lines and shouldn't cause any problems ",
                    "for other operations unless someone were to randomly put a keyword in ",
                    "but as long as thigns like LABEL, ENTER, CHARACTER, SAY, CHOICE, FOCUS, HIDE ",
                    "and all that other business like GOTO or similar aren't the first word it should ",
                    "be fiiiine.",
                ).to_owned()),]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_say_text_with_operations_between() {
        let ops = parse_scene_to_operations(concat!(
            "SAY some text on a line\n",
            "HIDE left\n",
            "SAY some text on another line\n",
        ));
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![
                    UnvalidatedNovelOps::Say("some text on a line".to_owned()),
                    UnvalidatedNovelOps::Hide("left".to_owned()),
                    UnvalidatedNovelOps::Say("some text on another line".to_owned()),
                ]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_say_text_with_empty_lines_between() {
        let ops = parse_scene_to_operations(concat!(
            "SAY like if someone is writing a paragraph of text\n",
            "\n",
            "and wanted to have a couple of those. the question is if\n",
            "that should be broken out into multiple says or not, or if\n",
            "the newline is respected at all and turned into another say?\n",
        ));
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![
                    UnvalidatedNovelOps::Say(
                        "like if someone is writing a paragraph of text".to_owned()
                    ),
                    UnvalidatedNovelOps::Say(
                        concat!(
                            "and wanted to have a couple of those. the question is if ",
                            "that should be broken out into multiple says or not, or if ",
                            "the newline is respected at all and turned into another say?",
                        )
                        .to_owned()
                    ),
                ]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_play_scene_op_is_failure() {
        let ops = parse_scene_to_operations("PLAY ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse PLAY sound effect name from empty string"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_sound_scene_op_successfully() {
        let ops = parse_scene_to_operations("PLAY blip");
        if let Ok(ops) = ops {
            assert_eq!(ops, vec![UnvalidatedNovelOps::PlaySfx("blip".to_owned()),]);
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_play_no_name_scene_op_failure() {
        let ops = parse_scene_to_operations("PLAY");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse sound effect name from PLAY"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_music_scene_op_is_failure() {
        let ops = parse_scene_to_operations("MUSIC ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse music name from empty string"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_music_scene_op_successfully() {
        let ops = parse_scene_to_operations("MUSIC pachebal");
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![UnvalidatedNovelOps::PlayMusic("pachebal".to_owned()),]
            );
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_music_no_name_scene_op_failure() {
        let ops = parse_scene_to_operations("MUSIC");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse music name from MUSIC"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_empty_background_to_scene_op_failure() {
        let ops = parse_scene_to_operations("BACKGROUND ");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse background name from empty string, add a background name to scene"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_missing_background_to_scene_op_failure() {
        let ops = parse_scene_to_operations("BACKGROUND");
        if let Err(problem) = ops {
            assert_eq!(
                problem.to_string(),
                "line 1 could not parse background name from BACKGROUND"
            );
        } else {
            panic!("expected unsuccessful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_background_to_scene_op_success() {
        let ops = parse_scene_to_operations("BACKGROUND battlefield");
        if let Ok([UnvalidatedNovelOps::Background(name)]) = ops.as_deref() {
            assert_eq!("battlefield", name);
        } else {
            panic!("expected successful parse, got {:?}", ops);
        }
    }

    #[test]
    fn parse_full_scene_ops_successfully() {
        let ops = parse_scene_to_operations(concat!(
            "\nGOTO foo",
            "\nLABEL foo",
            "\nBACKGROUND battlefield",
            "\nHIDE left",
            "\nPLAY blip",
            "\nCHARACTER miku IS idle",
            "\nMUSIC bgmusic",
            "\nSAY",
            "\nIt was a lovely day out and time for Miku to sing a song",
            "\nENTER miku FROM left",
            "\nCHOICE",
            "\n| label1 OPTION text for label one",
            "\n| label2 OPTION choice ends once no more pipes on left",
            "\n", // empty line is a no-op and means nothing
            "\nFOCUS left",
            "\nCHOICE",
            "\n| label3 OPTION text for label one of choice 2",
            "\n| label4 OPTION choice ends when there's no more stuff"
        ));
        if let Ok(ops) = ops {
            assert_eq!(
                ops,
                vec![
                    UnvalidatedNovelOps::Goto("foo".to_owned()),
                    UnvalidatedNovelOps::Label("foo".to_owned()),
                    UnvalidatedNovelOps::Background("battlefield".to_owned()),
                    UnvalidatedNovelOps::Hide("left".to_owned()),
                    UnvalidatedNovelOps::PlaySfx("blip".to_owned()),
                    UnvalidatedNovelOps::NewCharacterState("miku".to_owned(), "idle".to_owned()),
                    UnvalidatedNovelOps::PlayMusic("bgmusic".to_owned()),
                    UnvalidatedNovelOps::Say(
                        "It was a lovely day out and time for Miku to sing a song".to_owned()
                    ),
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

    #[test]
    fn when_show_has_multiple_errors_both_are_reported() {
        let result = validate_program(
            vec![UnvalidatedNovelOps::Show(
                "noone".to_owned(),
                "badpp".to_owned(),
            )],
            &TmpSceneLoadingContext::default(),
        );
        if let Err(problem) = result {
            assert_eq!(
                problem.to_string(),
                "cannot load texture for character \"noone\", cannot show character at position \"badpp\" as position is not valid"
            );
        } else {
            panic!("should have had an error, had: {:?}", result);
        }
    }
}
