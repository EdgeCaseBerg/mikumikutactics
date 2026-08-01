use asefile::AsepriteFile;
use image::RgbaImage;
use image::imageops;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=spritesheets/");
    let out_dir = env::var_os("OUT_DIR").expect("not OUT_DIR environmental variable specified");
    let out_dir = Path::new(".").join("assets");
    let in_dir = Path::new(".").join("spritesheets");
    let manifest = out_dir.join("manifest");
    let ase_files = process_dir(in_dir, out_dir);
    for (ase, relative_path) in ase_files.into_iter() {
        // TODO: write out manifest file
    }
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
        .expect(&format!("could not save file {}", out.display()));
}

// Recursively walk ase_folder, exporting each ase file and creating the png in the assets folder as it goes
fn process_dir(ase_folder: PathBuf, asset_folder: PathBuf) -> Vec<(AsepriteFile, PathBuf)> {
    let mut ase_files = vec![];
    let entries = fs::read_dir(&ase_folder).expect(&format!(
        "could not read input directory for read_dir {}",
        ase_folder.display()
    ));
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
            let mut ase_out = asset_folder.join(&entry.file_name());
            ase_out.set_extension("png");
            let ase = AsepriteFile::read_file(&path)
                .expect(&format!("could not open path {}", ase_out.display()));
            write_file_to_sheet(&ase, ase_out.clone());
            ase_files.push((ase, ase_out));
        }
    }
    ase_files
}
