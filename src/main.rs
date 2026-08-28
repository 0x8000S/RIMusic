#![windows_subsystem = "windows"]
use iced::{widget, Color};
use iced_aw::Menu;
use lofty::prelude::*;
use rodio::{Float, Source};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use iced::theme::{Custom, Palette};
use lofty::file::TaggedFile;
use serde::{Deserialize, Serialize};

fn cap() -> Palette {
    Palette {
        background: Color::from_rgb8(30, 30, 46),       // 深蓝灰背景
        text: Color::from_rgb8(205, 214, 244),          // 浅灰文字
        primary: Color::from_rgb8(137, 180, 250),       // 强调色（蓝）
        success: Color::from_rgb8(166, 227, 161),       // 成功绿
        danger: Color::from_rgb8(243, 139, 168),        // 危险红
        warning: Color::from_rgb8(216, 118, 0)          // 警告橙
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum PlayList {
    AllMusic,
    Tags(String),
    Artist(String)
}

#[derive(Clone, Serialize, Deserialize, Copy)]
enum PlaybackType {
    OnceStop,
    OneWhile,
    MusicNext,
    RadomPlay,
}
impl Display for PlaybackType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaybackType::OnceStop => write!(f, "单曲即停"),
            PlaybackType::OneWhile => write!(f, "单曲循环"),
            PlaybackType::MusicNext => write!(f, "下一曲"),
            PlaybackType::RadomPlay => write!(f, "随机播放"),
        }
    }
}

#[derive(Clone)]
enum Message {
    OnValueChanged(f64),
    PlayMusic(MusicFile),
    Sync,
    OnReleaseSlider,
    OnPSSwitchClicked,
    CloseMusicOpenFailureMsg,
    OnPlaybackTypeChanged(PlaybackType),
    SwitchSideBarShow,
    GoView(View),
    OpenSetTagMsg(MusicFile),
    AddTagTo(String, MusicFile),
    CloseSetTag,
    WhenNewTagType(String),
    OpenNewTagMsg,
    CloseNewTagMsg,
    AddTag,
    NextMusic,
    PrevMusic,
    RemoveMusicFromTag(MusicFile),
    RemoveTag(String),
    WhenSearchTextType(String),
    WhenExpSearchOriginClicked,
    WhenSettingChanged(SettingKeys),
    DeleteOriginPath(PathBuf),
    OnAddSearchOriginClicked,
    CloseSave,
    ReadFileData
}
#[derive(PartialEq)]
enum PlayState {
    Stop,
    Play,
}

#[derive(Clone, PartialEq)]
enum View {
    MainView,
    TagsView,
    TagView(String),
    SearchView,
    SettingsView,
    ArtistsView,
    ArtistView(String)
}

#[derive(Clone, Serialize, Deserialize)]
struct MusicFile {
    music: PathBuf,
    duration: Option<Duration>,
    artist: Option<String>
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
            } else {
                self.artist = Some("群星".to_string());
            }
        } else {
            self.artist = Some("群星".to_string());
        }
    }
    fn get_music_all_data(&mut self) {
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
    fn get_music_file_artist(&mut self) {
        println!("Read artist...");
        if self.artist.is_some() {
            return;
        }
        if let Ok(tf) = lofty::probe::Probe::open(self.music.clone()).unwrap().read() {
            self._set_value(tf)
        } else {
            self.artist = Some("群星".to_string());
        }
    }
    fn get_music_file_total_duration(&mut self) {
        println!("Read time...");
        if let Some(_) = self.duration {
            return;
        }
        if let Ok(tf) = lofty::probe::Probe::open(self.music.clone()).unwrap().read() {
            self.duration = Some(tf.properties().duration());
        } else {
            let filez = std::fs::File::open(&self.music).unwrap();
            if let Ok(source) = rodio::Decoder::new(std::io::BufReader::new(filez)) {
                if let Some(d) = source.total_duration() {
                    self.duration = Some(d);
                }
            } else {
                self.duration = Some(Duration::from_secs(0u64));
            }
        }
    }

}


struct CommonWidget {}
impl CommonWidget {
    fn side_bar_button(env: &RIMusic, name: String, view: View) -> iced::Element<'_, Message> {
        widget::button(widget::text(name))
            .width(iced::Fill)
            .on_press_maybe(
                (view == env.view)
                    .then(|| None)
                    .unwrap_or_else(|| Some(Message::GoView(view))),
            )
            .into()
    }
    fn modal<'a>(view: impl Into<iced::Element<'a, Message>>) -> iced::Element<'a, Message> {
        widget::container(widget::center(
            widget::container(view)
                .padding(18)
                .style(widget::container::rounded_box),
        ))
            .style(|_t| widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgba8(
                    0, 0, 0, 0.6,
                ))),
                ..widget::container::Style::default()
            })
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
    }
    fn view_builder<'a>(content: impl Into<iced::Element<'a, Message>>) -> iced::Element<'a, Message> {
        widget::container(content)
            .padding(8)
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
    }
    fn title_bar<'a>(view_name: String, action_buttons: Option<iced::Element<'a, Message>>, back_view: Option<Message>, settings: &'a Settings) -> iced::Element<'a, Message> {
        widget::row![
                if settings.show_side_bar {
                    widget::container(widget::space())
                } else {
                    widget::container(
                        widget::button(
                            back_view.is_some().then(|| "<")
                            .unwrap_or_else(|| "≡")
                            ).on_press(back_view.is_some()
                                .then(|| back_view.unwrap())
                                .unwrap_or_else(|| Message::SwitchSideBarShow)),
                    )
                },
                widget::text(view_name)
                    .size(28)
                    .style(widget::text::primary),
                widget::container(
                    widget::row![action_buttons])
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
            ]
            .align_y(iced::alignment::Vertical::Center)
            .spacing(8)
            .into()
    }
    fn setting_group(name: String, content: iced::Element<'_, Message>) -> iced::Element<'_, Message> {
        widget::column![
            widget::text(name).size(48),
            widget::space().height(18),
            content
        ].into()
    }
    fn panel(content: iced::Element<'_, Message>) -> iced::Element<'_, Message> {
        widget::container(content).padding(18)
            .style(|x| widget::container::Style {
                background: Some(iced::Background::Color(x.palette().background)),
                border: iced::Border {
                    color: x.palette().text,
                    width: 2.0,
                    radius: iced::border::Radius::new(4)
                },
                ..widget::container::Style::default()
            })
            .into()
    }
    fn setting_card(name: String, content: iced::Element<Message>) -> iced::Element<Message> {
        Self::panel(
            widget::row![
                widget::text(name),
                widget::container(
                    content
                ).width(iced::Fill).align_x(iced::alignment::Horizontal::Right)
            ].into()
        )
    }
    fn expand_content<'a>(title: String, content: iced::Element<'a, Message>, show: &bool, e: Message) -> iced::Element<'a, Message> {
        if *show {
            Self::panel(
                widget::column![
                    widget::stack![
                        widget::row![
                            widget::text(title).size(32),
                            widget::container(
                                widget::text("👆")
                            ).width(iced::Fill).align_x(iced::alignment::Horizontal::Right)
                        ],
                        widget::button("")
                            .style(widget::button::text)
                            .width(iced::Fill).height(iced::Fill)
                            .on_press(e)
                    ],
                    widget::space().height(24),
                    content
                ].into()
            ).into()
        } else {
            Self::panel(
                widget::stack![
                    widget::row![
                        widget::text(title).size(32),
                        widget::container(
                            widget::text("👇")
                        ).width(iced::Fill).align_x(iced::alignment::Horizontal::Right)
                    ],
                    widget::button("")
                    .style(widget::button::text)
                    .width(iced::Fill).height(iced::Fill)
                    .on_press(e)
                ].into()
            ).into()
        }
    }
    fn path_show(p: &PathBuf) -> iced::Element<'_, Message> {
        widget::container(
            widget::row![
                widget::text(p.to_string_lossy()).size(18),
                widget::container(
                    widget::button("删除").on_press(Message::DeleteOriginPath(p.clone()))
                ).width(iced::Fill).align_x(iced::alignment::Horizontal::Right)
            ]
        ).width(iced::Fill).into()
    }
}

struct MusicStore {
    search_origin: Vec<PathBuf>,
    music_files: Vec<MusicFile>,
    tags: HashMap<String, Vec<MusicFile>>,
    artists: HashMap<String, Vec<MusicFile>>,
    idx: usize,
    is_read: bool
}

impl MusicStore {
    fn new() -> Self {
        MusicStore {
            search_origin: vec![],
            music_files: vec![],
            tags: HashMap::new(),
            artists: HashMap::new(),
            idx: 0,
            is_read: true
        }
    }
    fn get_music_file() -> PathBuf {
        if let Some(v) = dirs::audio_dir() {
            v
        } else {
            PathBuf::new()
        }
    }
    fn sync(&mut self) {
        self.music_files.clear();
        self.artists.clear();
        self.sync_only_push();
    }
    fn sync_only_push(&mut self) {
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
                let f = f.unwrap();
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
        files
    }
    fn remove_tag(&mut self, player: &mut Player, tag: String) {
        self.tags.remove(&tag);
        if let PlayList::Tags(tx) = &player.play_list {
            if *tx == tag {
                player.player_default()
            }
        }
    }
    fn remove_music_from_tag(&mut self, player: &mut Player, tag: &String, music_file: MusicFile)  {
        let idx = self.tags.get(tag).unwrap()
            .iter().position(|x| *x == music_file).unwrap();
        self.tags.get_mut(tag).unwrap().remove(idx);
        if let Some(x) = &player.now_playing {
            if *x == music_file {
                player.player_default()
            }
        }
    }
    fn find_music(&self, player: &Player) -> usize {
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
    fn remove_search_origin(&mut self, p: PathBuf) {
        let idx = self.search_origin.iter()
            .position(|x| *x == p);
        if let Some(val) = idx {
            self.search_origin.remove(val);
        }
    }
    fn read_one(&mut self) {
        if self.idx < self.music_files.len() {
            println!("{}-{:?}", self.music_files.len(), self.music_files[self.idx].music);
            if Player::verification_file(&self.music_files[self.idx]) {
                self.music_files[self.idx].get_music_all_data();
                let art = self.music_files[self.idx].artist.clone().unwrap();
                if self.artists.get(&art).is_some() {
                    self.artists.get_mut(&art).unwrap().push(self.music_files[self.idx].clone());
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
    fn add_tag_for_music(&mut self, tag: String, mut music_file: MusicFile) {
        music_file.get_music_all_data();
        if let Some(v) = self.tags.get_mut(&tag) {
            v.push(music_file);
        } else {
            self.tags.insert(tag, vec![music_file]);
        }
    }
}

struct Player {
    _music_handle: rodio::MixerDeviceSink,
    music_player: rodio::Player,
    music_name: String,
    now_playing: Option<MusicFile>,
    total_music_time: Duration,
    play_state: PlayState,
    playback_type: PlaybackType,
    play_list: PlayList,
    value: f64,
}

impl Player {
    fn new() -> Self {
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
    fn verification_file(f: &MusicFile) -> bool {
        if let Ok(file) = std::fs::File::open(&f.music) {
            rodio::Decoder::try_from(file).is_ok()
        } else {
            false
        }
    }
    fn play_music(&mut self, p: &mut MusicFile, store: &mut MusicStore) -> Result<(), ()> {
        if !Self::verification_file(&p) {
            return Err(())
        }
        let file = std::fs::File::open(&p.music).unwrap();
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
    fn music_play_push(&mut self, pidx: i32, store: &mut MusicStore) -> Result<(), ()> {
        let idx = store.find_music(self);
        let mut f = match &self.play_list {
            PlayList::AllMusic => store.music_files[Self::calc_next_idx(store.music_files.len(), idx, pidx)].clone(),
            PlayList::Tags(t) => store.tags[t][Self::calc_next_idx(store.tags[t].len(), idx, pidx)].clone(),
            PlayList::Artist(a) => store.artists[a][Self::calc_next_idx(store.artists[a].len(), idx, pidx)].clone()
        };
        self.play_music(&mut f, store)
    }
    fn player_default(&mut self) {
        self.now_playing = None;
        self.music_player.stop();
        self.play_state = PlayState::Stop;
        self.value = 0.0;
        self.music_player.try_seek(Duration::from_secs(0)).unwrap();
        self.music_name = String::from("TEST MUSIC TITLE");
    }
    fn set_volume(&mut self, vol: Float) {
        self.music_player.set_volume(vol)
    }
    fn set_pos(&mut self, pos: Duration) {
        let _ = self.music_player.try_seek(pos);
        self.value = pos.as_millis() as f64
    }
    fn sync(&mut self, store: &mut MusicStore) -> Result<(), ()> {
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
    fn ps_switch(&mut self) -> bool {
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

#[derive(Clone)]
enum SettingKeys {
    ShowSideBar(bool),
    Volume(Float),
    KeepPlayState(bool)
}
#[derive(Serialize, Deserialize)]
struct Settings {
    show_side_bar: bool,
    volume: Float,
    search_origin: Vec<PathBuf>,
    tags: HashMap<String, Vec<MusicFile>>,
    last_music: Option<MusicFile>,
    last_position: Duration,
    last_playback: PlaybackType,
    last_playlist: PlayList,
    keep_state: bool
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
            keep_state: false
        }
    }
}

impl Settings {
    fn set_setting(&mut self, key: SettingKeys) {
        match key {
            SettingKeys::ShowSideBar(b) => self.show_side_bar = b,
            SettingKeys::Volume(v) => self.volume = v,
            SettingKeys::KeepPlayState(b) => self.keep_state = b
        }
    }
    fn save(&self, store: &MusicStore, player: &Player) -> Self {
        Settings {
            search_origin: store.search_origin.clone(),
            tags: store.tags.clone(),
            last_music: player.now_playing.clone(),
            last_position: player.music_player.get_pos(),
            last_playback: player.playback_type.clone(),
            last_playlist: player.play_list.clone(),
            ..*self
        }
    }
}

struct RIMusic {
    operate_files: Option<MusicFile>,
    player: Player,
    force_stop: bool,
    show_music_open_failure: bool,
    side_bar_show: bool,
    view: View,
    show_set_tag_modal: bool,
    new_tag: String,
    show_new_tag_modal: bool,
    search_text: String,
    show_exp_search_origin: bool,
    store: MusicStore,
    settings: Settings,
    exit: bool,
    idx: usize,
}

impl Default for RIMusic {
    fn default() -> Self {
        let settings = confy::load("RIMusic", None).unwrap_or_else(|_|Settings::default());
        let mut store = MusicStore::new();
        let mut player = Player::new();
        Self::init(&mut player, &mut store, &settings);
        RIMusic {
            store,
            operate_files: None,
            player,
            force_stop: false,
            show_music_open_failure: false,
            side_bar_show: false,
            view: View::MainView,
            show_set_tag_modal: false,
            new_tag: String::new(),
            show_new_tag_modal: false,
            search_text: String::new(),
            show_exp_search_origin: false,
            settings,
            exit: false,
            idx: 0
        }
    }
}

// UI逻辑
impl RIMusic {
    fn tags_card(&self, tags: String) -> iced::Element<'_, Message> {
        widget::button(
            widget::column![
                widget::text(tags.clone())
                    .size(28)
                    .width(iced::Fill)
                    .height(iced::Fill)
                    .wrapping(widget::text::Wrapping::WordOrGlyph),
                widget::text(self.store.tags.get(&tags).unwrap().len())
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .size(48)
                    .style(widget::text::base)
            ].width(iced::Fill)
        )
            .on_press(Message::GoView(View::TagView(tags)))
            .padding(8)
            .into()
    }
    fn artists_card(&self, artist: String) -> iced::Element<'_, Message> {
        widget::button(
            widget::column![
                widget::text(artist.clone())
                    .size(28)
                    .width(iced::Fill)
                    .height(iced::Fill)
                    .wrapping(widget::text::Wrapping::WordOrGlyph),
                widget::text(self.store.artists.get(&artist).unwrap().len())
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .size(48)
                    .style(widget::text::base)
            ].width(iced::Fill)
        )
            .on_press(Message::GoView(View::ArtistView(artist)))
            .padding(8)
            .into()
    }

    fn side_bar(&self) -> iced::Element<'_, Message> {
        widget::row([
            widget::container(
                widget::column![
                    widget::text("RIMusic")
                        .size(48)
                        .style(widget::text::primary),
                    widget::space().height(48),
                    CommonWidget::side_bar_button(&self, String::from("曲库"), View::MainView),
                    CommonWidget::side_bar_button(&self, String::from("标签"), View::TagsView),
                    CommonWidget::side_bar_button(&self, String::from("搜索"), View::SearchView),
                    CommonWidget::side_bar_button(&self, String::from("艺术家"), View::ArtistsView),
                    widget::container(
                        CommonWidget::side_bar_button(&self, String::from("设置"), View::SettingsView)
                    ).height(iced::Fill).align_y(iced::alignment::Vertical::Bottom)
                ]
                    .spacing(8)
                    .align_x(iced::alignment::Horizontal::Center),
            )
                .style(widget::container::bordered_box)
                .padding(8)
                .width(iced::Shrink)
                .height(iced::Fill)
                .into(),
            self.settings.show_side_bar
                .then(|| widget::container(widget::space()).width(0).into())
                .unwrap_or_else(|| widget::container(
                    widget::button("")
                        .style(|_, _| widget::button::Style {
                            background: Some(iced::Background::Color(Color::from_rgba8(
                                0, 0, 0, 0.4,
                            ))),
                            ..widget::button::Style::default()
                        })
                        .width(iced::Fill)
                        .height(iced::Fill)
                        .on_press(Message::SwitchSideBarShow),
                ).width(iced::Fill)
                    .height(iced::Fill)
                    .into())
        ])
            .into()
    }
    fn check_music_in_tag(&self, k: &String) -> Option<Message> {
        if let Some(x) = self.store.tags.get(k) {
            return if x.contains(&self.operate_files.clone().unwrap_or_else(|| MusicFile::default())) {
                None
            } else {
                Some(Message::AddTagTo(
                    k.clone(),
                    self.operate_files.clone().unwrap_or_else(|| MusicFile::default()),
                ))
            };
        }
        Some(Message::AddTagTo(
            k.clone(),
            self.operate_files.clone().unwrap_or_else(|| MusicFile::default() ),
        ))
    }

    fn check_tag_add(&self) -> Option<Message> {
        if self.new_tag.is_empty() {
            None
        } else {
            if let Some(_) = self.store.tags.get(&self.new_tag) {
                None
            } else {
                Some(Message::AddTag)
            }
        }
    }
    fn modal_add_tag(&self) -> iced::Element<'_, Message> {
        let mut names = vec![];
        for (k, _v) in self.store.tags.iter() {
            names.push(
                widget::button(
                    widget::text(k.clone()).wrapping(widget::text::Wrapping::WordOrGlyph),
                )
                    .on_press_maybe(self.check_music_in_tag(k))
                    .into(),
            )
        }
        CommonWidget::modal(
            widget::column![
                widget::text("选择欲添加的标签🏷")
                    .size(48)
                    .style(widget::text::primary),
                widget::center_x(
                    widget::grid(names).fluid(100).spacing(8)
                ),
                widget::container(
                    widget::button("关闭")
                        .style(widget::button::danger)
                        .on_press(Message::CloseSetTag)
                )
                .width(iced::Fill)
                .align_x(iced::alignment::Horizontal::Right)
            ]
                .spacing(8)
                .width(iced::Shrink),
        )
    }
    fn modal_create_tag(&self) -> iced::Element<'_, Message> {
        CommonWidget::modal(
            widget::column![
                widget::text("新建标签🏷")
                    .size(48)
                    .style(widget::text::primary),
                widget::text_input("请输入标签名称", &self.new_tag).on_input(Message::WhenNewTagType),
                widget::container(
                    widget::row![
                        widget::button("添加").on_press_maybe(self.check_tag_add()),
                        widget::button("关闭")
                            .style(widget::button::danger)
                            .on_press(Message::CloseNewTagMsg)
                    ]
                    .spacing(8)
                )
                .width(iced::Fill)
                .align_x(iced::alignment::Horizontal::Right)
            ]
                .spacing(8)
                .width(iced::Shrink),
        )
    }
    fn modal_music_open_fault(&self) -> iced::Element<'_, Message> {
        CommonWidget::modal(
            widget::column![
                widget::text("音频文件错误")
                    .size(48)
                    .style(widget::text::danger),
                widget::container(widget::button("关闭").on_press(Message::CloseMusicOpenFailureMsg))
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right),
            ]
                .spacing(8)
                .width(iced::Shrink),
        )
    }

    fn music_card(&self, title: MusicFile) -> iced::Element<'_, Message> {
        let p = title.clone();
        let z = title.clone();
        CommonWidget::panel(
            widget::column![
            widget::text(p.music.file_name().unwrap().to_string_lossy().to_string()).size(24),
            widget::row![
                    widget::row![
                        widget::text(title.duration.is_some()
                            .then(|| Self::format_time(title.duration.unwrap().as_secs_f64()))
                            .unwrap_or_else(|| "--:--".to_string())),
                        widget::text("|"),
                        widget::text(z.artist.is_some()
                            .then(|| z.artist.unwrap().clone().to_owned())
                            .unwrap_or_else(|| "--".to_string())
                        )
                    ],
                    widget::container(
                        widget::row![
                            match &self.view {
                                View::TagView(_) =>
                                    widget::button("移除该标签").style(widget::button::danger).on_press(Message::RemoveMusicFromTag(title.clone())),
                                _ => widget::button("").width(0),
                            },
                            widget::button("TAGS").on_press(Message::OpenSetTagMsg(title.clone())),
                            widget::button("PLAY").on_press(Message::PlayMusic(p.clone()))
                        ]
                        .spacing(8)
                    )
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right),

                ]
                .width(iced::Fill)
                .align_y(iced::alignment::Vertical::Center),
                ].into()
        )
    }

    fn main_view(&self) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in self.store.music_files.iter() {
            cards.push(self.music_card(f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("MainView".to_string(), None, None, &self.settings),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "空空如也,像冬天的落叶一样", act),
        ].spacing(12))
    }
    fn tags_view(&self) -> iced::Element<'_, Message> {
        let mut tagcs = vec![];
        for k in self.store.tags.keys() {
            tagcs.push(self.tags_card(k.to_string()))
        }
        let act = tagcs.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("TagsView".to_string(),
                Some(widget::button("+").on_press(Message::OpenNewTagMsg).into()),
                None,
                &self.settings),
            Self::show_or_text(widget::grid(tagcs).fluid(200).spacing(8).into(), "什么都没有,来创建一个新的标签吧!", act),
        ].spacing(12)
        )
    }
    fn search_view(&self) -> iced::Element<'_, Message> {
        let mut show = vec![];
        for i in self.search_music_file(&self.search_text) {
            show.push(self.music_card(i))
        }
        let act = self.search_text.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("SearchView".to_string(), None, None, &self.settings),
            widget::text_input("搜你所爱", &self.search_text)
                .on_input(Message::WhenSearchTextType)
                .width(iced::Fill),
            Self::show_or_text(widget::column(show).spacing(12).into(), "搜索从这里开始!", act),
            ].spacing(12)
        )
    }
    fn tag_view(&self, tag: String) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in self.store.tags.get(&tag).unwrap() {
            cards.push(self.music_card(f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar(
                tag.clone(),
                Some(widget::button("🗑").style(widget::button::danger).on_press(Message::RemoveTag(tag)).into()),
                Some(Message::GoView(View::TagsView)),
                &self.settings),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "嗯...加点什么好呢?", act),
        ].spacing(12)
        )
    }
    fn settings_view(&self) -> iced::Element<'_, Message> {
        let mut so = vec![];
        for i in &self.store.search_origin {
            so.push(CommonWidget::path_show(i))
        }
        CommonWidget::view_builder(
            widget::column![
                CommonWidget::title_bar("SettingsView".to_string(), None, None, &self.settings),
                widget::scrollable(
                    widget::column(
                        [
                            CommonWidget::setting_group("常规".to_string(),
                                widget::column![
                                    CommonWidget::setting_card("保留播放状态".to_string(), widget::toggler(self.settings.keep_state).on_toggle(|x| Message::WhenSettingChanged(SettingKeys::KeepPlayState(x))).into()),
                                    CommonWidget::expand_content("搜索源".to_string(),
                                        widget::container(
                                            widget::column![
                                                widget::scrollable(
                                                    widget::column(so).spacing(12)
                                                ).height(200),
                                                widget::container(
                                                    widget::button("添加").on_press(Message::OnAddSearchOriginClicked)
                                                ).width(iced::Fill).align_x(iced::alignment::Horizontal::Center)
                                            ]
                                        ).into(),
                                        &self.show_exp_search_origin, Message::WhenExpSearchOriginClicked),
                                ].spacing(12).into()
                            ),
                            CommonWidget::setting_group("外观".to_string(),
                                widget::column![
                                    CommonWidget::setting_card("侧边栏常驻".to_string(), widget::toggler(self.settings.show_side_bar)
                                        .on_toggle(|x| Message::WhenSettingChanged(SettingKeys::ShowSideBar(x))).into())
                                ].spacing(12).into()
                            ),
                            CommonWidget::setting_group("关于".to_string(), widget::column![
                                widget::text("版本: v1.0.0").size(18),
                                widget::text("作者: 氢気氚").size(18)
                            ].into())
                        ]
                    ).spacing(32)
                )
            ]

        )
    }
    fn artists_view(&self) -> iced::Element<'_, Message> {
        let mut artists = vec![];
        for k in self.store.artists.keys() {
            artists.push(self.artists_card(k.to_string()))
        }
        let act = artists.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("ArtistsView".to_string(),
                None,
                None,
                &self.settings),
            Self::show_or_text(widget::grid(artists).fluid(200).spacing(8).into(), "群星...", act),
        ].spacing(12)
        )
    }
    fn artist_view(&self, artist: String) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in self.store.artists.get(&artist).unwrap() {
            cards.push(self.music_card(f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar(
                artist.clone(),
                None,
                Some(Message::GoView(View::ArtistsView)),
                &self.settings),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "嗯...为什么是空的呢,但你是不可能看到这个字的!😨", act),
        ].spacing(12)
        )
    }
    fn play_bar(&self) -> iced::Element<'_, Message> {
        let sub1 = iced_aw::menu::Item::with_menu(
            widget::button(widget::text(self.player.playback_type.to_string()))
                .style(widget::button::text),
            Menu::new(
                [
                    iced_aw::menu::Item::new(
                        widget::button("单曲即停")
                            .style(widget::button::text)
                            .on_press(Message::OnPlaybackTypeChanged(PlaybackType::OnceStop)),
                    ),
                    iced_aw::menu::Item::new(
                        widget::button("单曲循环")
                            .style(widget::button::text)
                            .on_press(Message::OnPlaybackTypeChanged(PlaybackType::OneWhile)),
                    ),
                    iced_aw::menu::Item::new(
                        widget::button("下一曲")
                            .style(widget::button::text)
                            .on_press(Message::OnPlaybackTypeChanged(PlaybackType::MusicNext)),
                    ),
                    iced_aw::menu::Item::new(
                        widget::button("随机挑选")
                            .style(widget::button::text)
                            .on_press(Message::OnPlaybackTypeChanged(PlaybackType::RadomPlay)),
                    ),
                ]
                    .into(),
            )
                .width(iced::Length::Shrink),
        );
        let bar = iced_aw::MenuBar::new([sub1].into());
        let vols = iced_aw::menu::Item::with_menu(
            widget::button(widget::text(format!("音量:{}", ((self.settings.volume as f32) * 100f32) as i32)))
                .style(widget::button::text),
            Menu::new(
                [
                    iced_aw::menu::Item::new(
                        widget::vertical_slider(0..=100, ((self.settings.volume as f32) * 100f32) as i32, |x| Message::WhenSettingChanged(SettingKeys::Volume(x as f32 / 100f32 as Float)))
                            .step(1)
                            .height(200)
                    )
                ].into()
            ).padding(18).width(iced::Length::Shrink),
        );
        let vol = iced_aw::MenuBar::new([vols].into());
        widget::container(
            widget::column![
                widget::text(&self.player.music_name).size(48),
                widget::row![
                    widget::text(Self::format_time(
                        Duration::from_millis(self.player.value as u64).as_secs_f64()
                    ))
                    .size(28),
                    widget::button("⏮").on_press_maybe(self.player.now_playing.is_some().then(|| Some(Message::PrevMusic)).unwrap_or_else(|| None)),
                    widget::button(
                        (self.player.play_state == PlayState::Play)
                            .then(|| "⏸")
                            .unwrap_or_else(|| "▶️")
                    )
                    .on_press(Message::OnPSSwitchClicked),
                    widget::button("️⏭").on_press_maybe(self.player.now_playing.is_some().then(|| Some(Message::NextMusic)).unwrap_or_else(|| None)),
                    bar,
                    vol,
                    widget::slider(
                        0f64..=self.player.total_music_time.as_millis() as f64,
                        self.player.value,
                        Message::OnValueChanged
                    )
                    .step(0.01)
                    .on_release(Message::OnReleaseSlider)
                ]
                .spacing(8)
                .align_y(iced::alignment::Vertical::Center),
            ]
                .padding(8),
        )
            .width(iced::Fill)
            .style(|x| widget::container::secondary(x))
            .into()
    }
    fn content(&self) -> iced::Element<'_, Message> {
        widget::column![
            widget::row![
                self.settings.show_side_bar
                    .then(|| widget::container(self.side_bar()))
                    .unwrap_or_else(|| widget::container(widget::space())),
                match &self.view {
                    View::MainView => self.main_view(),
                    View::TagsView => self.tags_view(),
                    View::TagView(t) => self.tag_view(t.clone()),
                    View::SearchView => self.search_view(),
                    View::SettingsView => self.settings_view(),
                    View::ArtistsView => self.artists_view(),
                    View::ArtistView(a) => self.artist_view(a.clone())
                }
            ],
            self.play_bar()
        ]
            .height(iced::Fill)
            .into()
    }
    fn show_or_space(view: iced::Element<Message>, act: bool) -> iced::Element<Message> {
        act.then(|| view).unwrap_or_else(|| widget::space().into())
    }
    fn show_or_text<'a>(list: iced::Element<'a, Message>, text: &'a str, act: bool) -> iced::Element<'a, Message> {
        act.then(||
                widget::container(
                    widget::center(
                        widget::text(text).size(48)
                    )
                )
            ).unwrap_or_else(||
            widget::container(
                widget::scrollable(list).spacing(8)
            )
        ).into()
    }
    fn view(&self) -> iced::Element<'_, Message> {
        widget::stack([
            self.content(),
            Self::show_or_space(self.modal_music_open_fault(), self.show_music_open_failure),
            Self::show_or_space(self.modal_add_tag(), self.show_set_tag_modal),
            Self::show_or_space(self.modal_create_tag(), self.show_new_tag_modal),
            Self::show_or_space(self.side_bar(), self.side_bar_show),
        ])
            .into()
    }
}

// 工具方法
impl RIMusic {
    fn search_music_file(&self, text: &String) -> Vec<MusicFile> {
        let ret: Vec<_> = self.store.music_files.iter()
            .filter(|x| x.music.to_string_lossy().to_lowercase().contains(&text.to_lowercase()))
            .map(|x| x.clone())
            .collect();
        ret
    }
    fn format_time(seconds: f64) -> String {
        let secs = seconds as u64;
        format!("{}:{:02}", secs / 60, secs % 60)
    }
    fn init(player: &mut Player, store: &mut MusicStore, settings: &Settings) {
        store.search_origin = settings.search_origin.clone();
        store.tags = settings.tags.clone();
        store.sync();
        player.set_volume(settings.volume);
        if settings.keep_state {
            player.total_music_time = settings.last_music.is_some()
                .then(|| {
                    let lm = &mut settings.last_music.clone().unwrap();
                    lm.get_music_file_total_duration();
                    lm.duration.unwrap()
                })
                .unwrap_or_else(|| Duration::from_secs(0));
            player.playback_type = settings.last_playback;
            player.play_list = settings.last_playlist.clone();
            if settings.last_music.is_some() {
                let _ = player.play_music(&mut settings.last_music.clone().unwrap(), store);
                player.music_player.pause();
                player.set_pos(settings.last_position);
                player.play_state = PlayState::Stop;
            }
        }
    }
}

// 执行逻辑
impl RIMusic {
    fn update(&mut self, msg: Message) -> iced::Task<Message> {
        match msg {
            Message::OnValueChanged(x) => {
                self.player.music_player.pause();
                self.player.play_state = PlayState::Stop;
                self.player.value = x;
                self.player.music_player
                    .try_seek(Duration::from_millis(x as u64))
                    .unwrap();
            }
            Message::PlayMusic(mut p) => {
                match &self.view {
                    View::TagView(t) => self.player.play_list = PlayList::Tags(t.clone()),
                    View::MainView => self.player.play_list = PlayList::AllMusic,
                    View::TagsView => (),
                    View::SearchView => self.player.play_list = PlayList::AllMusic,
                    View::SettingsView => (),
                    View::ArtistsView => (),
                    View::ArtistView(a) => self.player.play_list = PlayList::Artist(a.clone())
                }
                if let Err(_) = self.player.play_music(&mut p, &mut self.store) {
                    self.show_music_open_failure = true
                }
            }
            Message::Sync => {
                if let Err(_) = self.player.sync(&mut self.store) {
                    self.show_music_open_failure = true;
                }
            }
            Message::OnReleaseSlider => {
                if !self.force_stop {
                    self.player.music_player.play();
                    self.player.play_state = PlayState::Play;
                }
            }
            Message::OnPSSwitchClicked => self.force_stop = self.player.ps_switch(),
            Message::CloseMusicOpenFailureMsg => self.show_music_open_failure = false,
            Message::OnPlaybackTypeChanged(p) => self.player.playback_type = p,
            Message::SwitchSideBarShow => self.side_bar_show = !self.side_bar_show,
            Message::GoView(v) => {
                self.view = v;
                self.side_bar_show = false;
            }
            Message::OpenSetTagMsg(p) => {
                self.show_set_tag_modal = true;
                self.operate_files = Some(p)
            }
            Message::AddTagTo(t, p) => {
                self.store.add_tag_for_music(t, p);
                self.show_set_tag_modal = false;
            }
            Message::CloseSetTag => self.show_set_tag_modal = false,
            Message::WhenNewTagType(t) => self.new_tag = t,
            Message::OpenNewTagMsg => self.show_new_tag_modal = true,
            Message::CloseNewTagMsg => {
                self.show_new_tag_modal = false;
                self.new_tag.clear();
            }
            Message::AddTag => {
                self.store.tags.insert(self.new_tag.clone(), vec![]);
                self.show_new_tag_modal = false;
                self.new_tag.clear();
            }
            Message::PrevMusic => {
                if let Err(_) = self.player.music_play_push(-1, &mut self.store) {
                    self.show_music_open_failure = true
                }
            },
            Message::NextMusic => {
                if let Err(_) = self.player.music_play_push(1, &mut self.store) {
                    self.show_music_open_failure = true
                }
            }
            Message::RemoveMusicFromTag(p) => {
                if let View::TagView(t) = &self.view {
                    self.store.remove_music_from_tag(&mut self.player, t, p)
                }
            }
            Message::RemoveTag(t) => {
                self.view = View::TagsView;
                self.store.remove_tag(&mut self.player, t);
            }
            Message::WhenSearchTextType(s) => {
                self.search_text = s
            }
            Message::WhenExpSearchOriginClicked => self.show_exp_search_origin = !self.show_exp_search_origin,
            Message::WhenSettingChanged(t) => {
                self.settings.set_setting(t.clone());
                if let SettingKeys::Volume(f) = t {
                    self.player.set_volume(f);
                }
            }
            Message::DeleteOriginPath(p) => {
                if let Some(val) = &self.player.now_playing {
                    if let Some(par) = val.music.parent() {
                        if par == p.as_path() {
                            self.player.player_default();
                        }
                    }
                }
                self.store.remove_search_origin(p);
                self.store.sync();
                self.idx = 0;
                self.store.is_read = true
            }
            Message::OnAddSearchOriginClicked => {
                let path = rfd::FileDialog::new().pick_folder();
                if let Some(pat) = path {
                    let idx = self.store.search_origin.iter().position(|x| *x == pat);
                    if let None = idx {
                    self.store.search_origin.push(pat);
                    self.store.sync_only_push();
                    }
                    self.idx = 0;
                    self.store.is_read = true
                }
            }
            Message::CloseSave => {
                let export = self.settings.save(&self.store, &self.player);
                let _ = confy::store("RIMusic", None, export);
                self.exit = true
            }
            Message::ReadFileData => {
                self.store.read_one();
            }
        }
        if self.exit {
            return iced::exit()
        }
        iced::Task::none()
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::window::close_requests().map(|_| Message::CloseSave),
            (self.player.play_state == PlayState::Play)
                .then(|| iced::time::every(Duration::from_millis(50)).map(|_| Message::Sync))
                .unwrap_or_else(|| iced::Subscription::none()),
            self.store.is_read
                .then(|| iced::time::every(Duration::from_millis(100)).map(|_| Message::ReadFileData))
                .unwrap_or_else(|| iced::Subscription::none())

        ])
    }
}

fn main() -> iced::Result {
    let m = iced::Theme::Custom(Arc::new(Custom::new("Caption".to_string(), cap())));
    iced::application(RIMusic::default, RIMusic::update, RIMusic::view)
        .subscription(RIMusic::subscription)
        .theme(m)
        .exit_on_close_request(false)
        .run()
}
