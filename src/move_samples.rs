use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::path::Component;
use glob::glob;
use m8_files::{reader::*, Instrument};

use crate::types::M8FstoErr;

enum Swap {
    Dir { from: String, to: String },
    File { from: String, to: String }
}

impl Swap {
    pub fn apply_fs(&self) -> Result<(), M8FstoErr> {
        match self {
            Swap::Dir { from, to } => {
                std::fs::rename(from, to).map_err(| _| M8FstoErr::RenameFailure { path: from.clone() })
            }
            Swap::File { from, to } => {
                println!("MOVING FILE {:?} -> {:?}", from, to);
                std::fs::rename(from, to).map_err(| _| M8FstoErr::RenameFailure { path: from.clone() })
            }
        }
    }

    pub fn try_swap(&self, sample_path: &str) -> Option<String> {
        match self {
            Swap::File { from, to } if sample_path == from =>
                Some(to.clone()),
            Swap::File { from: _, to: _} => None,
            Swap::Dir { from, to } => {
                if !sample_path.starts_with(from) {
                    return None
                }
                let final_path =
                    sample_path.strip_prefix(from).unwrap();
                Some(format!("{}{}", to, final_path))
            }
        }
    }
}

struct SwappedFile {
    file_data : Vec<u8>,
    touched: Vec<(usize, String, String, String)>
}

fn on_file_blob(dry_run: bool, swap: &Swap, path: &Path, data: Vec<u8>) -> Result<Option<SwappedFile>, M8FstoErr> {
    let mut reader = Reader::new(data.clone());
    let mut touched = vec![];
    let mut song = m8_files::Song::read_from_reader(&mut reader)
        .map_err(|e| M8FstoErr::UnparseableM8File {
            path: path.to_path_buf(),
            reason: format!("{:?}", e)
        })?;

    for (i, instr) in song.instruments.iter_mut().enumerate() {
        match instr {
            Instrument::Sampler(sampler) => {
                match swap.try_swap(&sampler.sample_path) {
                    None => {}
                    Some(new_path) => {
                        touched.push((i, sampler.name.clone(), sampler.sample_path.clone(), new_path.clone()));
                        sampler.sample_path = new_path;
                    }
                }
            }
            _ => {}
        }
    }

    if touched.len() > 0 {
        if dry_run {
            return Ok(Some(SwappedFile {
                file_data: Vec::new(),
                touched
            }))
        }

        let mut writer =
            m8_files::writer::Writer::new(data);
        println!("SONG VER: {:?}", song.version);
        song.write(&mut writer)
            .map_err(|reason| M8FstoErr::SongSerializationError { reason })?;

        Ok(Some(SwappedFile {
            file_data: writer.finish(),
            touched
        }))
    } else {
        Ok(None)
    }

}

fn on_dir(force: bool, dry_run : bool, cwd: &Path, swap: &Swap) -> Result<(), M8FstoErr> {
    let mut errors = vec![];
    let mut matched_not_serializable = vec![];
    let mut to_write= vec![];
    let search_pattern =
        glob(&format!("{}/**/*.m8s", cwd.to_str().unwrap()))
        .expect("Failed to read glob pattern");

    for entry in search_pattern {
        match entry {
            Err(e) => println!("{:?}", e),
            Ok(path) => {
                let try_as_file = fs::read(&path);
                match try_as_file {
                    Err(e) => {
                        errors.push(M8FstoErr::CannotReadFile {
                            path: path.to_path_buf(),
                            reason: format!("{:?}", e)
                        })
                    }
                    Ok(file_blob) => {
                        match on_file_blob(dry_run, swap, &path, file_blob) {
                            Ok(None) => {}
                            Err(M8FstoErr::SongSerializationError { reason: _ }) => matched_not_serializable.push(path),
                            Err(m8err) => errors.push(m8err),
                            Ok(Some(swapped)) if dry_run => {
                                println!("Song {:?}", &path);
                                for (inst, name, path, new_path) in swapped.touched {
                                    println!(" - {} {} \"{}\" -> \"{}\"", inst, name, path, new_path)
                                }
                            }
                            Ok(Some(swapped)) => {
                                println!("Song {:?}", &path);
                                for (inst, name, path, new_path) in swapped.touched {
                                    println!(" - {} {} \"{}\" -> \"{}\"", inst, name, path, new_path)
                                }

                                to_write.push((path, swapped.file_data));
                                
                            }
                        }
                    }
                }
            }
        }
    }

    // If we have some file we can't translate, but still want to write
    // the files
    if matched_not_serializable.len() == 0 || force {
        for (path, data) in to_write {
            match fs::write(&path, data) {
                Ok(()) => {}
                Err(_) => {
                    errors.push(M8FstoErr::SongSerializationError { reason: format!("Error while writing file {:?}", path) });
                }
            }
        }
    }

    if errors.len() == 0 {
        Ok(())
    } else if errors.len() == 1 {
        Err(errors[0].clone())
    } else {
        Err(M8FstoErr::MultiErrs { inner: errors })
    }
}


pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}


pub fn move_samples(cwd: &Path, force: bool, dry_run: bool, from: String, to: String) -> Result<(), M8FstoErr> {
    let cwd = normalize_path(cwd);

    println!("CWD: {:?}", cwd);
    println!("FROM: {:?}", from);
    println!("TO: {:?}", to);
    let from_path = PathBuf::from(from);
    if !from_path.exists() {
        return Err(M8FstoErr::InvalidPath { reason: format!("Folder {:?} doesn't exists", from_path) })
    }

    let to_path = PathBuf::from(to);
    let to_canon =
        if to_path.is_relative() {
            let mut abs = cwd.clone();
            abs.push(to_path);
            abs
        } else {
            to_path
        };

    let from_canon = normalize_path(&from_path);
    
    println!("FROM_CANON {:?}", from_canon);
    let rel_from =
        from_canon.strip_prefix(&cwd)
            .map_err(| _| M8FstoErr::InvalidPath { reason: "from relativisation error".into() })?
            .to_str()
            .unwrap()
            .replace("\\","/");

    println!("REL_FROM {:?}", rel_from);
    println!("TO_CANON {:?}", to_canon);
    let rel_to =
        to_canon.strip_prefix(&cwd)
            .map_err(| _| M8FstoErr::InvalidPath { reason: "destination canonicalization error".into() })?
            .to_str()
            .unwrap()
            .replace("\\","/");

    println!("REL_TO {:?}", rel_to);
    println!("dry_run: {:?}", dry_run);
    let move_order =
        if from_path.is_dir() {
            Swap::Dir {
                from: format!("/{}", rel_from),
                to: format!("/{}", rel_to),
            }
        } else if from_path.is_file() {
            Swap::File {
                from: format!("/{}", rel_from),
                to: format!("/{}", rel_to),
            }
        } else {
            return Err(M8FstoErr::CannotReadFile { path: from_path, reason: String::from("Neither file nor directory")})
        };

    match on_dir(force, dry_run, &cwd, &move_order) {
        Ok(()) => {
            std::fs::rename(&from_canon, to_canon)
                .map_err(|_| M8FstoErr::RenameFailure { path: format!("{:?}", from_canon) })
        }
        Err(errs) if force => {
            std::fs::rename(&from_canon, to_canon)
                .map_err(|_| M8FstoErr::RenameFailure { path: format!("{:?}", from_canon) })?;
            Err(errs)
        }
        Err(errs) => Err(errs)
    }
}
