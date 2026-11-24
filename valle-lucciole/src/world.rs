//! Mappa: terreno, oggetti, colture, generazione e "cottura" del terreno in un'unica texture.

use crate::canvas::*;
use crate::items::*;
use crate::util::*;

pub const W: i32 = 96;
pub const H: i32 = 72;
pub const TS: f32 = 16.0;
/// Oltre questa colonna c'è il paese: niente zappa lì.
pub const FARM_MAX_X: i32 = 61;

const TEAL: [Rgba; 6] = [hex(0x1b6670), hex(0x227c86), hex(0x2c939c), hex(0x45abb0), hex(0x96d8d0), hex(0xdcf8ee)];
const FLAG: [Rgba; 5] = [hex(0x9a6e40), hex(0xc49a64), hex(0xd9b37c), hex(0xeacb94), hex(0xfbe4b4)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ground {
    Grass,
    Dirt,
    Water,
    Cliff,
    Stairs,
    Bridge,
    Cobble,
    /// warm flagstone path
    Path,
    /// wooden dock over water
    Deck,
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
    Maple,
    Apple,
    Hedge,
    Keg,
    Crate,
    HayBale,
    Brazier,
    Coop,
    FlowerBed,
    Mushroom,
    TallGrass,
    Sign,
}

impl Kind {
    pub const ALL: [Kind; 33] = [
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
        Kind::Maple,
        Kind::Apple,
        Kind::Hedge,
        Kind::Keg,
        Kind::Crate,
        Kind::HayBale,
        Kind::Brazier,
        Kind::Coop,
        Kind::FlowerBed,
        Kind::Mushroom,
        Kind::TallGrass,
        Kind::Sign,
    ];
    pub fn size(self) -> (i32, i32) {
        match self {
            Kind::House => (6, 2),
            Kind::Cottage => (5, 2),
            Kind::Bin | Kind::Well | Kind::Bench | Kind::Hedge => (2, 1),
            Kind::Fountain => (3, 2),
            Kind::Stall => (3, 2),
            Kind::Boulder => (2, 2),
            Kind::Coop => (4, 2),
            _ => (1, 1),
        }
    }
    pub fn hp(self) -> i32 {
        match self {
            Kind::Oak | Kind::Pine | Kind::Cherry | Kind::Maple | Kind::Apple => 6,
            Kind::Stump => 4,
            Kind::Rock => 2,
            Kind::Boulder => 8,
            Kind::Weed | Kind::Stick | Kind::TallGrass | Kind::Mushroom => 1,
            _ => 0,
        }
    }
    pub fn is_tree(self) -> bool {
        matches!(self, Kind::Oak | Kind::Pine | Kind::Cherry | Kind::Maple | Kind::Apple)
    }
    /// Can you walk through it?
    pub fn blocks(self) -> bool {
        !matches!(self, Kind::TallGrass | Kind::Mushroom)
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
    /// water tiles of ponds edged with stones
    pub rim: Vec<bool>,
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

/// Jittered-grid cells used for flagstones and rim stones:
/// returns (cell centre, distance to nearest centre, to second nearest, cell id).
pub fn voronoi(px: f32, py: f32, size: f32, seed: u32) -> (f32, f32, f32, f32, i32) {
    let gx = (px / size).floor() as i32;
    let gy = (py / size).floor() as i32;
    let (mut d1, mut d2) = (1e9f32, 1e9f32);
    let (mut cx, mut cy, mut id) = (0.0, 0.0, 0);
    for oy in -1..=1 {
        for ox in -1..=1 {
            let (x, y) = (gx + ox, gy + oy);
            let jx = (x as f32 + 0.15 + 0.7 * hashf(x, y, seed)) * size;
            let jy = (y as f32 + 0.15 + 0.7 * hashf(x, y, seed + 1)) * size;
            let d = ((px - jx).powi(2) + (py - jy).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
                cx = jx;
                cy = jy;
                id = (hash(x, y, seed + 2) & 0xffff) as i32;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (cx, cy, d1, d2, id)
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
        let o = self.occ[idx(x, y)];
        !matches!(g, Ground::Water | Ground::Cliff) && (o < 0 || !self.objs[o as usize].kind.blocks())
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
                if matches!(self.g(xx, yy), Ground::Water | Ground::Cliff | Ground::Stairs | Ground::Bridge | Ground::Deck) {
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
            rim: vec![false; n],
            reserved: vec![false; n],
        };
        let mut r = Rng::new(seed);
        fn fill(w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32, g: Ground, reserve: bool) {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if inb(x, y) {
                        w.ground[idx(x, y)] = g;
                        if reserve {
                            w.reserved[idx(x, y)] = true;
                        }
                    }
                }
            }
        }
        fn reserve(w: &mut World, x0: i32, y0: i32, x1: i32, y1: i32) {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if inb(x, y) {
                        w.reserved[idx(x, y)] = true;
                    }
                }
            }
        }
        fn pond(w: &mut World, cx: f32, cy: f32, rx: f32, ry: f32, wob: f32, seed: u32, rim: bool) {
            for y in (cy - ry - 3.0) as i32..=(cy + ry + 3.0) as i32 {
                for x in (cx - rx - 3.0) as i32..=(cx + rx + 3.0) as i32 {
                    if !inb(x, y) {
                        continue;
                    }
                    let dx = (x as f32 + 0.5 - cx) / rx;
                    let dy = (y as f32 + 0.5 - cy) / ry;
                    // rimmed ponds are squarer, natural ones rounder
                    let d = if rim { dx.abs().powf(3.0) + dy.abs().powf(3.0) } else { dx * dx + dy * dy };
                    let d = d + (fbm(x as f32 * 0.3, y as f32 * 0.3, seed) - 0.5) * wob;
                    if d < 1.0 {
                        w.ground[idx(x, y)] = Ground::Water;
                        w.rim[idx(x, y)] = rim;
                        w.reserved[idx(x, y)] = true;
                    }
                }
            }
        }

        // --- soil: sandy farmland with grassy patches, tidy dirt in the yard and garden
        for y in 21..53 {
            for x in 18..61 {
                if fbm(x as f32 * 0.16, y as f32 * 0.16, 7) < 0.56 {
                    w.ground[idx(x, y)] = Ground::Dirt;
                }
            }
        }
        fill(&mut w, 29, 18, 50, 23, Ground::Dirt, false);
        fill(&mut w, 34, 25, 49, 31, Ground::Dirt, false);
        fill(&mut w, 52, 18, 57, 22, Ground::Dirt, false);

        // --- water
        pond(&mut w, 11.5, 32.5, 6.4, 4.6, 0.25, 31, true);
        pond(&mut w, 54.5, 61.5, 5.2, 3.4, 0.5, 37, false);
        pond(&mut w, 18.0, 6.5, 3.2, 2.2, 0.4, 41, false);

        // --- flagstone paths
        fill(&mut w, 36, 20, 47, 21, Ground::Path, true); // patio in front of the house
        fill(&mut w, 41, 20, 42, 35, Ground::Path, true); // house -> crossroads
        fill(&mut w, 20, 34, 63, 35, Ground::Path, true); // main road to the bridge
        fill(&mut w, 20, 31, 21, 33, Ground::Path, true); // to the dock
        fill(&mut w, 28, 14, 29, 21, Ground::Path, true); // stairs -> yard
        fill(&mut w, 28, 20, 35, 21, Ground::Path, true);
        fill(&mut w, 69, 34, 76, 35, Ground::Path, true); // bridge -> square
        fill(&mut w, 85, 24, 86, 26, Ground::Path, true); // cottage
        fill(&mut w, 41, 36, 42, 53, Ground::Path, true); // to the meadow
        fill(&mut w, 41, 54, 42, 62, Ground::Dirt, true);

        // --- river and bridge
        for y in 0..H {
            let cx = 66.0 + (y as f32 * 0.13).sin() * 2.0 + (y as f32 * 0.047).cos() * 1.4;
            for x in 56..76 {
                if (x as f32 + 0.5 - cx).abs() < 2.5 {
                    w.ground[idx(x, y)] = Ground::Water;
                }
            }
        }
        for y in 34..36 {
            for x in 58..76 {
                if w.g(x, y) == Ground::Water {
                    w.ground[idx(x, y)] = Ground::Bridge;
                }
            }
        }
        // --- dock on the pond
        fill(&mut w, 13, 31, 19, 32, Ground::Deck, true);
        // --- plateau and cliff
        for x in 0..61 {
            for y in 12..14 {
                if w.g(x, y) != Ground::Water {
                    w.ground[idx(x, y)] = if x == 28 || x == 29 { Ground::Stairs } else { Ground::Cliff };
                }
            }
        }
        // --- town square
        fill(&mut w, 74, 27, 90, 41, Ground::Cobble, true);

        // --- buildings and the farm yard
        w.place(Kind::House, 38, 17, 0);
        w.place(Kind::Bin, 45, 18, 0);
        w.place(Kind::Mailbox, 36, 19, 0);
        w.place(Kind::Well, 48, 20, 0);
        w.place(Kind::Coop, 52, 16, 0);
        w.place(Kind::Sign, 39, 36, 0);
        for (i, x) in (31..36).enumerate() {
            w.place(if i == 0 { Kind::Crate } else { Kind::Keg }, x, 18, i as u32);
        }
        w.place(Kind::Crate, 30, 18, 1);
        w.place(Kind::HayBale, 56, 17, 0);
        w.place(Kind::HayBale, 57, 17, 1);
        w.place(Kind::HayBale, 57, 18, 2);
        for (x, y) in [(40, 23), (43, 23)] {
            w.place(Kind::Brazier, x, y, 0);
        }
        for (i, x) in [35, 36, 37, 38, 45, 46, 47, 48].iter().enumerate() {
            w.place(Kind::FlowerBed, *x, 22, i as u32);
        }
        for (x, y) in [(40, 33), (43, 33), (22, 33), (62, 33)] {
            w.place(Kind::Lamp, x, y, 0);
        }
        w.place(Kind::Bench, 23, 30, 0);
        w.place(Kind::Scarecrow, 45, 25, 0);
        for (x, y, v) in [(37, 17, 0), (44, 17, 1), (37, 18, 1)] {
            w.place(Kind::Bush, x, y, v);
        }
        // chicken yard
        for y in 16..24 {
            for x in [51, 58] {
                w.place(Kind::Fence, x, y, 0);
            }
        }
        for x in [56, 57] {
            w.place(Kind::Fence, x, 16, 0);
        }
        for x in 51..59 {
            if !(53..=54).contains(&x) {
                w.place(Kind::Fence, x, 23, 0);
            }
        }
        reserve(&mut w, 52, 18, 57, 22);
        // garden
        for x in 33..51 {
            for y in [24, 32] {
                if !(41..=42).contains(&x) {
                    w.place(Kind::Fence, x, y, 0);
                }
            }
        }
        for y in 25..32 {
            w.place(Kind::Fence, 33, y, 0);
            w.place(Kind::Fence, 50, y, 0);
        }
        reserve(&mut w, 29, 14, 50, 31);
        // orchard
        for (i, (x, y)) in [(47, 38), (51, 38), (55, 38), (49, 42), (53, 42), (57, 42)].iter().enumerate() {
            w.place(Kind::Apple, *x, *y, i as u32);
        }
        reserve(&mut w, 45, 37, 59, 44);
        // farm boundary fence
        for x in 16..61 {
            if !(40..=43).contains(&x) {
                w.place(Kind::Fence, x, 53, 0);
            }
        }

        // --- the town
        w.place(Kind::Stall, 80, 30, 0);
        w.place(Kind::Cottage, 83, 22, 0);
        for (x, y) in [(75, 28), (89, 28), (75, 40), (89, 40)] {
            w.place(Kind::Lamp, x, y, 0);
        }
        w.place(Kind::Fountain, 80, 35, 0);
        for (x, y) in [(76, 35), (86, 35), (79, 39), (84, 39)] {
            w.place(Kind::Bench, x, y, 0);
        }
        for (i, (x, y)) in [(78, 28), (86, 28), (76, 31), (88, 31), (77, 40), (87, 40)].iter().enumerate() {
            w.place(Kind::Planter, *x, *y, i as u32);
        }
        for (i, x) in [80, 81, 82, 88, 89].iter().enumerate() {
            w.place(Kind::FlowerBed, *x, 25, i as u32 + 3);
        }
        reserve(&mut w, 79, 19, 90, 26);

        // --- nature
        for y in 0..H {
            for x in 0..W {
                let edge = x < 3 || x >= W - 2 || y >= H - 3 || y < 2;
                let plateau = y < 12 && x < 61;
                let town = x >= 71;
                let field = (18..61).contains(&x) && (36..53).contains(&y);
                let yard = (18..61).contains(&x) && (14..36).contains(&y);
                let meadow = y >= 54;
                let west = x < 18 && y >= 14;
                let v = r.range(0, 3) as u32;
                let tree = match r.range(0, 12) {
                    0..=3 => Kind::Pine,
                    4..=7 => Kind::Oak,
                    8..=9 => Kind::Maple,
                    _ => Kind::Cherry,
                };
                let f = r.f();
                if edge {
                    if f < 0.45 {
                        w.place_free(Kind::Hedge, x, y, v);
                    } else if f < 0.85 {
                        w.place_free(if r.chance(0.55) { Kind::Pine } else { Kind::Oak }, x, y, v);
                    }
                } else if plateau {
                    if f < 0.2 {
                        w.place_free(tree, x, y, v);
                    } else if f < 0.26 {
                        w.place_free(Kind::Hedge, x, y, v);
                    } else if f < 0.29 {
                        w.place_free(Kind::Rock, x, y, v);
                    } else if f < 0.34 {
                        w.place_free(Kind::TallGrass, x, y, v);
                    } else if f < 0.36 {
                        w.place_free(Kind::Mushroom, x, y, v);
                    } else if f < 0.39 {
                        w.place_free(Kind::Weed, x, y, v);
                    }
                } else if town {
                    if w.g(x, y) == Ground::Grass {
                        if f < 0.07 {
                            w.place_free(if r.chance(0.5) { Kind::Cherry } else { tree }, x, y, v);
                        } else if f < 0.1 {
                            w.place_free(Kind::Hedge, x, y, v);
                        } else if f < 0.14 {
                            w.place_free(Kind::TallGrass, x, y, v);
                        }
                    }
                } else if field {
                    if f < 0.004 {
                        w.place_free(Kind::Boulder, x, y, 0);
                    } else if f < 0.03 {
                        w.place_free(if r.chance(0.5) { Kind::Oak } else { Kind::Pine }, x, y, v);
                    } else if f < 0.17 {
                        w.place_free(Kind::Weed, x, y, v);
                    } else if f < 0.23 {
                        w.place_free(Kind::Rock, x, y, v);
                    } else if f < 0.28 {
                        w.place_free(Kind::Stick, x, y, v % 2);
                    } else if f < 0.36 {
                        w.place_free(Kind::TallGrass, x, y, v);
                    }
                } else if yard {
                    if w.g(x, y) == Ground::Grass && f < 0.05 {
                        w.place_free(Kind::TallGrass, x, y, v);
                    } else if f < 0.012 {
                        w.place_free(Kind::Weed, x, y, v);
                    }
                } else if meadow || west {
                    if f < 0.07 {
                        w.place_free(tree, x, y, v);
                    } else if f < 0.11 {
                        w.place_free(Kind::Hedge, x, y, v);
                    } else if f < 0.22 {
                        w.place_free(Kind::TallGrass, x, y, v);
                    } else if f < 0.24 {
                        w.place_free(Kind::Mushroom, x, y, v);
                    } else if f < 0.27 {
                        w.place_free(Kind::Weed, x, y, v);
                    } else if f < 0.28 {
                        w.place_free(Kind::Rock, x, y, v);
                    }
                }
            }
        }
        // a first small plot, already hoed
        for y in 26..28 {
            for x in 35..40 {
                w.tilled[idx(x, y)] = true;
            }
        }
        w
    }

    // -------------------------------------------------------------- cottura del terreno

    /// Returns the baked ground and, for every pixel, which ground type is visible there.
    pub fn bake(&self) -> (Canvas, Vec<u8>) {
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
            Dirt | Stairs | Bridge | Path | Deck => 0.0,
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
                // flagstones: a stone belongs to the path if its centre lies on a path tile,
                // so stones overlap tile edges and the path border is irregular
                let near_path = g != Water
                    && (g == Path || [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| self.g(px / 16 + dx, py / 16 + dy) == Path));
                if near_path {
                    let (cx, cy, _, _, _) = voronoi(px as f32, py as f32, 8.5, 501);
                    if self.g((cx / 16.0).floor() as i32, (cy / 16.0).floor() as i32) == Path {
                        tm[(py * pw + px) as usize] = code(Path);
                        continue;
                    }
                }
                let g = if g == Path { Dirt } else { g };
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
        // stone rim around the edged ponds: find those pixels now so neighbours treat them as stone
        let rimc = [hex(0x58524a), hex(0x857f74), hex(0xa59d8f), hex(0xc4bcac), hex(0xe4dccc)];
        let mut rim_px: Vec<(i32, i32, f32)> = Vec::new();
        {
            let (mut x0, mut y0, mut x1, mut y1) = (W, H, 0, 0);
            for ty in 0..H {
                for tx in 0..W {
                    if self.rim[idx(tx, ty)] {
                        x0 = x0.min(tx);
                        y0 = y0.min(ty);
                        x1 = x1.max(tx);
                        y1 = y1.max(ty);
                    }
                }
            }
            let mut offs: Vec<(i32, i32, f32)> = Vec::new();
            for dy in -8..=8i32 {
                for dx in -8..=8i32 {
                    let d = ((dx * dx + dy * dy) as f32).sqrt();
                    if d <= 8.3 {
                        offs.push((dx, dy, d));
                    }
                }
            }
            offs.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            if x1 >= x0 {
                for py in (y0 * 16 - 8).max(0)..((y1 + 1) * 16 + 8).min(ph) {
                    for px in (x0 * 16 - 8).max(0)..((x1 + 1) * 16 + 8).min(pw) {
                        let t = tm[(py * pw + px) as usize];
                        if t == code(Water) || t == code(Deck) {
                            continue;
                        }
                        let width = 6.0 + vnoise(px as f32 * 0.2, py as f32 * 0.2, 88) * 2.2;
                        for &(dx, dy, d) in &offs {
                            if d > width {
                                break;
                            }
                            let (qx, qy) = (px + dx, py + dy);
                            if qx < 0 || qy < 0 || qx >= pw || qy >= ph {
                                continue;
                            }
                            if tm[(qy * pw + qx) as usize] == code(Water) && self.rim[idx(qx / 16, qy / 16)] {
                                rim_px.push((px, py, d));
                                break;
                            }
                        }
                    }
                }
            }
            for &(px, py, _) in &rim_px {
                tm[(py * pw + px) as usize] = code(Cobble);
            }
        }
        let all = [Grass, Dirt, Water, Cliff, Stairs, Bridge, Cobble, Path, Deck];
        let tt = |x: i32, y: i32| -> Ground {
            let x = x.clamp(0, pw - 1);
            let y = y.clamp(0, ph - 1);
            all[tm[(y * pw + x) as usize] as usize]
        };
        let landish = |g: Ground| !matches!(g, Water | Bridge | Deck);
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
                        let rimmed = self.rim[idx(px / 16, py / 16)];
                        let wp = if rimmed { &TEAL } else { &WAT };
                        let mut col;
                        // parete della sponda sotto la terra
                        let bank = (1..=3).find(|&k| landish(tt(px, py - k)));
                        let bridge_above = (1..=4).any(|k| matches!(tt(px, py - k), Bridge | Deck));
                        if let Some(k) = bank {
                            col = match (k, rimmed) {
                                (1, false) => D[1],
                                (2, false) => D[0],
                                (1, true) => rimc[1],
                                (2, true) => rimc[0],
                                _ => wp[0],
                            };
                        } else {
                            let near = |d: i32| {
                                [(d, 0), (-d, 0), (0, d), (0, -d), (d, d), (-d, d), (d, -d), (-d, -d)]
                                    .iter()
                                    .any(|&(dx, dy)| landish(tt(px + dx, py + dy)))
                            };
                            col = if near(1) {
                                wp[5]
                            } else if near(2) {
                                wp[4]
                            } else if near(5) {
                                wp[3]
                            } else if near(10) {
                                wp[2]
                            } else {
                                let n = fbm(px as f32 * 0.05, py as f32 * 0.05, 23);
                                if n < 0.45 {
                                    wp[1]
                                } else {
                                    wp[2]
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
                        if (1..=5).any(|k| tt(px, py - k) == Deck) && px.rem_euclid(16) < 2 {
                            col = if px.rem_euclid(16) == 0 { WOOD[2] } else { WOOD[0] };
                        }
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
                    Path => {
                        let (cx, cy, d1, d2, id) = voronoi(px as f32, py as f32, 8.5, 501);
                        let mut col = if d2 - d1 < 1.1 {
                            if hashf(px, py, 41) < 0.25 { D[0] } else { D[1] }
                        } else {
                            let tone = hashf(id, 0, 502);
                            let base = if tone < 0.3 { FLAG[1] } else if tone < 0.82 { FLAG[2] } else { FLAG[3] };
                            let dx = px as f32 + 0.5 - cx;
                            let dy = py as f32 + 0.5 - cy;
                            let rimz = d2 - d1 < 2.0;
                            if rimz && dy > 0.8 {
                                FLAG[0]
                            } else if rimz && dx > 1.5 {
                                FLAG[1]
                            } else if rimz && dy < -0.5 && dx < 1.0 {
                                FLAG[4]
                            } else if hashf(px, py, 43) < 0.06 {
                                shade(base, 0.9)
                            } else {
                                base
                            }
                        };
                        if tt(px, py - 1) == Grass {
                            col = shade(col, 0.68);
                        } else if tt(px, py - 2) == Grass {
                            col = shade(col, 0.84);
                        }
                        if cliff_above {
                            col = shade(col, 0.74);
                        }
                        col
                    }
                    Deck => {
                        let tx = px / 16;
                        let mut top = py / 16;
                        while top > 0 && self.g(tx, top - 1) == Deck {
                            top -= 1;
                        }
                        let mut bot = py / 16;
                        while bot < H - 1 && self.g(tx, bot + 1) == Deck {
                            bot += 1;
                        }
                        let mut left = tx;
                        while left > 0 && self.g(left - 1, py / 16) == Deck {
                            left -= 1;
                        }
                        let by = py - top * 16;
                        let fb = (bot + 1) * 16 - 1 - py;
                        let lx = px - left * 16;
                        if by == 0 {
                            WOOD[4]
                        } else if fb == 0 {
                            WOOD[0]
                        } else if fb <= 2 {
                            WOOD[1]
                        } else if lx == 0 {
                            WOOD[1]
                        } else if px % 4 == 3 {
                            WOOD[1]
                        } else if (by == 3 || fb == 4) && px % 4 == 1 {
                            WOOD[0]
                        } else if hashf(px / 4, 0, 71) < 0.4 {
                            WOOD[2]
                        } else {
                            WOOD[3]
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
        // ---- stone rim around the edged ponds
        for &(px, py, d) in &rim_px {
            let (cx, cy, d1, d2, id) = voronoi(px as f32, py as f32, 6.5, 601);
            let col = if d2 - d1 < 0.9 {
                rimc[0]
            } else {
                let tone = hashf(id, 0, 602);
                let base = if tone < 0.35 { rimc[2] } else if tone < 0.85 { rimc[3] } else { mix(rimc[2], D[2], 0.35) };
                let dx = px as f32 + 0.5 - cx;
                let dy = py as f32 + 0.5 - cy;
                let edge = d2 - d1 < 2.0;
                let mut col = if edge && (dy > 0.3 || dx > 1.2) {
                    rimc[1]
                } else if edge && dy < 0.0 && dx < 0.8 {
                    rimc[4]
                } else {
                    base
                };
                if d < 1.6 {
                    col = shade(col, 0.82); // wet stones by the water
                }
                if hashf(id, 1, 603) < 0.22 && dy > 0.8 && hashf(px, py, 604) < 0.7 {
                    col = if hashf(px, py, 605) < 0.5 { G[1] } else { G[2] }; // moss
                }
                col
            };
            c.set(px, py, col);
        }
        // ---- life in the water: lily pads, reeds, rocks
        let mut r2 = Rng::new(777);
        for ty in 0..H {
            for tx in 0..W {
                if self.g(tx, ty) != Water {
                    continue;
                }
                let rimmed = self.rim[idx(tx, ty)];
                let wp = if rimmed { &TEAL } else { &WAT };
                let river = (56..=76).contains(&tx) && !rimmed;
                let bx = tx * 16 + r2.range(3, 13);
                let by = ty * 16 + r2.range(3, 13);
                let open = [(7, 0), (-7, 0), (0, 6), (0, -6), (5, 5), (-5, 5), (5, -5), (-5, -5)].iter().all(|&(dx, dy)| tt(bx + dx, by + dy) == Water);
                let roll = r2.f();
                if !river && open && roll < 0.2 {
                    let rx = 3.2 + r2.f() * 1.3;
                    let ry = rx * 0.66;
                    let notch = r2.rf(0.0, 6.28);
                    for y in (by as f32 - ry - 1.0) as i32..=(by as f32 + ry + 1.0) as i32 {
                        for x in (bx as f32 - rx - 1.0) as i32..=(bx as f32 + rx + 1.0) as i32 {
                            let dx = (x as f32 + 0.5 - bx as f32) / rx;
                            let dy = (y as f32 + 0.5 - by as f32) / ry;
                            let d = dx * dx + dy * dy;
                            if d > 1.0 {
                                continue;
                            }
                            let a = dy.atan2(dx);
                            let da = ((a - notch + 9.42) % 6.283 - 3.14).abs();
                            if da < 0.32 && d > 0.05 {
                                continue;
                            }
                            let col = if d > 0.7 && dy > 0.0 { G[0] } else if dy < -0.2 && dx < 0.3 { G[3] } else { G[2] };
                            c.set(x, y, col);
                        }
                        let below = (by as f32 + ry + 1.0) as i32;
                        c.set(bx, below, wp[0]);
                    }
                    c.set(bx - 1, by - 1, G[4]);
                    if r2.chance(0.3) {
                        let pk = [hex(0xf4a6c8), hex(0xffe0f0), hex(0xd870a0)];
                        c.set(bx + 1, by - 1, pk[0]);
                        c.set(bx, by - 2, pk[1]);
                        c.set(bx + 2, by - 2, pk[1]);
                        c.set(bx + 1, by - 2, hex(0xf8d048));
                        c.set(bx + 1, by, pk[2]);
                    }
                } else if open && roll < 0.24 {
                    c.ellipse(bx as f32 + 0.5, by as f32 + 1.5, 3.5, 1.4, wp[4]);
                    c.ellipse(bx as f32, by as f32, 2.6, 1.8, STONE[2]);
                    c.hline(bx - 1, bx + 1, by - 1, STONE[4]);
                    c.set(bx + 2, by + 1, STONE[0]);
                }
                // reeds along natural shores
                if !rimmed {
                    let land = [(0, -1), (-1, 0), (1, 0), (0, 1)].iter().find(|&&(dx, dy)| matches!(self.g(tx + dx, ty + dy), Grass | Dirt));
                    if let Some(&(dx, dy)) = land {
                        if r2.chance(if river { 0.14 } else { 0.4 }) {
                            let sx = tx * 16 + 8 + dx * 6 + r2.range(-3, 4);
                            let sy = ty * 16 + 10 + dy * 5;
                            for k in 0..r2.range(4, 7) {
                                let x = sx + k - 2 + r2.range(-1, 1);
                                let hgt = r2.range(5, 11);
                                for t in 0..hgt {
                                    let col = if t < 2 { G[0] } else if t > hgt - 3 { G[3] } else { G[2] };
                                    c.set(x + if t > hgt * 2 / 3 { (k % 2) * 2 - 1 } else { 0 }, sy - t, col);
                                }
                                if k % 2 == 0 && r2.chance(0.6) {
                                    c.set(x, sy - hgt, hex(0x7a4a24));
                                    c.set(x, sy - hgt - 1, hex(0x9a6030));
                                    c.set(x, sy - hgt + 1, hex(0x7a4a24));
                                }
                            }
                            c.hline(sx - 3, sx + 3, sy + 1, wp[4]);
                        }
                    }
                }
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
                    for _ in 0..5 {
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
        (c, tm)
    }
}
