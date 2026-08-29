use std::collections::HashMap;
use std::path::PathBuf;
use crate::music_file::MusicFile;
use crate::player::Player;
use crate::state::PlayList;

#[derive(Debug)]
pub struct MusicStore {
    pub(crate) search_origin: Vec<PathBuf>,
    pub(crate) music_files: Vec<MusicFile>,
    pub(crate) tags: HashMap<String, Vec<MusicFile>>,
    pub(crate) artists: HashMap<String, Vec<MusicFile>>,
    pub(crate) idx: usize,
    pub(crate) is_read: bool
}

impl MusicStore {
    pub(crate) fn new() -> Self {
        MusicStore {
            search_origin: vec![],
            music_files: vec![],
            tags: HashMap::new(),
            artists: HashMap::new(),
            idx: 0,
            is_read: true
        }
    }
    pub(crate) fn get_music_file() -> PathBuf {
        if let Some(v) = dirs::audio_dir() {
            v
        } else {
            PathBuf::new()
        }
    }
    pub(crate) fn sync(&mut self) {
        self.music_files.clear();
        self.artists.clear();
        self.sync_only_push();
    }
    pub(crate) fn sync_only_push(&mut self) {
        for p in self.search_origin.iter() {
            for f in Self::get_music_file_from_path(p) {
                let mf = MusicFile {
                    music: f,
                    duration: None,
                    artist: None
                };
                self.music_files.push(mf);
            }
        }
    }
    fn get_music_file_from_path(p: &PathBuf) -> Vec<PathBuf> {
        let mut files = vec![];
        if let Ok(val) = std::fs::read_dir(p) {
            for f in val {
                if let Ok(f) = f {
                    let path = f.path();
                    if path.is_file() {
                        if let Some(val) = path.extension() {
                            if val == "mp3" || val == "wav" || val == "ogg" {
                                files.push(path)
                            }
                        }
                    }
                }
            }

        }
        files
    }
    pub(crate) fn remove_tag(&mut self, player: &mut Player, tag: String) {
        self.tags.remove(&tag);
        if let PlayList::Tags(tx) = &player.play_list {
            if *tx == tag {
                player.player_default()
            }
        }
    }
    pub(crate) fn remove_music_from_tag(&mut self, player: &mut Player, tag: &String, music_file: MusicFile)  {
        let idx = self.tags.get(tag).unwrap()
            .iter().position(|x| *x == music_file).unwrap();
        self.tags.get_mut(tag).unwrap().remove(idx);
        if let Some(x) = &player.now_playing {
            if *x == music_file {
                player.player_default()
            }
        }
    }
    pub(crate) fn find_music(&self, player: &Player) -> usize {
        match &player.play_list {
            PlayList::AllMusic => {
                self.music_files
                    .iter()
                    .position(|x| {
                        x.clone() == player.now_playing.clone().unwrap()
                    })
                    .unwrap()
            }
            PlayList::Tags(t) => {
                let music_files = &self.tags[t];
                music_files
                    .iter()
                    .position(|x| {
                        x.clone() == player.now_playing.clone().unwrap()
                    })
                    .unwrap()
            }
            PlayList::Artist(a) => {
                let music_files = &self.artists[a];
                music_files
                    .iter()
                    .position(|x| {
                        x.clone() == player.now_playing.clone().unwrap()
                    })
                    .unwrap()
            }
        }
    }
    pub(crate) fn remove_search_origin(&mut self, p: PathBuf) {
        let idx = self.search_origin.iter()
            .position(|x| *x == p);
        if let Some(val) = idx {
            self.search_origin.remove(val);
        }
    }
    pub(crate) fn read_one(&mut self) {
        if self.idx < self.music_files.len() {
            if Player::verification_file(&self.music_files[self.idx]) {
                self.music_files[self.idx].get_music_all_data();
                let art = self.music_files[self.idx].artist.clone().unwrap();
                if self.artists.get(&art).is_some() {
                    if let Some(v) = self.artists.get(&art) {
                        if !v.contains(&self.music_files[self.idx]) {
                            self.artists.get_mut(&art).unwrap().push(self.music_files[self.idx].clone());
                        }
                    }
                } else {
                    self.artists.insert(art, vec![self.music_files[self.idx].clone()]);
                }
            }
            self.idx += 1
        } else {
            self.is_read = false;
            self.idx = 0;
        }
    }
    pub(crate) fn add_tag_for_music(&mut self, tag: String, mut music_file: MusicFile) {
        music_file.get_music_all_data();
        if let Some(v) = self.tags.get_mut(&tag) {
            v.push(music_file);
        } else {
            self.tags.insert(tag, vec![music_file]);
        }
    }
    pub(crate) fn search_music_file(&self, text: &String) -> Vec<MusicFile> {
        let ret: Vec<_> = self.music_files.iter()
            .filter(|x| x.music.to_string_lossy().to_lowercase().contains(&text.to_lowercase()))
            .map(|x| x.clone())
            .collect();
        ret
    }
}