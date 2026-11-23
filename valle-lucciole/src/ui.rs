//! Interfaccia: barra degli oggetti, orologio, energia, negozio, dialoghi, titolo.

use crate::game::*;
use crate::items::*;
use macroquad::prelude::*;

const INK: Color = Color::new(0.29, 0.16, 0.09, 1.0);
const SHADOW: Color = Color::new(0.08, 0.05, 0.04, 0.85);

fn hexc(v: u32) -> Color {
    Color::from_rgba((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}

pub fn panel(x: f32, y: f32, w: f32, h: f32) {
    let (x, y) = (x.round(), y.round());
    draw_rectangle(x, y, w, h, hexc(0x2a1a16));
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, hexc(0x8a4a22));
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, h - 4.0, hexc(0xd88a44));
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, 1.0, hexc(0xf0b060));
    draw_rectangle(x + 3.0, y + 3.0, w - 6.0, h - 6.0, hexc(0x9a5a2a));
    draw_rectangle(x + 4.0, y + 4.0, w - 8.0, h - 8.0, hexc(0xf6dcaa));
    // borchie agli angoli
    for (cx, cy) in [(x + 1.0, y + 1.0), (x + w - 3.0, y + 1.0), (x + 1.0, y + h - 3.0), (x + w - 3.0, y + h - 3.0)] {
        draw_rectangle(cx, cy, 2.0, 2.0, hexc(0xf4cc6a));
    }
}

fn slot(x: f32, y: f32, sel: bool) {
    draw_rectangle(x, y, 18.0, 18.0, hexc(0xb8763e));
    draw_rectangle(x + 1.0, y + 1.0, 17.0, 17.0, hexc(0xf0c88a));
    draw_rectangle(x + 1.0, y + 1.0, 16.0, 16.0, hexc(0xe4ae6c));
    if sel {
        draw_rectangle_lines(x - 1.0, y - 1.0, 20.0, 20.0, 2.0, hexc(0xd8342a));
    }
}

fn wrap(s: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for w in s.split(' ') {
        if cur.chars().count() + w.chars().count() + 1 > max_chars && !cur.is_empty() {
            lines.push(cur);
            cur = String::new();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

impl Game {
    fn mouse_ui(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        vec2(mx / self.scale, my / self.scale)
    }

    fn hotbar_origin(&self) -> Vec2 {
        let w = SLOTS as f32 * 20.0 + 6.0;
        vec2(((self.vw - w) / 2.0).floor(), self.vh - 29.0)
    }

    pub fn hotbar_hit(&self) -> Option<usize> {
        let o = self.hotbar_origin();
        let m = self.mouse_ui();
        (0..SLOTS).find(|&i| Rect::new(o.x + 4.0 + i as f32 * 20.0, o.y + 4.0, 18.0, 18.0).contains(m))
    }

    fn shop_rect(&self) -> Rect {
        let w = 262.0;
        let h = 52.0 + self.shop_items().len() as f32 * 25.0 + 18.0;
        Rect::new(((self.vw - w) / 2.0).floor(), ((self.vh - h) / 2.0 - 10.0).floor(), w, h)
    }

    pub fn shop_hit(&self) -> Option<usize> {
        let r = self.shop_rect();
        let m = self.mouse_ui();
        (0..self.shop_items().len()).find(|&i| Rect::new(r.x + 8.0, r.y + 40.0 + i as f32 * 25.0, r.w - 16.0, 23.0).contains(m))
    }

    fn sleep_rect(&self) -> Rect {
        Rect::new(((self.vw - 200.0) / 2.0).floor(), ((self.vh - 64.0) / 2.0).floor(), 200.0, 64.0)
    }

    pub fn sleep_hit(&self) -> Option<bool> {
        let r = self.sleep_rect();
        let m = self.mouse_ui();
        if Rect::new(r.x + 40.0, r.y + 36.0, 50.0, 18.0).contains(m) {
            Some(true)
        } else if Rect::new(r.x + 110.0, r.y + 36.0, 50.0, 18.0).contains(m) {
            Some(false)
        } else {
            None
        }
    }

    pub fn draw_ui(&self) {
        let f = &self.font;
        if self.mode == Mode::Title {
            self.draw_title();
            return;
        }
        // --- barra oggetti
        let o = self.hotbar_origin();
        panel(o.x, o.y, SLOTS as f32 * 20.0 + 6.0, 26.0);
        for i in 0..SLOTS {
            let x = o.x + 4.0 + i as f32 * 20.0;
            let y = o.y + 4.0;
            slot(x, y, i == self.sel);
            if let Some((it, n)) = self.inv[i] {
                draw_texture(self.art.icon(it), x + 1.0, y + 1.0, WHITE);
                if !it.is_tool() && n > 1 {
                    let s = n.to_string();
                    let w = f.width(&s, 1.0);
                    f.draw_shadow(&s, x + 17.0 - w, y + 10.0, 1.0, WHITE, SHADOW);
                }
                if it == Item::Can {
                    let k = self.water as f32 / MAX_WATER as f32;
                    draw_rectangle(x + 2.0, y + 15.0, 14.0, 2.0, hexc(0x3a2a20));
                    draw_rectangle(x + 2.0, y + 15.0, (14.0 * k).round(), 2.0, hexc(0x5ab0f0));
                }
            }
            if self.vw > 300.0 {
                let label = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="][i];
                f.draw(label, x + 1.0, y - 9.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.0));
            }
        }
        // nome dell'oggetto in mano
        if self.held_t > 0.0 {
            if let Some(it) = self.held() {
                let a = self.held_t.min(1.0);
                f.draw_center(&it.name(), self.vw / 2.0, o.y - 11.0, 1.0, Color::new(1.0, 1.0, 1.0, a), Color::new(0.08, 0.05, 0.04, a * 0.85));
            }
        }

        // --- orologio
        let cw = 86.0;
        let cx = self.vw - cw - 4.0;
        panel(cx, 4.0, cw, 50.0);
        let wd = DAYS[((self.day - 1) % 7) as usize];
        f.draw(&format!("{} {}", wd, self.day), cx + 8.0, 9.0, 1.0, INK);
        let icon = if self.raining { &self.art.rain } else { &self.art.sun };
        draw_texture(icon, cx + cw - 19.0, 7.0, WHITE);
        let h = ((self.time / 60.0) as i32) % 24;
        let m = ((self.time % 60.0) / 10.0) as i32 * 10;
        f.draw(&format!("{}:{:02}", h, m), cx + 8.0, 21.0, 1.0, INK);
        // barra del giorno
        let k = ((self.time - DAY_START) / (DAY_END - DAY_START)).clamp(0.0, 1.0);
        draw_rectangle(cx + 40.0, 23.0, 38.0, 3.0, hexc(0xc89a64));
        draw_rectangle(cx + 40.0, 23.0, (38.0 * k).round(), 3.0, if k > 0.8 { hexc(0x5a4aa0) } else { hexc(0xe8a030) });
        draw_rectangle(cx + 6.0, 31.0, cw - 12.0, 1.0, hexc(0xd8b480));
        draw_texture(&self.art.coin, cx + 8.0, 36.0, WHITE);
        f.draw(&format!("{}", self.money), cx + 20.0, 37.0, 1.0, INK);

        // --- energia
        let ex = self.vw - 17.0;
        let ey = self.vh - 84.0;
        panel(ex - 3.0, ey - 12.0, 16.0, 82.0);
        f.draw("E", ex + 2.0, ey - 8.0, 1.0, INK);
        draw_rectangle(ex + 1.0, ey, 8.0, 64.0, hexc(0x3a2a20));
        let k = (self.energy / MAX_ENERGY).clamp(0.0, 1.0);
        let col = if k > 0.5 {
            hexc(0x5ac84a)
        } else if k > 0.2 {
            hexc(0xe8c030)
        } else {
            hexc(0xe0402a)
        };
        let hgt = (62.0 * k).round();
        draw_rectangle(ex + 2.0, ey + 63.0 - hgt, 6.0, hgt, col);
        draw_rectangle(ex + 2.0, ey + 63.0 - hgt, 2.0, hgt, Color::new(1.0, 1.0, 1.0, 0.3));

        // --- messaggio
        if let Some((s, t)) = &self.toast {
            let a = t.min(0.5) * 2.0;
            let maxw = (self.vw - 120.0).min(300.0);
            let lines = wrap(s, ((maxw - 16.0) / 6.0) as usize);
            let w = lines.iter().map(|l| f.width(l, 1.0)).fold(0.0, f32::max) + 16.0;
            let h = lines.len() as f32 * 10.0 + 12.0;
            let x = ((self.vw - w) / 2.0).floor();
            let y = 8.0 - (1.0 - a) * 30.0;
            panel(x, y, w, h);
            for (i, l) in lines.iter().enumerate() {
                f.draw(l, x + 8.0, y + 7.0 + i as f32 * 10.0, 1.0, INK);
            }
        }

        match self.mode {
            Mode::Shop => self.draw_shop(),
            Mode::Sleep => {
                let r = self.sleep_rect();
                panel(r.x, r.y, r.w, r.h);
                f.draw_center("Andare a dormire?", r.x + r.w / 2.0, r.y + 12.0, 1.0, INK, Color::new(0.0, 0.0, 0.0, 0.0));
                f.draw_center("La giornata finirà qui.", r.x + r.w / 2.0, r.y + 23.0, 1.0, hexc(0x8a5a3a), Color::new(0.0, 0.0, 0.0, 0.0));
                for (i, (label, yes)) in [("Sì", true), ("No", false)].iter().enumerate() {
                    let bx = r.x + 40.0 + i as f32 * 70.0;
                    let by = r.y + 38.0;
                    let on = self.sleep_yes == *yes;
                    draw_rectangle(bx, by, 50.0, 16.0, if on { hexc(0xd8342a) } else { hexc(0xb8763e) });
                    draw_rectangle(bx + 1.0, by + 1.0, 48.0, 14.0, if on { hexc(0xf8e0a8) } else { hexc(0xe4ae6c) });
                    f.draw_center(label, bx + 25.0, by + 5.0, 1.0, INK, Color::new(0.0, 0.0, 0.0, 0.0));
                }
            }
            Mode::Help => self.draw_help(),
            Mode::Night => self.draw_night(),
            _ => {}
        }
    }

    fn draw_shop(&self) {
        let f = &self.font;
        let r = self.shop_rect();
        draw_rectangle(0.0, 0.0, self.vw, self.vh, Color::new(0.05, 0.03, 0.08, 0.45));
        panel(r.x, r.y, r.w, r.h);
        // ritratto della mercante
        draw_rectangle(r.x + 8.0, r.y + 8.0, 26.0, 26.0, hexc(0xb8763e));
        draw_rectangle(r.x + 9.0, r.y + 9.0, 24.0, 24.0, hexc(0xa8d0e8));
        draw_texture_ex(
            &self.art.merchant.walk[0][0],
            r.x + 9.0,
            r.y + 5.0,
            WHITE,
            DrawTextureParams { source: Some(Rect::new(0.0, 0.0, 16.0, 19.0)), dest_size: Some(vec2(24.0, 28.5)), ..Default::default() },
        );
        draw_rectangle(r.x + 9.0, r.y + 33.0, 24.0, 3.0, hexc(0xf6dcaa));
        f.draw("Bancarella di Mira", r.x + 40.0, r.y + 10.0, 1.0, INK);
        f.draw("Semi freschi, ogni giorno!", r.x + 40.0, r.y + 21.0, 1.0, hexc(0x8a5a3a));
        for (i, it) in self.shop_items().iter().enumerate() {
            let y = r.y + 40.0 + i as f32 * 25.0;
            let on = i == self.shop_sel;
            draw_rectangle(r.x + 8.0, y, r.w - 16.0, 23.0, if on { hexc(0xfff0c8) } else { hexc(0xefcf98) });
            if on {
                draw_rectangle_lines(r.x + 8.0, y, r.w - 16.0, 23.0, 1.0, hexc(0xd8342a));
            }
            draw_texture(self.art.icon(*it), r.x + 11.0, y + 4.0, WHITE);
            f.draw(&it.name(), r.x + 32.0, y + 4.0, 1.0, INK);
            if let Item::Seed(k) = it {
                f.draw(&format!("{} giorni", k.days()), r.x + 32.0, y + 14.0, 1.0, hexc(0x9a6a44));
                let p = k.seed_price().to_string();
                let w = f.width(&p, 1.0);
                f.draw(&p, r.x + r.w - 14.0 - w, y + 8.0, 1.0, INK);
                draw_texture(&self.art.coin, r.x + r.w - 26.0 - w, y + 7.0, WHITE);
            }
        }
        let by = r.y + r.h - 16.0;
        draw_texture(&self.art.coin, r.x + 10.0, by - 1.0, WHITE);
        f.draw(&format!("{}", self.money), r.x + 22.0, by, 1.0, INK);
        f.draw("Clic: compra  Shift: ×5  Esc: esci", r.x + 70.0, by, 1.0, hexc(0x9a6a44));
    }

    fn draw_help(&self) {
        let f = &self.font;
        let lines = [
            ("WASD / Frecce", "Muoviti (Shift per correre)"),
            ("Clic sinistro / C", "Usa l'attrezzo o pianta"),
            ("Clic destro / E", "Raccogli, parla, apri, mangia"),
            ("1-0, rotella, Q/Tab", "Cambia oggetto"),
            ("Cassa", "Spedisci: pagato la notte"),
            ("Casa", "Dormi e salva la partita"),
            ("H", "Mostra / nascondi i comandi"),
        ];
        let w = 312.0;
        let h = 30.0 + lines.len() as f32 * 12.0 + 14.0;
        let x = ((self.vw - w) / 2.0).floor();
        let y = ((self.vh - h) / 2.0).floor();
        draw_rectangle(0.0, 0.0, self.vw, self.vh, Color::new(0.05, 0.03, 0.08, 0.45));
        panel(x, y, w, h);
        f.draw_center("Comandi", x + w / 2.0, y + 9.0, 1.0, INK, Color::new(0.0, 0.0, 0.0, 0.0));
        for (i, (k, v)) in lines.iter().enumerate() {
            let yy = y + 26.0 + i as f32 * 12.0;
            f.draw(k, x + 10.0, yy, 1.0, hexc(0xb83a2a));
            f.draw(v, x + 136.0, yy, 1.0, INK);
        }
    }

    fn draw_night(&self) {
        let f = &self.font;
        let t = self.night_t;
        let a = if t < 1.0 {
            t
        } else if t > 4.5 {
            1.0 - (t - 4.5)
        } else {
            1.0
        };
        draw_rectangle(0.0, 0.0, self.vw, self.vh, Color::new(0.03, 0.03, 0.07, a.clamp(0.0, 1.0)));
        if self.night_done && t < 5.0 {
            let ta = ((t - 1.0) * 2.0).clamp(0.0, 1.0) * a;
            let cx = self.vw / 2.0;
            let mut y = self.vh / 2.0 - 40.0;
            let wd = DAYS[((self.day - 1) % 7) as usize];
            f.draw_center(&format!("Giorno {} - {}", self.day, wd), cx, y, 2.0, Color::new(1.0, 0.9, 0.6, ta), Color::new(0.0, 0.0, 0.0, ta));
            y += 26.0;
            for l in &self.summary {
                f.draw_center(l, cx, y, 1.0, Color::new(0.9, 0.9, 1.0, ta), Color::new(0.0, 0.0, 0.0, ta));
                y += 11.0;
            }
            // lucciole decorative
            for i in 0..12 {
                let k = i as f32;
                let x = cx + (self.t * 0.4 + k * 2.1).sin() * 150.0;
                let yy = self.vh / 2.0 + (self.t * 0.6 + k * 1.7).cos() * 80.0;
                let al = ((self.t * 2.0 + k).sin() * 0.5 + 0.5) * ta;
                draw_rectangle(x.round(), yy.round(), 1.0, 1.0, Color::new(0.9, 1.0, 0.5, al));
            }
        }
    }

    fn draw_title(&self) {
        let f = &self.font;
        let cx = self.vw / 2.0;
        draw_rectangle(0.0, 0.0, self.vw, self.vh, Color::new(0.05, 0.04, 0.1, 0.25));
        let y = (self.vh * 0.2).round();
        let title1 = "Valle delle";
        let title2 = "Lucciole";
        // contorno spesso
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0)] {
            let c = hexc(0x2a1a16);
            let w1 = f.width(title1, 2.0);
            f.draw(title1, (cx - w1 / 2.0).round() + dx * 2.0, y + dy * 2.0, 2.0, c);
            let w2 = f.width(title2, 4.0);
            f.draw(title2, (cx - w2 / 2.0).round() + dx * 2.0, y + 20.0 + dy * 2.0 + 4.0, 4.0, c);
        }
        let w1 = f.width(title1, 2.0);
        f.draw(title1, (cx - w1 / 2.0).round(), y, 2.0, hexc(0xf6dcaa));
        let w2 = f.width(title2, 4.0);
        f.draw(title2, (cx - w2 / 2.0).round(), y + 24.0, 4.0, hexc(0xf8d048));
        // lucciole intorno al titolo
        for i in 0..18 {
            let k = i as f32;
            let x = cx + (self.t * 0.5 + k * 1.9).sin() * 120.0;
            let yy = y + 30.0 + (self.t * 0.7 + k * 2.3).cos() * 34.0;
            let al = (self.t * 2.4 + k).sin() * 0.5 + 0.5;
            draw_rectangle(x.round() - 1.0, yy.round(), 3.0, 1.0, Color::new(0.8, 1.0, 0.4, al * 0.4));
            draw_rectangle(x.round(), yy.round(), 1.0, 1.0, Color::new(0.95, 1.0, 0.6, al));
        }
        f.draw_center("Un piccolo gioco di fattoria in pixel art", cx, y + 64.0, 1.0, WHITE, SHADOW);
        let blink = (self.t * 2.5).sin() > -0.3;
        let by = (self.vh * 0.72).round();
        if let Some(d) = self.has_save {
            if blink {
                f.draw_center(&format!("Invio: continua (giorno {})", d), cx, by, 1.0, hexc(0xfff0c0), SHADOW);
            }
            f.draw_center("N: nuova partita", cx, by + 12.0, 1.0, WHITE, SHADOW);
        } else if blink {
            f.draw_center("Premi Invio o clicca per iniziare", cx, by, 1.0, hexc(0xfff0c0), SHADOW);
        }
        f.draw_center("WASD per muoverti - H per i comandi", cx, self.vh - 14.0, 1.0, Color::new(1.0, 1.0, 1.0, 0.7), SHADOW);
    }
}
