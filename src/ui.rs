pub mod btn;
pub mod shape;
pub mod song;
pub mod text;

use crate::display::draw::*;
pub type Ctx = crate::display::draw::Ctx;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use song::Song;

pub struct Ui {
    hot_reload_tx: Sender<Vec<u8>>,

    open_song: Option<Song>,
    open_file_btn: btn::ButtonState,
    save_file_btn: btn::ButtonState,
    play_btn: btn::ButtonState,
}

impl Ui {
    pub fn new(hot_reload_tx: Sender<Vec<u8>>) -> Self {
        Ui {
            hot_reload_tx,
            open_song: None,
            open_file_btn: Default::default(),
            save_file_btn: Default::default(),
            play_btn: Default::default(),
        }
    }

    pub fn window_title(&self) -> String {
        match &self.open_song {
            None => "Mamar".to_string(),
            Some(song) => format!("{} - Mamar", song.file_name()),
        }
    }

    pub fn draw(&mut self, ctx: &mut Ctx, delta: f32) {
        shape::rect(ctx, {
            let size = ctx.display_size();
            rect(0.0, 0.0, size.width, size.height)
        }, color::BACKGROUND).draw(ctx);

        ctx.set_window_title(&self.window_title());

        let btn = btn::primary(
            ctx,
            delta,
            rect(0.0, 0.0, 96.0, 32.0),
            "Open File...",
            &mut self.open_file_btn,
        );
        btn.draw(ctx);
        if btn.is_click(ctx, MouseButton::Left) {
            // We use ctx.spawn here to defer opening of the file dialog until after drawing is complete.
            ctx.spawn(|| {
                // Note: we cannot open_file_dialog in this thread; it must be done on the main thread (macOS)
                // This is why we do everything in the below callback.

                move |ui: &mut Self| {
                    log::debug!("showing file open dialog");
                    let f =
                        tinyfiledialogs::open_file_dialog("Open File", "", Some((&["*.bgm", "*.bin", "*.mid", "*.midi"], "BGM files")));

                    if let Some(f) = f {
                        log::info!("loading song: {}", f);

                        match Song::open(PathBuf::from(f)) {
                            Ok(song) => ui.open_song = Some(song),
                            Err(error) => {
                                let msg = format!("{}", error);
                                log::error!("{}", msg);
                                tinyfiledialogs::message_box_ok(
                                    "Error opening file",
                                    &msg,
                                    tinyfiledialogs::MessageBoxIcon::Error,
                                );
                            }
                        }
                    } else {
                        log::debug!("user cancelled file open operation");
                    }
                }
            });
        }

        if let Some(song) = &mut self.open_song {
            song.draw(ctx, delta);

            let btn = btn::primary(ctx, delta, rect(100.0, 0.0, 64.0, 32.0), "Play", &mut self.play_btn);
            btn.draw(ctx);
            if btn.is_click(ctx, MouseButton::Left) {
                let data = song.bgm.as_bytes().unwrap(); // TODO handle error
                self.hot_reload_tx.send(data).unwrap();
            }

            let btn = btn::primary(ctx, delta, rect(168.0, 0.0, 96.0, 32.0), "Save As...", &mut self.save_file_btn);
            btn.draw(ctx);
            if btn.is_click(ctx, MouseButton::Left) {
                let current_path = song.path.to_string_lossy().to_string();
                ctx.spawn(|| {
                    move |ui: &mut Self| {
                        use std::fs::File;

                        log::debug!("showing file save dialog");
                        let path =
                            tinyfiledialogs::save_file_dialog_with_filter("Save As", &current_path, &["*.bgm", "*.bin"], "");

                        if let Some(path) = path {
                            log::info!("saving bgm to {}", path);
                            let mut f = File::create(&path).unwrap();

                            let song = ui.open_song.as_mut().unwrap();

                            song.path = PathBuf::from(path);
                            song.bgm.encode(&mut f).unwrap();
                        } else {
                            log::debug!("user cancelled file save operation");
                        }
                    }
                });
            }
        }
    }
}
