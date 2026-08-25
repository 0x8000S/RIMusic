use iced::{widget, Color};
use iced_aw::Menu;
use lofty::prelude::*;
use rodio::Source;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use iced::theme::{Custom, Palette};

fn metro() -> Palette {
    Palette {
        background: Color::from_rgb8(30, 30, 46),      // 深蓝灰背景
        text: Color::from_rgb8(205, 214, 244),          // 浅灰文字
        primary: Color::from_rgb8(137, 180, 250),       // 强调色（蓝）
        success: Color::from_rgb8(166, 227, 161),       // 成功绿
        danger: Color::from_rgb8(243, 139, 168),        // 危险红
        warning: Color::from_rgb8(216, 118, 0)
    }
}

#[derive(Clone)]
enum PlayList {
    AllMusic,
    Tags(String),
}

#[derive(Clone)]
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
    WhenSearchTextType(String)
}
#[derive(PartialEq)]
enum PlayState {
    Stop,
    Play,
}

#[derive(Clone, PartialEq)]
enum View {
    MainView,
    Tags,
    TagView(String),
    SearchView,
    // Settings
}

#[derive(Clone)]
struct MusicFile {
    music: PathBuf,
    duration: Option<Duration>
}
impl Default for MusicFile {
    fn default() -> Self {
        Self {
            music: PathBuf::new(),
            duration: None
        }
    }
}
impl PartialEq for MusicFile {
    fn eq(&self, other: &Self) -> bool {
        self.music == other.music
    }
}

impl MusicFile {
    fn get_music_file_total_duration(&mut self) -> Duration {
        if let Some(_) = self.duration {
            return self.duration.unwrap()
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
        self.duration.unwrap()
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
    fn title_bar(view_name: String, action_buttons: iced::Element<Message>, back_view: Option<Message>) -> iced::Element<Message> {
        widget::row![
                widget::button(
                back_view.is_some().then(|| "<")
                .unwrap_or_else(|| "=")
            ).on_press(back_view.is_some().then(|| back_view.unwrap()).unwrap_or_else(|| Message::SwitchSideBarShow)),
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
}

struct RIMusic {
    value: f64,
    music_files: Vec<MusicFile>,
    _music_handle: rodio::MixerDeviceSink,
    music_player: rodio::Player,
    total_dur: Duration,
    play_state: PlayState,
    music_name: String,
    force_stop: bool,
    now_playing: Option<MusicFile>,
    playback_type: PlaybackType,
    show_music_open_failure: bool,
    side_bar_show: bool,
    view: View,
    tags: HashMap<String, Vec<MusicFile>>,
    show_set_tag_modal: bool,
    operate_files: Option<MusicFile>,
    play_list: PlayList,
    new_tag: String,
    show_new_tag_modal: bool,
    search_text: String
}

impl Default for RIMusic {
    fn default() -> Self {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(&handle.mixer());
        let mut t = vec![];
        for i in RIMusic::get_music_file() {
            t.push(MusicFile {
                music: i,
                duration: None
            });
        }
        RIMusic {
            value: 0f64,
            music_files: t,
            _music_handle: handle,
            music_player: player,
            total_dur: Duration::from_secs(0),
            play_state: PlayState::Stop,
            music_name: String::from("TEST MUSIC TITLE"),
            force_stop: false,
            now_playing: None,
            playback_type: PlaybackType::OnceStop,
            show_music_open_failure: false,
            side_bar_show: false,
            view: View::MainView,
            tags: HashMap::new(),
            show_set_tag_modal: false,
            operate_files: None,
            play_list: PlayList::AllMusic,
            new_tag: String::new(),
            show_new_tag_modal: false,
            search_text: String::new()
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
                widget::text(self.tags.get(&tags).unwrap().len())
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

    fn side_bar(&self) -> iced::Element<'_, Message> {
        widget::row([
            widget::container(
                widget::column![
                    widget::text("RIMusic")
                        .size(48)
                        .style(widget::text::primary),
                    widget::space().height(48),
                    CommonWidget::side_bar_button(&self, String::from("曲库"), View::MainView),
                    CommonWidget::side_bar_button(&self, String::from("标签"), View::Tags),
                    CommonWidget::side_bar_button(&self, String::from("搜索"), View::SearchView),
                    // widget::container(
                    //     CommonWidget::side_bar_button(&self, String::from("设置"), View::SearchView)
                    // ).height(iced::Fill).align_y(iced::alignment::Vertical::Bottom)
                ]
                    .spacing(8)
                    .align_x(iced::alignment::Horizontal::Center),
            )
                .style(widget::container::bordered_box)
                .padding(8)
                .width(iced::Shrink)
                .height(iced::Fill)
                .into(),
            widget::button("")
                .style(|_, _| widget::button::Style {
                    background: Some(iced::Background::Color(Color::from_rgba8(
                        0, 0, 0, 0.4,
                    ))),
                    ..widget::button::Style::default()
                })
                .width(iced::Fill)
                .height(iced::Fill)
                .on_press(Message::SwitchSideBarShow)
                .into(),
        ])
            .width(iced::Fill)
            .height(iced::Fill)
            .into()
    }
    fn check_music_in_tag(&self, k: &String) -> Option<Message> {
        if let Some(x) = self.tags.get(k) {
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
            if let Some(_) = self.tags.get(&self.new_tag) {
                None
            } else {
                Some(Message::AddTag)
            }
        }
    }
    fn modal_add_tag(&self) -> iced::Element<'_, Message> {
        let mut names = vec![];
        for (k, _v) in self.tags.iter() {
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
        widget::container(
            widget::column![
            widget::text(p.music.file_name().unwrap().to_string_lossy().to_string()).size(24),
            widget::row![
                    widget::text((title.duration.is_some())
                        .then(|| Self::format_time(title.duration.unwrap().as_secs_f64()))
                        .unwrap_or_else(|| "--:--".to_string())),
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
                ]
        ).padding(18)
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

    fn main_view(&self) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in self.music_files.iter() {
            cards.push(self.music_card(f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("MainView".to_string(), widget::space().into(), None),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "空空如也,像冬天的落叶一样", act),
        ].spacing(12))
    }
    fn tags_view(&self) -> iced::Element<'_, Message> {
        let mut tagcs = vec![];
        for k in self.tags.keys() {
            tagcs.push(self.tags_card(k.to_string()))
        }
        let act = tagcs.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("TagsView".to_string(), widget::button("+").on_press(Message::OpenNewTagMsg).into(), None),
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
            CommonWidget::title_bar("SearchView".to_string(), widget::space().into(), None),
            widget::text_input("搜你所爱", &self.search_text)
                .on_input(Message::WhenSearchTextType)
                .width(iced::Fill),
            Self::show_or_text(widget::column(show).spacing(12).into(), "搜索从这里开始!", act),
            ].spacing(12)
        )
    }
    fn tag_view(&self, tag: String) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in self.tags.get(&tag).unwrap() {
            cards.push(self.music_card(f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar(tag.clone(), widget::button("🗑").style(widget::button::danger).on_press(Message::RemoveTag(tag)).into(), Some(Message::GoView(View::Tags))),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "嗯...加点什么好呢?", act),
        ].spacing(12)
        )
    }
    fn play_bar(&self) -> iced::Element<'_, Message> {
        let sub1 = iced_aw::menu::Item::with_menu(
            widget::button(widget::text(self.playback_type.to_string()))
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
        widget::container(
            widget::column![
                widget::text(&self.music_name).size(48),
                widget::row![
                    widget::text(Self::format_time(
                        Duration::from_millis(self.value as u64).as_secs_f64()
                    ))
                    .size(28),
                    widget::button("⏮").on_press_maybe(self.now_playing.is_some().then(|| Some(Message::PrevMusic)).unwrap_or_else(|| None)),
                    widget::button(
                        (self.play_state == PlayState::Play)
                            .then(|| "⏸")
                            .unwrap_or_else(|| "▶️")
                    )
                    .on_press(Message::OnPSSwitchClicked),
                    widget::button("️⏭").on_press_maybe(self.now_playing.is_some().then(|| Some(Message::NextMusic)).unwrap_or_else(|| None)),
                    bar,
                    widget::slider(
                        0f64..=self.total_dur.as_millis() as f64,
                        self.value,
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
            match &self.view {
                View::MainView => self.main_view(),
                View::Tags => self.tags_view(),
                View::TagView(t) => self.tag_view(t.clone()),
                View::SearchView => self.search_view()
            },
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
        let ret: Vec<_> = self.music_files.iter()
            .filter(|x| x.music.to_string_lossy().to_lowercase().contains(&text.to_lowercase()))
            .map(|x| x.clone())
            .collect();
        ret
    }
    fn get_music_file() -> Vec<PathBuf> {
        let mut files = vec![];
        if let Some(v) = dirs::audio_dir() {
            for f in std::fs::read_dir(v).unwrap() {
                let f = f.unwrap();
                let path = f.path();
                if path.is_file() {
                    files.push(path)
                }
            }
        } else {
            return vec![];
        }
        files
    }
    fn format_time(seconds: f64) -> String {
        let secs = seconds as u64;
        format!("{}:{:02}", secs / 60, secs % 60)
    }
    fn play_music(&mut self, p: &mut MusicFile) {
        let file = std::fs::File::open(&p.music).unwrap();
        self.now_playing = Some(p.clone());
        if let Ok(source) = rodio::Decoder::try_from(file) {
            self.play_state = PlayState::Play;
            self.total_dur = p.get_music_file_total_duration();
            match &self.play_list {
                PlayList::AllMusic => {
                    let idx = self.find_music();
                    p.get_music_file_total_duration();
                    self.music_files[idx] = p.clone();
                }
                PlayList::Tags(t) => {
                    let idx = self.find_music();
                    p.get_music_file_total_duration();
                    self.tags.get_mut(t).unwrap()[idx] = p.clone();
                }
            }
            self.music_player.clear();
            self.value = self.music_player.get_pos().as_millis() as f64;
            self.music_player.append(source);
            self.music_player.play();
        } else {
            self.show_music_open_failure = true;
        }
        let name = p.music.file_name().unwrap().to_str().unwrap().to_string();
        let chars: Vec<_> = name.chars().collect();
        if chars.iter().len() > 20 {
            let pre20 = String::from_iter(chars.get(..20).unwrap().to_owned().iter());
            self.music_name = format!("{}...", pre20);
        } else {
            self.music_name = name;
        }
    }
    fn find_music(&self) -> usize {
        match &self.play_list {
            PlayList::AllMusic => {
                self
                    .music_files
                    .iter()
                    .position(|x| {
                        x.clone() == self.now_playing.clone().unwrap()
                    })
                    .unwrap()
            }
            PlayList::Tags(t) => {
                let music_files = &self.tags[t];
                music_files
                    .iter()
                    .position(|x| {
                        x.clone() == self.now_playing.clone().unwrap()
                    })
                    .unwrap()
            }
        }
    }
    fn music_play_push(&mut self, pidx: i64) {
        match &self.play_list {
            PlayList::AllMusic => {
                let mut idx = self.find_music();
                let ret = idx  as i64 + pidx;
                if ret == self.music_files.len() as i64 {
                    idx = 0;
                } else if ret < 0 {
                    idx = self.music_files.len() - 1;
                } else {
                    idx = ret as usize;
                }
                self.play_music(&mut self.music_files[idx].clone());
            }
            PlayList::Tags(t) => {
                let music_files = &self.tags[t];
                let mut idx = self.find_music();
                let ret = idx  as i64 + pidx;
                if ret == music_files.len() as i64 {
                    idx = 0;
                } else if ret < 0 {
                    idx = music_files.len() - 1;
                } else {
                    idx = ret as usize;
                }
                self.play_music(&mut music_files[idx].clone());
            }
        }
    }
    fn player_default(&mut self) {
        self.now_playing = None;
        self.music_player.stop();
        self.play_state = PlayState::Stop;
        self.value = 0.0;
        self.music_player.try_seek(Duration::from_secs(0)).unwrap();
        self.music_name = String::from("TEST MUSIC TITLE");
    }
}

// 执行逻辑
impl RIMusic {
    fn update(&mut self, msg: Message) {
        match msg {
            Message::OnValueChanged(x) => {
                self.music_player.pause();
                self.play_state = PlayState::Stop;
                self.value = x;
                self.music_player
                    .try_seek(Duration::from_millis(x as u64))
                    .unwrap();
            }
            Message::PlayMusic(mut p) => {
                match &self.view {
                    View::TagView(t) => self.play_list = PlayList::Tags(t.clone()),
                    View::MainView => self.play_list = PlayList::AllMusic,
                    View::Tags => (),
                    View::SearchView => self.play_list = PlayList::AllMusic
                }
                self.play_music(&mut p);
            }
            Message::Sync => {
                self.value = self.music_player.get_pos().as_millis() as f64;
                if self.music_player.empty() {
                    if let Some(p) = &self.now_playing {
                        match self.playback_type {
                            PlaybackType::OnceStop => {
                                self.play_state = PlayState::Stop;
                            }
                            PlaybackType::OneWhile => {
                                self.play_music(&mut p.clone());
                                self.play_state = PlayState::Play;
                                self.music_player.play();
                            }
                            PlaybackType::MusicNext => self.music_play_push(1),
                            PlaybackType::RadomPlay => match &self.play_list {
                                PlayList::AllMusic => {
                                    let idx = rand::random_range(0..self.music_files.len());
                                    self.play_music(&mut self.music_files[idx].clone());
                                }
                                PlayList::Tags(t) => {
                                    let music_files = &self.tags[t];
                                    let idx = rand::random_range(0..music_files.len());
                                    self.play_music(&mut music_files[idx].clone());
                                }
                            },
                        }
                    }
                    self.value = 0.0;
                    self.music_player.try_seek(Duration::from_secs(0)).unwrap();
                }
            }
            Message::OnReleaseSlider => {
                if !self.force_stop {
                    self.music_player.play();
                    self.play_state = PlayState::Play;
                }
            }
            Message::OnPSSwitchClicked => match self.play_state {
                PlayState::Play => {
                    self.music_player.pause();
                    self.play_state = PlayState::Stop;
                    self.force_stop = true;
                }
                PlayState::Stop => {
                    self.music_player.play();
                    self.play_state = PlayState::Play;
                    self.force_stop = false;
                }
            },
            Message::CloseMusicOpenFailureMsg => self.show_music_open_failure = false,
            Message::OnPlaybackTypeChanged(p) => self.playback_type = p,
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
                if let Some(v) = self.tags.get_mut(&t) {
                    v.push(p);
                } else {
                    self.tags.insert(t, vec![p]);
                }
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
                self.tags.insert(self.new_tag.clone(), vec![]);
                self.show_new_tag_modal = false;
                self.new_tag.clear();
            }
            Message::PrevMusic => self.music_play_push(-1),
            Message::NextMusic => self.music_play_push(1),
            Message::RemoveMusicFromTag(p) => {
                if let View::TagView(t) = &self.view {
                    let idx = self.tags.get(t).unwrap()
                        .iter().position(|x| *x == p).unwrap();
                    self.tags.get_mut(t).unwrap().remove(idx);
                    if let Some(x) = &self.now_playing {
                        if *x == p {
                        self.player_default()
                        }
                    }
                }
            }
            Message::RemoveTag(t) => {
                self.view = View::Tags;
                self.tags.remove(&t);
                if let PlayList::Tags(tx) = &self.play_list {
                    if *tx == t {
                        self.player_default()
                    }
                }
            }
            Message::WhenSearchTextType(s) => {
                self.search_text = s
            }
        }
    }
    fn subscription(&self) -> iced::Subscription<Message> {
        if self.play_state == PlayState::Play {
            return iced::time::every(Duration::from_millis(50)).map(|_| Message::Sync);
        }
        iced::Subscription::none()
    }
}

fn main() -> iced::Result {
    let m = iced::Theme::Custom(Arc::new(Custom::new("Metro".to_string(), metro())));
    iced::application(RIMusic::default, RIMusic::update, RIMusic::view)
        .subscription(RIMusic::subscription)
        .theme(m)
        .run()
}
