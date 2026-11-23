//! Disegno del mondo: terreno, oggetti ordinati per profondità, ombre, luci.

use crate::game::*;
use crate::items::*;
use crate::util::*;
use crate::world::*;
use macroquad::prelude::*;

enum Dr {
    Obj(usize),
    Crop(i32, i32),
    Player,
    Merchant(Vec2),
    Chicken(usize),
    Fall(usize),
    Drop(usize),
}

pub fn cam_for(rt: &RenderTarget, top_left: Vec2, vw: f32, vh: f32) -> Camera2D {
    Camera2D {
        target: vec2(top_left.x + vw / 2.0, top_left.y + vh / 2.0),
        zoom: vec2(2.0 / vw, 2.0 / vh),
        render_target: Some(rt.clone()),
        ..Default::default()
    }
}

fn tex(t: &Texture2D, x: f32, y: f32, col: Color) {
    draw_texture(t, x.round(), y.round(), col);
}

impl Game {
    pub fn obj_sprite(&self, o: &Obj) -> (&Texture2D, Vec2) {
        let a = &self.art;
        let v = o.var as usize;
        match o.kind {
            Kind::Oak => (&a.oak[v % 3], vec2(-16.0, -46.0)),
            Kind::Cherry => (&a.cherry[v % 3], vec2(-16.0, -46.0)),
            Kind::Pine => (&a.pine[v % 3], vec2(-8.0, -46.0)),
            Kind::Stump => (&a.stump, vec2(0.0, 0.0)),
            Kind::Rock => (&a.rock[v % 3], vec2(0.0, 0.0)),
            Kind::Boulder => (&a.boulder, vec2(0.0, 0.0)),
            Kind::Weed => (&a.weed[v % 3], vec2(0.0, -1.0)),
            Kind::Stick => (&a.stick[v % 2], vec2(0.0, 0.0)),
            Kind::Bush => (&a.bush[v % 2], vec2(0.0, -8.0)),
            Kind::House => (&a.house, vec2(0.0, -48.0)),
            Kind::Cottage => (&a.cottage, vec2(0.0, -48.0)),
            Kind::Bin => (&a.bin, vec2(0.0, -15.0)),
            Kind::Well => (&a.well, vec2(0.0, -31.0)),
            Kind::Stall => (&a.stall, vec2(0.0, -30.0)),
            Kind::Fence => (&a.fence_post, vec2(0.0, 0.0)),
            Kind::Mailbox => (&a.mailbox, vec2(0.0, -15.0)),
            Kind::Scarecrow => (&a.scarecrow, vec2(0.0, -15.0)),
            Kind::Lamp => (&a.lamp, vec2(0.0, -23.0)),
            Kind::Fountain => (&a.fountain, vec2(0.0, -14.0)),
            Kind::Bench => (&a.bench, vec2(0.0, -8.0)),
            Kind::Planter => (&a.planter[v % 3], vec2(0.0, -8.0)),
        }
    }

    fn is_tall(k: Kind) -> bool {
        matches!(k, Kind::Oak | Kind::Pine | Kind::Cherry | Kind::House | Kind::Cottage | Kind::Stall | Kind::Well | Kind::Lamp | Kind::Fountain)
    }

    pub fn draw(&self) {
        let (vw, vh) = (self.vw, self.vh);
        let cam = self.cam.round();
        // 1) mondo
        set_camera(&cam_for(&self.scene, cam, vw, vh));
        clear_background(BLACK);
        self.draw_world(cam);

        // 2) luci: mappa di luce moltiplicata sulla scena
        let amb = self.ambient();
        let night = self.night();
        if amb.r < 0.999 || amb.g < 0.999 || amb.b < 0.999 {
            set_camera(&cam_for(&self.light, cam, vw, vh));
            clear_background(amb);
            gl_use_material(&self.add_mat);
            self.draw_lights(night);
            gl_use_default_material();
            set_camera(&cam_for(&self.scene, Vec2::ZERO, vw, vh));
            gl_use_material(&self.mul_mat);
            draw_texture_ex(&self.light.texture, 0.0, 0.0, WHITE, DrawTextureParams { dest_size: Some(vec2(vw, vh)), ..Default::default() });
            gl_use_default_material();
        }
        // 3) cose luminose sopra il buio (finestre accese, lucciole)
        set_camera(&cam_for(&self.scene, cam, vw, vh));
        self.draw_emissive(night);

        // 4) pioggia e interfaccia in coordinate schermo
        set_camera(&cam_for(&self.scene, Vec2::ZERO, vw, vh));
        if self.raining {
            for d in &self.rain {
                draw_line(d.x, d.y, d.x - 2.5, d.y + 8.0, 1.0, Color::new(0.75, 0.82, 1.0, 0.45));
            }
        }
        self.draw_ui();

        // 5) a schermo intero, ingrandito a scala intera (pixel nitidi)
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &self.scene.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams { dest_size: Some(vec2(vw * self.scale, vh * self.scale)), ..Default::default() },
        );
    }

    fn draw_world(&self, cam: Vec2) {
        let (vw, vh) = (self.vw, self.vh);
        // terreno cotto
        draw_texture_ex(
            &self.ground_tex,
            cam.x,
            cam.y,
            WHITE,
            DrawTextureParams { source: Some(Rect::new(cam.x, cam.y, vw, vh)), dest_size: Some(vec2(vw, vh)), ..Default::default() },
        );
        let tx0 = ((cam.x / TS).floor() as i32 - 3).max(0);
        let ty0 = ((cam.y / TS).floor() as i32 - 3).max(0);
        let tx1 = (((cam.x + vw) / TS).ceil() as i32 + 3).min(W - 1);
        let ty1 = (((cam.y + vh) / TS).ceil() as i32 + 5).min(H - 1);
        let w = &self.world;
        // luccichii sull'acqua
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                if w.g(tx, ty) != Ground::Water {
                    continue;
                }
                if w.g(tx, ty - 1) != Ground::Water || w.g(tx - 1, ty) != Ground::Water || w.g(tx + 1, ty) != Ground::Water {
                    continue;
                }
                let h = hash(tx, ty, 5);
                let ph = self.t * 0.9 + (h % 1000) as f32 / 100.0;
                let k = ph.rem_euclid(3.0);
                if k < 1.2 {
                    let a = (k / 1.2 * std::f32::consts::PI).sin();
                    let x = tx as f32 * TS + (h % 11) as f32 + (ph * 2.0).sin() * 1.5;
                    let y = ty as f32 * TS + ((h >> 5) % 13) as f32;
                    draw_rectangle(x.round(), y.round(), 3.0, 1.0, Color::new(0.85, 0.95, 1.0, 0.75 * a));
                    if h % 3 == 0 {
                        draw_rectangle((x + 5.0).round(), (y + 3.0).round(), 2.0, 1.0, Color::new(0.85, 0.95, 1.0, 0.5 * a));
                    }
                }
            }
        }
        // terreno zappato
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                let i = idx(tx, ty);
                if !w.tilled[i] {
                    continue;
                }
                let t = |x: i32, y: i32| inb(x, y) && w.tilled[idx(x, y)];
                let mask = (t(tx, ty - 1) as usize) | (t(tx + 1, ty) as usize) << 1 | (t(tx, ty + 1) as usize) << 2 | (t(tx - 1, ty) as usize) << 3;
                let tt = if w.wet[i] { &self.art.tilled_wet[mask] } else { &self.art.tilled[mask] };
                tex(tt, tx as f32 * TS, ty as f32 * TS, WHITE);
            }
        }

        // raccolta degli elementi da ordinare per profondità
        let mut list: Vec<(f32, Dr)> = Vec::with_capacity(512);
        let view = Rect::new(cam.x - 64.0, cam.y - 32.0, vw + 128.0, vh + 120.0);
        for (i, o) in w.objs.iter().enumerate() {
            if !o.alive {
                continue;
            }
            let (_, fh) = o.kind.size();
            let px = o.x as f32 * TS;
            let py = o.y as f32 * TS;
            if !view.contains(vec2(px, py)) && !view.contains(vec2(px + 96.0, py)) {
                continue;
            }
            list.push(((o.y + fh) as f32 * TS - 0.5, Dr::Obj(i)));
            if o.kind == Kind::Stall {
                list.push((py + 15.0, Dr::Merchant(vec2(px + 24.0, py + 16.0))));
            }
        }
        for ty in ty0..=ty1 {
            for tx in tx0..=tx1 {
                if w.crops[idx(tx, ty)].is_some() {
                    list.push((ty as f32 * TS + 11.0, Dr::Crop(tx, ty)));
                }
            }
        }
        if self.mode != Mode::Title {
            list.push((self.player.pos.y, Dr::Player));
        }
        for (i, c) in self.chickens.iter().enumerate() {
            list.push((c.pos.y, Dr::Chicken(i)));
        }
        for (i, f) in self.falling.iter().enumerate() {
            list.push((f.base.y, Dr::Fall(i)));
        }
        for (i, d) in self.drops.iter().enumerate() {
            list.push((d.pos.y, Dr::Drop(i)));
        }
        list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // ombre (prima di tutto, sul terreno)
        let shadow = |t: &Texture2D, cx: f32, cy: f32| {
            draw_texture(t, (cx - t.width() / 2.0).round(), (cy - t.height() / 2.0).round(), WHITE);
        };
        for (_, d) in &list {
            match d {
                Dr::Obj(i) => {
                    let o = &w.objs[*i];
                    let bx = o.x as f32 * TS;
                    let by = o.y as f32 * TS;
                    match o.kind {
                        k if k.is_tree() => shadow(&self.art.shadow_l, bx + 8.0, by + 14.0),
                        Kind::Rock | Kind::Stump | Kind::Bush => shadow(&self.art.shadow_m, bx + 8.0, by + 14.0),
                        Kind::Boulder => shadow(&self.art.shadow_l, bx + 16.0, by + 29.0),
                        Kind::Lamp | Kind::Mailbox | Kind::Scarecrow => shadow(&self.art.shadow_s, bx + 8.0, by + 15.0),
                        Kind::Bin | Kind::Well | Kind::Bench => shadow(&self.art.shadow_l, bx + 16.0, by + 15.0),
                        Kind::Planter => shadow(&self.art.shadow_m, bx + 8.0, by + 15.0),
                        _ => {}
                    }
                }
                Dr::Player => shadow(&self.art.shadow_s, self.player.pos.x, self.player.pos.y - 1.0),
                Dr::Chicken(i) => shadow(&self.art.shadow_s, self.chickens[*i].pos.x, self.chickens[*i].pos.y - 1.0),
                Dr::Drop(i) => shadow(&self.art.shadow_s, self.drops[*i].pos.x, self.drops[*i].pos.y),
                _ => {}
            }
        }

        let pp = self.player.pos;
        for (sy, d) in &list {
            match d {
                Dr::Obj(i) => {
                    let o = &w.objs[*i];
                    let (t, off) = self.obj_sprite(o);
                    let mut p = vec2(o.x as f32 * TS, o.y as f32 * TS) + off;
                    if o.shake > 0.0 {
                        p.x += (self.t * 70.0).sin() * o.shake * 5.0;
                    }
                    let mut col = WHITE;
                    if Self::is_tall(o.kind) && self.mode != Mode::Title {
                        let r = Rect::new(p.x + 2.0, p.y + 2.0, t.width() - 4.0, t.height() - 4.0);
                        if pp.y < *sy - 1.0 && r.contains(pp - vec2(0.0, 10.0)) {
                            col = Color::new(1.0, 1.0, 1.0, 0.5);
                        }
                    }
                    if o.kind == Kind::Fence {
                        if w.is_fence(o.x + 1, o.y) {
                            tex(&self.art.fence_h, p.x + 8.0, p.y, col);
                        }
                        if w.is_fence(o.x, o.y + 1) {
                            tex(&self.art.fence_v, p.x, p.y + 8.0, col);
                        }
                    }
                    tex(t, p.x, p.y, col);
                }
                Dr::Crop(x, y) => {
                    let c = w.crops[idx(*x, *y)].unwrap();
                    let t = &self.art.crops[c.kind.idx()][c.stage()];
                    let sway = if c.stage() >= 2 { ((self.t * 1.7 + *x as f32 * 0.7).sin() * 0.6).round() } else { 0.0 };
                    tex(t, *x as f32 * TS + sway, *y as f32 * TS - 16.0, WHITE);
                    if c.ripe() && self.mode == Mode::Play {
                        // scintilla sul raccolto pronto
                        let k = (self.t * 2.0 + *x as f32 * 1.3 + *y as f32).rem_euclid(2.5);
                        if k < 0.4 {
                            draw_rectangle((*x as f32 * TS + 12.0).round(), (*y as f32 * TS - 6.0).round(), 1.0, 1.0, WHITE);
                        }
                    }
                }
                Dr::Player => self.draw_player(),
                Dr::Merchant(p) => {
                    let bob = if (self.t * 0.8).rem_euclid(2.0) < 0.15 { -1.0 } else { 0.0 };
                    tex(&self.art.merchant.walk[0][0], p.x - 8.0, p.y - 31.0 + bob, WHITE);
                }
                Dr::Chicken(i) => {
                    let c = &self.chickens[*i];
                    let f = if c.peck > 0.0 { 2 } else { (c.anim as usize) % 2 };
                    let t = if c.right { &self.art.chicken_r[f] } else { &self.art.chicken[f] };
                    tex(t, c.pos.x - 8.0, c.pos.y - 15.0, WHITE);
                }
                Dr::Fall(i) => {
                    let f = &self.falling[*i];
                    let o = Obj { kind: f.kind, x: 0, y: 0, var: f.var, hp: 0, shake: 0.0, alive: true };
                    let (t, off) = self.obj_sprite(&o);
                    let p = vec2(f.base.x - 8.0, f.base.y - 14.0) + off;
                    let k = f.t.min(1.0);
                    let ang = f.dir * k * k * 1.45;
                    let a = if k > 0.8 { 1.0 - (k - 0.8) / 0.2 } else { 1.0 };
                    draw_texture_ex(
                        t,
                        p.x.round(),
                        p.y.round(),
                        Color::new(1.0, 1.0, 1.0, a),
                        DrawTextureParams { rotation: ang, pivot: Some(f.base), ..Default::default() },
                    );
                }
                Dr::Drop(i) => {
                    let d = &self.drops[*i];
                    let bob = if d.z == 0.0 { (self.t * 4.0 + d.pos.x).sin() * 1.2 } else { 0.0 };
                    draw_texture_ex(
                        self.art.icon(d.item),
                        (d.pos.x - 6.0).round(),
                        (d.pos.y - 13.0 - d.z + bob).round(),
                        WHITE,
                        DrawTextureParams { dest_size: Some(vec2(12.0, 12.0)), ..Default::default() },
                    );
                }
            }
        }

        // particelle
        for p in &self.parts {
            let a = 1.0 - p.life / p.max;
            let mut c = p.col;
            c.a *= a.clamp(0.0, 1.0);
            let s = if p.grav == 0.0 { p.size + p.life * 2.0 } else { p.size };
            draw_rectangle(p.pos.x.round(), (p.pos.y - p.z).round(), s.round(), s.round(), c);
        }
        // farfalle (di giorno)
        if self.night() < 0.35 && !self.raining {
            for b in &self.butterflies {
                let f = ((self.t * 9.0 + b.phase) as usize) % 2;
                tex(&self.art.butterfly[f], b.pos.x, b.pos.y, b.col);
            }
        }
        // casella bersaglio
        if self.mode == Mode::Play {
            let (tx, ty) = self.target_tile();
            let x = tx as f32 * TS;
            let y = ty as f32 * TS;
            let c = Color::new(1.0, 1.0, 1.0, 0.55 + (self.t * 5.0).sin() * 0.2);
            for (cx, cy, sx, sy) in [(x, y, 1.0, 1.0), (x + 15.0, y, -1.0, 1.0), (x, y + 15.0, 1.0, -1.0), (x + 15.0, y + 15.0, -1.0, -1.0)] {
                draw_rectangle(cx, cy, 1.0, 1.0, c);
                draw_rectangle(cx + sx, cy, 1.0, 1.0, c);
                draw_rectangle(cx + sx * 2.0, cy, 1.0, 1.0, c);
                draw_rectangle(cx, cy + sy, 1.0, 1.0, c);
                draw_rectangle(cx, cy + sy * 2.0, 1.0, 1.0, c);
            }
        }
        // testi fluttuanti
        for p in &self.popups {
            let a = if p.t > 1.0 { 1.0 - (p.t - 1.0) / 0.4 } else { 1.0 };
            let mut c = p.col;
            c.a = a;
            self.font.draw_center(&p.text, p.pos.x, p.pos.y - p.t * 12.0, 1.0, c, Color::new(0.1, 0.06, 0.04, a * 0.8));
        }
    }

    fn draw_player(&self) {
        let p = &self.player;
        let d = p.dir as usize;
        let acting = p.act_t > 0.0;
        let t = if acting {
            &self.art.farmer.act[d]
        } else if p.moving {
            &self.art.farmer.walk[d][(p.anim as usize) % 4]
        } else {
            &self.art.farmer.walk[d][0]
        };
        let x = p.pos.x - 8.0;
        let y = p.pos.y - 31.0;
        if acting && d == 1 {
            self.draw_tool();
        }
        tex(t, x, y, WHITE);
        if acting && d != 1 {
            self.draw_tool();
        }
    }

    fn draw_tool(&self) {
        let p = &self.player;
        let Some(it) = p.act_item else { return };
        let k = (1.0 - p.act_t / p.act_total).clamp(0.0, 1.0);
        let e = k * k * (3.0 - 2.0 * k);
        let pos = p.pos;
        let (hand, flip) = match p.dir {
            0 => (vec2(pos.x + 3.0, pos.y - 11.0), false),
            1 => (vec2(pos.x + 4.0, pos.y - 18.0), false),
            2 => (vec2(pos.x - 6.0, pos.y - 13.0), true),
            _ => (vec2(pos.x + 6.0, pos.y - 13.0), false),
        };
        let mut ang = match it {
            Item::Can => 0.2 + e * 0.6,
            Item::Scythe => -2.2 + e * 3.2,
            _ => -1.9 + e * 2.5,
        };
        if p.dir == 0 && it != Item::Can {
            ang = -1.4 + e * 2.3;
        }
        if flip {
            ang = -ang;
        }
        let tl = if flip { hand - vec2(13.0, 13.0) } else { hand - vec2(3.0, 13.0) };
        draw_texture_ex(
            self.art.icon(it),
            tl.x.round(),
            tl.y.round(),
            WHITE,
            DrawTextureParams { rotation: ang, pivot: Some(hand), flip_x: flip, ..Default::default() },
        );
    }

    /// Finestre e lampioni da illuminare: (centro, raggio, colore)
    fn light_sources(&self) -> Vec<(Vec2, f32, Color)> {
        let mut v = Vec::new();
        let warm = Color::new(1.0, 0.78, 0.42, 1.0);
        for o in self.world.objs.iter().filter(|o| o.alive) {
            let base = vec2(o.x as f32 * TS, o.y as f32 * TS);
            match o.kind {
                Kind::House => {
                    let top = base + vec2(0.0, -48.0);
                    v.push((top + vec2(22.0, 57.0), 46.0, warm));
                    v.push((top + vec2(76.0, 56.0), 40.0, warm));
                }
                Kind::Cottage => {
                    let top = base + vec2(0.0, -48.0);
                    v.push((top + vec2(22.0, 57.0), 42.0, warm));
                    v.push((top + vec2(62.0, 56.0), 38.0, warm));
                }
                Kind::Lamp => v.push((base + vec2(8.0, -15.0), 84.0, Color::new(1.0, 0.85, 0.5, 1.0))),
                _ => {}
            }
        }
        v
    }

    /// Le lucciole si accendono solo di sera (non quando piove di giorno)
    pub fn firefly_k(&self) -> f32 {
        let h = self.time / 60.0;
        if self.raining {
            return 0.0;
        }
        ((h - 19.4) * 1.3).clamp(0.0, 1.0)
    }

    fn draw_lights(&self, night: f32) {
        let k = night.clamp(0.0, 1.0);
        let glow = |p: Vec2, r: f32, c: Color| {
            draw_texture_ex(&self.art.glow, p.x - r, p.y - r, c, DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() });
        };
        for (p, r, c) in self.light_sources() {
            glow(p, r, Color::new(c.r, c.g, c.b, 0.85 * k));
        }
        if self.mode != Mode::Title {
            glow(self.player.pos - vec2(0.0, 12.0), 54.0, Color::new(0.55, 0.5, 0.42, 0.55 * k));
        }
        let fk = self.firefly_k();
        for f in &self.fireflies {
            let a = ((self.t * 2.3 + f.phase).sin() * 0.5 + 0.5) * fk;
            glow(f.pos, 12.0, Color::new(0.75, 1.0, 0.35, a));
        }
    }

    fn draw_emissive(&self, night: f32) {
        let k = ((night - 0.25) * 1.6).clamp(0.0, 1.0);
        if k <= 0.0 {
            return;
        }
        let lit = Color::new(1.0, 0.86, 0.45, k);
        for o in self.world.objs.iter().filter(|o| o.alive) {
            let top = vec2(o.x as f32 * TS, o.y as f32 * TS);
            match o.kind {
                Kind::House | Kind::Cottage => {
                    let top = top + vec2(0.0, -48.0);
                    let w = if o.kind == Kind::House { 96.0 } else { 80.0 };
                    draw_rectangle(top.x + 15.0, top.y + 51.0, 14.0, 12.0, lit);
                    let (rx, rw) = if w > 80.0 { (w - 25.0, 11.0) } else { (w - 23.0, 10.0) };
                    draw_rectangle(top.x + rx, top.y + 51.0, rw, rw, lit);
                }
                Kind::Lamp => draw_rectangle(top.x + 5.0, top.y - 18.0, 6.0, 5.0, Color::new(1.0, 0.95, 0.7, k)),
                _ => {}
            }
        }
        let fk = self.firefly_k();
        for f in &self.fireflies {
            let a = ((self.t * 2.3 + f.phase).sin() * 0.5 + 0.5) * fk;
            draw_rectangle(f.pos.x.round(), f.pos.y.round(), 1.0, 1.0, Color::new(0.95, 1.0, 0.6, a));
            draw_rectangle(f.pos.x.round() - 1.0, f.pos.y.round(), 3.0, 1.0, Color::new(0.8, 1.0, 0.4, a * 0.4));
        }
    }
}
