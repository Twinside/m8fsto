use std::{collections::{hash_map::Entry, HashMap}, fs, path::{Path, PathBuf}};
use m8_files::{reader::*, writer::Writer, Instrument};

use crate::{broken_search::sample_to_absolute_path, types::M8FstoErr};

fn on_file_blob(backup_root: &Path, song_path: &Path, out_folder: &Path, data: Vec<u8>) -> Result<(), M8FstoErr> {
    let mut reader = Reader::new(data.clone());
    let mut song = m8_files::Song::read_from_reader(&mut reader)
        .map_err(|e| M8FstoErr::UnparseableM8File {
            path: song_path.to_path_buf(),
            reason: format!("{:?}", e)
        })?;

    // First pass we verify that all the samples exists, before effectively
    // moving the files.
    for (i, instr) in song.instruments.iter().enumerate() {
        match instr {
            Instrument::Sampler(sampler) => {
                let full_sample_path =
                    sample_to_absolute_path(backup_root, song_path, &sampler.sample_path);

                if  !full_sample_path.exists() {
                    return Err(M8FstoErr::MissingSample { instr: i, path: full_sample_path })
                }
            }
            _ => {}
        }
    }

    let out_folder = out_folder.join(&song.name);
    std::fs::create_dir(&out_folder).map_err(|e|
        M8FstoErr::FolderCreationError {
            path: out_folder.clone(),
            reason: format!("{:?}", e)
        })?;

    let sample_folder_path = out_folder.join("Samples");
    std::fs::create_dir(&sample_folder_path).map_err(|e|
        M8FstoErr::FolderCreationError {
            path: out_folder.clone(),
            reason: format!("{:?}", e)
        })?;

    let mut samples : HashMap<String, String> = HashMap::new();

    // Let's move the samples and rewrite the sampler instruments
    for (i, instr) in song.instruments.iter_mut().enumerate() {
        match instr {
            Instrument::Sampler(sampler) => {
                let full_sample_path =
                    sample_to_absolute_path(backup_root, song_path, &sampler.sample_path);

                match samples.entry(sampler.sample_path.clone()) {
                    // if we already moved the same sample, we just reuse
                    // the file (deduplication happen)
                    Entry::Occupied(prev) => {
                        sampler.sample_path = prev.get().clone()
                    }
                    Entry::Vacant(v) => {
                        let file_name = full_sample_path.file_name()
                            .unwrap().to_str().unwrap();

                        let out_filename = format!("{}_{}", i, file_name);
                        let out_sample_path =
                            sample_folder_path.join(&out_filename);

                        std::fs::copy(&full_sample_path, &out_sample_path)
                            .map_err(|e| M8FstoErr::SampleCopyError {
                                path: full_sample_path.clone(),
                                to: out_sample_path,
                                reason: format!("{:?}", e) })?;

                        let relative_name = format!("Samples/{}", out_filename);
                        sampler.sample_path = relative_name.clone();
                        v.insert(relative_name);
                    }
                }
            }
            _ => {}
        }
    }


    let out_song_name =
        out_folder.join(song_path.file_name().unwrap());

    let mut writer = Writer::new(data);
    song.write(&mut writer)
        .map_err(|reason|
            M8FstoErr::SongSerializationError { 
                destination: format!("{:?}", &out_song_name),
                reason
            })?;

    std::fs::write(&out_song_name, writer.finish())
        .map_err(|reason|
            M8FstoErr::SongSerializationError {
                destination: format!("{:?}", out_song_name),
                reason: format!("{:?}", reason)
            })?;
    
    Ok(())
}

/// Try to list sample of a given path
pub fn bundle_song(cwd: &Path, path : &str, out_folder: &Option<String>) -> Result<(), M8FstoErr> {
    let file_blob = fs::read(path)
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: PathBuf::from(path), reason: format!("{:?}", e) })?;

    let as_path = Path::new(path);

    let out_folder =
        out_folder
            .clone()
            .map_or_else(
                || cwd.to_path_buf().join("bundle"),
                |e| PathBuf::from(e));

    on_file_blob(cwd, as_path, &out_folder, file_blob)
}