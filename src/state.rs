use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::music_file::MusicFile;
use crate::player::Player;
use crate::settings::{SettingKeys, Settings};
use crate::store::MusicStore;

#[derive(Clone)]
pub struct UiArgs<'a> {
    pub player: &'a Player,
    pub store: &'a MusicStore,
    pub settings: &'a Settings
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum PlayList {
    AllMusic,
    Tags(String),
    Artist(String)
}

#[derive(Clone, Serialize, Deserialize, Copy, Debug)]
pub enum PlaybackType {
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
pub enum UiMessage {
    OnValueChanged(f64),
    PlayMusic(MusicFile),
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
}

#[derive(Clone)]
pub enum BackendMessage {
    Sync,
    CloseSave,
    ReadFileData
}

#[derive(Clone)]
pub enum Message {
    Ui(UiMessage),
    Backend(BackendMessage)
}
#[derive(PartialEq)]
pub enum PlayState {
    Stop,
    Play,
}

#[derive(Clone, PartialEq)]
pub enum View {
    MainView,
    TagsView,
    TagView(String),
    SearchView,
    SettingsView,
    ArtistsView,
    ArtistView(String)
}