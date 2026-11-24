//! Stato di gioco, logica e rendering 2.5D (ordinamento per profondità, ombre, luci).

use crate::art::Art;
use crate::font::Font;
use crate::items::*;
use crate::util::*;
use crate::world::*;
use macroquad::prelude::*;
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};

pub const SLOTS: usize = 12;
pub const MAX_ENERGY: f32 = 200.0;
pub const MAX_WATER: i32 = 40;
pub const DAY_START: f32 = 6.0 * 60.0;
pub const DAY_END: f32 = 26.0 * 60.0;
/// minuti di gioco per secondo reale
pub const TIME_SPEED: f32 = 1.6;
const SAVE_PATH: &str = "salvataggio.txt";
pub const DAYS: [&str; 7] = ["LUN", "MAR", "MER", "GIO", "VEN", "SAB", "DOM"];

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Title,
    Play,
    Shop,
    Sleep,
    Night,
    Help,
}

pub struct Player {
    pub pos: Vec2,
    pub dir: u8, // 0 giù 1 su 2 sinistra 3 destra
    pub anim: f32,
    pub moving: bool,
    pub act_t: f32,
    pub act_total: f32,
    pub act_item: Option<Item>,
    pub act_target: (i32, i32),
    pub act_done: bool,
}

pub struct Drop {
    pub item: Item,
    pub pos: Vec2,
    pub vel: Vec2,
    pub z: f32,
    pub vz: f32,
    pub t: f32,
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub z: f32,
    pub vz: f32,
    pub grav: f32,
    pub life: f32,
    pub max: f32,
    pub col: Color,
    pub size: f32,
}

pub struct Chicken {
    pub pos: Vec2,
    pub target: Vec2,
    pub wait: f32,
    pub right: bool,
    pub peck: f32,
    pub anim: f32,
}

pub struct Flyer {
    pub pos: Vec2,
    pub base: Vec2,
    pub phase: f32,
    pub col: Color,
}

pub struct Falling {
    pub kind: Kind,
    pub var: u32,
    pub base: Vec2,
    pub t: f32,
    pub dir: f32,
}

pub struct Popup {
    pub text: String,
    pub pos: Vec2,
    pub t: f32,
    pub col: Color,
}

pub struct Game {
    pub art: Art,
    pub font: Font,
    pub world: World,
    pub ground_tex: Texture2D,
    /// visible ground type per pixel (to pick grass tufts or pebbles at object bases)
    pub ground_px: Vec<u8>,
    pub player: Player,
    pub inv: [Option<(Item, i32)>; SLOTS],
    pub sel: usize,
    pub money: i32,
    pub energy: f32,
    pub water: i32,
    pub day: i32,
    pub time: f32,
    pub raining: bool,
    pub shipped: Vec<(Item, i32)>,
    pub drops: Vec<Drop>,
    pub parts: Vec<Particle>,
    pub chickens: Vec<Chicken>,
    pub butterflies: Vec<Flyer>,
    pub fireflies: Vec<Flyer>,
    pub falling: Vec<Falling>,
    pub popups: Vec<Popup>,
    pub rain: Vec<Vec2>,
    pub mode: Mode,
    pub t: f32,
    pub cam: Vec2,
    pub rng: Rng,
    pub toast: Option<(String, f32)>,
    pub held_t: f32,
    pub shop_sel: usize,
    pub sleep_yes: bool,
    pub night_t: f32,
    pub night_done: bool,
    pub summary: Vec<String>,
    pub passed_out: bool,
    pub late_warned: bool,
    pub smoke_t: f32,
    pub mouse_last: Vec2,
    pub mouse_active: bool,
    pub has_save: Option<i32>,
    pub vw: f32,
    pub vh: f32,
    pub scale: f32,
    pub scene: RenderTarget,
    pub light: RenderTarget,
    /// silhouettes cast by the sun, composited once so overlaps don't double-darken
    pub shadows: RenderTarget,
    pub add_mat: Material,
    pub mul_mat: Material,
    pub tip_i: usize,
}

const VERT: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}"#;

const FRAG: &str = r#"#version 100
varying lowp vec4 color;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = color * texture2D(Texture, uv);
}"#;

fn blend_mat(b: BlendState) -> Material {
    load_material(
        ShaderSource::Glsl { vertex: VERT, fragment: FRAG },
        MaterialParams {
            pipeline_params: PipelineParams { color_blend: Some(b), alpha_blend: Some(b), ..Default::default() },
            ..Default::default()
        },
    )
    .unwrap()
}

fn rt(w: f32, h: f32) -> RenderTarget {
    let r = render_target(w as u32, h as u32);
    r.texture.set_filter(FilterMode::Nearest);
    r
}

pub fn c8(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

const TIPS: [&str; 5] = [
    "Benvenuto nella Valle delle Lucciole! Mira vende semi alla bancarella oltre il ponte.",
    "Le colture crescono solo nei giorni in cui il terreno è bagnato.",
    "Metti il raccolto nella cassa accanto a casa: verrà pagato durante la notte.",
    "Se ti senti stanco, mangia qualcosa (tasto destro con il cibo in mano).",
    "Di notte, vicino allo stagno, si accendono le lucciole.",
];

impl Game {
    pub fn new() -> Self {
        let art = Art::new();
        let font = Font::new();
        let world = World::generate(7);
        let (ground_canvas, ground_px) = world.bake();
        let ground_tex = ground_canvas.tex();
        let (vw, vh, scale) = Self::view_size();
        let mut g = Game {
            art,
            font,
            world,
            ground_tex,
            ground_px,
            player: Player {
                pos: vec2(41.0 * TS + 8.0, 19.0 * TS + 12.0),
                dir: 0,
                anim: 0.0,
                moving: false,
                act_t: 0.0,
                act_total: 0.3,
                act_item: None,
                act_target: (0, 0),
                act_done: true,
            },
            inv: [None; SLOTS],
            sel: 0,
            money: 300,
            energy: MAX_ENERGY,
            water: MAX_WATER,
            day: 1,
            time: DAY_START,
            raining: false,
            shipped: Vec::new(),
            drops: Vec::new(),
            parts: Vec::new(),
            chickens: Vec::new(),
            butterflies: Vec::new(),
            fireflies: Vec::new(),
            falling: Vec::new(),
            popups: Vec::new(),
            rain: Vec::new(),
            mode: Mode::Title,
            t: 0.0,
            cam: vec2(30.0 * TS, 14.0 * TS),
            rng: Rng::new(12345),
            toast: None,
            held_t: 0.0,
            shop_sel: 0,
            sleep_yes: true,
            night_t: 0.0,
            night_done: false,
            summary: Vec::new(),
            passed_out: false,
            late_warned: false,
            smoke_t: 0.0,
            mouse_last: Vec2::ZERO,
            mouse_active: false,
            has_save: None,
            vw,
            vh,
            scale,
            scene: rt(vw, vh),
            light: rt(vw, vh),
            shadows: rt(vw, vh),
            add_mat: blend_mat(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::SourceAlpha), BlendFactor::One)),
            mul_mat: blend_mat(BlendState::new(Equation::Add, BlendFactor::Value(BlendValue::DestinationColor), BlendFactor::Zero)),
            tip_i: 0,
        };
        g.new_game_inventory();
        g.spawn_life();
        g.has_save = g.peek_save();
        g
    }

    fn view_size() -> (f32, f32, f32) {
        let sw = screen_width().max(320.0);
        let sh = screen_height().max(200.0);
        let scale = (sh / 270.0).round().max(1.0);
        ((sw / scale).ceil(), (sh / scale).ceil(), scale)
    }

    fn new_game_inventory(&mut self) {
        self.inv = [None; SLOTS];
        self.inv[0] = Some((Item::Hoe, 1));
        self.inv[1] = Some((Item::Can, 1));
        self.inv[2] = Some((Item::Axe, 1));
        self.inv[3] = Some((Item::Pick, 1));
        self.inv[4] = Some((Item::Scythe, 1));
        self.inv[5] = Some((Item::Seed(Crop::Carota), 12));
    }

    fn spawn_life(&mut self) {
        let mut r = Rng::new(77);
        self.chickens.clear();
        for i in 0..4 {
            let p = vec2((34 + i * 4) as f32 * TS, 22.0 * TS);
            self.chickens.push(Chicken { pos: p, target: p, wait: r.rf(0.0, 2.0), right: i % 2 == 0, peck: 0.0, anim: 0.0 });
        }
        self.butterflies.clear();
        let cols = [c8(255, 255, 255, 255), c8(255, 214, 90, 255), c8(170, 210, 255, 255), c8(255, 170, 210, 255)];
        for i in 0..14 {
            let b = vec2(r.rf(6.0, 90.0) * TS, r.rf(15.0, 68.0) * TS);
            self.butterflies.push(Flyer { pos: b, base: b, phase: r.rf(0.0, 50.0), col: cols[i % 4] });
        }
        self.fireflies.clear();
        for i in 0..60 {
            let zone = i % 4;
            let b = match zone {
                0 => vec2(r.rf(3.0, 19.0), r.rf(33.0, 47.0)),
                1 => vec2(r.rf(4.0, 58.0), r.rf(3.0, 11.0)),
                2 => vec2(r.rf(46.0, 66.0), r.rf(55.0, 68.0)),
                _ => vec2(r.rf(18.0, 60.0), r.rf(22.0, 52.0)),
            } * TS;
            self.fireflies.push(Flyer { pos: b, base: b, phase: r.rf(0.0, 100.0), col: WHITE });
        }
    }

    // ------------------------------------------------------------ inventario

    pub fn add_item(&mut self, it: Item, n: i32) -> bool {
        if !it.is_tool() {
            for s in self.inv.iter_mut().flatten() {
                if s.0 == it {
                    s.1 += n;
                    return true;
                }
            }
        }
        for s in self.inv.iter_mut() {
            if s.is_none() {
                *s = Some((it, n));
                return true;
            }
        }
        false
    }

    fn take_selected(&mut self, n: i32) {
        if let Some(s) = &mut self.inv[self.sel] {
            s.1 -= n;
            if s.1 <= 0 {
                self.inv[self.sel] = None;
            }
        }
    }

    pub fn held(&self) -> Option<Item> {
        self.inv[self.sel].map(|s| s.0)
    }

    fn toast(&mut self, s: &str) {
        self.toast = Some((s.to_string(), 3.2));
    }

    fn popup(&mut self, s: String, pos: Vec2, col: Color) {
        self.popups.push(Popup { text: s, pos, t: 0.0, col });
    }

    // ------------------------------------------------------------ utilità

    pub fn player_tile(&self) -> (i32, i32) {
        ((self.player.pos.x / TS).floor() as i32, ((self.player.pos.y - 3.0) / TS).floor() as i32)
    }

    fn dir_vec(d: u8) -> (i32, i32) {
        match d {
            0 => (0, 1),
            1 => (0, -1),
            2 => (-1, 0),
            _ => (1, 0),
        }
    }

    pub fn mouse_world(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        vec2(mx / self.scale, my / self.scale) + self.cam
    }

    pub fn target_tile(&self) -> (i32, i32) {
        let (px, py) = self.player_tile();
        if self.mouse_active {
            let m = self.mouse_world();
            let (mx, my) = ((m.x / TS).floor() as i32, (m.y / TS).floor() as i32);
            if (mx - px).abs() <= 1 && (my - py).abs() <= 1 {
                return (mx, my);
            }
        }
        let (dx, dy) = Self::dir_vec(self.player.dir);
        (px + dx, py + dy)
    }

    fn face_toward(&mut self, tx: i32, ty: i32) {
        let (px, py) = self.player_tile();
        let (dx, dy) = (tx - px, ty - py);
        if dx == 0 && dy == 0 {
            return;
        }
        self.player.dir = if dx.abs() > dy.abs() {
            if dx < 0 {
                2
            } else {
                3
            }
        } else if dy < 0 {
            1
        } else {
            0
        };
    }

    fn box_free(&self, p: Vec2) -> bool {
        let hw = 4.5;
        let corners = [vec2(p.x - hw, p.y - 5.0), vec2(p.x + hw, p.y - 5.0), vec2(p.x - hw, p.y - 0.5), vec2(p.x + hw, p.y - 0.5)];
        corners.iter().all(|c| self.world.walkable((c.x / TS).floor() as i32, (c.y / TS).floor() as i32))
    }

    pub fn night(&self) -> f32 {
        let a = self.ambient();
        (1.0 - (a.r * 0.3 + a.g * 0.5 + a.b * 0.2)).clamp(0.0, 1.0) * 1.6
    }

    pub fn ambient(&self) -> Color {
        let h = self.time / 60.0;
        // ambient colour through the day: pink dawn, warm morning, neutral noon,
        // golden afternoon, orange sunset, blue night
        let keys: [(f32, [f32; 3]); 11] = [
            (5.0, [0.42, 0.44, 0.72]),
            (6.0, [0.9, 0.74, 0.8]),
            (7.3, [1.0, 0.93, 0.86]),
            (10.0, [1.0, 0.99, 0.96]),
            (14.5, [1.0, 1.0, 0.97]),
            (16.8, [1.0, 0.92, 0.78]),
            (18.2, [1.0, 0.78, 0.6]),
            (19.2, [0.84, 0.6, 0.62]),
            (20.3, [0.5, 0.47, 0.74]),
            (21.5, [0.3, 0.32, 0.56]),
            (27.0, [0.24, 0.26, 0.48]),
        ];
        let mut c = keys[keys.len() - 1].1;
        for w in keys.windows(2) {
            if h >= w[0].0 && h < w[1].0 {
                let t = (h - w[0].0) / (w[1].0 - w[0].0);
                for i in 0..3 {
                    c[i] = w[0].1[i] + (w[1].1[i] - w[0].1[i]) * t;
                }
            }
        }
        if self.raining {
            c[0] *= 0.72;
            c[1] *= 0.76;
            c[2] *= 0.86;
        }
        Color::new(c[0], c[1], c[2], 1.0)
    }

    fn burst(&mut self, pos: Vec2, col: Color, n: usize, spd: f32) {
        for _ in 0..n {
            let a = self.rng.rf(0.0, 6.283);
            let s = self.rng.rf(spd * 0.3, spd);
            self.parts.push(Particle {
                pos,
                vel: vec2(a.cos() * s, a.sin() * s * 0.5),
                z: self.rng.rf(2.0, 8.0),
                vz: self.rng.rf(20.0, 60.0),
                grav: 160.0,
                life: 0.0,
                max: self.rng.rf(0.4, 0.8),
                col,
                size: if self.rng.chance(0.3) { 2.0 } else { 1.0 },
            });
        }
    }

    fn spawn_drop(&mut self, it: Item, n: i32, pos: Vec2) {
        for _ in 0..n {
            let a = self.rng.rf(0.0, 6.283);
            let s = self.rng.rf(10.0, 30.0);
            self.drops.push(Drop { item: it, pos, vel: vec2(a.cos() * s, a.sin() * s * 0.6), z: 4.0, vz: self.rng.rf(50.0, 80.0), t: 0.0 });
        }
    }

    fn tile_center(x: i32, y: i32) -> Vec2 {
        vec2(x as f32 * TS + 8.0, y as f32 * TS + 8.0)
    }

    // ------------------------------------------------------------ update

    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        let (vw, vh, sc) = Self::view_size();
        if vw != self.vw || vh != self.vh {
            self.vw = vw;
            self.vh = vh;
            self.scene = rt(vw, vh);
            self.light = rt(vw, vh);
            self.shadows = rt(vw, vh);
        }
        self.scale = sc;
        let m = Vec2::from(mouse_position());
        if (m - self.mouse_last).length() > 0.5 {
            self.mouse_active = true;
        }
        self.mouse_last = m;

        match self.mode {
            Mode::Title => self.update_title(dt),
            Mode::Play => self.update_play(dt),
            Mode::Shop => self.update_shop(),
            Mode::Sleep => self.update_sleep(),
            Mode::Night => self.update_night(dt),
            Mode::Help => {
                if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) {
                    self.mode = Mode::Play;
                }
            }
        }
        if self.mode != Mode::Title && self.mode != Mode::Night {
            self.update_ambient(dt);
        } else if self.mode == Mode::Title {
            self.update_ambient(dt);
        }
        if let Some((_, t)) = &mut self.toast {
            *t -= dt;
            if *t <= 0.0 {
                self.toast = None;
            }
        }
        self.held_t = (self.held_t - dt).max(0.0);
    }

    fn update_title(&mut self, dt: f32) {
        // la camera vaga lentamente sopra la fattoria
        let t = self.t * 0.05;
        let target = vec2((38.0 + t.sin() * 10.0) * TS - self.vw / 2.0, (26.0 + (t * 0.7).cos() * 6.0) * TS - self.vh / 2.0);
        self.cam = self.cam.lerp(target, (dt * 0.6).min(1.0));
        self.time = 17.6 * 60.0 + (self.t * 0.6).sin() * 30.0;
        self.clamp_cam();
        let start = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left);
        if start {
            if self.has_save.is_some() {
                self.load();
            } else {
                self.time = DAY_START;
            }
            self.mode = Mode::Play;
            self.toast("Premi H per vedere i comandi.");
        } else if is_key_pressed(KeyCode::N) {
            self.reset_new();
            self.mode = Mode::Play;
            self.toast("Premi H per vedere i comandi.");
        }
    }

    fn reset_new(&mut self) {
        self.world = World::generate(7);
        let (gc, gp) = self.world.bake();
        self.ground_tex = gc.tex();
        self.ground_px = gp;
        self.new_game_inventory();
        self.money = 300;
        self.energy = MAX_ENERGY;
        self.water = MAX_WATER;
        self.day = 1;
        self.time = DAY_START;
        self.raining = false;
        self.shipped.clear();
        self.drops.clear();
        self.player.pos = vec2(41.0 * TS + 8.0, 19.0 * TS + 12.0);
        self.player.dir = 0;
    }

    fn update_play(&mut self, dt: f32) {
        // tempo
        self.time += dt * TIME_SPEED;
        if self.time >= 22.0 * 60.0 && !self.late_warned {
            self.late_warned = true;
            self.toast("Si sta facendo tardi... meglio tornare a casa.");
        }
        if self.time >= DAY_END {
            self.passed_out = true;
            self.toast("Sei crollato dalla stanchezza!");
            self.start_night();
            return;
        }
        // selezione slot
        let keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
            KeyCode::Key0,
            KeyCode::Minus,
            KeyCode::Equal,
        ];
        for (i, k) in keys.iter().enumerate() {
            if is_key_pressed(*k) {
                self.sel = i;
                self.held_t = 1.6;
            }
        }
        let (_, wy) = mouse_wheel();
        if wy.abs() > 0.1 {
            self.sel = if wy < 0.0 { (self.sel + 1) % SLOTS } else { (self.sel + SLOTS - 1) % SLOTS };
            self.held_t = 1.6;
        }
        if is_key_pressed(KeyCode::Q) {
            self.sel = (self.sel + SLOTS - 1) % SLOTS;
            self.held_t = 1.6;
        }
        if is_key_pressed(KeyCode::Tab) {
            self.sel = (self.sel + 1) % SLOTS;
            self.held_t = 1.6;
        }
        if is_key_pressed(KeyCode::H) {
            self.mode = Mode::Help;
            return;
        }
        // clic sulla barra: seleziona slot
        let mut clicked_ui = false;
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = self.hotbar_hit() {
                self.sel = i;
                self.held_t = 1.6;
                clicked_ui = true;
            }
        }

        // movimento
        let p = &mut self.player;
        let mut mv = Vec2::ZERO;
        if p.act_t <= 0.0 {
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                mv.x -= 1.0;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                mv.x += 1.0;
            }
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                mv.y -= 1.0;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                mv.y += 1.0;
            }
        }
        let moving = mv.length_squared() > 0.0;
        if moving {
            if mv.x.abs() > 0.0 && mv.y.abs() == 0.0 {
                p.dir = if mv.x < 0.0 { 2 } else { 3 };
            } else if mv.y != 0.0 && mv.x == 0.0 {
                p.dir = if mv.y < 0.0 { 1 } else { 0 };
            } else if !(p.dir == 2 && mv.x < 0.0 || p.dir == 3 && mv.x > 0.0) && !(p.dir == 1 && mv.y < 0.0 || p.dir == 0 && mv.y > 0.0) {
                p.dir = if mv.x < 0.0 { 2 } else { 3 };
            }
            self.mouse_active = false;
            let speed = if is_key_down(KeyCode::LeftShift) { 105.0 } else { 78.0 } * if self.energy <= 0.0 { 0.6 } else { 1.0 };
            let step = mv.normalize() * speed * dt;
            let pos = self.player.pos;
            let nx = vec2(pos.x + step.x, pos.y);
            if self.box_free(nx) {
                self.player.pos = nx;
            }
            let ny = vec2(self.player.pos.x, self.player.pos.y + step.y);
            if self.box_free(ny) {
                self.player.pos = ny;
            }
            self.player.anim += dt * 8.0;
        } else {
            self.player.anim = 0.0;
        }
        self.player.moving = moving;
        let maxp = vec2(W as f32 * TS - 6.0, H as f32 * TS - 1.0);
        self.player.pos = self.player.pos.clamp(vec2(6.0, 8.0), maxp);

        // azioni
        if self.player.act_t > 0.0 {
            self.player.act_t -= dt;
            if !self.player.act_done && self.player.act_t <= self.player.act_total * 0.45 {
                self.player.act_done = true;
                if let Some(it) = self.player.act_item {
                    let (tx, ty) = self.player.act_target;
                    self.apply_tool(it, tx, ty);
                }
            }
        } else {
            let use_btn = (is_mouse_button_pressed(MouseButton::Left) && !clicked_ui) || is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::Space);
            let int_btn = is_mouse_button_pressed(MouseButton::Right) || is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::X);
            if use_btn {
                self.use_held();
            } else if int_btn {
                self.interact();
            }
        }

        // camera
        let target = vec2(self.player.pos.x - self.vw / 2.0, self.player.pos.y - 14.0 - self.vh / 2.0);
        self.cam = self.cam.lerp(target, (dt * 10.0).min(1.0));
        self.clamp_cam();

        self.update_world_things(dt);
    }

    fn clamp_cam(&mut self) {
        let maxx = (W as f32 * TS - self.vw).max(0.0);
        let maxy = (H as f32 * TS - self.vh).max(0.0);
        self.cam.x = self.cam.x.clamp(0.0, maxx);
        self.cam.y = self.cam.y.clamp(0.0, maxy);
    }

    fn update_world_things(&mut self, dt: f32) {
        // oggetti
        for o in self.world.objs.iter_mut() {
            if o.shake > 0.0 {
                o.shake = (o.shake - dt).max(0.0);
            }
        }
        // alberi che cadono
        let mut done = Vec::new();
        for (i, f) in self.falling.iter_mut().enumerate() {
            f.t += dt;
            if f.t > 1.0 {
                done.push(i);
            }
        }
        for i in done.into_iter().rev() {
            let f = self.falling.remove(i);
            let land = f.base + vec2(f.dir * 30.0, -4.0);
            let n = self.rng.range(4, 8);
            self.spawn_drop(Item::Wood, n, land);
            let col = match f.kind {
                Kind::Cherry => c8(240, 168, 205, 255),
                Kind::Pine => c8(44, 116, 86, 255),
                _ => c8(88, 170, 64, 255),
            };
            self.burst(land, col, 30, 70.0);
            if f.kind == Kind::Cherry && self.rng.chance(0.5) {
                self.spawn_drop(Item::Fiber, 1, land);
            }
        }
        // drop: fisica + calamita verso il giocatore
        let pp = self.player.pos - vec2(0.0, 6.0);
        let mut picked: Vec<usize> = Vec::new();
        for (i, d) in self.drops.iter_mut().enumerate() {
            d.t += dt;
            if d.z > 0.0 || d.vz > 0.0 {
                d.vz -= 260.0 * dt;
                d.z += d.vz * dt;
                d.pos += d.vel * dt;
                if d.z <= 0.0 {
                    d.z = 0.0;
                    if d.vz.abs() > 25.0 {
                        d.vz = -d.vz * 0.35;
                        d.z = 0.01;
                    } else {
                        d.vz = 0.0;
                    }
                    d.vel *= 0.5;
                }
            }
            let dist = (pp - d.pos).length();
            if d.t > 0.45 && dist < 40.0 {
                let dirv = (pp - d.pos).normalize_or_zero();
                d.pos += dirv * (60.0 + (40.0 - dist) * 6.0) * dt;
                if dist < 7.0 {
                    picked.push(i);
                }
            }
        }
        for i in picked.into_iter().rev() {
            let it = self.drops[i].item;
            if self.add_item(it, 1) {
                self.drops.remove(i);
                // raggruppa i popup dello stesso oggetto
                let name = it.name();
                if let Some(p) = self.popups.iter_mut().find(|p| p.t < 0.6 && p.text.ends_with(&name)) {
                    let n: i32 = p.text.trim_start_matches('+').split(' ').next().and_then(|s| s.parse().ok()).unwrap_or(1);
                    p.text = format!("+{} {}", n + 1, name);
                    p.t = 0.0;
                } else {
                    let pos = self.player.pos - vec2(0.0, 34.0);
                    self.popup(format!("+1 {}", name), pos, c8(255, 255, 255, 255));
                }
            } else if self.toast.is_none() {
                self.toast("Lo zaino è pieno!");
            }
        }
        // galline
        let mut r = self.rng.clone();
        for ch in self.chickens.iter_mut() {
            ch.peck = (ch.peck - dt).max(0.0);
            if ch.wait > 0.0 {
                ch.wait -= dt;
                if ch.wait <= 0.0 {
                    let home = vec2(42.0 * TS, 22.0 * TS);
                    ch.target = home + vec2(r.rf(-8.0, 8.0) * TS, r.rf(-1.5, 3.5) * TS);
                }
                if r.chance(dt * 0.6) {
                    ch.peck = 0.5;
                }
            } else {
                let d = ch.target - ch.pos;
                if d.length() < 2.0 {
                    ch.wait = r.rf(1.0, 4.0);
                } else {
                    let step = d.normalize() * 16.0 * dt;
                    let np = ch.pos + step;
                    let (tx, ty) = ((np.x / TS).floor() as i32, ((np.y - 2.0) / TS).floor() as i32);
                    if self.world.walkable(tx, ty) {
                        ch.pos = np;
                        ch.right = step.x > 0.0;
                        ch.anim += dt * 6.0;
                    } else {
                        ch.wait = r.rf(0.5, 2.0);
                    }
                }
            }
        }
        self.rng = r;
        // farfalle e lucciole
        let tt = self.t;
        for b in self.butterflies.iter_mut() {
            let ph = b.phase;
            b.pos = b.base + vec2((tt * 0.37 + ph).sin() * 40.0 + (tt * 0.9 + ph).sin() * 10.0, (tt * 0.53 + ph * 1.3).cos() * 24.0);
        }
        for f in self.fireflies.iter_mut() {
            let ph = f.phase;
            f.pos = f.base + vec2((tt * 0.21 + ph).sin() * 30.0 + (tt * 1.3 + ph).sin() * 4.0, (tt * 0.27 + ph * 0.7).cos() * 22.0);
        }
        // popup
        for p in self.popups.iter_mut() {
            p.t += dt;
        }
        self.popups.retain(|p| p.t < 1.4);
    }

    /// Particelle, fumo, pioggia: animano anche il titolo.
    fn update_ambient(&mut self, dt: f32) {
        for p in self.parts.iter_mut() {
            p.life += dt;
            p.pos += p.vel * dt;
            p.vz -= p.grav * dt;
            p.z = (p.z + p.vz * dt).max(0.0);
            if p.z == 0.0 && p.grav > 0.0 {
                p.vel *= 0.8;
            }
        }
        self.parts.retain(|p| p.life < p.max);
        // fumo dal camino
        self.smoke_t -= dt;
        if self.smoke_t <= 0.0 {
            self.smoke_t = 0.35;
            if let (Some(h), Some((cx, cy))) = (self.world.find(Kind::House), self.art.house_info.chimney) {
                let top = vec2(h.x as f32 * TS, (h.y + 2) as f32 * TS - self.art.house_info.base_y as f32);
                let base = top + vec2(cx as f32, cy as f32 - 1.0);
                let g = 0.75 + self.rng.f() * 0.15;
                self.parts.push(Particle {
                    pos: base + vec2(self.rng.rf(-2.0, 2.0), 0.0),
                    vel: vec2(self.rng.rf(3.0, 9.0), -12.0),
                    z: 0.0,
                    vz: 0.0,
                    grav: 0.0,
                    life: 0.0,
                    max: 2.6,
                    col: Color::new(g, g, g + 0.03, 0.55),
                    size: 3.0,
                });
            }
        }
        // zampilli della fontana
        if let Some(f) = self.world.find(Kind::Fountain) {
            let top = vec2(f.x as f32 * TS + 24.0, (f.y + 2) as f32 * TS - 46.0 + 5.0);
            for _ in 0..2 {
                let a = self.rng.rf(-1.0, 1.0);
                self.parts.push(Particle {
                    pos: vec2(top.x + a, top.y + 34.0),
                    vel: vec2(a * 9.0, 0.0),
                    z: 34.0,
                    vz: self.rng.rf(12.0, 26.0),
                    grav: 60.0,
                    life: 0.0,
                    max: 1.4,
                    col: Color::new(0.78, 0.9, 1.0, 0.85),
                    size: 1.0,
                });
            }
        }
        // pioggia (coordinate schermo)
        if self.raining {
            while self.rain.len() < 220 {
                self.rain.push(vec2(self.rng.rf(0.0, self.vw + 60.0), self.rng.rf(-self.vh, self.vh)));
            }
            for d in self.rain.iter_mut() {
                d.x -= 70.0 * dt;
                d.y += 260.0 * dt;
            }
            let (vw, vh) = (self.vw, self.vh);
            let mut r = self.rng.clone();
            for d in self.rain.iter_mut() {
                if d.y > vh || d.x < -10.0 {
                    *d = vec2(r.rf(0.0, vw + 60.0), r.rf(-40.0, 0.0));
                }
            }
            self.rng = r;
        } else {
            self.rain.clear();
        }
    }

    // ------------------------------------------------------------ azioni del giocatore

    fn use_held(&mut self) {
        let Some(it) = self.held() else { return };
        let (tx, ty) = self.target_tile();
        if it.is_tool() {
            let cost = match it {
                Item::Can => 1.0,
                Item::Scythe => 0.5,
                _ => 2.0,
            };
            if self.energy <= -20.0 {
                self.toast("Sei esausto. Mangia qualcosa o vai a dormire.");
                return;
            }
            self.face_toward(tx, ty);
            self.energy -= cost;
            let p = &mut self.player;
            p.act_total = if it == Item::Scythe { 0.24 } else { 0.34 };
            p.act_t = p.act_total;
            p.act_item = Some(it);
            p.act_target = (tx, ty);
            p.act_done = false;
            return;
        }
        match it {
            Item::Seed(k) => {
                if inb(tx, ty) && self.world.tilled[idx(tx, ty)] && self.world.crops[idx(tx, ty)].is_none() && self.world.obj_at(tx, ty).is_none() {
                    self.world.crops[idx(tx, ty)] = Some(CropState { kind: k, growth: 0 });
                    self.take_selected(1);
                    self.face_toward(tx, ty);
                    self.burst(Self::tile_center(tx, ty), c8(170, 110, 60, 255), 6, 20.0);
                } else if self.toast.is_none() {
                    self.toast("I semi vanno piantati nel terreno zappato.");
                }
            }
            Item::Produce(_) => self.eat(),
            _ => {}
        }
    }

    fn eat(&mut self) {
        let Some(it) = self.held() else { return };
        let e = it.energy();
        if e <= 0 {
            self.toast("Questo non si mangia! Vendilo nella cassa.");
            return;
        }
        if self.energy >= MAX_ENERGY {
            self.toast("Sei già pieno di energie.");
            return;
        }
        self.energy = (self.energy + e as f32).min(MAX_ENERGY);
        self.take_selected(1);
        let pos = self.player.pos - vec2(0.0, 36.0);
        self.popup(format!("+{} energia", e), pos, c8(140, 240, 120, 255));
    }

    fn harvest(&mut self, tx: i32, ty: i32) -> bool {
        if !inb(tx, ty) {
            return false;
        }
        if let Some(c) = self.world.crops[idx(tx, ty)] {
            if c.ripe() {
                let (a, b) = c.kind.yield_range();
                let n = self.rng.range(a, b + 1);
                self.world.crops[idx(tx, ty)] = None;
                self.spawn_drop(Item::Produce(c.kind), n, Self::tile_center(tx, ty));
                self.burst(Self::tile_center(tx, ty), c8(110, 190, 70, 255), 10, 40.0);
                return true;
            }
        }
        false
    }

    fn interact(&mut self) {
        let (tx, ty) = self.target_tile();
        self.face_toward(tx, ty);
        if self.harvest(tx, ty) {
            return;
        }
        // anche la casella sotto i piedi (comodo per il raccolto)
        let (px, py) = self.player_tile();
        if self.harvest(px, py) {
            return;
        }
        if let Some(i) = self.world.obj_at(tx, ty) {
            let o = self.world.objs[i];
            match o.kind {
                Kind::House => {
                    self.sleep_yes = true;
                    self.mode = Mode::Sleep;
                    return;
                }
                Kind::Cottage => {
                    self.toast("La casa di Mira. La porta è chiusa a chiave.");
                    return;
                }
                Kind::Bin => {
                    self.ship_selected();
                    return;
                }
                Kind::Stall => {
                    let h = self.time / 60.0;
                    if (8.0..20.0).contains(&h) {
                        self.mode = Mode::Shop;
                    } else {
                        self.toast("Mira è tornata a casa. La bancarella apre alle 8:00.");
                    }
                    return;
                }
                Kind::Well => {
                    self.refill();
                    return;
                }
                Kind::Mailbox => {
                    let tip = TIPS[self.tip_i % TIPS.len()];
                    self.tip_i += 1;
                    self.toast(tip);
                    return;
                }
                Kind::Scarecrow => {
                    self.toast("Lo spaventapasseri Bruno veglia sui campi.");
                    return;
                }
                _ => {}
            }
        }
        if self.world.g(tx, ty) == Ground::Water && self.held() == Some(Item::Can) {
            self.refill();
            return;
        }
        if matches!(self.held(), Some(Item::Produce(_))) {
            self.eat();
        }
    }

    fn refill(&mut self) {
        if self.inv.iter().flatten().any(|s| s.0 == Item::Can) {
            self.water = MAX_WATER;
            let (tx, ty) = self.target_tile();
            self.burst(Self::tile_center(tx, ty), c8(120, 180, 250, 255), 12, 40.0);
            self.toast("Annaffiatoio pieno!");
        }
    }

    fn ship_selected(&mut self) {
        let Some((it, n)) = self.inv[self.sel] else {
            self.toast("Seleziona qualcosa da spedire, poi usa la cassa.");
            return;
        };
        if it.is_tool() || it.sell_price() == 0 {
            self.toast("Questo non si può vendere.");
            return;
        }
        self.inv[self.sel] = None;
        if let Some(s) = self.shipped.iter_mut().find(|s| s.0 == it) {
            s.1 += n;
        } else {
            self.shipped.push((it, n));
        }
        let b = self.world.find(Kind::Bin).map(|o| vec2(o.x as f32 * TS + 16.0, o.y as f32 * TS)).unwrap_or(self.player.pos);
        self.burst(b, c8(250, 210, 90, 255), 10, 30.0);
        self.popup(format!("{} ×{} spedito", it.name(), n), b - vec2(0.0, 20.0), c8(255, 230, 120, 255));
    }

    fn apply_tool(&mut self, it: Item, tx: i32, ty: i32) {
        let center = Self::tile_center(tx, ty);
        if !inb(tx, ty) {
            return;
        }
        let obj = self.world.obj_at(tx, ty);
        match it {
            Item::Hoe => {
                if obj.is_none() && self.world.farmable(tx, ty) && !self.world.tilled[idx(tx, ty)] {
                    self.world.tilled[idx(tx, ty)] = true;
                    if self.raining {
                        self.world.wet[idx(tx, ty)] = true;
                    }
                    self.burst(center, c8(150, 92, 46, 255), 10, 35.0);
                } else if obj.is_none() && tx > FARM_MAX_X {
                    if self.toast.is_none() {
                        self.toast("Qui non si può zappare: è il paese!");
                    }
                } else {
                    self.burst(center, c8(200, 200, 200, 255), 3, 20.0);
                }
            }
            Item::Can => {
                if self.world.g(tx, ty) == Ground::Water || obj.map(|i| self.world.objs[i].kind == Kind::Well).unwrap_or(false) {
                    self.refill();
                    return;
                }
                if self.water <= 0 {
                    self.toast("L'annaffiatoio è vuoto! Riempilo al pozzo o allo stagno.");
                    return;
                }
                self.water -= 1;
                if self.world.tilled[idx(tx, ty)] {
                    self.world.wet[idx(tx, ty)] = true;
                }
                for _ in 0..14 {
                    let a = self.rng.rf(0.0, 6.283);
                    self.parts.push(Particle {
                        pos: center + vec2(a.cos() * 5.0, a.sin() * 3.0),
                        vel: vec2(a.cos() * 20.0, a.sin() * 10.0),
                        z: self.rng.rf(6.0, 12.0),
                        vz: self.rng.rf(-10.0, 30.0),
                        grav: 200.0,
                        life: 0.0,
                        max: 0.5,
                        col: c8(140, 200, 255, 255),
                        size: 1.0,
                    });
                }
            }
            Item::Axe => {
                if let Some(i) = obj {
                    let o = self.world.objs[i];
                    match o.kind {
                        Kind::Oak | Kind::Pine | Kind::Cherry => {
                            self.world.objs[i].hp -= 1;
                            self.world.objs[i].shake = 0.3;
                            let col = match o.kind {
                                Kind::Cherry => c8(245, 175, 210, 255),
                                Kind::Pine => c8(50, 120, 90, 255),
                                _ => c8(90, 170, 66, 255),
                            };
                            self.burst(center - vec2(0.0, 26.0), col, 8, 40.0);
                            self.burst(center, c8(170, 110, 50, 255), 4, 30.0);
                            if self.world.objs[i].hp <= 0 {
                                let dir = if self.player.pos.x < center.x { 1.0 } else { -1.0 };
                                self.falling.push(Falling { kind: o.kind, var: o.var, base: vec2(center.x, ty as f32 * TS + 14.0), t: 0.0, dir });
                                self.world.objs[i].kind = Kind::Stump;
                                self.world.objs[i].hp = Kind::Stump.hp();
                            }
                        }
                        Kind::Stump => {
                            self.world.objs[i].hp -= 1;
                            self.world.objs[i].shake = 0.2;
                            self.burst(center, c8(170, 110, 50, 255), 8, 35.0);
                            if self.world.objs[i].hp <= 0 {
                                self.world.remove(i);
                                self.spawn_drop(Item::Wood, 3, center);
                            }
                        }
                        Kind::Stick => {
                            self.world.remove(i);
                            self.spawn_drop(Item::Wood, 1, center);
                            self.burst(center, c8(170, 110, 50, 255), 6, 30.0);
                        }
                        Kind::Bush | Kind::Weed => {
                            self.world.objs[i].shake = 0.25;
                            self.burst(center, c8(90, 170, 66, 255), 6, 30.0);
                        }
                        _ => {}
                    }
                }
            }
            Item::Pick => {
                if let Some(i) = obj {
                    let o = self.world.objs[i];
                    if matches!(o.kind, Kind::Rock | Kind::Boulder) {
                        self.world.objs[i].hp -= 1;
                        self.world.objs[i].shake = 0.2;
                        let c = if o.kind == Kind::Boulder { vec2(o.x as f32 * TS + 16.0, o.y as f32 * TS + 16.0) } else { center };
                        self.burst(c, c8(170, 170, 185, 255), 8, 45.0);
                        if self.world.objs[i].hp <= 0 {
                            self.world.remove(i);
                            let n = if o.kind == Kind::Boulder { self.rng.range(6, 10) } else { self.rng.range(1, 3) };
                            self.spawn_drop(Item::Stone, n, c);
                        }
                    }
                } else if self.world.tilled[idx(tx, ty)] && self.world.crops[idx(tx, ty)].is_none() {
                    self.world.tilled[idx(tx, ty)] = false;
                    self.world.wet[idx(tx, ty)] = false;
                    self.burst(center, c8(150, 92, 46, 255), 8, 30.0);
                }
            }
            Item::Scythe => {
                // la falce taglia un arco di 3 caselle davanti
                let (px, py) = self.player_tile();
                let (dx, dy) = (tx - px, ty - py);
                let cells: Vec<(i32, i32)> = if dx.abs() + dy.abs() == 1 {
                    if dx != 0 {
                        vec![(tx, ty - 1), (tx, ty), (tx, ty + 1)]
                    } else {
                        vec![(tx - 1, ty), (tx, ty), (tx + 1, ty)]
                    }
                } else {
                    vec![(tx, ty)]
                };
                for (x, y) in cells {
                    let c = Self::tile_center(x, y);
                    if let Some(i) = self.world.obj_at(x, y) {
                        if self.world.objs[i].kind == Kind::Weed {
                            self.world.remove(i);
                            self.burst(c, c8(90, 170, 66, 255), 10, 40.0);
                            if self.rng.chance(0.7) {
                                self.spawn_drop(Item::Fiber, 1, c);
                            }
                            if self.rng.chance(0.06) {
                                let k = Crop::ALL[self.rng.range(0, 4) as usize];
                                self.spawn_drop(Item::Seed(k), 1, c);
                            }
                        }
                    } else {
                        self.harvest(x, y);
                    }
                }
            }
            _ => {}
        }
    }

    // ------------------------------------------------------------ negozio / sonno / notte

    pub fn shop_items(&self) -> Vec<Item> {
        Crop::ALL.iter().map(|&k| Item::Seed(k)).collect()
    }

    fn update_shop(&mut self) {
        let items = self.shop_items();
        let n = items.len();
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::E) || is_mouse_button_pressed(MouseButton::Right) {
            self.mode = Mode::Play;
            return;
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.shop_sel = (self.shop_sel + 1) % n;
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.shop_sel = (self.shop_sel + n - 1) % n;
        }
        let mut buy = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if let Some(i) = self.shop_hit() {
            if self.mouse_active {
                self.shop_sel = i;
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                self.shop_sel = i;
                buy = true;
            }
        }
        if buy {
            let it = items[self.shop_sel];
            let qty = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) { 5 } else { 1 };
            let price = match it {
                Item::Seed(k) => k.seed_price(),
                _ => 0,
            } * qty;
            if self.money < price {
                self.toast("Non hai abbastanza monete.");
            } else if self.add_item(it, qty) {
                self.money -= price;
                self.held_t = 0.0;
            } else {
                self.toast("Lo zaino è pieno!");
            }
        }
    }

    fn update_sleep(&mut self) {
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::D) {
            self.sleep_yes = !self.sleep_yes;
        }
        let mut confirm = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if is_key_pressed(KeyCode::Y) || is_key_pressed(KeyCode::S) {
            self.sleep_yes = true;
            confirm = true;
        }
        if is_key_pressed(KeyCode::N) || is_key_pressed(KeyCode::Escape) {
            self.mode = Mode::Play;
            return;
        }
        if let Some(yes) = self.sleep_hit() {
            if self.mouse_active {
                self.sleep_yes = yes;
            }
            if is_mouse_button_pressed(MouseButton::Left) {
                self.sleep_yes = yes;
                confirm = true;
            }
        }
        if confirm {
            if self.sleep_yes {
                self.passed_out = false;
                self.start_night();
            } else {
                self.mode = Mode::Play;
            }
        }
    }

    fn start_night(&mut self) {
        self.mode = Mode::Night;
        self.night_t = 0.0;
        self.night_done = false;
    }

    fn update_night(&mut self, dt: f32) {
        self.night_t += dt;
        if !self.night_done && self.night_t > 1.0 {
            self.night_done = true;
            self.new_day();
        }
        let skip = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Left);
        if self.night_done && self.night_t < 4.5 && skip {
            self.night_t = 4.5;
        }
        if self.night_t > 5.5 {
            self.mode = Mode::Play;
        }
    }

    fn new_day(&mut self) {
        // colture
        for i in 0..(W * H) as usize {
            if let Some(c) = &mut self.world.crops[i] {
                if self.world.wet[i] && c.growth < c.kind.days() {
                    c.growth += 1;
                }
            }
            self.world.wet[i] = false;
        }
        // pagamento spedizioni
        let mut earned = 0;
        self.summary.clear();
        for (it, n) in &self.shipped {
            let v = it.sell_price() * n;
            earned += v;
            self.summary.push(format!("{} ×{}   {}", it.name(), n, v));
        }
        self.shipped.clear();
        self.money += earned;
        self.summary.insert(0, if earned > 0 { format!("Guadagni dalla cassa: {}", earned) } else { "Nessuna spedizione ieri.".into() });
        // nuove erbacce e rametti
        for _ in 0..10 {
            let x = self.rng.range(18, 61);
            let y = self.rng.range(21, 53);
            let k = match self.rng.range(0, 6) {
                0 => Kind::Stick,
                1 => Kind::Rock,
                _ => Kind::Weed,
            };
            let near_player = (Self::tile_center(x, y) - self.player.pos).length() < 40.0;
            if !near_player && self.world.crops[idx(x, y)].is_none() {
                let v = self.rng.range(0, 3) as u32;
                self.world.place(k, x, y, v);
            }
        }
        self.day += 1;
        self.time = DAY_START;
        self.late_warned = false;
        self.raining = self.rng.chance(0.22);
        if self.raining {
            for i in 0..(W * H) as usize {
                if self.world.tilled[i] {
                    self.world.wet[i] = true;
                }
            }
            self.summary.push("Oggi piove: le colture si annaffiano da sole!".into());
        }
        self.energy = if self.passed_out { MAX_ENERGY * 0.5 } else { MAX_ENERGY };
        self.water = MAX_WATER;
        self.player.pos = vec2(41.0 * TS + 8.0, 19.0 * TS + 12.0);
        self.player.dir = 0;
        self.player.act_t = 0.0;
        self.drops.clear();
        self.falling.clear();
        self.save();
        self.summary.push("Partita salvata.".into());
    }

    /// Prepara una scena dimostrativa (usato solo per le schermate di prova).
    pub fn debug_setup(&mut self, what: &str) {
        if what == "title" {
            return;
        }
        if what == "selftest" {
            self.selftest();
            std::process::exit(0);
        }
        self.mode = Mode::Play;
        self.time = 10.0 * 60.0;
        // un campo di esempio con tutte le colture in vari stadi
        let mut k = 0;
        for y in 25..29 {
            for x in 36..48 {
                if self.world.obj_at(x, y).is_none() {
                    let i = idx(x, y);
                    self.world.tilled[i] = true;
                    self.world.wet[i] = (x + y) % 3 != 0;
                    if y != 27 {
                        let kind = Crop::ALL[((x - 36) / 3) as usize % 4];
                        let g = ((k * 7) % (kind.days() as i32 + 1)) as u8;
                        self.world.crops[i] = Some(CropState { kind, growth: g });
                    }
                    k += 1;
                }
            }
        }
        self.player.pos = vec2(42.0 * TS + 8.0, 24.0 * TS + 10.0);
        self.player.dir = 0;
        self.cam = vec2(self.player.pos.x - self.vw / 2.0, self.player.pos.y - 14.0 - self.vh / 2.0);
        self.inv[6] = Some((Item::Produce(Crop::Zucca), 3));
        self.inv[7] = Some((Item::Wood, 24));
        self.inv[8] = Some((Item::Seed(Crop::Girasole), 5));
        self.sel = 0;
        match what {
            "night" => self.time = 21.5 * 60.0,
            "help" => self.mode = Mode::Help,
            "toast" => self.toast(TIPS[0]),
            "dusk" => self.time = 18.8 * 60.0,
            "morning" => self.time = 7.6 * 60.0,
            "golden" => self.time = 17.7 * 60.0,
            "noon" => self.time = 12.5 * 60.0,
            "house" => {
                self.time = 16.2 * 60.0;
                self.player.pos = vec2(42.0 * TS + 8.0, 21.0 * TS + 10.0);
            }
            "cottage" => {
                self.time = 11.0 * 60.0;
                self.player.pos = vec2(86.0 * TS + 8.0, 26.0 * TS + 10.0);
            }
            "rain" => self.raining = true,
            "shop" => {
                self.player.pos = vec2(81.0 * TS + 8.0, 33.0 * TS + 10.0);
                self.mode = Mode::Shop;
            }
            "town" => {
                self.player.pos = vec2(80.0 * TS + 8.0, 34.0 * TS + 10.0);
                self.time = 20.6 * 60.0;
            }
            "forest" => {
                self.player.pos = vec2(28.5 * TS, 15.0 * TS + 10.0);
                self.player.dir = 1;
            }
            "chop" => {
                self.player.pos = vec2(30.0 * TS + 8.0, 38.0 * TS + 10.0);
                self.sel = 2;
                self.player.dir = 3;
                self.player.act_total = 0.34;
                self.player.act_t = 0.12;
                self.player.act_item = Some(Item::Axe);
                self.player.act_done = true;
            }
            _ => {}
        }
        if matches!(what, "shop" | "town" | "forest" | "chop" | "house" | "cottage") {
            self.cam = vec2(self.player.pos.x - self.vw / 2.0, self.player.pos.y - 14.0 - self.vh / 2.0);
        }
    }

    /// Verifica automatica del ciclo di gioco completo.
    fn selftest(&mut self) {
        let mut ok = true;
        let mut check = |name: &str, c: bool| {
            println!("[{}] {}", if c { "OK" } else { "FALLITO" }, name);
            ok &= c;
        };
        self.mode = Mode::Play;
        let (tx, ty) = (46, 29);
        if let Some(i) = self.world.obj_at(tx, ty) {
            self.world.remove(i);
        }
        self.apply_tool(Item::Hoe, tx, ty);
        check("la zappa lavora il terreno", self.world.tilled[idx(tx, ty)]);
        self.player.pos = vec2(tx as f32 * TS + 8.0, (ty - 1) as f32 * TS + 12.0);
        self.player.dir = 0;
        self.mouse_active = false;
        self.sel = 5;
        let seeds_before = self.inv[5].unwrap().1;
        self.use_held();
        check("si pianta una carota", self.world.crops[idx(tx, ty)].is_some() && self.inv[5].unwrap().1 == seeds_before - 1);
        let w0 = self.water;
        self.apply_tool(Item::Can, tx, ty);
        check("l'annaffiatoio bagna il terreno", self.world.wet[idx(tx, ty)] && self.water == w0 - 1);
        for _ in 0..Crop::Carota.days() {
            self.world.wet[idx(tx, ty)] = true;
            self.new_day();
        }
        check("la carota matura in 4 giorni", self.world.crops[idx(tx, ty)].map(|c| c.ripe()).unwrap_or(false));
        self.raining = false;
        check("raccolto", self.harvest(tx, ty));
        let n_drops = self.drops.len();
        check("cadono dei frutti", n_drops >= 1);
        for d in std::mem::take(&mut self.drops) {
            self.add_item(d.item, 1);
        }
        let slot = self.inv.iter().position(|s| matches!(s, Some((Item::Produce(Crop::Carota), _)))).unwrap();
        self.sel = slot;
        let money0 = self.money;
        self.ship_selected();
        self.new_day();
        check(&format!("vendita pagata di notte (+{})", self.money - money0), self.money == money0 + Crop::Carota.sell() * n_drops as i32);
        // albero: 6 colpi d'ascia -> ceppo
        let tree = self.world.objs.iter().position(|o| o.alive && o.kind == Kind::Oak && o.x > 18 && o.x < 60 && o.y > 21 && o.y < 52).unwrap();
        let (ox, oy) = (self.world.objs[tree].x, self.world.objs[tree].y);
        for _ in 0..6 {
            self.apply_tool(Item::Axe, ox, oy);
        }
        check("l'albero diventa un ceppo", self.world.objs[tree].kind == Kind::Stump && self.falling.len() == 1);
        for _ in 0..4 {
            self.apply_tool(Item::Axe, ox, oy);
        }
        check("il ceppo sparisce", !self.world.objs[tree].alive && self.world.obj_at(ox, oy).is_none());
        let money1 = self.money;
        self.mode = Mode::Shop;
        self.shop_sel = 0;
        if self.money >= 20 && self.add_item(Item::Seed(Crop::Carota), 1) {
            self.money -= 20;
        }
        check("il negozio vende semi", self.money == money1 - 20);
        self.save();
        let day = self.day;
        self.day = 99;
        self.load();
        check("salvataggio e caricamento", self.day == day);
        let _ = std::fs::remove_file(SAVE_PATH);
        println!("{}", if ok { "TUTTI I TEST SUPERATI" } else { "ALCUNI TEST FALLITI" });
    }

    // ------------------------------------------------------------ salvataggio

    fn peek_save(&self) -> Option<i32> {
        let s = std::fs::read_to_string(SAVE_PATH).ok()?;
        s.lines().find_map(|l| l.strip_prefix("day ").and_then(|v| v.parse().ok()))
    }

    fn save(&self) {
        let mut s = String::from("valle-lucciole v1\n");
        s += &format!("day {}\ntime {}\nmoney {}\nenergy {}\nwater {}\nrain {}\nsel {}\n", self.day, self.time, self.money, self.energy, self.water, self.raining as i32, self.sel);
        for (i, sl) in self.inv.iter().enumerate() {
            if let Some((it, n)) = sl {
                s += &format!("inv {} {} {}\n", i, it.code(), n);
            }
        }
        for y in 0..H {
            for x in 0..W {
                let i = idx(x, y);
                if self.world.tilled[i] {
                    let (ck, cg) = self.world.crops[i].map(|c| (c.kind.idx() as i32, c.growth as i32)).unwrap_or((-1, 0));
                    s += &format!("tile {} {} {} {} {}\n", x, y, self.world.wet[i] as i32, ck, cg);
                }
            }
        }
        for o in self.world.objs.iter().filter(|o| o.alive) {
            s += &format!("obj {} {} {} {} {}\n", o.kind.code(), o.x, o.y, o.var, o.hp);
        }
        let _ = std::fs::write(SAVE_PATH, s);
    }

    fn load(&mut self) {
        let Ok(s) = std::fs::read_to_string(SAVE_PATH) else { return };
        let mut objs = Vec::new();
        self.inv = [None; SLOTS];
        let n = (W * H) as usize;
        self.world.tilled = vec![false; n];
        self.world.wet = vec![false; n];
        self.world.crops = vec![None; n];
        for l in s.lines() {
            let p: Vec<&str> = l.split_whitespace().collect();
            let num = |i: usize| p.get(i).and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
            match p.first().copied() {
                Some("day") => self.day = num(1) as i32,
                Some("time") => self.time = num(1),
                Some("money") => self.money = num(1) as i32,
                Some("energy") => self.energy = num(1),
                Some("water") => self.water = num(1) as i32,
                Some("rain") => self.raining = num(1) > 0.0,
                Some("sel") => self.sel = (num(1) as usize).min(SLOTS - 1),
                Some("inv") => {
                    if let (Some(it), i) = (p.get(2).and_then(|c| Item::from_code(c)), num(1) as usize) {
                        if i < SLOTS {
                            self.inv[i] = Some((it, num(3) as i32));
                        }
                    }
                }
                Some("tile") => {
                    let (x, y) = (num(1) as i32, num(2) as i32);
                    if inb(x, y) {
                        let i = idx(x, y);
                        self.world.tilled[i] = true;
                        self.world.wet[i] = num(3) > 0.0;
                        let ck = num(4) as i32;
                        if ck >= 0 {
                            self.world.crops[i] = Some(CropState { kind: Crop::from_idx(ck as usize), growth: num(5) as u8 });
                        }
                    }
                }
                Some("obj") => {
                    let k = num(1) as usize;
                    if k < Kind::ALL.len() {
                        objs.push(Obj { kind: Kind::ALL[k], x: num(2) as i32, y: num(3) as i32, var: num(4) as u32, hp: num(5) as i32, shake: 0.0, alive: true });
                    }
                }
                _ => {}
            }
        }
        if !objs.is_empty() {
            self.world.set_objects(objs);
        }
        self.player.pos = vec2(41.0 * TS + 8.0, 19.0 * TS + 12.0);
    }
}
