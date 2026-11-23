//! Valle delle Lucciole: un gioco di fattoria 2.5D in pixel art.
//! Tutta la grafica è generata via codice (vedi sprites.rs e world.rs::bake).

mod art;
mod canvas;
mod font;
mod game;
mod items;
mod render;
mod sprites;
mod ui;
mod util;
mod world;

use macroquad::prelude::*;

fn conf() -> Conf {
    Conf {
        window_title: "Valle delle Lucciole".to_owned(),
        window_width: 1440,
        window_height: 810,
        high_dpi: false,
        window_resizable: true,
        ..Default::default()
    }
}

async fn run() {
    let mut g = game::Game::new();
    // Modalità di prova per catturare schermate: VALLE_SHOT=percorso[,frame,ora]
    let shot = std::env::var("VALLE_SHOT").ok();
    let mut frame = 0u32;
    loop {
        let dt = get_frame_time().min(1.0 / 20.0);
        if let Some(s) = &shot {
            let parts: Vec<&str> = s.split(',').collect();
            if frame == 1 && parts.len() > 2 {
                g.debug_setup(parts[2]);
            }
        }
        g.update(dt);
        g.draw();
        if let Some(s) = &shot {
            let parts: Vec<&str> = s.split(',').collect();
            let at: u32 = parts.get(1).and_then(|v| v.parse().ok()).unwrap_or(30);
            if frame == at {
                get_screen_data().export_png(parts[0]);
                break;
            }
        }
        frame += 1;
        next_frame().await;
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--export-art") {
        art::export_all("assets/sprites");
        println!("Sprite esportati in assets/sprites");
        return;
    }
    macroquad::Window::from_config(conf(), run());
}
