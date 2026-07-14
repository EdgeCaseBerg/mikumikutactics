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
#[derive(Debug, Clone)]
pub enum NovelOps {
    Say(String),
    Choices(Vec<Choice>),
    Label(String),
    Goto(String),
    Show(CHARACTER, PortraitPosition),
    Hide(PortraitPosition),
    SetCharacterState(CHARACTER, CharacterState),
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
pub struct CHARACTER {
    name: String,
    state: CharacterState,
}

#[derive(Debug, Clone)]
pub enum CharacterState {
    Idle,
    Happy,
    Mad,
}

#[derive(Debug, Clone)]
pub enum PortraitPosition {
    Left,
    Center,
    Right,
}
