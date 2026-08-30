use std::path::PathBuf;
use std::time::Duration;
use iced::{widget, Color};
use iced_aw::Menu;
use rodio::Float;
use crate::music_file::MusicFile;
use crate::player::Player;
use crate::settings::{SettingKeys, Settings};
use crate::state::{PlayState, PlaybackType, Message, View, UiMessage, PlayList};
use crate::state::UiArgs;
use crate::store::MusicStore;

pub struct CommonWidget {}
impl CommonWidget {
    fn side_bar_button(env: &Ui, name: String, view: View) -> iced::Element<'_, Message> {
        widget::button(widget::text(name))
            .width(iced::Fill)
            .on_press_maybe(
                (view == env.view)
                    .then(|| None)
                    .unwrap_or_else(|| Some(Message::Ui(UiMessage::GoView(view)))),
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
    fn title_bar(view_name: String, action_buttons: Option<iced::Element<Message>>, back_view: Option<Message>, show_side_bar: bool) -> iced::Element<Message> {
        widget::row![
                if show_side_bar {
                    widget::container(widget::space())
                } else {
                    widget::container(
                        widget::button(
                            back_view.is_some().then(|| "<")
                            .unwrap_or_else(|| "≡")
                            ).on_press(back_view.is_some()
                                .then(|| back_view.unwrap())
                                .unwrap_or_else(|| Message::Ui(UiMessage::SwitchSideBarShow))),
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
                    widget::button("删除").on_press(Message::Ui(UiMessage::DeleteOriginPath(p.clone())))
                ).width(iced::Fill).align_x(iced::alignment::Horizontal::Right)
            ]
        ).width(iced::Fill).into()
    }
}

pub struct Ui {
    operate_files: Option<MusicFile>,
    force_stop: bool,
    pub show_music_open_failure: bool,
    side_bar_show: bool,
    view: View,
    show_set_tag_modal: bool,
    new_tag: String,
    show_new_tag_modal: bool,
    search_text: String,
    show_exp_search_origin: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            operate_files: None,
            force_stop: false,
            show_music_open_failure: false,
            side_bar_show: false,
            view: View::MainView,
            show_set_tag_modal: false,
            new_tag: String::new(),
            show_new_tag_modal: false,
            search_text: String::new(),
            show_exp_search_origin: false,
        }
    }
}

impl Ui {
    fn content<'a>(&'a self, args: UiArgs<'a>) -> iced::Element<'a, Message> {
        widget::column![
            widget::row![
                args.settings.show_side_bar
                    .then(|| widget::container(self.side_bar(args.clone())))
                    .unwrap_or_else(|| widget::container(widget::space())),
                match &self.view {
                    View::MainView => self.main_view(args.clone()),
                    View::TagsView => Ui::tags_view(args.clone()),
                    View::TagView(t) => self.tag_view(args.clone(), t.clone()),
                    View::SearchView => self.search_view(args.clone()),
                    View::SettingsView => self.settings_view(args.clone()),
                    View::ArtistsView => Ui::artists_view(args.clone()),
                    View::ArtistView(a) => self.artist_view(args.clone(), a.clone())
                }
            ],
            Ui::play_bar(args)
        ]
            .height(iced::Fill)
            .into()
    }

    pub fn view<'a>(&'a self, args: UiArgs<'a>) -> iced::Element<'a, Message> {
        widget::stack([
            self.content(args.clone()),
            Self::show_or_space(Ui::modal_music_open_fault(), self.show_music_open_failure),
            Self::show_or_space(self.modal_add_tag(args.clone()), self.show_set_tag_modal),
            Self::show_or_space(self.modal_create_tag(args.clone()), self.show_new_tag_modal),
            Self::show_or_space(self.side_bar(args.clone()), self.side_bar_show),
        ])
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
    fn format_time(seconds: f64) -> String {
        let secs = seconds as u64;
        format!("{}:{:02}", secs / 60, secs % 60)
    }
    fn tags_card(args: UiArgs<'_>, tags: String) -> iced::Element<'_, Message> {
        widget::button(
            widget::column![
                widget::text(tags.clone())
                    .size(28)
                    .width(iced::Fill)
                    .height(iced::Fill)
                    .wrapping(widget::text::Wrapping::WordOrGlyph),
                widget::text(args.store.tags.get(&tags).unwrap().len())
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .size(48)
                    .style(widget::text::base)
            ].width(iced::Fill)
        )
            .on_press(Message::Ui(UiMessage::GoView(View::TagView(tags))))
            .padding(8)
            .into()
    }
    fn artists_card(args: UiArgs<'_>, artist: String) -> iced::Element<'_, Message> {
        widget::button(
            widget::column![
                widget::text(artist.clone())
                    .size(28)
                    .width(iced::Fill)
                    .height(iced::Fill)
                    .wrapping(widget::text::Wrapping::WordOrGlyph),
                widget::text(args.store.artists.get(&artist).unwrap().len())
                    .width(iced::Fill)
                    .align_x(iced::alignment::Horizontal::Right)
                    .size(48)
                    .style(widget::text::base)
            ].width(iced::Fill)
        )
            .on_press(Message::Ui(UiMessage::GoView(View::ArtistView(artist))))
            .padding(8)
            .into()
    }

    fn side_bar(&self, args: UiArgs) -> iced::Element<'_, Message> {
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
            args.settings.show_side_bar
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
                        .on_press(Message::Ui(UiMessage::SwitchSideBarShow)),
                ).width(iced::Fill)
                    .height(iced::Fill)
                    .into())
        ])
            .into()
    }
    fn check_music_in_tag(&self, args: UiArgs, k: &String) -> Option<Message> {
        if let Some(x) = args.store.tags.get(k) {
            return if x.contains(&self.operate_files.clone().unwrap_or_else(|| MusicFile::default())) {
                None
            } else {
                Some(Message::Ui(UiMessage::AddTagTo(
                    k.clone(),
                    self.operate_files.clone().unwrap_or_else(|| MusicFile::default()),
                )))
            };
        }
        Some(Message::Ui(UiMessage::AddTagTo(
            k.clone(),
            self.operate_files.clone().unwrap_or_else(|| MusicFile::default() ),
        )))
    }

    fn check_tag_add(&self, args: UiArgs) -> Option<Message> {
        if self.new_tag.is_empty() {
            None
        } else {
            if let Some(_) = args.store.tags.get(&self.new_tag) {
                None
            } else {
                Some(Message::Ui(UiMessage::AddTag))
            }
        }
    }
    fn modal_add_tag(&self, args: UiArgs) -> iced::Element<'_, Message> {
        let mut names = vec![];
        for (k, _v) in args.store.tags.iter() {
            names.push(
                widget::button(
                    widget::text(k.clone()).wrapping(widget::text::Wrapping::WordOrGlyph),
                )
                    .on_press_maybe(Self::check_music_in_tag(&self, args.clone(), k))
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
                        .on_press(Message::Ui(UiMessage::CloseSetTag))
                )
                .width(iced::Fill)
                .align_x(iced::alignment::Horizontal::Right)
            ]
                .spacing(8)
                .width(iced::Shrink),
        )
    }
    fn modal_create_tag(&self, args: UiArgs) -> iced::Element<'_, Message> {
        CommonWidget::modal(
            widget::column![
                widget::text("新建标签🏷")
                    .size(48)
                    .style(widget::text::primary),
                widget::text_input("请输入标签名称", &self.new_tag).on_input(|x| Message::Ui(UiMessage::WhenNewTagType(x))),
                widget::container(
                    widget::row![
                        widget::button("添加").on_press_maybe(Self::check_tag_add(self, args)),
                        widget::button("关闭")
                            .style(widget::button::danger)
                            .on_press(Message::Ui(UiMessage::CloseNewTagMsg))
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
    fn modal_music_open_fault<'a>() -> iced::Element<'a, Message> {
        CommonWidget::modal(
            widget::column![
                widget::text("音频文件错误")
                    .size(48)
                    .style(widget::text::danger),
                widget::container(widget::button("关闭").on_press(Message::Ui(UiMessage::CloseMusicOpenFailureMsg)))
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
                                    widget::button("移除该标签").style(widget::button::danger).on_press(Message::Ui(UiMessage::RemoveMusicFromTag(title.clone()))),
                                _ => widget::button("").width(0),
                            },
                            widget::button("TAGS").on_press(Message::Ui(UiMessage::OpenSetTagMsg(title.clone()))),
                            widget::button("PLAY").on_press(Message::Ui(UiMessage::PlayMusic(p.clone())))
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

    fn main_view(&self, args: UiArgs) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in args.store.music_files.iter() {
            cards.push(Self::music_card(self, f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("MainView".to_string(), None, None, args.settings.show_side_bar),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "空空如也,像冬天的落叶一样", act),
        ].spacing(12))
    }
    fn tags_view(args: UiArgs<'_>) -> iced::Element<'_, Message> {
        let mut tags = vec![];
        for k in args.clone().store.tags.keys() {
            tags.push(Self::tags_card(args.clone(), k.to_string()))
        }
        let act = tags.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("TagsView".to_string(),
                Some(widget::button("+").on_press(Message::Ui(UiMessage::OpenNewTagMsg)).into()),
                None,
                args.settings.show_side_bar),
            Self::show_or_text(widget::grid(tags).fluid(200).spacing(8).into(), "什么都没有,来创建一个新的标签吧!", act),
        ].spacing(12)
        )
    }
    fn search_view(&self, args: UiArgs) -> iced::Element<'_, Message> {
        let mut show = vec![];
        for i in args.store.search_music_file(&self.search_text) {
            show.push(Self::music_card(self, i))
        }
        let act = self.search_text.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("SearchView".to_string(), None, None, args.settings.show_side_bar),
            widget::text_input("搜你所爱", &self.search_text)
                .on_input(|x| Message::Ui(UiMessage::WhenSearchTextType(x)))
                .width(iced::Fill),
            Self::show_or_text(widget::column(show).spacing(12).into(), "搜索从这里开始!", act),
            ].spacing(12)
        )
    }
    fn tag_view(&self, args: UiArgs, tag: String) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in args.store.tags.get(&tag).unwrap() {
            cards.push(Self::music_card(self, f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar(
                tag.clone(),
                Some(widget::button("🗑").style(widget::button::danger)
                    .on_press(Message::Ui(UiMessage::RemoveTag(tag))).into()),
                Some(Message::Ui(UiMessage::GoView(View::TagsView))),
                args.settings.show_side_bar),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "嗯...加点什么好呢?", act),
        ].spacing(12)
        )
    }
    fn settings_view<'a>(&'a self, args: UiArgs<'a>) -> iced::Element<'a, Message> {
        let mut so = vec![];
        for i in &args.store.search_origin {
            so.push(CommonWidget::path_show(i))
        }
        CommonWidget::view_builder(
            widget::column![
                CommonWidget::title_bar("SettingsView".to_string(), None, None, args.settings.show_side_bar),
                widget::scrollable(
                    widget::column(
                        [
                            CommonWidget::setting_group("常规".to_string(),
                                widget::column![
                                    CommonWidget::setting_card("保留播放状态".to_string(),
                                        widget::toggler(args.settings.keep_state)
                                            .on_toggle(|x| Message::Ui(UiMessage::WhenSettingChanged(SettingKeys::KeepPlayState(x)))).into()),
                                    CommonWidget::expand_content("搜索源".to_string(),
                                        widget::container(
                                            widget::column![
                                                widget::scrollable(
                                                    widget::column(so).spacing(12)
                                                ).height(200),
                                                widget::container(
                                                    widget::button("添加").on_press(Message::Ui(UiMessage::OnAddSearchOriginClicked))
                                                ).width(iced::Fill).align_x(iced::alignment::Horizontal::Center)
                                            ]
                                        ).into(),
                                        &self.show_exp_search_origin, Message::Ui(UiMessage::WhenExpSearchOriginClicked)),
                                ].spacing(12).into()
                            ),
                            CommonWidget::setting_group("外观".to_string(),
                                widget::column![
                                    CommonWidget::setting_card("侧边栏常驻".to_string(), widget::toggler(args.settings.show_side_bar)
                                        .on_toggle(|x| Message::Ui(UiMessage::WhenSettingChanged(SettingKeys::ShowSideBar(x)))).into())
                                ].spacing(12).into()
                            ),
                            CommonWidget::setting_group("关于".to_string(), widget::column![
                                widget::text("版本: v1.1.0").size(18),
                                widget::text("作者: 氢気氚").size(18)
                            ].into())
                        ]
                    ).spacing(32)
                )
            ]

        )
    }
    fn artists_view(args: UiArgs<'_>) -> iced::Element<'_, Message> {
        let mut artists = vec![];
        for k in args.store.artists.keys() {
            artists.push(Self::artists_card(args.clone(), k.to_string()))
        }
        let act = artists.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar("ArtistsView".to_string(),
                None,
                None,
                args.settings.show_side_bar),
            Self::show_or_text(widget::grid(artists).fluid(200).spacing(8).into(), "群星...", act),
        ].spacing(12)
        )
    }
    fn artist_view(&self, args: UiArgs, artist: String) -> iced::Element<'_, Message> {
        let mut cards = vec![];
        for f in args.store.artists.get(&artist).unwrap() {
            cards.push(Self::music_card(self, f.clone()))
        }
        let act = cards.is_empty();
        CommonWidget::view_builder(widget::column![
            CommonWidget::title_bar(
                artist.clone(),
                None,
                Some(Message::Ui(UiMessage::GoView(View::ArtistsView))),
                args.settings.show_side_bar),
            Self::show_or_text(widget::column(cards).spacing(12).into(), "嗯...为什么是空的呢,但你是不可能看到这个字的!😨", act),
        ].spacing(12)
        )
    }
    fn play_bar(args: UiArgs<'_>) -> iced::Element<'_, Message> {
        let sub1 = iced_aw::menu::Item::with_menu(
            widget::button(widget::text(args.player.playback_type.to_string()))
                .style(widget::button::text),
            Menu::new(
                [
                    iced_aw::menu::Item::new(
                        widget::button("单曲即停")
                            .style(widget::button::text)
                            .on_press(Message::Ui(UiMessage::OnPlaybackTypeChanged(PlaybackType::OnceStop))),
                    ),
                    iced_aw::menu::Item::new(
                        widget::button("单曲循环")
                            .style(widget::button::text)
                            .on_press(Message::Ui(UiMessage::OnPlaybackTypeChanged(PlaybackType::OneWhile))),
                    ),
                    iced_aw::menu::Item::new(
                        widget::button("下一曲")
                            .style(widget::button::text)
                            .on_press(Message::Ui(UiMessage::OnPlaybackTypeChanged(PlaybackType::MusicNext))),
                    ),
                    iced_aw::menu::Item::new(
                        widget::button("随机挑选")
                            .style(widget::button::text)
                            .on_press(Message::Ui(UiMessage::OnPlaybackTypeChanged(PlaybackType::RadomPlay))),
                    ),
                ]
                    .into(),
            )
                .width(iced::Length::Shrink),
        );
        let bar = iced_aw::MenuBar::new([sub1].into());
        let vols = iced_aw::menu::Item::with_menu(
            widget::button(widget::text(format!("音量:{}", ((args.settings.volume as f32) * 100f32) as i32)))
                .style(widget::button::text),
            Menu::new(
                [
                    iced_aw::menu::Item::new(
                        widget::vertical_slider(0..=100,
                                                ((args.settings.volume as f32) * 100f32) as i32,
                                                |x| Message::Ui(UiMessage::WhenSettingChanged(SettingKeys::Volume(x as f32 / 100f32 as Float))))
                            .step(1)
                            .height(200)
                    )
                ].into()
            ).padding(18).width(iced::Length::Shrink),
        );
        let vol = iced_aw::MenuBar::new([vols].into());
        widget::container(
            widget::column![
                widget::text(&args.player.music_name).size(48),
                widget::row![
                    widget::text(Self::format_time(
                        Duration::from_millis(args.player.value as u64).as_secs_f64()
                    ))
                    .size(28),
                    widget::button("⏮").on_press_maybe(args.player.now_playing.is_some()
                        .then(|| Some(Message::Ui(UiMessage::PrevMusic))).unwrap_or_else(|| None)),
                    widget::button(
                        (args.player.play_state == PlayState::Play)
                            .then(|| "⏸")
                            .unwrap_or_else(|| "▶️")
                    )
                    .on_press(Message::Ui(UiMessage::OnPSSwitchClicked)),
                    widget::button("️⏭").on_press_maybe(args.player.now_playing.is_some()
                        .then(|| Some(Message::Ui(UiMessage::NextMusic))).unwrap_or_else(|| None)),
                    bar,
                    vol,
                    widget::slider(
                        0f64..=args.player.total_music_time.as_millis() as f64,
                        args.player.value,
                        |x| Message::Ui(UiMessage::OnValueChanged(x))
                    )
                    .step(0.01)
                    .on_release(Message::Ui(UiMessage::OnReleaseSlider))
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
}

impl Ui {
    pub fn update(&mut self, msg: UiMessage, player: &mut Player, store: &mut MusicStore, settings: &mut Settings) {
        match msg {
            UiMessage::OnValueChanged(x) => {
                player.music_player.pause();
                player.play_state = PlayState::Stop;
                player.value = x;
                player.music_player
                    .try_seek(Duration::from_millis(x as u64))
                    .unwrap();
            }
            UiMessage::PlayMusic(mut p) => {
                match &self.view {
                    View::TagView(t) => player.play_list = PlayList::Tags(t.clone()),
                    View::MainView => player.play_list = PlayList::AllMusic,
                    View::TagsView => (),
                    View::SearchView => player.play_list = PlayList::AllMusic,
                    View::SettingsView => (),
                    View::ArtistsView => (),
                    View::ArtistView(a) => player.play_list = PlayList::Artist(a.clone())
                }
                if let Err(_) = player.play_music(&mut p, store) {
                    self.show_music_open_failure = true
                }
            }
            UiMessage::OnReleaseSlider => {
                if !self.force_stop {
                    player.music_player.play();
                    player.play_state = PlayState::Play;
                }
            }
            UiMessage::OnPSSwitchClicked => self.force_stop = player.ps_switch(),
            UiMessage::CloseMusicOpenFailureMsg => self.show_music_open_failure = false,
            UiMessage::OnPlaybackTypeChanged(p) => player.playback_type = p,
            UiMessage::SwitchSideBarShow => self.side_bar_show = !self.side_bar_show,
            UiMessage::GoView(v) => {
                self.view = v;
                self.side_bar_show = false;
            }
            UiMessage::OpenSetTagMsg(p) => {
                self.show_set_tag_modal = true;
                self.operate_files = Some(p)
            }
            UiMessage::AddTagTo(t, p) => {
                store.add_tag_for_music(t, p);
                self.show_set_tag_modal = false;
            }
            UiMessage::CloseSetTag => self.show_set_tag_modal = false,
            UiMessage::WhenNewTagType(t) => self.new_tag = t,
            UiMessage::OpenNewTagMsg => self.show_new_tag_modal = true,
            UiMessage::CloseNewTagMsg => {
                self.show_new_tag_modal = false;
                self.new_tag.clear();
            }
            UiMessage::AddTag => {
                store.tags.insert(self.new_tag.clone(), vec![]);
                self.show_new_tag_modal = false;
                self.new_tag.clear();
            }
            UiMessage::PrevMusic => {
                if let Err(_) = player.music_play_push(-1, store) {
                    self.show_music_open_failure = true
                }
            },
            UiMessage::NextMusic => {
                if let Err(_) = player.music_play_push(1, store) {
                    self.show_music_open_failure = true
                }
            }
            UiMessage::RemoveMusicFromTag(p) => {
                if let View::TagView(t) = &self.view {
                    store.remove_music_from_tag(player, t, p)
                }
            }
            UiMessage::RemoveTag(t) => {
                self.view = View::TagsView;
                store.remove_tag(player, t);
            }
            UiMessage::WhenSearchTextType(s) => {
                self.search_text = s
            }
            UiMessage::WhenExpSearchOriginClicked => self.show_exp_search_origin = !self.show_exp_search_origin,
            UiMessage::WhenSettingChanged(t) => {
                settings.set_setting(t.clone());
                if let SettingKeys::Volume(f) = t {
                    player.set_volume(f);
                }
            }
            UiMessage::DeleteOriginPath(p) => {
                if let Some(val) = &player.now_playing {
                    if let Some(par) = val.music.parent() {
                        if par == p.as_path() {
                            player.player_default();
                        }
                    }
                }
                store.remove_search_origin(p);
                store.sync();
                store.idx = 0;
                store.is_read = true
            }
            UiMessage::OnAddSearchOriginClicked => {
                let path = rfd::FileDialog::new().pick_folder();
                if let Some(pat) = path {
                    let idx = store.search_origin.iter().position(|x| *x == pat);
                    if let None = idx {
                        store.search_origin.push(pat);
                        store.sync_only_push();
                    }
                    store.idx = 0;
                    store.is_read = true
                }
            }
        }
    }
}