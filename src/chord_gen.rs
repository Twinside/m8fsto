use m8_file_parser::writer::Writer;
use m8_file_parser::{ AHDEnv, FMWave, FmAlgo, Instrument, InstrumentWithEq, LfoShape, LimitType, Mod, Operator, SynthParams, Table, Version, LFO };
use m8_file_parser::FMSynth;

use crate::types::M8FstoErr;

#[derive(Clone)]
struct Chord {
    name: String,
    offsets: Vec<usize>
}

impl Chord {
    pub fn make(n : &str, offsets: Vec<usize>) -> Chord {
        Chord {
            name: String::from(n),
            offsets
        }
    }

    pub fn len(&self) -> usize {
        self.offsets.len()
    }

    fn make_mod_op(&self) -> Operator {
        Operator {
            shape: FMWave::SIN,
            ratio: 0x01,
            ratio_fine: 0x00,
            level: 0x80,
            feedback: 0x00,
            retrigger: 0x00,
            mod_a: 0x00,
            mod_b: 0x00,
        }
    }

    fn make_op(&self, ix: usize) -> Operator {
        let ofs = self.offsets[ix];
        let freq : f64 = (2 as f64).powf((ofs as f64) / 12.0);
        let fine = (freq - ((freq as i64) as f64)) * 100.0;

        Operator {
            shape: FMWave::SAW,
            ratio: freq as u8,
            ratio_fine: fine as u8,
            level: 0x80,
            feedback: 0x00,
            retrigger: 0x00,
            mod_a: 0x00,
            mod_b: 0x00,
        }
    }

    pub fn as_fm(&self, inversion: u8) -> FMSynth {
        let ahd = AHDEnv {
            dest: 0,
            amount: 0xFF,
            attack: 0,
            hold: 0,
            decay: 0x80,
        };

        let lfo = LFO {
            shape: LfoShape::TRI,
            dest: 0,
            trigger_mode: m8_file_parser::LfoTriggerMode::FREE,
            freq: 0x10,
            amount: 0xFF,
            retrigger: 0,
        };

        let operators = [
            if self.offsets.len() > 3 {
                self.make_op(3)
            } else {
                self.make_mod_op()
            },

            self.make_op(2),
            self.make_op(1),
            self.make_op(0)
        ];

        let synth_params = SynthParams {
            volume: 0x0,
            pitch: 0,
            fine_tune: 0x80,
            filter_type: 0,
            filter_cutoff: 0xFF,
            filter_res: 0x0,
            amp: 0,
            limit: LimitType(0),
            mixer_pan: 0x80,
            mixer_dry: 0xC0,
            mixer_mfx: 0,
            mixer_delay: 0,
            mixer_reverb: 0x00,
            associated_eq: 0x80,
            mods: [
                Mod::AHDEnv(ahd.clone()),
                Mod::AHDEnv(ahd.clone()),
                Mod::LFO(lfo.clone()),
                Mod::LFO(lfo.clone()),
            ],
        };

        let algo =
            if self.offsets.len() == 4 {
                FmAlgo(0xB)
            } else {
                FmAlgo(0x8)
            };

        FMSynth { 
            number: 0,
            name: 
                if inversion > 0 {
                    format!("{}_INV{}", self.name, inversion)
                } else {
                    self.name.clone()
                },
            transpose: true,
            table_tick: 1,
            synth_params,
            algo,
            operators,
            mod1: 0,
            mod2: 0,
            mod3: 0,
            mod4: 0
        }
    }
}

fn build_chords() -> Vec<Chord> {
    vec![
        Chord::make("MAJ", vec![0, 4, 7]),
        Chord::make("MAJ6", vec![0, 4, 7, 9]),
        Chord::make("DOM7", vec![0, 4, 7, 10]),
        Chord::make("MAJ7", vec![0, 4, 7, 11]),
        Chord::make("AUG", vec![0, 4, 8]),
        Chord::make("AUG7", vec![0, 4, 8, 10]),
        Chord::make("MIN", vec![0, 3, 7]),
        Chord::make("MIN6", vec![0, 3, 7, 9]),
        Chord::make("MIN7", vec![0, 3, 7, 10]),
        Chord::make("MINMAJ7", vec![0, 3, 7, 11]),
        Chord::make("DIM", vec![0, 3, 6]),
        Chord::make("DIM7", vec![0, 3, 6, 9]),
        Chord::make("HDIM7", vec![0, 3, 6, 10]),
    ]
}

fn wrap(synth: FMSynth) -> InstrumentWithEq {
    let ver = Version {
        major: 4, minor: 2, patch: 0
    };

    InstrumentWithEq {
        instrument: Instrument::FMSynth(synth),
        table: Table::default_ver(ver),
        eq: None,
        version: ver,
    }
}

pub fn generate() -> Result<(), M8FstoErr> {

    for chord in &build_chords() {
        let folder_prefix = format!("FM_CHORDS/{}", &chord.name);

        // one folder per chord
        std::fs::create_dir_all(&folder_prefix).map_err(|err|
            M8FstoErr::FolderCreationError {
                path: (&chord.name).into(),
                reason: format!("{}", err)
        })?;

        let instrument =
            wrap(chord.as_fm(0));

        let mut w = Writer::new_instrument_writer(false);
        let instr_name = format!("{}/{}.m8i", &folder_prefix, &chord.name);
        instrument.write(&mut w);

        std::fs::write(&instr_name, &w.finish())
            .map_err(|err|
                M8FstoErr::SongSerializationError {
                    destination: instr_name.clone(),
                    reason: format!("{}", err)
                })?;

        let mut cchord = chord.clone();
        for inversion in 0 .. chord.len() - 1 {
            let instr_name = format!("{}/{}_INV{}.m8i", &folder_prefix, &chord.name, inversion);
            cchord.offsets[inversion] += 12;
            let inv_instr =
                wrap(cchord.as_fm((inversion + 1) as u8));
            let mut inv_w = Writer::new_instrument_writer(false);
            inv_instr.write(&mut inv_w);
            std::fs::write(&instr_name, &inv_w.finish())
                .map_err(|err|
                    M8FstoErr::SongSerializationError {
                        destination: instr_name.clone(),
                        reason: format!("{}", err)
                    })?;
        }
    }

    Ok(())
}
