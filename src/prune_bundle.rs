use std::{collections::HashSet, fs, path::{Path, PathBuf}};
use m8_files::{reader::*, Instrument};

use crate::{broken_search::is_sample_absolute, types::{FlagBag, M8FstoErr}};

fn on_file_blob(flags : FlagBag, song_path: &Path, data: Vec<u8>) -> Result<(), M8FstoErr> {
    let mut reader = Reader::new(data.clone());
    let song = m8_files::Song::read_from_reader(&mut reader)
        .map_err(|e| M8FstoErr::UnparseableM8File {
            path: song_path.to_path_buf(),
            reason: format!("{:?}", e)
        })?;

    let mut all_samples = HashSet::new();

    // First pass we gather all relative sample path, raise an error if one
    // is absolute.
    for (i, instr) in song.instruments.iter().enumerate() {
        match instr {
            Instrument::Sampler(sampler) => {
                let sample_path = String::from(&sampler.sample_path);
                if is_sample_absolute(&sample_path) {
                    return Err(M8FstoErr::SampleInBundleNotRelative {
                        sample_path, instrument: i
                    });
                }

                all_samples.insert(sample_path);
            }
            _ => {}
        }
    }

    let song_folder = song_path.parent().unwrap();
    let sample_folder = song_folder.join("Samples");

    let mut to_remove = Vec::new();
    for f in sample_folder
        .read_dir()
        .map_err(|_e| M8FstoErr::InvalidPath {reason: "Can't read path".into() })? {

        let entry = f.map_err(|_e| M8FstoErr::InvalidPath {reason: "".into()} )?;
        let entry_path = entry.path();
        let rel_folder = entry_path.strip_prefix(song_folder).unwrap();

        let as_string = String::from(rel_folder.to_str().unwrap())
            // M8 use '/' as folder separator 
            .replace('\\', "/");

        if !all_samples.contains(&as_string) {
            to_remove.push(entry_path);
        }
    }

    if to_remove.len() == 0 {
        println!("Sample folder is clean, nothing to do!");
        return Ok(())
    }

    if flags.dry_run {
        println!("Extra samples to be removed:");
        for pb in &to_remove {
            println!(" * '{:?}'", pb);
        }
    } else {
        for pb in &to_remove {
            println!("Removing '{:?}'", pb);
            fs::remove_file(&pb)
                .map_err(|e|
                    M8FstoErr::FileRemovalFailure { path: pb.clone(), reason: format!("{:?}", e) })?;
        }
    }

    Ok(())
}

/// Try to list sample of a given path
pub fn prune_bundle(flags: FlagBag, path : &str) -> Result<(), M8FstoErr> {
    let file_blob = fs::read(path)
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: PathBuf::from(path), reason: format!("{:?}", e) })?;

    let song_path = Path::new(path);

    on_file_blob(flags, song_path, file_blob)
}