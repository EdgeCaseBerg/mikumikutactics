use asefile::AsepriteFile;
use image::RgbaImage;
use image::imageops;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=spritesheets/");
    println!("cargo::rerun-if-changed=build.rs");
    let out_dir = env::var_os("OUT_DIR").expect("not OUT_DIR environmental variable specified");
    let out_dir = Path::new(".").join("assets");
    let in_dir = Path::new(".").join("spritesheets");
    let manifest = out_dir.join("manifest");
    let ase_files = process_dir(in_dir);
    for (ase, relative_path) in ase_files.into_iter() {
        let mut ase_out = out_dir.join(&relative_path);
        ase_out.set_extension("png");
        let ase_out =
            std::path::absolute(&ase_out).expect("couldnt turn path into an absolute one");
        // interestingly, width and height is of the SPRITE size when you've saved the .ase after setting it, not the full width of the image itself.
        let w = ase.width() as u32;
        let h = ase.height() as u32;
        let mut spritesheet = RgbaImage::new(w * ase.num_frames(), h);
        for frame in 0..ase.num_frames() {
            let f = ase.frame(frame);
            let img = f.image();
            let offset = w * frame;
            imageops::overlay(&mut spritesheet, &img, offset as i64, 0);
        }
        // this doesn't actually work, we need a mkdir -f situation because right now
        // I have to manually create the folders to avoid an io write err
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&ase_out.as_path());
        spritesheet
            .save(&ase_out.as_path())
            .expect(&format!("could not save file {}", ase_out.display()));
    }
    // TODO: write out manifest file
}

// Note if we experience memory pressure from loading every .ase into memory
// then cry a little and refactor this to write things out as it processes instead.
fn process_dir(path: PathBuf) -> Vec<(AsepriteFile, PathBuf)> {
    let mut ase_files = vec![];
    let entries = fs::read_dir(&path).expect(&format!(
        "could not read path for read_dir {}",
        path.display()
    ));
    for result in entries {
        let Ok(entry) = result else {
            continue;
        };
        if let Ok(metadata) = entry.metadata()
            && metadata.is_dir()
        {
            let dir_files = process_dir(entry.path());
            for (subfile, relative_path) in dir_files.into_iter() {
                let tuple = (subfile, relative_path);
                ase_files.push(tuple);
            }
        } else {
            let path = entry.path();
            let ase = AsepriteFile::read_file(&path)
                .expect(&format!("could not open path {}", path.display()));
            ase_files.push((ase, path));
        }
    }
    ase_files
}
