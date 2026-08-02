use jzon;
use jzon::JsonValue;
use std::error::Error;

type AssetResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Manifest {
    pub spritesheets: Vec<Spritesheet>,
}

#[derive(Debug)]
pub struct Spritesheet {
    pub id: usize,
    pub name: String,
    pub asset_path: String,
    pub frames: Vec<SpriteFrame>,
    pub tags: Vec<FrameTag>,
}

#[derive(Debug)]
pub struct SpriteFrame {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub duration_ms: u32,
}

#[derive(Debug)]
pub struct FrameTag {
    pub name: String,
    pub from: u32,
    pub to: u32,
}

// Writes are simple:
impl Into<JsonValue> for Manifest {
    fn into(self) -> JsonValue {
        jzon::object! {
            spritesheets: self.spritesheets
        }
    }
}
impl Into<JsonValue> for Spritesheet {
    fn into(self) -> JsonValue {
        jzon::object! {
            id: self.id,
            name: self.name,
            asset_path: self.asset_path,
            frames: self.frames,
            tags: self.tags,
        }
    }
}
impl Into<JsonValue> for SpriteFrame {
    fn into(self) -> JsonValue {
        jzon::object! {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
            duration_ms: self.duration_ms,
        }
    }
}
impl Into<JsonValue> for FrameTag {
    fn into(self) -> JsonValue {
        jzon::object! {
            name: self.name,
            from: self.from,
            to: self.to,
        }
    }
}

// Reads are a little more complicated:

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
            .into_iter()
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
            .into_iter()
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
