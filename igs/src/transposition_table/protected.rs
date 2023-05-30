use crate::dbs::{NimbersProvider, NimbersStorer, HasLen};
use crate::game::{Game, SerializableGame};
use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, BufWriter, Write};

/// Transposition Table that protects nimbers of some positions
/// (usually the positions that are close to a root of a search tree).
///
/// Nimbers of protected positions are stored in `protected_part`,
/// which never overwrites them and is saved in `backup` (which usually is a file).
/// Nimbers of the rest positions are stored in `unprotected_part`.
/// The predicate `should_be_protected` points which positions are protected.
pub struct ProtectedTT<'g, G: Game, UnprotectedTT, ProtectPred: Fn(&G, &G::Position) -> bool, F> {
    game: &'g G,
    /// Stores nimbers of unprotected positions.
    unprotected_part: UnprotectedTT,
    /// Stores nimbers of protected positions.
    protected_part: HashMap<G::Position, u8>,
    /// The predicate that points if given position is protected.
    should_be_protected: ProtectPred,
    /// A copy of the protected part; usually the file.
    backup: F
}

impl<'g, G, UnprotectedTT, ProtectPred> ProtectedTT<'g, G, UnprotectedTT, ProtectPred, BufWriter<File>>
where G: Game + SerializableGame,
      <G as Game>::Position: Eq + Hash,
      UnprotectedTT: NimbersStorer<G::Position>,
      ProtectPred: Fn(&G, &G::Position) -> bool
{
    pub fn new<P: AsRef<Path>>(game: &'g G, backup_file_name: P, should_be_protected: ProtectPred, mut unprotected_part: UnprotectedTT) -> Self {
        let mut protected_part = HashMap::<G::Position, u8>::new();
        let mut backup_position = 0;
        let mut backup = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            //.truncate(false)
            .open(backup_file_name)
            .unwrap();
        let mut backup_has_extra_positions = false;
        while let Ok(position) = game.read_position(&mut backup) {
            let mut nimber = 0u8;
            if backup.read_exact(std::slice::from_mut(&mut nimber)).is_ok() {
                if should_be_protected(game, &position) {
                    protected_part.store_nimber(position, nimber);
                    backup_position = backup.stream_position().unwrap();
                } else {    // file has been created with different predicate and some position are not protected now:
                    unprotected_part.store_nimber(position, nimber);
                    backup_has_extra_positions = true;
                }
            } else {
                break;
            }
        }
        backup.seek(SeekFrom::Start(backup_position)).unwrap();
        let mut backup = BufWriter::with_capacity(game.position_size_bytes() + 1, backup);
        if backup_has_extra_positions {
            backup.rewind().unwrap();
            for (p, n) in &protected_part {
                game.write_position(&mut backup, p).expect("ProtectedTT cannot write the position to the backup");
                backup.write_all(&n.to_ne_bytes()).expect("ProtectedTT cannot write the nimber to the backup");
            }
            backup.flush().expect("ProtectedTT cannot flush the backup");
            let current_size = backup.stream_position().expect("ProtectedTT cannot shrink the file");
            backup.get_mut().set_len(current_size).expect("ProtectedTT cannot shrink the file");
        }
        Self {
            game,
            unprotected_part,
            protected_part,
            should_be_protected,
            backup
        }
    }
}

impl<'g, G, UnprotectedTT, ProtectPred, F> NimbersProvider<G::Position> for ProtectedTT<'g, G, UnprotectedTT, ProtectPred, F>
where G: Game,
      <G as Game>::Position: Eq + Hash,
      UnprotectedTT: NimbersProvider<G::Position>,
    ProtectPred: Fn(&G, &G::Position) -> bool
{
    #[inline(always)]
    fn get_nimber(&self, position: &G::Position) -> Option<u8> {
        if (self.should_be_protected)(self.game, position) {
            self.protected_part.get_nimber(position)
        } else {
            self.unprotected_part.get_nimber(position)
        }
    }

    #[inline(always)]
    fn get_nimber_and_self_organize(&mut self, position: &G::Position) -> Option<u8> {
        if (self.should_be_protected)(self.game, position) {
            self.protected_part.get_nimber_and_self_organize(position)
        } else {
            self.unprotected_part.get_nimber_and_self_organize(position)
        }
    }
}

impl<'g, G, UnprotectedTT, ProtectPred, F> NimbersStorer<G::Position> for ProtectedTT<'g, G, UnprotectedTT, ProtectPred, F>
    where G: Game + SerializableGame,
          <G as Game>::Position: Eq + Hash,
          UnprotectedTT: NimbersStorer<G::Position> + NimbersProvider<G::Position>,
          ProtectPred: Fn(&G, &G::Position) -> bool,
        F: io::Write
{
    fn store_nimber(&mut self, position: G::Position, nimber: u8) {
        if (self.should_be_protected)(self.game, &position) {
            /*let mut buff = Vec::<u8>::new();
            self.game.write_position(&mut buff, &position).expect("ProtectedTT cannot write the position to the backup");
            buff.push(nimber);
            self.backup.write_all(&buff);*/
            self.game.write_position(&mut self.backup, &position).expect("ProtectedTT cannot write the position to the backup");
            self.backup.write_all(&nimber.to_ne_bytes()).expect("ProtectedTT cannot write the nimber to the backup");
            self.backup.flush().expect("ProtectedTT cannot flush the backup");
            self.protected_part.store_nimber(position, nimber)
        } else {
            self.unprotected_part.store_nimber(position, nimber)
        }
    }
}

impl<'g, G, UnprotectedTT, ProtectPred, F> HasLen for ProtectedTT<'g, G, UnprotectedTT, ProtectPred, F>
where G: Game, ProtectPred: Fn(&G, &G::Position) -> bool, UnprotectedTT: HasLen
{
    #[inline] fn len(&self) -> usize {
        self.protected_part.len() + self.unprotected_part.len()
    }
}