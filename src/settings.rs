use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use rodio::Float;
use serde::{Deserialize, Serialize};
use crate::music_file::MusicFile;
use crate::state::{PlaybackType, PlayList};
use crate::player::Player;
use crate::store::MusicStore;

#[derive(Clone)]
pub enum SettingKeys {
    ShowSideBar(bool),
    Volume(Float),
    KeepPlayState(bool)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub show_side_bar: bool,
    pub volume: Float,
    pub search_origin: Vec<PathBuf>,
    pub tags: HashMap<String, Vec<MusicFile>>,
    pub last_music: Option<MusicFile>,
    pub last_position: Duration,
    pub last_playback: PlaybackType,
    pub last_playlist: PlayList,
    pub keep_state: bool,
    pub artist: HashMap<String, Vec<MusicFile>>
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            show_side_bar: false,
            volume: 1 as Float,
            search_origin: vec![MusicStore::get_music_file()],
            tags: HashMap::new(),
            last_music: None,
            last_position: Duration::from_secs(0),
            last_playback: PlaybackType::OnceStop,
            last_playlist: PlayList::AllMusic,
            keep_state: false,
            artist: HashMap::new()
        }
    }
}

impl Settings {
    pub fn set_setting(&mut self, key: SettingKeys) {
        match key {
            SettingKeys::ShowSideBar(b) => self.show_side_bar = b,
            SettingKeys::Volume(v) => self.volume = v,
            SettingKeys::KeepPlayState(b) => self.keep_state = b
        }
    }
    pub fn save(&self, store: &MusicStore, player: &Player) -> Self {
        Settings {
            search_origin: store.search_origin.clone(),
            tags: store.tags.clone(),
            last_music: player.now_playing.clone(),
            last_position: player.music_player.get_pos(),
            last_playback: player.playback_type.clone(),
            last_playlist: player.play_list.clone(),
            artist: store.artists.clone(),
            ..*self
        }
    }
}