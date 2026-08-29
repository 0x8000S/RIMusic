#![windows_subsystem = "windows"]

mod ui;
mod state;
mod backend;
mod app;
mod settings;
mod player;
mod store;
mod music_file;

use iced::Color;
use std::sync::Arc;
use iced::theme::{Custom, Palette};

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

fn main() -> iced::Result {
    let m = iced::Theme::Custom(Arc::new(Custom::new("Caption".to_string(), cap())));
    iced::application(app::RIMusic::default, app::RIMusic::update, app::RIMusic::view)
        .subscription(app::RIMusic::subscription)
        .theme(m)
        .exit_on_close_request(false)
        .run()
}
