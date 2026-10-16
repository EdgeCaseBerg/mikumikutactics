use asefile::AsepriteFile;
use image::RgbaImage;
use image::imageops;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub type BuildResult<T> = Result<T, Box<dyn Error>>;

/* The goal of the build script is to process the .ase files in the spritesheets
 * folder, as well as any other assets that need to be assigned ids for the catalog
 * and then create the manifest file that the game will use to resolve those assets
 * {
 *    spritesheets: [
 *        {
 *            id: 0,
 *            name: miku,
 *            asset_path: "characters/miku.png",
 *            frames: [
 *                {
 *                    x: 0,
 *                    y: 0,
 *                    w: 320,
 *                    h: 480,
 *                    duration_ms: 100,
 *                },
 *                ...
 *            ],
 *            tags: [
 *                {
 *                    name: "miku",
 *                    from: 0,
 *                    to: 2
 *                },
 *                {
 *                    name: "idle",
 *                    from: 0,
 *                    to: 0
 *                }
 *            ]
 *        }
 *    ]
 * }
*/

fn main() {
    // Rerun build script if the spritesheets change
    println!("cargo::rerun-if-changed=spritesheets/");
    println!("cargo::rerun-if-changed=assets/");

    let assets_dir = Path::new(".").join("assets");
    let spritesheets = spritesheets(&assets_dir);
    let scenes = load_file_assets(&assets_dir, "scenes");
    let sounds = load_file_assets(&assets_dir, "sounds");
    let music = load_file_assets(&assets_dir, "music");
    let weapons = load_weapon_assets(&assets_dir);
    let maps = load_map_assets(&assets_dir, &spritesheets);

    // I? I am the monument to all of your sins. t. Gravemind - Halo 2.
    let manifest_path = assets_dir.join("manifest");
    let manifest = Manifest {
        spritesheets,
        scenes,
        sounds,
        music,
        weapons,
        maps,
    };
    let j: jzon::JsonValue = manifest.into();
    let string = j.pretty(2);
    fs::write(manifest_path, string).expect("failed to write manifest file");
}

include!("src/manifest.rs");

fn load_file_assets(assets_dir: &Path, subdirectory: &str) -> Vec<FileAsset> {
    let scenes_path = assets_dir.join(subdirectory);
    let mut files = vec![];
    let entries = fs::read_dir(&scenes_path).unwrap_or_else(|_| {
        panic!(
            "could not read input directory for read_dir {}",
            scenes_path.display()
        )
    });
    for result in entries {
        let Ok(entry) = result else {
            continue;
        };
        if let Ok(metadata) = entry.metadata()
            && metadata.is_dir()
        {
            let entries = load_file_assets(&entry.path(), subdirectory);
            for file in entries {
                files.push(file);
            }
        } else {
            let mut path = entry.path();
            let name = entry
                .path()
                .file_stem()
                .expect("could not create name for file asset from path")
                .to_string_lossy()
                .to_string();
            if let Ok(stripped) = path.strip_prefix("./") {
                path = stripped.to_path_buf();
            }
            let path = path.display().to_string();
            files.push(FileAsset {
                id: 0,
                name,
                asset_path: path,
            });
        }
    }
    files
        .into_iter()
        .enumerate()
        .map(|(i, mut f)| {
            f.id = i;
            f
        })
        .collect()
}

fn spritesheets(assets_dir: &Path) -> Vec<Spritesheet> {
    let in_dir = Path::new(".").join("spritesheets");
    let ase_files = process_dir_for_ase(in_dir, assets_dir.to_path_buf())
        .into_iter()
        .map(|(a, p)| {
            // remove assets/ from the path. GameOptions has assets in it for joining against the cur directory
            // so leave it out for now and be consistent with how id_to_relative_path was used before
            (a, p.strip_prefix(assets_dir).unwrap().to_path_buf())
        })
        .collect();
    make_spritesheets(ase_files)
}

fn write_file_to_sheet(ase: &AsepriteFile, out: PathBuf) {
    let w = ase.width() as u32;
    let h = ase.height() as u32;
    let mut spritesheet = RgbaImage::new(w * ase.num_frames(), h);
    for i in 0..ase.num_frames() {
        let offset = w * i;
        let frame = ase.frame(i);
        let img = frame.image();
        imageops::overlay(&mut spritesheet, &img, offset as i64, 0);
    }
    spritesheet
        .save(&out.as_path())
        .unwrap_or_else(|_| panic!("could not save file {}", out.display()))
}

// Recursively walk ase_folder, exporting each ase file and creating the png in the assets folder as it goes
fn process_dir_for_ase(ase_folder: PathBuf, asset_folder: PathBuf) -> Vec<(AsepriteFile, PathBuf)> {
    let mut ase_files = vec![];
    let entries = fs::read_dir(&ase_folder).unwrap_or_else(|_| {
        panic!(
            "could not read input directory for read_dir {}",
            ase_folder.display()
        )
    });
    for result in entries {
        let Ok(entry) = result else {
            continue;
        };
        if let Ok(metadata) = entry.metadata()
            && metadata.is_dir()
        {
            let mirrored_path = asset_folder.join(entry.file_name());
            fs::create_dir_all(&mirrored_path).expect("failed to create directories");
            let entries = process_dir_for_ase(entry.path(), mirrored_path);
            for (ase, path) in entries {
                ase_files.push((ase, path));
            }
        } else {
            if !entry.file_name().to_string_lossy().ends_with("ase") {
                continue;
            }
            let path = entry.path();
            let mut ase_out = asset_folder.join(entry.file_name());
            ase_out.set_extension("png");
            let ase = AsepriteFile::read_file(&path)
                .unwrap_or_else(|_| panic!("could not open path {}", ase_out.display()));
            write_file_to_sheet(&ase, ase_out.clone());
            ase_files.push((ase, ase_out));
        }
    }
    ase_files
}

fn get_ase_tags(ase: &AsepriteFile) -> Vec<FrameTag> {
    (0..ase.num_tags())
        .map(|tag_id| {
            let tag = ase.tag(tag_id);
            FrameTag {
                name: tag.name().to_owned(),
                from: tag.from_frame(),
                to: tag.to_frame(),
            }
        })
        .collect()
}

fn get_ase_frames(ase: &AsepriteFile) -> Vec<SpriteFrame> {
    (0..ase.num_frames())
        .map(|frame_idx| {
            let frame = ase.frame(frame_idx);
            // Technically we could pack things more tightly maybe or use less vram if we use
            // the cel's top_left across all the cels in a layer and shrink the sprite down.
            // But its a bit easier to work with rectangles we know about rather than different
            // sized pieces I think. A future optimization perhaps.
            //
            // similarly, the x,y coordinates assume only one direction for frames in the sheet
            // (which will be horizontal in our case) but if we packed multiple characters together
            // in the future or something as a releasing step if it made sense, then we'd tweak those
            // things around a bit I think and then be able to shrink the number of textures to load
            // down quite a bit.
            SpriteFrame {
                x: ase.width() as u32 * frame_idx,
                y: 0,
                w: ase.width() as u32,
                h: ase.height() as u32,
                duration_ms: frame.duration(),
            }
        })
        .collect()
}

fn make_spritesheets(ase_files: Vec<(AsepriteFile, PathBuf)>) -> Vec<Spritesheet> {
    ase_files
        .into_iter()
        .enumerate()
        .map(|(id, (ase, relative_path))| {
            let tags = get_ase_tags(&ase);
            let frames = get_ase_frames(&ase);
            let mut asset_path = relative_path;
            let name = asset_path
                .file_stem()
                .expect("could not create name for spritesheet from path")
                .to_string_lossy()
                .to_string();
            if let Ok(stripped) = asset_path.strip_prefix("./") {
                asset_path = stripped.to_path_buf();
            }
            let asset_path = asset_path.display().to_string();
            Spritesheet {
                id,
                name,
                asset_path,
                frames,
                tags,
            }
        })
        .collect()
}

fn load_weapon_assets(assets_dir: &Path) -> Vec<Weapon> {
    let in_dir = assets_dir.join("weapons");
    let file = in_dir.join("weapons.csv");
    let raw_csv = fs::read_to_string(file).expect("could not load weapon csv");
    raw_csv
        .lines()
        .skip(1)
        .enumerate()
        .map(
            |(id, line)| match line.split(",").collect::<Vec<_>>()[0..4] {
                [name, attack, min, max] => {
                    let name = name.trim().to_owned();
                    let attack: usize = attack.trim().parse().unwrap_or_else(|e| {
                        panic!("could not parse attack power from value '{attack}' {e}")
                    });
                    let min_range: usize = min.trim().parse().unwrap_or_else(|e| {
                        panic!("could not parse min_range power from value '{min}' {e}")
                    });
                    let max_range: usize = max.trim().parse().unwrap_or_else(|e| {
                        panic!("could not parse max_range power from value '{max}' {e}")
                    });
                    Weapon {
                        id,
                        name,
                        range: min_range..=max_range,
                        attack,
                    }
                }
                _ => {
                    panic!("Bad line! {:?}", line)
                }
            },
        )
        .collect()
}

fn load_map_assets(assets_dir: &Path, spritesheets: &Vec<Spritesheet>) -> Vec<TerrainMap> {
    let in_dir = assets_dir.join("maps");
    let mut map_files = vec![];
    let entries = fs::read_dir(&in_dir).unwrap_or_else(|_| {
        panic!(
            "could not read input directory for read_dir {}",
            in_dir.display()
        )
    });
    for result in entries {
        let Ok(entry) = result else {
            continue;
        };
        if let Ok(metadata) = entry.metadata()
            && metadata.is_dir()
        {
            let entries = load_map_assets(&entry.path(), spritesheets);
            for map in entries {
                map_files.push(map);
            }
        } else {
            if !entry.file_name().to_string_lossy().ends_with("map") {
                continue;
            }
            let path = entry.path();
            let map = parse_map_file(&path);
            map_files.push(map);
        }
    }
    map_files
}

fn parse_map_file(path: &Path) -> TerrainMap {
    let terrain_name = path
        .file_name()
        .expect("no filename?")
        .to_string_lossy()
        .into_owned();
    let raw = fs::read_to_string(path).expect("could not load map file to string");
    let mut lines = raw.lines();
    let spritesheet_name = lines
        .next()
        .expect("expected palette to start map file line")
        .split_once(':')
        .map(|(_, name)| name.trim())
        .expect("palette line incorrect");

    let mut defs = HashMap::new();
    for line in lines.by_ref() {
        // Stop at first empty line (the line before the grid)
        if line.is_empty() {
            break;
        }
        let (key, rest) = line.split_once(':').expect("invalid tile definition");
        let (tag_name, flag_str) = rest
            .rsplit_once(' ')
            .expect("could not parse flag and tag name");
        let flags: usize = flag_str.parse().expect("could not parse flags to number");
        defs.insert(
            key.to_owned(),
            TerrainTile {
                tag_name: tag_name.to_owned(),
                flags,
            },
        );
    }

    let mut grid = String::with_capacity(raw.len());
    let mut buf = [0; 4]; // avoid str allocation per character
    for (line_number, line) in lines.enumerate() {
        for (char_number, c) in line.chars().enumerate() {
            let key = c.encode_utf8(&mut buf);
            assert!(
                defs.contains_key(key),
                "key {c} at line {line_number}, column {char_number} not found in definitions",
            );
            grid.push(c);
        }
        grid.push('\n');
    }

    TerrainMap {
        name: terrain_name,
        sheet: spritesheet_name.to_owned(),
        tile_definitions: defs,
        grid,
    }
}

fn validate_map(map: &TerrainMap, spritesheets: &Vec<Spritesheet>) -> BuildResult<()> {
    let Some(sheet) = spritesheets.iter().find(|sheet| sheet.name == map.sheet) else {
        return Err("map spritesheet does not exist in manifest".into());
    };

    let mut unique_tiles: std::collections::HashSet<char> = map.grid.chars().collect();
    unique_tiles.remove(&'\n');
    let expected_count = unique_tiles.len();
    let actual_acount = map.tile_definitions.len();
    assert_eq!(
        expected_count, actual_acount,
        "count of tile definitions and found grid tiles does not match"
    );

    let mut confirmed_exists_in_sheet = 0;
    let mut buf = [0; 4];
    for c in unique_tiles {
        let key = c.encode_utf8(&mut buf);
        let def = map
            .tile_definitions
            .get(key)
            .expect("impossible? cant find tile definition for key");
        sheet
            .tags
            .iter()
            .find(|tag| tag.name == def.tag_name)
            .expect("key did not match a tag in spritesheet");
    }

    Ok(())
}
