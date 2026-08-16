use jzon::JsonValue;
use std::error::Error;

/*

IMPORTANT NOTE!!!

⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣶⣿⣿⣷⣦⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣶⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⣿⡟⢡⣤⣬⣝⠻⣿⣷⣄⠀⠀⠀⠀⠀⠀⣠⣿⣿⣿⣿⠟⣫⣥⡝⣷⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⢧⡿⠁⠀⠈⢻⣦⠹⣿⣶⠶⠒⠒⠒⠚⠛⠿⠿⢭⣖⠉⠀⠀⣇⢸⡇⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣼⡇⠀⠀⢀⡤⠛⠓⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠓⣄⣠⡿⢸⡇⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⡇⣧⣤⠞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠛⢣⣼⡇⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⡷⠋⠀⢠⠆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠱⣆⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣿⣿⣿⡟⠁⠀⣰⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠀⠀⠀⠀⠀⠸⡄⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⡿⠀⠀⢰⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡟⡄⠀⠀⠀⠀⠀⣷⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⣍⡻⣿⣿⣿⠁⠀⢰⠇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⠞⠀⢇⠀⠀⠀⠀⠀⢹⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⣿⣿⣧⢻⣿⣿⠀⠀⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⡀⠀⠀⠀⠀⠀⣠⠛⠀⠀⠸⡆⢠⡄⠀⠀⣸⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⢨⣿⣿⡿⣸⣿⣿⠀⢰⠃⠀⢠⣼⠆⠀⠀⠀⠀⢀⡀⠙⠢⠀⠀⢀⠞⠁⠀⠀⠀⣀⠜⢞⡇⠀⠀⡿⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢠⠎⢿⣏⣱⣿⣿⠏⠀⠾⠶⠶⢻⠘⠶⠶⠶⠤⠤⠞⠧⠤⣄⣠⠴⠋⠀⠀⠀⠀⠈⠁⠀⠈⡇⠀⢰⠁⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⡎⠀⠈⠻⢿⣿⣿⠀⠀⠀⠀⠀⢸⠀⠀⠙⢿⣶⣶⣤⣄⣤⠀⠀⠀⠀⠀⠀⠀⢀⣤⣶⣶⡶⠿⢶⣺⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⢇⡀⣠⡀⠀⢹⠃⠀⠀⠀⣀⣠⠜⠀⠀⠀⠀⠀⠈⠉⠉⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠀⠀⠀⠀⠀⠹⣄⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠋⢳⡄⣈⣷⣤⣠⣾⣿⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢢⣀⡠⣄⣀⣀⡀⠀⠀⠀⠀⠀⠀⠀⢀⣿⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠻⣿⣿⣿⢿⣿⣿⠒⠦⣤⣄⡀⠀⠀⠀⠀⠀⢸⣇⣀⣬⠟⠉⠀⠀⠀⢀⣠⣤⣴⣶⣿⣿⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⣿⠀⠀⠀⠹⡔⠚⠁⡠⠛⡍⠙⣛⠿⡖⠒⠦⢤⡤⠒⢶⢛⠟⠛⠣⢤⡤⠜⠛⠛⠛⠒⠤⡀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡴⠋⣀⡤⠶⠶⣧⢤⣼⡁⠀⠻⣺⠉⠀⣵⢄⡴⠊⠁⣀⣼⠋⠀⠀⠀⣾⡇⠀⠀⠀⠀⠀⠀⣻⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⡬⠛⠉⣤⠚⠛⠳⢼⡇⠀⠙⣦⣼⠁⠀⠀⢸⣇⡤⠞⠋⢠⠃⠀⠀⠀⠀⢣⠳⣤⣀⣀⣠⣤⠖⠃⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢧⠀⣠⠟⣠⠴⠒⠒⠦⣽⡶⣼⣿⣿⣷⣶⣶⣿⣿⣿⡧⠤⠼⢤⡀⠀⠀⠀⠈⠣⢄⣀⣀⠟⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠶⣈⢦⣿⠀⠀⠀⠀⠈⡇⠻⣿⣿⣿⣿⣿⣿⣿⡿⠁⣀⡠⠶⠻⣦⣀⠀⠀⠀⢈⣹⠏⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣤⠛⠛⣿⠷⣄⠀⠀⢠⣇⣴⣿⠿⠛⠘⠟⢻⣿⣷⣏⠁⠀⠀⠀⠈⣮⡛⢢⣴⣿⣿⣿⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⣾⣿⣴⡛⠀⠀⠈⠉⣉⣵⣿⣿⣛⠀⠀⠀⠀⠀⠻⣿⣿⣷⣄⠀⠀⢀⣸⣿⣿⣿⣿⡿⠟⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢿⣿⣿⣶⣀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣶⣶⣶⣶⣾⣿⣿⣿⣷⣶⣿⣿⣿⣿⡿⠏⠁⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣾⣿⣭⣭⠿⠉⠉⠉⠉⠉⠉⠹⢦⣶⣿⣿⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠈⢻⣿⣿⣿⣿⣿⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⠿⠟⠛⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠛⠛⠿⠟⠛⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀

Momoi has stopped you here to let you know that this file is included into
the build.rs file via the include macro. So don't add anything to it that
would break because the dependencies between the project and the build file
are separate!

*/

type AssetResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Manifest {
    pub spritesheets: Vec<Spritesheet>,
    pub sounds: Vec<FileAsset>,
    pub music: Vec<FileAsset>,
    pub scenes: Vec<FileAsset>,
}

#[derive(Debug)]
pub struct Spritesheet {
    pub id: usize,
    pub name: String,
    pub asset_path: String,
    pub frames: Vec<SpriteFrame>,
    pub tags: Vec<FrameTag>,
}

#[derive(Debug, Clone)]
pub struct SpriteFrame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub duration_ms: u32,
}

#[derive(Debug, Clone)]
pub struct FrameTag {
    pub name: String,
    pub from: u32,
    pub to: u32,
}

#[derive(Debug)]
pub struct FileAsset {
    pub id: usize,
    pub name: String,
    pub asset_path: String,
}

// Writes are simple:
impl From<Manifest> for JsonValue {
    fn from(val: Manifest) -> Self {
        jzon::object! {
            spritesheets: val.spritesheets,
            sounds: val.sounds,
            music: val.music,
            scenes: val.scenes,
        }
    }
}
impl From<Spritesheet> for JsonValue {
    fn from(val: Spritesheet) -> Self {
        jzon::object! {
            id: val.id,
            name: val.name,
            asset_path: val.asset_path,
            frames: val.frames,
            tags: val.tags,
        }
    }
}
impl From<SpriteFrame> for JsonValue {
    fn from(val: SpriteFrame) -> Self {
        jzon::object! {
            x: val.x,
            y: val.y,
            w: val.w,
            h: val.h,
            duration_ms: val.duration_ms,
        }
    }
}
impl From<FrameTag> for JsonValue {
    fn from(val: FrameTag) -> Self {
        jzon::object! {
            name: val.name,
            from: val.from,
            to: val.to,
        }
    }
}
impl From<FileAsset> for JsonValue {
    fn from(val: FileAsset) -> Self {
        jzon::object! {
            id: val.id,
            name: val.name,
            asset_path: val.asset_path,
        }
    }
}

// Reads are a little more complicated:

impl TryFrom<JsonValue> for Manifest {
    type Error = Box<dyn Error>;
    fn try_from(obj: JsonValue) -> AssetResult<Manifest> {
        let Some(root) = obj.as_object() else {
            return Err("unexpected shape of manifest file, root object was not an object".into());
        };

        let Some(spritesheets) = root["spritesheets"].as_array() else {
            return Err(
                "unexpected shape of manifest file, root object missing spritesheets list".into(),
            );
        };

        let Some(sounds) = root["sounds"].as_array() else {
            return Err(
                "unexpected shape of manifest file, root object missing sounds list".into(),
            );
        };

        let Some(scenes) = root["scenes"].as_array() else {
            return Err(
                "unexpected shape of manifest file, root object missing scenes list".into(),
            );
        };

        let Some(music) = root["music"].as_array() else {
            return Err("unexpected shape of manifest file, root object missing music list".into());
        };

        let mut errors = vec![];
        let spritesheets: Vec<Spritesheet> = spritesheets
            .iter()
            .filter_map(|json_val| {
                let f: AssetResult<Spritesheet> = json_val.try_into();
                match f {
                    Ok(s) => Some(s),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                }
            })
            .collect();

        let sounds: Vec<FileAsset> = sounds
            .iter()
            .filter_map(|json_val| {
                let f: AssetResult<FileAsset> = json_val.try_into();
                match f {
                    Ok(s) => Some(s),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                }
            })
            .collect();

        let scenes: Vec<FileAsset> = scenes
            .iter()
            .filter_map(|json_val| {
                let f: AssetResult<FileAsset> = json_val.try_into();
                match f {
                    Ok(s) => Some(s),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                }
            })
            .collect();

        let music: Vec<FileAsset> = music
            .iter()
            .filter_map(|json_val| {
                let f: AssetResult<FileAsset> = json_val.try_into();
                match f {
                    Ok(s) => Some(s),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(" ")
                .into());
        }

        Ok(Manifest {
            spritesheets,
            sounds,
            music,
            scenes,
        })
    }
}

impl TryFrom<&JsonValue> for Spritesheet {
    type Error = Box<dyn Error>;

    fn try_from(obj: &JsonValue) -> Result<Self, Self::Error> {
        let mut errors = vec![];
        let id = match obj["id"].as_usize() {
            Some(id) => id,
            None => {
                errors.push("missing or invalid id field in Spritesheet".into());
                0
            }
        };
        let name = match obj["name"].as_str() {
            Some(name) => name,
            None => {
                errors.push("missing or invalid name field in Spritesheet".into());
                ""
            }
        };
        let asset_path = match obj["asset_path"].as_str() {
            Some(asset_path) => asset_path,
            None => {
                errors.push("missing or invalid asset_path field in Spritesheet".into());
                ""
            }
        };
        let raw_frames = match obj["frames"].as_array() {
            Some(raw) => raw,
            None => {
                errors.push("missing or invalid frames field in Spritesheet".into());
                &vec![]
            }
        };
        let raw_tags = match obj["tags"].as_array() {
            Some(raw) => raw,
            None => {
                errors.push("missing or invalid tags field in Spritesheet".into());
                &vec![]
            }
        };

        let frames = raw_frames
            .iter()
            .filter_map(|json_val| {
                let f: AssetResult<SpriteFrame> = json_val.try_into();
                match f {
                    Ok(f) => Some(f),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                }
            })
            .collect();

        let tags = raw_tags
            .iter()
            .filter_map(|json_val| {
                let t: AssetResult<FrameTag> = json_val.try_into();
                match t {
                    Ok(t) => Some(t),
                    Err(e) => {
                        errors.push(e);
                        None
                    }
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(" ")
                .into());
        }

        Ok(Spritesheet {
            id,
            name: name.to_owned(),
            asset_path: asset_path.to_owned(),
            frames,
            tags,
        })
    }
}

impl TryFrom<&JsonValue> for SpriteFrame {
    type Error = Box<dyn Error>;
    fn try_from(obj: &JsonValue) -> AssetResult<SpriteFrame> {
        let Some(x) = obj["x"].as_u32() else {
            return Err("missing or invalid x field in sprite frame".into());
        };
        let Some(y) = obj["y"].as_u32() else {
            return Err("missing or invalid y field in sprite frame".into());
        };
        let Some(w) = obj["w"].as_u32() else {
            return Err("missing or invalid w field in sprite frame".into());
        };
        let Some(h) = obj["h"].as_u32() else {
            return Err("missing or invalid h field in sprite frame".into());
        };
        let Some(duration_ms) = obj["duration_ms"].as_u32() else {
            return Err("missing or invalid duration_ms field in sprite frame".into());
        };

        Ok(SpriteFrame {
            x,
            y,
            w,
            h,
            duration_ms,
        })
    }
}

impl TryFrom<&JsonValue> for FrameTag {
    type Error = Box<dyn Error>;
    fn try_from(obj: &JsonValue) -> AssetResult<FrameTag> {
        let Some(name) = obj["name"].as_str() else {
            return Err("missing or invalid name field in frame tag".into());
        };
        let Some(from) = obj["from"].as_u32() else {
            return Err("missing or invalid from field in frame tag".into());
        };
        let Some(to) = obj["to"].as_u32() else {
            return Err("missing or invalid to field in frame tag".into());
        };

        Ok(FrameTag {
            name: name.to_owned(),
            from,
            to,
        })
    }
}

impl TryFrom<&JsonValue> for FileAsset {
    type Error = Box<dyn Error>;
    fn try_from(obj: &JsonValue) -> AssetResult<FileAsset> {
        let Some(name) = obj["name"].as_str() else {
            return Err("missing or invalid name field in file asset".into());
        };
        let Some(id) = obj["id"].as_usize() else {
            return Err("missing or invalid id field in file asset".into());
        };
        let Some(asset_path) = obj["asset_path"].as_str() else {
            return Err("missing or invalid asset_path field in file asset".into());
        };

        Ok(FileAsset {
            name: name.to_owned(),
            asset_path: asset_path.to_owned(),
            id,
        })
    }
}
