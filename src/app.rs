use std::time::Duration;
use crate::settings::{Settings};
use crate::state::{PlayState, UiArgs};
use crate::state::{Message, BackendMessage};
use crate::player::Player;
use crate::ui::Ui;
use crate::store::MusicStore;
use crate::backend;

pub struct RIMusic {
    pub player: Player,
    pub store: MusicStore,
    pub settings: Settings,
    pub exit: bool,
    run_once: bool,
    pub ui: Ui
}

impl Default for RIMusic {
    fn default() -> Self {
        let settings = confy::load("RIMusic", None).unwrap_or_else(|_|Settings::default());
        RIMusic {
            store: MusicStore::new(),
            player: Player::new(),
            settings,
            exit: false,
            run_once: true,
            ui: Ui::default()
        }
    }
}

// UI逻辑
impl RIMusic {
    pub fn view(&self) -> iced::Element<'_, Message> {
        self.ui.view(UiArgs {
            player: &self.player,
            store: &self.store,
            settings: &self.settings
        })
    }

}

// 工具方法
impl RIMusic {

    fn read_var(&mut self) {
        self.store.search_origin = self.settings.search_origin.clone();
        self.store.tags = self.settings.tags.clone();
        self.store.artists = self.settings.artist.clone();
        self.store.sync_only_push();
        self.player.set_volume(self.settings.volume);
    }
    fn read_play(&mut self) {
        if self.settings.keep_state {
            self.player.total_music_time = self.settings.last_music.is_some()
                .then(|| {
                    let lm = &mut self.settings.last_music.clone().unwrap();
                    lm.get_music_file_total_duration();
                    lm.duration.unwrap()
                })
                .unwrap_or_else(|| Duration::from_secs(0));
            self.player.playback_type = self.settings.last_playback;
            self.player.play_list = self.settings.last_playlist.clone();
            if self.settings.last_music.is_some() {
                let _ = self.player.play_music(&mut self.settings.last_music.clone().unwrap(), &mut self.store);
                self.player.music_player.pause();
                self.player.set_pos(self.settings.last_position);
                self.player.play_state = PlayState::Stop;
            }
        }
    }
}

// 执行逻辑
impl RIMusic {
    pub fn update(&mut self, msg: Message) -> iced::Task<Message> {
        if self.run_once {
            self.read_var();
            self.read_play();
            self.run_once = false;
        }
        match msg {
            Message::Ui(m) => self.ui.update(m, &mut self.player, &mut self.store, &mut self.settings),
            Message::Backend(m) => backend::update(m, self)
        }
        if self.exit {
            return iced::exit()
        }
        iced::Task::none()
    }
    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch([
            iced::window::close_requests().map(|_| Message::Backend(BackendMessage::CloseSave)),
            (self.player.play_state == PlayState::Play)
                .then(|| iced::time::every(Duration::from_millis(50)).map(|_| Message::Backend(BackendMessage::Sync)))
                .unwrap_or_else(|| iced::Subscription::none()),
            self.store.is_read
                .then(|| iced::time::every(Duration::from_millis(100)).map(|_| Message::Backend(BackendMessage::ReadFileData)))
                .unwrap_or_else(|| iced::Subscription::none())

        ])
    }
}