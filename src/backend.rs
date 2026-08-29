use crate::state::BackendMessage;
use crate::app::RIMusic;

pub fn update(msg: BackendMessage, app: &mut RIMusic) {
    match msg {
        BackendMessage::Sync => {
            if let Err(_) = app.player.sync(&mut app.store) {
                app.ui.show_music_open_failure = true;
            }
        }
        BackendMessage::CloseSave => {
            let export = app.settings.save(&app.store, &app.player);
            let _ = confy::store("RIMusic", None, export);
            app.exit = true
        }
        BackendMessage::ReadFileData => {
            app.store.read_one();
        }
    }
}