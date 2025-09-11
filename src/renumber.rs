use std::{fs, path::PathBuf};

use m8_file_parser::{reader::Reader, remapper::Remapper, writer::Writer};

use crate::{types::M8FstoErr, RenumberCommand, RenumberTarget};

fn renumber_from_song(show: &RenumberCommand, _w: &mut dyn std::io::Write, song: &mut m8_file_parser::Song) -> Result<(), M8FstoErr> {
    match show.renum_command {
        RenumberTarget::Instrument {from, to} => {
            let mut remapper = Remapper::default_ver(song.version);
            remapper.instrument_mapping.mapping[from] = to as u8;
            remapper.instrument_mapping.to_move.push(from as u8);
            remapper.renumber(song);
        }
    }

    Ok(())
}

pub fn renumber_element(show: RenumberCommand, w: &mut dyn std::io::Write) -> Result<(), M8FstoErr> {
    let in_filename = show.file.clone();
    let song_path = PathBuf::from(in_filename.clone());
    let file_blob = fs::read(song_path.clone())
        .map_err(|e|
            M8FstoErr::CannotReadFile { path: song_path.clone(), reason: format!("{:?}", e) })?;

    let mut reader = Reader::new(file_blob.clone());

    match m8_file_parser::Song::read_from_reader(&mut reader) {
        Ok(mut song) => {
            renumber_from_song(&show, w, &mut song)?;

            let o_name = match &show.out_file {
                None => in_filename.clone(),
                Some(of) => of.clone()
            };

            if !show.dry_run {
                let mut writer = Writer::new(file_blob);
                song.write(&mut writer)
                    .map_err(|reason|
                        M8FstoErr::SongSerializationError { 
                            destination: format!("{:?}", &o_name),
                            reason
                        })
            } else {
                Ok(())
            }
        },
        Err(e) => {
            Err(M8FstoErr::UnparseableM8File {
                path: song_path,
                reason: format!("{:?}", e)
            })
        }
    }
}
