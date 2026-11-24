//! Disegno del mondo: terreno, oggetti ordinati per profondità, ombre, luci.

use crate::game::*;
use crate::items::*;
use crate::util::*;
use crate::world::*;
use macroquad::models::{draw_mesh, Mesh, Vertex};
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
            Kind::House => (&a.house, vec2(0.0, 32.0 - a.house_info.base_y as f32)),
            Kind::Cottage => (&a.cottage, vec2(0.0, 32.0 - a.cottage_info.base_y as f32)),
            Kind::Bin => (&a.bin, vec2(0.0, -15.0)),
            Kind::Well => (&a.well, vec2(0.0, -33.0)),
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
        // 0) sun shadows into their own layer
        let (shear, sun_k) = self.sun();
        if sun_k > 0.01 {
            set_camera(&cam_for(&self.shadows, cam, vw, vh));
            clear_background(Color::new(0.0, 0.0, 0.0, 0.0));
            self.draw_cast_shadows(cam, shear);
        }
        // 1) world
        set_camera(&cam_for(&self.scene, cam, vw, vh));
        clear_background(BLACK);
        self.draw_world(cam, sun_k);

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
        // 3) glowing things above the darkness (lit windows, fireflies) and daylight beams
        set_camera(&cam_for(&self.scene, cam, vw, vh));
        self.draw_emissive(night);
        set_camera(&cam_for(&self.scene, Vec2::ZERO, vw, vh));
        self.draw_sunlight(sun_k);

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

    fn draw_world(&self, cam: Vec2, sun_k: f32) {
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

        // sun shadows (cool tint, ambient light fills them in)
        if sun_k > 0.01 {
            draw_texture_ex(
                &self.shadows.texture,
                cam.x,
                cam.y,
                Color::new(0.08, 0.1, 0.24, 0.3 * sun_k),
                DrawTextureParams { dest_size: Some(vec2(vw, vh)), ..Default::default() },
            );
        }
        // shadows first, on the ground: a wide soft shadow plus a darker contact shadow
        let shadow = |t: &Texture2D, cx: f32, cy: f32| {
            draw_texture(t, (cx - t.width() / 2.0).round(), (cy - t.height() / 2.0).round(), WHITE);
        };
        let stretch = |t: &Texture2D, cx: f32, cy: f32, w: f32, h: f32| {
            draw_texture_ex(t, (cx - w / 2.0).round(), (cy - h / 2.0).round(), WHITE, DrawTextureParams { dest_size: Some(vec2(w, h)), ..Default::default() });
        };
        let a = &self.art;
        for (_, d) in &list {
            match d {
                Dr::Obj(i) => {
                    let o = &w.objs[*i];
                    let bx = o.x as f32 * TS;
                    let by = o.y as f32 * TS;
                    match o.kind {
                        k if k.is_tree() => {
                            shadow(&a.shadow_xl, bx + 10.0, by + 13.0);
                            shadow(&a.contact, bx + 8.0, by + 15.0);
                        }
                        Kind::Stump | Kind::Bush => {
                            shadow(&a.shadow_m, bx + 8.0, by + 14.0);
                            shadow(&a.contact, bx + 8.0, by + 15.0);
                        }
                        Kind::Rock => {
                            shadow(&a.shadow_m, bx + 9.0, by + 14.0);
                            stretch(&a.contact, bx + 8.0, by + 15.0, 12.0, 4.0);
                        }
                        Kind::Boulder => {
                            shadow(&a.shadow_l, bx + 17.0, by + 29.0);
                            stretch(&a.contact, bx + 16.0, by + 31.0, 28.0, 5.0);
                        }
                        Kind::Lamp | Kind::Mailbox | Kind::Scarecrow | Kind::Planter => {
                            shadow(&a.shadow_s, bx + 9.0, by + 15.0);
                            stretch(&a.contact, bx + 8.0, by + 15.5, 8.0, 3.0);
                        }
                        Kind::Fence => stretch(&a.contact, bx + 8.0, by + 15.0, 8.0, 3.0),
                        Kind::Bin | Kind::Bench => {
                            shadow(&a.shadow_l, bx + 17.0, by + 15.0);
                            stretch(&a.contact, bx + 16.0, by + 15.5, 30.0, 4.0);
                        }
                        Kind::Well => {
                            stretch(&a.shadow_xl, bx + 17.0, by + 15.0, 40.0, 13.0);
                            stretch(&a.contact, bx + 16.0, by + 16.0, 32.0, 5.0);
                        }
                        Kind::Stall => stretch(&a.shadow_xl, bx + 25.0, by + 31.0, 54.0, 10.0),
                        Kind::Fountain => stretch(&a.shadow_xl, bx + 25.0, by + 31.0, 56.0, 12.0),
                        Kind::House | Kind::Cottage => {
                            let iw = if o.kind == Kind::House { a.house_info.w } else { a.cottage_info.w } as f32;
                            stretch(&a.shadow_xl, bx + iw / 2.0 + 3.0, by + 47.0, iw + 14.0, 12.0);
                        }
                        _ => {}
                    }
                }
                Dr::Player => {
                    shadow(&a.shadow_s, self.player.pos.x, self.player.pos.y - 1.0);
                    stretch(&a.contact, self.player.pos.x, self.player.pos.y - 0.5, 8.0, 3.0);
                }
                Dr::Chicken(i) => shadow(&a.shadow_s, self.chickens[*i].pos.x, self.chickens[*i].pos.y - 1.0),
                Dr::Drop(i) => shadow(&a.shadow_s, self.drops[*i].pos.x, self.drops[*i].pos.y),
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
                    self.draw_decals(o, true);
                    tex(t, p.x, p.y, col);
                    self.draw_decals(o, false);
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

        // cloud shadows drifting over everything
        if sun_k > 0.01 && !self.raining {
            for i in 0..7 {
                let t = &self.art.clouds[i % 2];
                let span = W as f32 * TS + 400.0;
                let x = ((i as f32 * 397.0 + self.t * 7.0) % span) - 200.0;
                let y = (i as f32 * 211.0 + (self.t * 0.05 + i as f32).sin() * 20.0) % (H as f32 * TS);
                if x + 352.0 < cam.x || x > cam.x + vw || y + 192.0 < cam.y || y > cam.y + vh {
                    continue;
                }
                draw_texture_ex(t, x.round(), y.round(), Color::new(0.06, 0.08, 0.2, 0.13 * sun_k), DrawTextureParams { dest_size: Some(vec2(352.0, 192.0)), ..Default::default() });
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

    /// Sun direction as a shear: how far a point `d` pixels above the ground is
    /// displaced (x to the side, y towards the back). Plus how strong shadows are now.
    pub fn sun(&self) -> (Vec2, f32) {
        let h = self.time / 60.0;
        let t = ((h - 6.0) / 14.0).clamp(0.0, 1.0);
        // morning: sun in the east, shadows to the west; evening the opposite
        let sx = -1.15 + 2.3 * t;
        let sy = 0.3 + (t - 0.5).abs() * 0.55;
        let day = ((h - 5.6) * 1.5).clamp(0.0, 1.0) * ((20.4 - h) * 1.2).clamp(0.0, 1.0);
        let k = day * if self.raining { 0.2 } else { 1.0 };
        (vec2(sx, sy), k)
    }

    fn cast(&self, t: &Texture2D, x: f32, y: f32, base: f32, shear: Vec2) {
        let (w, h) = (t.width(), t.height());
        let p = |px: f32, py: f32| {
            let d = base - py;
            vec2(px + shear.x * d, base - shear.y * d)
        };
        let c = [p(x, y), p(x + w, y), p(x + w, y + h), p(x, y + h)];
        let uv = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let vertices = (0..4).map(|i| Vertex::new(c[i].x, c[i].y, 0.0, uv[i].0, uv[i].1, WHITE)).collect();
        draw_mesh(&Mesh { vertices, indices: vec![0, 1, 2, 0, 2, 3], texture: Some(t.clone()) });
    }

    fn draw_cast_shadows(&self, cam: Vec2, shear: Vec2) {
        let view = Rect::new(cam.x - 200.0, cam.y - 64.0, self.vw + 400.0, self.vh + 200.0);
        let w = &self.world;
        for o in w.objs.iter().filter(|o| o.alive) {
            let (fw, fh) = o.kind.size();
            let px = o.x as f32 * TS;
            let py = o.y as f32 * TS;
            if !view.contains(vec2(px, py)) && !view.contains(vec2(px + fw as f32 * TS, py)) {
                continue;
            }
            if matches!(o.kind, Kind::Weed | Kind::Stick) {
                continue;
            }
            let (t, off) = self.obj_sprite(o);
            let base = (o.y + fh) as f32 * TS - 1.0;
            self.cast(t, px + off.x, py + off.y, base, shear);
            if o.kind == Kind::Fence {
                if w.is_fence(o.x + 1, o.y) {
                    self.cast(&self.art.fence_h, px + 8.0, py, base, shear);
                }
            }
            if o.kind == Kind::Stall {
                self.cast(&self.art.merchant.walk[0][0], px + 16.0, py - 15.0, py + 16.0, shear);
            }
        }
        let (tx0, ty0) = (((cam.x - 64.0) / TS) as i32, ((cam.y - 32.0) / TS) as i32);
        let (tx1, ty1) = (((cam.x + self.vw + 64.0) / TS) as i32, ((cam.y + self.vh + 64.0) / TS) as i32);
        for ty in ty0.max(0)..ty1.min(H) {
            for tx in tx0.max(0)..tx1.min(W) {
                if let Some(c) = w.crops[idx(tx, ty)] {
                    if c.stage() >= 2 {
                        let t = &self.art.crops[c.kind.idx()][c.stage()];
                        self.cast(t, tx as f32 * TS, ty as f32 * TS - 16.0, ty as f32 * TS + 13.0, shear);
                    }
                }
            }
        }
        if self.mode != Mode::Title {
            let p = &self.player;
            let d = p.dir as usize;
            let t = if p.moving { &self.art.farmer.walk[d][(p.anim as usize) % 4] } else { &self.art.farmer.walk[d][0] };
            self.cast(t, p.pos.x - 8.0, p.pos.y - 31.0, p.pos.y - 1.0, shear);
        }
        for c in &self.chickens {
            let t = if c.right { &self.art.chicken_r[0] } else { &self.art.chicken[0] };
            self.cast(t, c.pos.x - 8.0, c.pos.y - 15.0, c.pos.y - 1.0, shear);
        }
    }

    /// Daylight: soft beams slanting down from the sun, and a warm glow at golden hour.
    fn draw_sunlight(&self, sun_k: f32) {
        if sun_k <= 0.01 || self.raining {
            return;
        }
        let h = self.time / 60.0;
        let golden = ((h - 16.0) / 2.0).clamp(0.0, 1.0) * ((20.0 - h) / 1.2).clamp(0.0, 1.0);
        let morning = ((h - 5.8) / 1.0).clamp(0.0, 1.0) * ((10.0 - h) / 2.5).clamp(0.0, 1.0);
        let warmth = golden.max(morning);
        gl_use_material(&self.add_mat);
        let (vw, vh) = (self.vw, self.vh);
        let ang = if h < 13.0 { -0.55 } else { 0.55 };
        for i in 0..5 {
            let k = i as f32;
            let drift = (self.t * 0.03 + k * 1.7).sin() * 30.0;
            let x = vw * (0.1 + k * 0.22) + drift - self.cam.x * 0.05 % 60.0;
            let a = (0.035 + 0.03 * warmth) * sun_k * (0.6 + 0.4 * (self.t * 0.2 + k * 2.3).sin());
            let col = Color::new(1.0, 0.9 - 0.15 * warmth, 0.7 - 0.3 * warmth, a);
            let bw = 26.0 + (k * 13.0) % 30.0;
            draw_texture_ex(
                &self.art.beam,
                x - bw / 2.0,
                -vh * 0.3,
                col,
                DrawTextureParams { dest_size: Some(vec2(bw, vh * 1.6)), rotation: ang, ..Default::default() },
            );
        }
        if warmth > 0.0 {
            let r = vw.max(vh) * 0.9;
            let cx = if h < 13.0 { vw * 0.95 } else { vw * 0.05 };
            draw_texture_ex(
                &self.art.glow,
                cx - r,
                -r * 0.75,
                Color::new(1.0, 0.62, 0.3, 0.16 * warmth * sun_k),
                DrawTextureParams { dest_size: Some(vec2(r * 2.0, r * 2.0)), ..Default::default() },
            );
        }
        gl_use_default_material();
    }

    /// Grass tufts or pebbles around an object's base, so it grows out of the ground
    /// instead of floating on it. `behind` = the ones drawn before the sprite.
    fn draw_decals(&self, o: &Obj, behind: bool) {
        let anchors: &[(f32, f32)] = match (o.kind, behind) {
            (k, true) if k.is_tree() => &[(1.0, 13.0), (15.0, 13.0)],
            (Kind::Oak | Kind::Cherry, false) => &[(-4.0, 16.0), (20.0, 16.0), (7.0, 17.0)],
            (Kind::Pine, false) => &[(0.0, 16.0), (16.0, 16.0)],
            (Kind::Stump, false) => &[(0.0, 16.0), (16.0, 16.0)],
            (Kind::Rock, false) => &[(1.0, 16.0), (15.0, 16.0)],
            (Kind::Boulder, false) => &[(1.0, 32.0), (31.0, 32.0), (16.0, 33.0)],
            (Kind::Bush, false) => &[(2.0, 16.0), (14.0, 16.0)],
            (Kind::House, false) => &[(-2.0, 46.0), (6.0, 48.0), (18.0, 49.0), (30.0, 48.0), (80.0, 48.0), (92.0, 49.0), (99.0, 46.0)],
            (Kind::House, true) => &[(-4.0, 30.0), (100.0, 30.0)],
            (Kind::Cottage, false) => &[(2.0, 34.0), (12.0, 35.0), (20.0, 46.0), (60.0, 46.0), (68.0, 35.0), (79.0, 34.0)],
            (Kind::Well, false) => &[(-1.0, 16.0), (33.0, 16.0), (9.0, 18.0), (24.0, 18.0)],
            (Kind::Bin, false) => &[(0.0, 16.0), (32.0, 16.0), (16.0, 17.0)],
            (Kind::Bench, false) => &[(1.0, 16.0), (31.0, 16.0)],
            (Kind::Fence, false) if (o.x + o.y) % 2 == 0 => &[(8.0, 16.0)],
            (Kind::Mailbox | Kind::Scarecrow | Kind::Planter, false) => &[(8.0, 16.0)],
            (Kind::Stall, false) => &[(0.0, 32.0), (48.0, 32.0)],
            _ => &[],
        };
        let base = vec2(o.x as f32 * TS, o.y as f32 * TS);
        let pw = W * 16;
        for (k, &(ax, ay)) in anchors.iter().enumerate() {
            let p = base + vec2(ax, ay);
            let (px, py) = (p.x as i32, p.y as i32 - 1);
            if px < 0 || py < 0 || px >= pw || py >= H * 16 {
                continue;
            }
            let h = hash(o.x * 13 + k as i32, o.y * 7 + behind as i32, 21);
            match self.ground_px[(py * pw + px) as usize] {
                0 => {
                    let t = &self.art.tufts[(h % 8) as usize];
                    tex(t, p.x - 8.0, p.y - 9.0, WHITE);
                }
                1 if !behind && h % 3 != 0 => {
                    let t = &self.art.pebbles[(h % 4) as usize];
                    tex(t, p.x - 8.0, p.y - 6.0, WHITE);
                }
                _ => {}
            }
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

    fn building(&self, o: &Obj) -> Option<(Vec2, &crate::buildings::HouseInfo)> {
        let info = match o.kind {
            Kind::House => &self.art.house_info,
            Kind::Cottage => &self.art.cottage_info,
            _ => return None,
        };
        let top = vec2(o.x as f32 * TS, (o.y + 2) as f32 * TS - info.base_y as f32);
        Some((top, info))
    }

    /// Windows, lanterns and street lamps that light up at night: (centre, radius, colour)
    fn light_sources(&self) -> Vec<(Vec2, f32, Color)> {
        let mut v = Vec::new();
        let warm = Color::new(1.0, 0.78, 0.42, 1.0);
        for o in self.world.objs.iter().filter(|o| o.alive) {
            if let Some((top, info)) = self.building(o) {
                for &(x, y, w, h) in &info.windows {
                    let r = 18.0 + (w * h) as f32 * 0.12;
                    v.push((top + vec2(x as f32 + w as f32 / 2.0, y as f32 + h as f32 / 2.0 + 4.0), r.min(44.0), warm));
                }
                for &(x, y) in &info.lanterns {
                    v.push((top + vec2(x as f32, y as f32), 34.0, Color::new(1.0, 0.84, 0.5, 1.0)));
                }
            } else if o.kind == Kind::Lamp {
                v.push((vec2(o.x as f32 * TS + 8.0, o.y as f32 * TS - 15.0), 84.0, Color::new(1.0, 0.85, 0.5, 1.0)));
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
            if let Some((top, info)) = self.building(o) {
                for &(x, y, w, h) in &info.windows {
                    draw_rectangle(top.x + x as f32, top.y + y as f32, w as f32, h as f32, Color::new(1.0, 0.86, 0.45, k * 0.85));
                }
                for &(x, y) in &info.lanterns {
                    draw_rectangle(top.x + x as f32 - 1.0, top.y + y as f32 - 2.0, 2.0, 3.0, Color::new(1.0, 0.96, 0.75, k));
                }
            } else if o.kind == Kind::Lamp {
                let p = vec2(o.x as f32 * TS, o.y as f32 * TS);
                draw_rectangle(p.x + 5.0, p.y - 18.0, 6.0, 5.0, Color::new(1.0, 0.95, 0.7, k));
            }
        }
        let _ = lit;
        let fk = self.firefly_k();
        for f in &self.fireflies {
            let a = ((self.t * 2.3 + f.phase).sin() * 0.5 + 0.5) * fk;
            draw_rectangle(f.pos.x.round(), f.pos.y.round(), 1.0, 1.0, Color::new(0.95, 1.0, 0.6, a));
            draw_rectangle(f.pos.x.round() - 1.0, f.pos.y.round(), 3.0, 1.0, Color::new(0.8, 1.0, 0.4, a * 0.4));
        }
    }
}
