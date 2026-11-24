//! Farm and nature decor: more trees, hedges, fences, barrels, coop, flowers...

use crate::buildings::{roof, stones};
use crate::canvas::*;
use crate::sprites::{blobs, leaf, leaf_marks, trunk};
use crate::util::*;
use std::f32::consts::PI;

pub const MAPLE: [Rgba; 5] = [hex(0x6a2410), hex(0x9a3a16), hex(0xc8601e), hex(0xe8902c), hex(0xf8c050)];

/// Broadleaf tree with any palette, optionally carrying fruit.
pub fn leafy(var: u32, pal: &[Rgba; 5], fruit: Option<[Rgba; 2]>, seed: u32) -> Canvas {
    let mut c = Canvas::new(48, 64);
    trunk(&mut c, 24, 36, 61, 8, var + seed);
    let j = |a: f32, k: u32| a + (hashf(var as i32, k as i32, seed) - 0.5) * 3.0;
    let list = [
        (j(24.0, 1), j(13.0, 2), 11.5),
        (j(13.0, 3), j(21.0, 4), 10.0),
        (j(35.0, 5), j(21.0, 6), 10.0),
        (j(17.0, 7), j(31.0, 8), 10.5),
        (j(31.0, 9), j(31.0, 10), 10.5),
        (j(24.0, 11), j(24.0, 12), 11.0),
    ];
    blobs(&mut c, &list, pal, seed + 100 + var, 2.6, 0.6);
    leaf_marks(&mut c, pal, seed + 300 + var);
    if let Some(f) = fruit {
        let mut r = Rng::new((seed + var) as u64);
        for _ in 0..14 {
            let x = r.range(6, 42);
            let y = r.range(6, 40);
            if c.get(x, y) == pal[2] || c.get(x, y) == pal[3] {
                c.ellipse(x as f32, y as f32, 1.5, 1.5, f[0]);
                c.set(x - 1, y - 1, f[1]);
            }
        }
    }
    c.outline();
    c
}

/// Big flowering hedge, used to build lush walls along the borders.
pub fn hedge(var: u32) -> Canvas {
    let mut c = Canvas::new(32, 40);
    let j = |a: f32, k: u32| a + (hashf(var as i32, k as i32, 81) - 0.5) * 2.5;
    blobs(
        &mut c,
        &[
            (j(16.0, 1), j(14.0, 2), 10.0),
            (j(8.0, 3), j(21.0, 4), 8.5),
            (j(24.0, 5), j(21.0, 6), 8.5),
            (j(6.0, 7), j(30.0, 8), 6.5),
            (j(26.0, 9), j(30.0, 10), 6.5),
            (16.0, j(29.0, 11), 9.0),
        ],
        &LEAF,
        900 + var,
        2.4,
        0.7,
    );
    leaf_marks(&mut c, &LEAF, 950 + var);
    let fl = [[hex(0xf4b8d8), hex(0xfff0f8)], [hex(0xfff6f0), hex(0xffe070)], [hex(0xe8a0f0), hex(0xfff0ff)]][var as usize % 3];
    let mut r = Rng::new(var as u64 + 400);
    for _ in 0..16 {
        let x = r.range(3, 29);
        let y = r.range(4, 34);
        if c.solid(x, y) && c.solid(x + 1, y) {
            c.set(x, y, fl[0]);
            if r.chance(0.5) {
                c.set(x + 1, y, fl[0]);
                c.set(x, y - 1, fl[1]);
            }
        }
    }
    c.outline();
    c
}

pub fn fence_post() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for y in 3..15 {
        for x in 5..11 {
            let col = match x {
                5 => WOOD[4],
                6 => WOOD[3],
                9 => WOOD[1],
                10 => WOOD[0],
                _ => WOOD[2],
            };
            c.set(x, y, if (y + x) % 5 == 0 && x > 6 && x < 10 { WOOD[1] } else { col });
        }
    }
    c.ellipse(8.0, 3.0, 3.2, 1.9, WOOD[3]);
    c.ellipse(7.6, 2.7, 2.2, 1.2, WOOD[4]);
    c.set(8, 3, WOOD[2]);
    c.hline(5, 10, 14, WOOD[0]);
    c.outline();
    c
}

pub fn fence_rail_h() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for x in 0..16 {
        for (y0, k) in [(5, 0), (10, 1)] {
            c.set(x, y0, WOOD[4]);
            c.set(x, y0 + 1, if (x + k * 3) % 7 == 0 { WOOD[1] } else { WOOD[2] });
            c.set(x, y0 + 2, WOOD[0]);
        }
    }
    c.outline();
    c
}

pub fn fence_rail_v() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for y in 0..16 {
        c.set(6, y, WOOD[4]);
        c.set(7, y, WOOD[2]);
        c.set(8, y, if y % 6 == 0 { WOOD[1] } else { WOOD[2] });
        c.set(9, y, WOOD[0]);
    }
    c.outline();
    c
}

pub fn keg(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 24);
    for y in 6..23 {
        let bulge = if (y - 14i32).abs() < 4 { 1 } else { 0 };
        for x in 3 - bulge..13 + bulge {
            let rel = (x - 2) as f32 / 12.0;
            let mut col = if rel < 0.22 { WOOD[4] } else if rel < 0.5 { WOOD[3] } else if rel < 0.8 { WOOD[2] } else { WOOD[1] };
            if (x - 3) % 3 == 2 {
                col = shade(col, 0.86);
            }
            c.set(x, y, col);
        }
    }
    for hy in [8, 14, 20] {
        for x in 1..15 {
            if c.solid(x, hy) {
                c.set(x, hy, if x < 6 { STONE[4] } else if x < 10 { STONE[2] } else { STONE[1] });
                c.set(x, hy + 1, shade(c.get(x, hy + 1), 0.8));
            }
        }
    }
    c.ellipse(8.0, 6.0, 5.0, 2.0, WOOD[3]);
    c.ellipse(8.0, 6.0, 3.8, 1.2, WOOD[4]);
    c.set(8, 6, WOOD[1]);
    // tap
    c.rect(7, 16, 3, 2, BRASS_C[1]);
    c.set(8, 18, BRASS_C[0]);
    if var % 2 == 1 {
        c.rect(4, 10, 4, 3, hex(0xf2e6c6));
        c.hline(4, 6, 11, hex(0xa89070));
    }
    c.outline();
    c
}

const BRASS_C: [Rgba; 2] = [hex(0xb08420), hex(0xf2c440)];

pub fn crate_box(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 18);
    for y in 4..17 {
        for x in 1..15 {
            let edge = x == 1 || x == 14 || y == 4 || y == 16;
            let col = if edge {
                WOOD[1]
            } else if (y - 4) % 4 == 0 {
                WOOD[2]
            } else if x < 4 {
                WOOD[4]
            } else {
                WOOD[3]
            };
            c.set(x, y, col);
        }
    }
    c.line(2, 15, 13, 5, WOOD[1]);
    c.line(3, 15, 13, 6, WOOD[2]);
    c.rect(1, 2, 14, 2, WOOD[4]);
    c.hline(1, 14, 3, WOOD[2]);
    if var % 2 == 0 {
        // a few apples on top
        for (x, y) in [(5, 1), (8, 1), (11, 1)] {
            c.ellipse(x as f32, y as f32 + 0.5, 1.6, 1.4, hex(0xd8302a));
            c.set(x - 1, y, hex(0xff8a70));
        }
    }
    c.outline();
    c
}

pub fn haybale(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 16);
    let hay = [hex(0x9a7420), hex(0xc8962c), hex(0xe8c050), hex(0xfadc80)];
    for y in 3..15 {
        for x in 1..15 {
            let corner = (y == 3 || y == 14) && (x == 1 || x == 14);
            if corner {
                continue;
            }
            let n = hashf(x, y, var + 5);
            let mut col = if y < 5 { hay[3] } else if x > 11 || y > 12 { hay[1] } else { hay[2] };
            if n < 0.18 {
                col = hay[if y < 6 { 2 } else { 1 }];
            } else if n > 0.93 {
                col = hay[3];
            }
            c.set(x, y, col);
        }
    }
    for x in [4, 11] {
        c.vline(x, 3, 14, WOOD[1]);
    }
    c.set(0, 8, hay[2]);
    c.set(15, 10, hay[1]);
    c.set(6, 2, hay[3]);
    c.outline();
    c
}

pub fn brazier() -> Canvas {
    let mut c = Canvas::new(16, 24);
    stones(&mut c, 5, 11, 12, 24, 77);
    c.hline(4, 11, 11, STONE[3]);
    c.ellipse(8.0, 10.0, 6.0, 2.6, IRON_C[1]);
    c.ellipse(8.0, 9.5, 4.8, 1.6, hex(0x2a1810));
    for (x, col) in [(6, hex(0xe8501e)), (8, hex(0xffb030)), (10, hex(0xc83018)), (7, hex(0xff8020))] {
        c.set(x, 9, col);
    }
    c.hline(2, 13, 11, IRON_C[0]);
    c.outline();
    c
}

const IRON_C: [Rgba; 3] = [hex(0x22222c), hex(0x3a3a48), hex(0x5c5c6e)];

/// Flame frames for braziers.
pub fn flame(frame: u32) -> Canvas {
    let mut c = Canvas::new(10, 12);
    let cols = [hex(0xc83018), hex(0xf06a1e), hex(0xffb030), hex(0xfff0a0)];
    for y in 0..12 {
        for x in 0..10 {
            let dx = (x as f32 + 0.5 - 5.0) / 4.5;
            let t = y as f32 / 11.0; // 0 top, 1 bottom
            let wob = (vnoise(x as f32 * 0.6, y as f32 * 0.5 + frame as f32 * 1.7, 13) - 0.5) * 0.7;
            let width = t.powf(0.8) * (1.0 - t * 0.25) + wob * 0.4;
            let v = width - dx.abs();
            if v <= 0.0 {
                continue;
            }
            let lvl = if v > 0.55 && t > 0.35 { 3 } else if v > 0.32 { 2 } else if v > 0.12 { 1 } else { 0 };
            c.set(x, y, cols[lvl]);
        }
    }
    c
}

pub fn flowerbed(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 24);
    c.ellipse(8.0, 21.5, 7.0, 2.4, SOIL[2]);
    c.hline(3, 12, 20, SOIL[3]);
    let kinds = [
        [hex(0xd83a2a), hex(0xff8a70)],
        [hex(0xf0c028), hex(0xfff0a0)],
        [hex(0xe86aa8), hex(0xffc8e0)],
        [hex(0x5a6ae0), hex(0xb0c0ff)],
        [hex(0xa060e0), hex(0xe0c8ff)],
        [hex(0xfff6f0), hex(0xffffff)],
    ];
    let pal = kinds[var as usize % kinds.len()];
    let spikes = var % 3 == 1;
    for (i, x) in [3, 6, 9, 12].iter().enumerate() {
        let top = 6 + ((var as i32 + i as i32 * 3) % 4);
        for y in top..21 {
            c.set(*x, y, G[1]);
        }
        leaf(&mut c, *x as f32, 18.0, if i % 2 == 0 { -PI * 0.8 } else { -PI * 0.2 }, 4.0, 1.4, &LEAF);
        if spikes {
            for k in 0..5 {
                let y = top + k * 2 - 2;
                c.set(*x - 1, y, pal[0]);
                c.set(*x + 1, y, pal[0]);
                c.set(*x, y - 1, pal[1]);
            }
        } else {
            // tulip cup
            c.rect(*x - 1, top - 3, 3, 3, pal[0]);
            c.set(*x - 1, top - 4, pal[0]);
            c.set(*x + 1, top - 4, pal[0]);
            c.set(*x, top - 3, pal[1]);
        }
    }
    c.outline();
    c
}

pub fn mushrooms(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 16);
    let red = var % 2 == 0;
    let cap = if red { [hex(0x9a2018), hex(0xd8402c), hex(0xf07a58)] } else { [hex(0x6a4428), hex(0x9a6a3c), hex(0xc89a62)] };
    for (x, y, s) in [(5.0, 11.0, 1.0), (10.0, 12.5, 0.75), (8.0, 14.0, 0.55)] {
        let h = (5.0 * s) as i32;
        c.rect(x as i32 - 1, y as i32 - h + 2, 2, h, hex(0xf2e6c6));
        c.ellipse(x, y - h as f32 + 1.0, 3.4 * s + 0.6, 2.2 * s + 0.4, cap[1]);
        c.hline((x - 2.0 * s) as i32, (x + 1.0) as i32, (y - h as f32 + 0.0) as i32, cap[2]);
        if red {
            c.set(x as i32 - 1, (y - h as f32 + 1.0) as i32, hex(0xfff6f0));
            c.set(x as i32 + 1, (y - h as f32) as i32, hex(0xfff6f0));
        }
    }
    c.outline();
    c
}

pub fn tallgrass(var: u32, frame: u32) -> Canvas {
    let mut c = Canvas::new(16, 20);
    let mut r = Rng::new(var as u64 * 13 + 1);
    let lean = if frame == 0 { 0 } else { 1 };
    for i in 0..11 {
        let x = 2 + i + r.range(-1, 2);
        let hgt = r.range(7, 15);
        let side = if i < 5 { -1 } else { 1 };
        for t in 0..hgt {
            let off = if t > hgt / 2 { side + if t > hgt * 3 / 4 { lean } else { 0 } } else { 0 };
            let col = if t < 2 {
                G[0]
            } else if t == hgt - 1 {
                G[4]
            } else if t > hgt * 2 / 3 {
                G[3]
            } else if i % 3 == 0 {
                G[1]
            } else {
                G[2]
            };
            c.set(x + off, 19 - t, col);
        }
    }
    if var % 3 == 0 {
        c.set(5, 7, hex(0xfff6f0));
        c.set(11, 6, hex(0xffe070));
    }
    c
}

pub fn sign() -> Canvas {
    let mut c = Canvas::new(16, 24);
    for x in [3, 11] {
        c.rect(x, 10, 2, 13, WOOD[2]);
        c.vline(x + 1, 10, 22, WOOD[1]);
    }
    for y in 3..12 {
        for x in 0..16 {
            let col = if y == 3 { WOOD[4] } else if y == 11 { WOOD[0] } else if (y - 3) % 4 == 0 { WOOD[2] } else { WOOD[3] };
            c.set(x, y, col);
        }
    }
    for (y, x0, x1) in [(6, 3, 12), (8, 4, 11)] {
        for x in x0..x1 {
            if x % 3 != 2 {
                c.set(x, y, WOOD[0]);
            }
        }
    }
    c.set(13, 6, hex(0xf06a8a));
    c.outline();
    c
}

/// Green-painted chicken coop with a ramp, hay window and a rooster weathervane.
pub fn coop() -> Canvas {
    let (w, h) = (64, 64);
    let mut c = Canvas::new(w, h);
    let green = [hex(0x1e4428), hex(0x2e6238), hex(0x43824c), hex(0x64a462)];
    for y in 26..54 {
        for x in 4..60 {
            let lx = (x - 4) % 6;
            let k = (y - 26) as f32 / 28.0;
            let mut col = match lx {
                0 => green[0],
                1 => green[3],
                _ => {
                    if hashf((x - 4) / 6, y / 7, 5) < 0.2 {
                        green[1]
                    } else {
                        green[2]
                    }
                }
            };
            if k > 0.75 {
                col = shade(col, 0.88);
            }
            c.set(x, y, col);
        }
    }
    for (px, light) in [(4, true), (57, false)] {
        for y in 26..54 {
            for x in px..px + 3 {
                c.set(x, y, if x == px { if light { WOOD[4] } else { WOOD[3] } } else if x == px + 2 { WOOD[0] } else { WOOD[2] });
            }
        }
    }
    stones(&mut c, 3, 61, 52, 57, 91);
    // hay window
    c.rect(11, 33, 14, 11, WOOD[0]);
    c.rect(12, 34, 12, 9, WOOD[3]);
    c.rect(13, 35, 10, 7, hex(0x2a1810));
    for x in 13..23 {
        let hgt = 2 + (hash(x, 0, 3) % 4) as i32;
        for t in 0..hgt {
            c.set(x, 41 - t, if t == hgt - 1 { hex(0xfadc80) } else { hex(0xe8c050) });
        }
    }
    c.set(15, 36, hex(0xe8c050));
    c.hline(10, 25, 44, WOOD[4]);
    c.hline(10, 25, 45, WOOD[1]);
    // chicken door with ramp
    c.rect(35, 38, 12, 16, WOOD[0]);
    c.rect(36, 39, 10, 15, hex(0x1a120c));
    c.ellipse(41.0, 39.5, 5.0, 2.0, WOOD[0]);
    for y in 54..64 {
        let inset = (y - 54) / 4;
        for x in 35 - inset..47 + inset {
            let col = if (y - 54) % 3 == 0 { WOOD[1] } else if x < 38 { WOOD[4] } else { WOOD[3] };
            c.set(x, y, col);
        }
    }
    c.hline(34, 47, 63, WOOD[0]);
    // egg sign
    c.rect(50, 31, 6, 7, hex(0xf2e6c6));
    c.ellipse(53.0, 34.5, 1.6, 2.1, hex(0xfff8ee));
    c.set(52, 34, hex(0xe0d0b0));
    // roof
    let rp = [hex(0x3a2418), hex(0x5a3622), hex(0x7a4a2c), hex(0x9a643a), hex(0xb88050)];
    roof(&mut c, 6, 26, 0.3, &rp, 0, 93);
    for x in 4..60 {
        for (k, f) in [(29, 0.6), (30, 0.75), (31, 0.88)] {
            let p = c.get(x, k);
            if p[3] > 0 {
                c.set(x, k, shade(p, f));
            }
        }
    }
    // rooster weathervane
    c.vline(32, 0, 6, IRON_C[1]);
    c.hline(28, 36, 2, IRON_C[0]);
    c.rect(30, 0, 4, 2, IRON_C[2]);
    c.set(34, 0, IRON_C[2]);
    c.set(29, 0, IRON_C[0]);
    c.outline();
    c
}
