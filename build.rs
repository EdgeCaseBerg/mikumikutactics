use asefile::AsepriteFile;
use image::RgbaImage;
use image::imageops;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/* The goal of the build script is to process the .ase files in the spritesheets
 * folder, as well as any other assets that need to be assigned ids for the catalog
 * and then create the manifest file that the game will use to resolve those assets
{
   spritesheets: [
       {
           id: 0,
           name: miku,
           asset_path: "characters/miku.png",
           frames: [
               {
                   x: 0,
                   y: 0,
                   w: 320,
                   h: 480,
                   duration_ms: 100,
               },
               ...
           ],
           tags: [
               {
                   name: "miku",
                   from: 0,
                   to: 2
               },
               {
                   name: "idle",
                   from: 0,
                   to: 0
               }
           ]
       }
   ]
}
*/

fn main() {
    // Rerun build script if the spritesheets change
    println!("cargo::rerun-if-changed=spritesheets/");

    let out_dir = Path::new(".").join("assets");
    let in_dir = Path::new(".").join("spritesheets");
    let ase_files = process_dir(in_dir, out_dir.clone())
        .into_iter()
        .map(|(a, p)| {
            // remove assets/ from the path. GameOptions has assets in it for joining against the cur directory
            // so leave it out for now and be consistent with how id_to_relative_path was used before
            (a, p.strip_prefix(&out_dir).unwrap().to_path_buf())
        })
        .collect();
    let spritesheets = make_spritesheets(ase_files);
    let manifest_path = out_dir.join("manifest");
    let manifest = Manifest { spritesheets };
    let j = serde_json::to_string_pretty(&manifest).expect("failed to convert manifest into json");
    fs::write(manifest_path, j).expect("failed to write manifest file");
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
                y: ase.height() as u32 * frame_idx,
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

// TODO: add in audio to the manifest as well to refactor those constant ids too (later!)
#[derive(Debug, Serialize)]
struct Manifest {
    spritesheets: Vec<Spritesheet>,
}

#[derive(Debug, Serialize)]
struct Spritesheet {
    id: usize,
    name: String,
    asset_path: String,
    frames: Vec<SpriteFrame>,
    tags: Vec<FrameTag>,
}

#[derive(Debug, Serialize)]
struct SpriteFrame {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    duration_ms: u32,
}

#[derive(Debug, Serialize)]
struct FrameTag {
    name: String,
    from: u32,
    to: u32,
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
fn process_dir(ase_folder: PathBuf, asset_folder: PathBuf) -> Vec<(AsepriteFile, PathBuf)> {
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
            let entries = process_dir(entry.path(), mirrored_path);
            for (ase, path) in entries {
                ase_files.push((ase, path));
            }
        } else {
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
