//! Mappa: terreno, oggetti, colture, generazione e "cottura" del terreno in un'unica texture.

use crate::canvas::*;
use crate::items::*;
use crate::util::*;

pub const W: i32 = 96;
pub const H: i32 = 72;
pub const TS: f32 = 16.0;
/// Oltre questa colonna c'è il paese: niente zappa lì.
pub const FARM_MAX_X: i32 = 61;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ground {
    Grass,
    Dirt,
    Water,
    Cliff,
    Stairs,
    Bridge,
    Cobble,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Oak,
    Pine,
    Cherry,
    Stump,
    Rock,
    Boulder,
    Weed,
    Stick,
    Bush,
    House,
    Cottage,
    Bin,
    Well,
    Stall,
    Fence,
    Mailbox,
    Scarecrow,
    Lamp,
    Fountain,
    Bench,
    Planter,
}

impl Kind {
    pub const ALL: [Kind; 21] = [
        Kind::Oak,
        Kind::Pine,
        Kind::Cherry,
        Kind::Stump,
        Kind::Rock,
        Kind::Boulder,
        Kind::Weed,
        Kind::Stick,
        Kind::Bush,
        Kind::House,
        Kind::Cottage,
        Kind::Bin,
        Kind::Well,
        Kind::Stall,
        Kind::Fence,
        Kind::Mailbox,
        Kind::Scarecrow,
        Kind::Lamp,
        Kind::Fountain,
        Kind::Bench,
        Kind::Planter,
    ];
    pub fn size(self) -> (i32, i32) {
        match self {
            Kind::House => (6, 2),
            Kind::Cottage => (5, 2),
            Kind::Bin | Kind::Well | Kind::Bench => (2, 1),
            Kind::Fountain => (3, 2),
            Kind::Stall => (3, 2),
            Kind::Boulder => (2, 2),
            _ => (1, 1),
        }
    }
    pub fn hp(self) -> i32 {
        match self {
            Kind::Oak | Kind::Pine | Kind::Cherry => 6,
            Kind::Stump => 4,
            Kind::Rock => 2,
            Kind::Boulder => 8,
            Kind::Weed | Kind::Stick => 1,
            _ => 0,
        }
    }
    pub fn is_tree(self) -> bool {
        matches!(self, Kind::Oak | Kind::Pine | Kind::Cherry)
    }
    pub fn code(self) -> usize {
        Kind::ALL.iter().position(|k| *k == self).unwrap()
    }
}

#[derive(Clone, Copy)]
pub struct Obj {
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub var: u32,
    pub hp: i32,
    pub shake: f32,
    pub alive: bool,
}

#[derive(Clone, Copy)]
pub struct CropState {
    pub kind: Crop,
    pub growth: u8,
}

impl CropState {
    pub fn stage(&self) -> usize {
        let d = self.kind.days();
        if self.growth >= d {
            4
        } else if self.growth == 0 {
            0
        } else {
            (1 + (self.growth as usize - 1) * 3 / (d as usize - 1).max(1)).min(3)
        }
    }
    pub fn ripe(&self) -> bool {
        self.growth >= self.kind.days()
    }
}

pub struct World {
    pub ground: Vec<Ground>,
    pub tilled: Vec<bool>,
    pub wet: Vec<bool>,
    pub crops: Vec<Option<CropState>>,
    pub objs: Vec<Obj>,
    pub occ: Vec<i32>,
    reserved: Vec<bool>,
}

#[inline]
pub fn idx(x: i32, y: i32) -> usize {
    (y * W + x) as usize
}
#[inline]
pub fn inb(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < W && y < H
}

impl World {
    pub fn g(&self, x: i32, y: i32) -> Ground {
        if inb(x, y) {
            self.ground[idx(x, y)]
        } else {
            Ground::Grass
        }
    }

    pub fn obj_at(&self, x: i32, y: i32) -> Option<usize> {
        if !inb(x, y) {
            return None;
        }
        let o = self.occ[idx(x, y)];
        if o >= 0 {
            Some(o as usize)
        } else {
            None
        }
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        if !inb(x, y) {
            return false;
        }
        let g = self.ground[idx(x, y)];
        !matches!(g, Ground::Water | Ground::Cliff) && self.occ[idx(x, y)] < 0
    }

    pub fn farmable(&self, x: i32, y: i32) -> bool {
        inb(x, y) && x <= FARM_MAX_X && matches!(self.g(x, y), Ground::Grass | Ground::Dirt) && self.occ[idx(x, y)] < 0
    }

    pub fn is_fence(&self, x: i32, y: i32) -> bool {
        self.obj_at(x, y).map(|i| self.objs[i].kind == Kind::Fence).unwrap_or(false)
    }

    pub fn can_place(&self, kind: Kind, x: i32, y: i32) -> bool {
        let (w, h) = kind.size();
        for yy in y..y + h {
            for xx in x..x + w {
                if !inb(xx, yy) || self.occ[idx(xx, yy)] >= 0 || self.tilled[idx(xx, yy)] {
                    return false;
                }
                if matches!(self.g(xx, yy), Ground::Water | Ground::Cliff | Ground::Stairs | Ground::Bridge) {
                    return false;
                }
            }
        }
        true
    }

    pub fn place(&mut self, kind: Kind, x: i32, y: i32, var: u32) -> bool {
        if !self.can_place(kind, x, y) {
            return false;
        }
        let (w, h) = kind.size();
        let id = self.objs.len() as i32;
        self.objs.push(Obj { kind, x, y, var, hp: kind.hp(), shake: 0.0, alive: true });
        for yy in y..y + h {
            for xx in x..x + w {
                self.occ[idx(xx, yy)] = id;
            }
        }
        true
    }

    fn place_free(&mut self, kind: Kind, x: i32, y: i32, var: u32) -> bool {
        let (w, h) = kind.size();
        for yy in y..y + h {
            for xx in x..x + w {
                if !inb(xx, yy) || self.reserved[idx(xx, yy)] {
                    return false;
                }
            }
        }
        self.place(kind, x, y, var)
    }

    pub fn remove(&mut self, i: usize) {
        let o = self.objs[i];
        let (w, h) = o.kind.size();
        for yy in o.y..o.y + h {
            for xx in o.x..o.x + w {
                if inb(xx, yy) && self.occ[idx(xx, yy)] == i as i32 {
                    self.occ[idx(xx, yy)] = -1;
                }
            }
        }
        self.objs[i].alive = false;
    }

    /// Ricostruisce oggetti (usato dal caricamento salvataggi)
    pub fn set_objects(&mut self, list: Vec<Obj>) {
        self.objs.clear();
        self.occ.iter_mut().for_each(|o| *o = -1);
        for o in list {
            let (w, h) = o.kind.size();
            let id = self.objs.len() as i32;
            self.objs.push(o);
            for yy in o.y..o.y + h {
                for xx in o.x..o.x + w {
                    if inb(xx, yy) {
                        self.occ[idx(xx, yy)] = id;
                    }
                }
            }
        }
    }

    pub fn find(&self, kind: Kind) -> Option<&Obj> {
        self.objs.iter().find(|o| o.alive && o.kind == kind)
    }

    // -------------------------------------------------------------- generazione

    pub fn generate(seed: u64) -> World {
        let n = (W * H) as usize;
        let mut w = World {
            ground: vec![Ground::Grass; n],
            tilled: vec![false; n],
            wet: vec![false; n],
            crops: vec![None; n],
            objs: Vec::new(),
            occ: vec![-1; n],
            reserved: vec![false; n],
        };
        let mut r = Rng::new(seed);
        let set = |w: &mut World, x: i32, y: i32, g: Ground| {
            if inb(x, y) {
                w.ground[idx(x, y)] = g;
            }
        };
        let reserve = |w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32| {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if inb(x, y) {
                        w.reserved[idx(x, y)] = true;
                    }
                }
            }
        };

        // terra della fattoria (macchie organiche)
        for y in 21..53 {
            for x in 18..61 {
                let nz = fbm(x as f32 * 0.16, y as f32 * 0.16, 7);
                if nz < 0.56 {
                    set(&mut w, x, y, Ground::Dirt);
                }
            }
        }
        // cortile davanti a casa
        for y in 19..25 {
            for x in 33..54 {
                if !(y == 24 && (x < 36 || x > 51)) {
                    set(&mut w, x, y, Ground::Dirt);
                }
            }
        }
        // sentieri
        let path = |w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32| {
            for y in y0.min(y1)..=y0.max(y1) {
                for x in x0.min(x1)..=x0.max(x1) {
                    if inb(x, y) {
                        w.ground[idx(x, y)] = Ground::Dirt;
                        w.reserved[idx(x, y)] = true;
                    }
                }
            }
        };
        path(&mut w, 43, 24, 44, 35); // casa -> sud
        path(&mut w, 43, 34, 63, 35); // verso il ponte
        path(&mut w, 28, 14, 29, 20); // scale -> cortile
        path(&mut w, 28, 19, 36, 20);
        path(&mut w, 69, 34, 76, 35); // ponte -> piazza
        path(&mut w, 43, 35, 44, 60); // verso il prato a sud
        reserve(&mut w, 32, 13, 54, 24); // area di casa libera
        reserve(&mut w, 36, 25, 50, 29); // primo campo già ripulito

        // stagno a ovest
        for y in 30..50 {
            for x in 2..22 {
                let dx = (x as f32 + 0.5 - 10.5) / 6.8;
                let dy = (y as f32 + 0.5 - 39.5) / 4.8;
                let d = dx * dx + dy * dy + (fbm(x as f32 * 0.3, y as f32 * 0.3, 31) - 0.5) * 0.6;
                if d < 1.0 {
                    set(&mut w, x, y, Ground::Water);
                }
            }
        }
        // laghetto a sud-est
        for y in 56..68 {
            for x in 50..64 {
                let dx = (x as f32 + 0.5 - 56.0) / 4.6;
                let dy = (y as f32 + 0.5 - 61.5) / 3.2;
                let d = dx * dx + dy * dy + (fbm(x as f32 * 0.35, y as f32 * 0.35, 37) - 0.5) * 0.5;
                if d < 1.0 {
                    set(&mut w, x, y, Ground::Water);
                }
            }
        }
        // fiume
        for y in 0..H {
            let cx = 66.0 + (y as f32 * 0.13).sin() * 2.0 + (y as f32 * 0.047).cos() * 1.4;
            for x in 56..76 {
                if (x as f32 + 0.5 - cx).abs() < 2.5 {
                    set(&mut w, x, y, Ground::Water);
                }
            }
        }
        // ponte
        for y in 34..36 {
            for x in 58..76 {
                if w.g(x, y) == Ground::Water {
                    set(&mut w, x, y, Ground::Bridge);
                }
            }
        }
        // altopiano con scogliera
        for x in 0..W {
            for y in 12..14 {
                if w.g(x, y) == Ground::Grass && x < 61 {
                    set(&mut w, x, y, if x == 28 || x == 29 { Ground::Stairs } else { Ground::Cliff });
                }
            }
        }
        // piazza del paese
        for y in 27..42 {
            for x in 74..91 {
                set(&mut w, x, y, Ground::Cobble);
                w.reserved[idx(x, y)] = true;
            }
        }
        path(&mut w, 85, 24, 86, 26);

        // --- edifici
        w.place(Kind::House, 38, 17, 0);
        w.place(Kind::Bin, 45, 18, 0);
        w.place(Kind::Mailbox, 36, 19, 0);
        w.place(Kind::Well, 50, 19, 0);
        w.place(Kind::Scarecrow, 31, 31, 0);
        w.place(Kind::Stall, 80, 30, 0);
        w.place(Kind::Cottage, 83, 22, 0);
        for (x, y) in [(75, 28), (89, 28), (75, 40), (89, 40)] {
            w.place(Kind::Lamp, x, y, 0);
        }
        w.place(Kind::Fountain, 80, 35, 0);
        for (x, y) in [(76, 35), (86, 35), (79, 39), (84, 39)] {
            w.place(Kind::Bench, x, y, 0);
        }
        for (i, (x, y)) in [(78, 28), (86, 28), (76, 31), (88, 31), (77, 40), (87, 40), (82, 25), (88, 24)].iter().enumerate() {
            w.place(Kind::Planter, *x, *y, i as u32);
        }
        // cespugli ai lati della casa
        for (x, y, v) in [(37, 17, 0), (44, 17, 1), (37, 18, 1), (55, 22, 0), (32, 21, 1)] {
            w.place(Kind::Bush, x, y, v);
        }
        // staccionata della fattoria
        for x in 16..61 {
            if !(42..=45).contains(&x) {
                w.place(Kind::Fence, x, 53, 0);
            }
        }
        for y in 21..54 {
            if !(38..=41).contains(&y) {
                w.place(Kind::Fence, 16, y, 0);
            }
        }

        // --- vegetazione
        for y in 0..H {
            for x in 0..W {
                let edge = x < 3 || x >= W - 2 || y >= H - 3 || y < 2;
                let plateau = y < 12 && x < 61;
                let town = x >= 71;
                let farm = (18..61).contains(&x) && (21..53).contains(&y);
                let meadow = y >= 54;
                let west = x < 16 && y >= 14;
                let v = r.range(0, 3) as u32;
                let tree = match r.range(0, 10) {
                    0..=4 => Kind::Pine,
                    5..=8 => Kind::Oak,
                    _ => Kind::Cherry,
                };
                if edge {
                    if r.chance(0.75) {
                        w.place_free(if r.chance(0.6) { Kind::Pine } else { Kind::Oak }, x, y, v);
                    }
                } else if plateau {
                    let f = r.f();
                    if f < 0.22 {
                        w.place_free(tree, x, y, v);
                    } else if f < 0.27 {
                        w.place_free(Kind::Bush, x, y, v);
                    } else if f < 0.31 {
                        w.place_free(Kind::Rock, x, y, v);
                    } else if f < 0.36 {
                        w.place_free(Kind::Weed, x, y, v);
                    }
                } else if town {
                    if w.g(x, y) == Ground::Grass && r.chance(0.07) {
                        w.place_free(if r.chance(0.5) { Kind::Cherry } else { tree }, x, y, v);
                    } else if r.chance(0.02) {
                        w.place_free(Kind::Bush, x, y, v);
                    }
                } else if farm {
                    let f = r.f();
                    if f < 0.004 {
                        w.place_free(Kind::Boulder, x, y, 0);
                    } else if f < 0.03 {
                        w.place_free(if r.chance(0.5) { Kind::Oak } else { Kind::Pine }, x, y, v);
                    } else if f < 0.2 {
                        w.place_free(Kind::Weed, x, y, v);
                    } else if f < 0.26 {
                        w.place_free(Kind::Rock, x, y, v);
                    } else if f < 0.31 {
                        w.place_free(Kind::Stick, x, y, v % 2);
                    }
                } else if meadow || west {
                    let f = r.f();
                    if f < 0.07 {
                        w.place_free(tree, x, y, v);
                    } else if f < 0.1 {
                        w.place_free(Kind::Bush, x, y, v);
                    } else if f < 0.14 {
                        w.place_free(Kind::Weed, x, y, v);
                    } else if f < 0.15 {
                        w.place_free(Kind::Rock, x, y, v);
                    }
                } else if r.chance(0.05) {
                    w.place_free(Kind::Weed, x, y, v);
                }
            }
        }
        // primo campo zappato per iniziare
        for y in 26..28 {
            for x in 38..42 {
                w.tilled[idx(x, y)] = true;
            }
        }
        w
    }

    // -------------------------------------------------------------- cottura del terreno

    pub fn bake(&self) -> Canvas {
        use Ground::*;
        let pw = W * 16;
        let ph = H * 16;
        let code = |g: Ground| g as u8;
        let mut tm = vec![0u8; (pw * ph) as usize];
        // Campi continui (interpolazione bilineare tra i centri delle caselle + rumore):
        // bordi organici e arrotondati invece che a quadretti.
        let wv = |g: Ground| if g == Water { 1.0 } else { 0.0 };
        let gv = |g: Ground| match g {
            Grass | Water | Cliff => 1.0,
            Dirt | Stairs | Bridge => 0.0,
            Cobble => 0.35,
        };
        let bilerp = |fx: f32, fy: f32, f: &dyn Fn(Ground) -> f32| {
            let x0 = fx.floor() as i32;
            let y0 = fy.floor() as i32;
            let tx = fx - x0 as f32;
            let ty = fy - y0 as f32;
            let a = f(self.g(x0, y0));
            let b = f(self.g(x0 + 1, y0));
            let c = f(self.g(x0, y0 + 1));
            let d = f(self.g(x0 + 1, y0 + 1));
            let top = a + (b - a) * tx;
            let bot = c + (d - c) * tx;
            top + (bot - top) * ty
        };
        for py in 0..ph {
            for px in 0..pw {
                let g = self.g(px / 16, py / 16);
                let t = match g {
                    Grass | Dirt | Water => {
                        let fx = px as f32 / 16.0 - 0.5;
                        let fy = py as f32 / 16.0 - 0.5;
                        let n1 = vnoise(px as f32 * 0.09, py as f32 * 0.09, 11) - 0.5;
                        let n2 = vnoise(px as f32 * 0.27, py as f32 * 0.27, 13) - 0.5;
                        let w = bilerp(fx, fy, &wv) + n1 * 0.5 + n2 * 0.14;
                        if w > 0.5 {
                            Water
                        } else {
                            let n3 = vnoise(px as f32 * 0.07, py as f32 * 0.07, 17) - 0.5;
                            let n4 = vnoise(px as f32 * 0.22, py as f32 * 0.22, 19) - 0.5;
                            let v = bilerp(fx, fy, &gv) + n3 * 0.62 + n4 * 0.18;
                            if v > 0.5 {
                                Grass
                            } else {
                                Dirt
                            }
                        }
                    }
                    _ => g,
                };
                tm[(py * pw + px) as usize] = code(t);
            }
        }
        let all = [Grass, Dirt, Water, Cliff, Stairs, Bridge, Cobble];
        let tt = |x: i32, y: i32| -> Ground {
            let x = x.clamp(0, pw - 1);
            let y = y.clamp(0, ph - 1);
            all[tm[(y * pw + x) as usize] as usize]
        };
        let landish = |g: Ground| !matches!(g, Water | Bridge);
        let mut c = Canvas::new(pw, ph);
        for py in 0..ph {
            for px in 0..pw {
                let t = tt(px, py);
                let jit = (hashf(px, py, 1) - 0.5) * 0.1;
                let cliff_above = (1..=5).any(|k| tt(px, py - k) == Cliff);
                let col = match t {
                    Grass => {
                        let n = fbm(px as f32 * 0.035, py as f32 * 0.035, 3) + jit;
                        let mut col = if n > 0.63 { G[3] } else if n < 0.40 { G[1] } else { G[2] };
                        let h = hashf(px, py, 7);
                        if h < 0.035 {
                            col = G[3];
                        } else if h > 0.972 {
                            col = G[1];
                        }
                        let b1 = tt(px, py + 1);
                        let b2 = tt(px, py + 2);
                        if b1 != Grass && b1 != Cliff && b1 != Stairs {
                            col = G[0];
                        } else if b2 != Grass && b2 != Cliff && b2 != Stairs {
                            col = G[1];
                        } else if tt(px, py - 1) != Grass && tt(px, py - 1) != Cliff {
                            col = G[4];
                        } else if tt(px - 1, py) != Grass || tt(px + 1, py) != Grass {
                            col = G[1];
                        }
                        if cliff_above {
                            col = shade(col, 0.74);
                        }
                        col
                    }
                    Dirt => {
                        let n = fbm(px as f32 * 0.04, py as f32 * 0.04, 5) + jit;
                        let mut col = if n > 0.64 { D[3] } else if n < 0.38 { D[1] } else { D[2] };
                        let h = hashf(px, py, 8);
                        if h < 0.025 {
                            col = D[1];
                        } else if h > 0.985 {
                            col = D[4];
                        }
                        if tt(px, py - 1) == Grass {
                            col = D[0];
                        } else if tt(px, py - 2) == Grass {
                            col = D[1];
                        } else if tt(px, py - 3) == Grass {
                            col = shade(col, 0.93);
                        }
                        if cliff_above {
                            col = shade(col, 0.74);
                        }
                        col
                    }
                    Cobble => {
                        let row = py / 5;
                        let off = (row % 2) * 3;
                        let col_i = (px + off) / 7;
                        let lx = (px + off) % 7;
                        let ly = py % 5;
                        let tone = hashf(col_i, row, 17);
                        let mut col = if lx == 0 || ly == 0 {
                            COBBLE[0]
                        } else if tone < 0.35 {
                            COBBLE[1]
                        } else if tone < 0.85 {
                            COBBLE[2]
                        } else {
                            COBBLE[3]
                        };
                        if lx == 1 && ly == 1 {
                            col = COBBLE[3];
                        } else if ly == 4 && lx != 0 {
                            col = shade(col, 0.88);
                        }
                        if tt(px, py - 1) != Cobble || tt(px - 1, py) != Cobble || tt(px + 1, py) != Cobble || tt(px, py + 1) != Cobble {
                            col = COBBLE[0];
                        }
                        col
                    }
                    Water => {
                        let mut col;
                        // parete della sponda sotto la terra
                        let bank = (1..=3).find(|&k| landish(tt(px, py - k)));
                        let bridge_above = (1..=4).any(|k| tt(px, py - k) == Bridge);
                        if let Some(k) = bank {
                            col = match k {
                                1 => D[1],
                                2 => D[0],
                                _ => WAT[0],
                            };
                        } else {
                            let near = |d: i32| {
                                [(d, 0), (-d, 0), (0, d), (0, -d), (d, d), (-d, d), (d, -d), (-d, -d)]
                                    .iter()
                                    .any(|&(dx, dy)| landish(tt(px + dx, py + dy)))
                            };
                            col = if near(1) {
                                WAT[5]
                            } else if near(2) {
                                WAT[4]
                            } else if near(5) {
                                WAT[3]
                            } else if near(10) {
                                WAT[2]
                            } else {
                                let n = fbm(px as f32 * 0.05, py as f32 * 0.05, 23);
                                if n < 0.45 {
                                    WAT[1]
                                } else {
                                    WAT[2]
                                }
                            };
                            if tt(px, py - 4) != Water && !landish(tt(px, py - 4)) {
                                // niente
                            }
                        }
                        if bridge_above {
                            col = shade(col, 0.6);
                        }
                        // pali del ponte
                        if (1..=4).any(|k| tt(px, py + k) == Bridge) && px.rem_euclid(24) < 3 {
                            col = if px.rem_euclid(24) == 0 { WOOD[3] } else { WOOD[1] };
                        }
                        col
                    }
                    Cliff => {
                        let tx = px / 16;
                        let mut top = py / 16;
                        while top > 0 && self.g(tx, top - 1) == Cliff {
                            top -= 1;
                        }
                        let mut bot = py / 16;
                        while bot < H - 1 && self.g(tx, bot + 1) == Cliff {
                            bot += 1;
                        }
                        let local = py - top * 16;
                        let from_bot = (bot + 1) * 16 - 1 - py;
                        let lip = 4 + (vnoise(px as f32 * 0.35, 3.0, 41) * 4.0) as i32;
                        if local < lip {
                            if local == lip - 1 {
                                G[0]
                            } else if local == 0 {
                                G[4]
                            } else if hashf(px, py, 3) < 0.2 {
                                G[3]
                            } else {
                                G[2]
                            }
                        } else {
                            let wob = vnoise(py as f32 * 0.15, px as f32 * 0.02, 43) * 4.0;
                            let ci = ((px as f32 + wob) / 6.0).floor() as i32;
                            let xi = ((px as f32 + wob) as i32).rem_euclid(6);
                            let tone = hashf(ci, 0, 44);
                            let mut col = if xi == 0 {
                                CLIFF[0]
                            } else if xi == 1 {
                                CLIFF[4]
                            } else if tone < 0.4 {
                                CLIFF[2]
                            } else {
                                CLIFF[3]
                            };
                            let ledge = 15 + ci.rem_euclid(4);
                            if local == ledge {
                                col = CLIFF[1];
                            } else if local == ledge - 1 {
                                col = CLIFF[4];
                            }
                            if local == lip {
                                col = CLIFF[0];
                            } else if local == lip + 1 {
                                col = CLIFF[1];
                            }
                            if from_bot < 3 {
                                col = shade(col, 0.72);
                            }
                            col
                        }
                    }
                    Stairs => {
                        let sx = px - (px / 16) * 16 + if self.g(px / 16 - 1, py / 16) == Stairs { 16 } else { 0 };
                        if sx < 2 || sx >= 30 {
                            STONE[1]
                        } else {
                            match (py - 12 * 16).rem_euclid(6) {
                                0 => STONE[4],
                                1..=3 => STONE[3],
                                4 => STONE[2],
                                _ => STONE[0],
                            }
                        }
                    }
                    Bridge => {
                        let mut top = py / 16;
                        while top > 0 && self.g(px / 16, top - 1) == Bridge {
                            top -= 1;
                        }
                        let by = py - top * 16;
                        if by < 3 {
                            if by == 0 {
                                WOOD[3]
                            } else {
                                WOOD[1]
                            }
                        } else if by >= 29 {
                            if by == 31 {
                                WOOD[0]
                            } else {
                                WOOD[1]
                            }
                        } else {
                            let pl = px / 4;
                            if px % 4 == 3 {
                                WOOD[1]
                            } else if (by == 6 || by == 25) && px % 4 == 1 {
                                WOOD[0]
                            } else if hashf(pl, 0, 61) < 0.4 {
                                WOOD[2]
                            } else if by == 3 {
                                WOOD[4]
                            } else {
                                WOOD[3]
                            }
                        }
                    }
                };
                c.set(px, py, col);
            }
        }
        // ---- dettagli: ciuffi, fiori, sassolini
        let mut r = Rng::new(99);
        for ty in 0..H {
            for tx in 0..W {
                let g = self.g(tx, ty);
                let bx = tx * 16;
                let by = ty * 16;
                if g == Grass {
                    for _ in 0..3 {
                        let x = bx + r.range(1, 15);
                        let y = by + r.range(3, 15);
                        if c.get(x, y) == G[2] || c.get(x, y) == G[1] || c.get(x, y) == G[3] {
                            c.set(x, y, G[1]);
                            c.set(x + 1, y, G[1]);
                            c.set(x - 1, y - 1, G[3]);
                            c.set(x, y - 1, G[3]);
                            c.set(x + 1, y - 2, G[4]);
                        }
                    }
                    if r.chance(0.16) {
                        let x = bx + r.range(2, 14);
                        let y = by + r.range(3, 13);
                        let fc = [hex(0xfff6f0), hex(0xf4a6c8), hex(0xa890e8), hex(0xffe070), hex(0x8ec8f4)][r.range(0, 5) as usize];
                        if tt(x, y) == Grass && tt(x, y + 3) == Grass {
                            c.set(x, y + 1, G[0]);
                            c.set(x, y + 2, G[1]);
                            c.set(x - 1, y, fc);
                            c.set(x + 1, y, fc);
                            c.set(x, y - 1, fc);
                            c.set(x, y + 1, shade(fc, 0.8));
                            c.set(x, y, hex(0xf8c838));
                        }
                    }
                } else if g == Dirt && r.chance(0.35) {
                    let x = bx + r.range(2, 13);
                    let y = by + r.range(2, 13);
                    if tt(x, y) == Dirt && tt(x + 1, y) == Dirt {
                        c.set(x, y, hex(0xb08654));
                        c.set(x + 1, y, hex(0x9a7044));
                        c.set(x, y - 1, D[4]);
                        c.set(x, y + 1, D[0]);
                        c.set(x + 1, y + 1, D[0]);
                    }
                }
            }
        }
        c
    }
}
