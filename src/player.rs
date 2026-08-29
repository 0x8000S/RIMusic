use std::time::Duration;
use rodio::Float;
use crate::music_file::MusicFile;
use crate::state::{PlayState, PlaybackType, PlayList};
use crate::store::MusicStore;

pub struct Player {
    pub _music_handle: rodio::MixerDeviceSink,
    pub music_player: rodio::Player,
    pub music_name: String,
    pub now_playing: Option<MusicFile>,
    pub total_music_time: Duration,
    pub play_state: PlayState,
    pub playback_type: PlaybackType,
    pub play_list: PlayList,
    pub value: f64,
}

impl Player {
    pub(crate) fn new() -> Self {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(&handle.mixer());

        Player {
            _music_handle: handle,
            music_player: player,
            total_music_time: Duration::from_secs(0),
            play_state: PlayState::Stop,
            music_name: String::from("TEST MUSIC TITLE"),
            now_playing: None,
            playback_type: PlaybackType::OnceStop,
            play_list: PlayList::AllMusic,
            value: 0f64,
        }
    }
    pub(crate) fn verification_file(f: &MusicFile) -> bool {
        if let Ok(file) = std::fs::File::open(&f.music) {
            rodio::Decoder::try_from(file).is_ok()
        } else {
            false
        }
    }
    pub(crate) fn play_music(&mut self, p: &mut MusicFile, store: &mut MusicStore) -> Result<(), ()> {
        if !Self::verification_file(&p) {
            return Err(())
        }
        let file = std::fs::File::open(&p.music);
        if let Err(_) = file {
            return Err(())
        }
        let file = file.unwrap();
        self.now_playing = Some(p.clone());
        if let Ok(source) = rodio::Decoder::try_from(file) {
            self.play_state = PlayState::Play;
            p.get_music_file_total_duration();
            self.total_music_time = p.duration.unwrap();
            match &self.play_list {
                PlayList::AllMusic => {
                    let idx = store.find_music(self);
                    p.get_music_file_total_duration();
                    p.get_music_file_artist();
                    store.music_files[idx] = p.clone();
                }
                PlayList::Tags(t) => {
                    let idx = store.find_music(self);
                    p.get_music_file_total_duration();
                    p.get_music_file_artist();
                    store.tags.get_mut(t).unwrap()[idx] = p.clone();
                }
                PlayList::Artist(a) => {
                    let idx = store.find_music(self);
                    p.get_music_file_total_duration();
                    p.get_music_file_artist();
                    store.artists.get_mut(a).unwrap()[idx] = p.clone();
                }
            }
            self.music_player.clear();
            self.value = self.music_player.get_pos().as_millis() as f64;
            self.music_player.append(source);
            self.music_player.play();
        } else {
            return Err(())
        }
        let name = p.music.file_name().unwrap().to_str().unwrap().to_string();
        let chars: Vec<_> = name.chars().collect();
        if chars.iter().len() > 20 {
            let pre20 = String::from_iter(chars.get(..20).unwrap().to_owned().iter());
            self.music_name = format!("{}...", pre20);
        } else {
            self.music_name = name;
        }
        Ok(())
    }
    fn calc_next_idx(len: usize, pos: usize, pidx: i32) -> usize {
        let mut ret = pos as i32 + pidx;
        if ret < 0 {
            ret = len as i32 + pidx;
        } else {
            ret = ret % len as i32
        }
        if len == 0 {
            ret = 0;
        }
        ret as usize
    }
    pub(crate) fn music_play_push(&mut self, pidx: i32, store: &mut MusicStore) -> Result<(), ()> {
        let idx = store.find_music(self);
        let mut f = match &self.play_list {
            PlayList::AllMusic => store.music_files[Self::calc_next_idx(store.music_files.len(), idx, pidx)].clone(),
            PlayList::Tags(t) => store.tags[t][Self::calc_next_idx(store.tags[t].len(), idx, pidx)].clone(),
            PlayList::Artist(a) => store.artists[a][Self::calc_next_idx(store.artists[a].len(), idx, pidx)].clone()
        };
        self.play_music(&mut f, store)
    }
    pub(crate) fn player_default(&mut self) {
        self.now_playing = None;
        self.music_player.stop();
        self.play_state = PlayState::Stop;
        self.value = 0.0;
        self.music_player.try_seek(Duration::from_secs(0)).unwrap();
        self.music_name = String::from("TEST MUSIC TITLE");
    }
    pub(crate) fn set_volume(&mut self, vol: Float) {
        self.music_player.set_volume(vol)
    }
    pub fn set_pos(&mut self, pos: Duration) {
        let _ = self.music_player.try_seek(pos);
        self.value = pos.as_millis() as f64
    }
    pub(crate) fn sync(&mut self, store: &mut MusicStore) -> Result<(), ()> {
        self.value = self.music_player.get_pos().as_millis() as f64;
        if self.music_player.empty() {
            if let Some(p) = &self.now_playing {
                match self.playback_type {
                    PlaybackType::OnceStop => {
                        self.play_state = PlayState::Stop;
                    }
                    PlaybackType::OneWhile => {
                        self.play_music(&mut p.clone(), store)?;
                        self.play_state = PlayState::Play;
                        self.music_player.play();
                    }
                    PlaybackType::MusicNext => {
                        self.music_play_push(1, store)?;
                    }
                    PlaybackType::RadomPlay => match &self.play_list {
                        PlayList::AllMusic => {
                            let idx = rand::random_range(0..store.music_files.len());
                            self.play_music(&mut store.music_files[idx].clone(), store)?;

                        }
                        PlayList::Tags(t) => {
                            let music_files = &store.tags[t];
                            let idx = rand::random_range(0..music_files.len());
                            self.play_music(&mut music_files[idx].clone(), store)?;
                        }
                        PlayList::Artist(a) => {
                            let music_files = &store.artists[a];
                            let idx = rand::random_range(0..music_files.len());
                            self.play_music(&mut music_files[idx].clone(), store)?;
                        }
                    },
                }
            }
            self.value = 0.0;
            self.music_player.try_seek(Duration::from_secs(0)).unwrap();
        }
        Ok(())
    }
    pub(crate) fn ps_switch(&mut self) -> bool {
        match self.play_state {
            PlayState::Play => {
                self.music_player.pause();
                self.play_state = PlayState::Stop;
                true
            }
            PlayState::Stop => {
                self.music_player.play();
                self.play_state = PlayState::Play;
                false
            }
        }
    }
}