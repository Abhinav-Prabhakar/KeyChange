//! KeyChange — a MIDI text instrument.
//!
//! One window, one screen. The engine owns the typing rules, the MIDI module
//! owns the wire, and the UI only draws.

mod engine;
mod midi;
mod ui;

use std::borrow::Cow;

use gpui::prelude::*;
use gpui::{
    point, px, size, App, Application, Bounds, Focusable, TitlebarOptions, WindowBounds,
    WindowOptions,
};

fn load_fonts(cx: &mut App) {
    let fonts: [&'static [u8]; 11] = [
        include_bytes!("../assets/fonts/InstrumentSerif0.ttf"),
        include_bytes!("../assets/fonts/InstrumentSerif1.ttf"),
        include_bytes!("../assets/fonts/JetBrainsMono-400.ttf"),
        include_bytes!("../assets/fonts/JetBrainsMono-600.ttf"),
        include_bytes!("../assets/fonts/JetBrainsMono-700.ttf"),
        include_bytes!("../assets/fonts/SpaceGrotesk-400.ttf"),
        include_bytes!("../assets/fonts/SpaceGrotesk-500.ttf"),
        include_bytes!("../assets/fonts/SpaceGrotesk-700.ttf"),
        include_bytes!("../assets/fonts/Unbounded-600.ttf"),
        include_bytes!("../assets/fonts/Unbounded-800.ttf"),
        include_bytes!("../assets/fonts/Unbounded-900.ttf"),
    ];
    let _ = cx
        .text_system()
        .add_fonts(fonts.iter().map(|f| Cow::Borrowed(*f)).collect());
}

fn main() {
    Application::new().run(|cx: &mut App| {
        load_fonts(cx);

        let bounds = Bounds::centered(None, size(px(1180.), px(900.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("KeyChange".into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(18.), px(20.))),
                }),
                window_min_size: Some(size(px(1040.), px(820.))),
                ..Default::default()
            },
            |window, cx| {
                let app = cx.new(ui::KeyChangeApp::new);
                let focus = app.read(cx).focus_handle(cx);
                window.focus(&focus);
                app
            },
        )
        .unwrap();
    });
}
