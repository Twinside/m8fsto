use std::{fs, path::PathBuf};

use m8_file_parser::{reader::Reader, remapper::Remapper};

use crate::{types::M8FstoErr, RenumberCommand, RenumberTarget};

fn renumber_from_song(show: RenumberCommand, w: &mut dyn std::io::Write, mut song: m8_file_parser::Song) -> Result<(), M8FstoErr> {
    match show.renum_command {
        RenumberTarget::Instrument {from, to} => {
            let mut remapper = Remapper::default_ver(song.version);
            remapper.instrument_mapping.mapping[from] = to as u8;
            remapper.instrument_mapping.to_move.push(from as u8);
            remapper.renumber(&mut song);
        }
    }

    Ok(())
}

pub fn renumber_element(show: RenumberCommand, w: &mut dyn std::io::Write) -> Result<(), M8FstoErr> {
    let song_path = PathBuf::from(show.file.clone());
    let file_blob = fs::read(song_path.clone())
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: song_path.clone(), reason: format!("{:?}", e) })?;

    let mut reader = Reader::new(file_blob);

    match m8_file_parser::Song::read_from_reader(&mut reader) {
        Ok(song) => renumber_from_song(show, w, song),
        Err(e) => {
            Err(M8FstoErr::UnparseableM8File {
                path: song_path,
                reason: format!("{:?}", e)
            })
        }
    }
}
