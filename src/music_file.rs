use std::path::PathBuf;
use std::time::Duration;
use lofty::file::{AudioFile, TaggedFile};
use lofty::prelude::{Accessor, TaggedFileExt};
use rodio::Source;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MusicFile {
    pub(crate) music: PathBuf,
    pub(crate) duration: Option<Duration>,
    pub(crate) artist: Option<String>
}
impl Default for MusicFile {
    fn default() -> Self {
        Self {
            music: PathBuf::new(),
            duration: None,
            artist: None
        }
    }
}
impl PartialEq for MusicFile {
    fn eq(&self, other: &Self) -> bool {
        self.music == other.music
    }
}

impl MusicFile {
    fn _set_value(&mut self, val: TaggedFile) {
        self.duration = Some(val.properties().duration());
        if let Some(val) = val.primary_tag() {
            if let Some(art) = val.artist() {
                self.artist = Some(art.to_string());
                return;
            }
        }
        self.artist = Some("群星".to_string());
    }
    pub(crate) fn get_music_all_data(&mut self) {
        if self.artist.is_some() {
            return;
        }
        if let Ok(v) = lofty::probe::Probe::open(self.music.clone()) {
            if let Ok(val) = v.read() {
                self._set_value(val)
            }
        }  else {
            self.get_music_file_total_duration();
            self.artist = Some("群星".to_string());
        }
    }
    pub(crate) fn get_music_file_artist(&mut self) {
        if self.artist.is_some() {
            return;
        }
        if let Ok(tf) = lofty::probe::Probe::open(self.music.clone()) {
            if let Ok(tf) = tf.read() {
                self._set_value(tf);
                return;
            }
        }
        self.artist = Some("群星".to_string());
    }
    pub(crate) fn get_music_file_total_duration(&mut self) {
        if let Some(_) = self.duration {
            return;
        }
        if let Ok(tf) = lofty::probe::Probe::open(self.music.clone()) {
            if let Ok(tf) = tf.read() {
                self.duration = Some(tf.properties().duration());
                return;
            }
        }
        if let Ok(filez) = std::fs::File::open(&self.music) {
            if let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(filez)) {
                if let Some(d) = source.total_duration() {
                    self.duration = Some(d);
                    return;
                }
            }
        }
        self.duration = Some(Duration::from_secs(0u64));
    }

}