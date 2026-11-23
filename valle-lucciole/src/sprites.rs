//! Tutti gli sprite del gioco, dipinti pixel per pixel via codice.

use crate::canvas::*;
use crate::items::*;
use crate::util::*;
use std::f32::consts::PI;

// ------------------------------------------------------------------ helper

/// Chioma "a nuvole": cerchi ombreggiati con luce da in alto a sinistra.
fn blobs(c: &mut Canvas, list: &[(f32, f32, f32)], pal: &[Rgba; 5], seed: u32, wobble: f32, texture: f32) {
    for (i, &(cx, cy, r)) in list.iter().enumerate() {
        let y0 = (cy - r - 3.0) as i32;
        let y1 = (cy + r + 3.0) as i32;
        let x0 = (cx - r - 3.0) as i32;
        let x1 = (cx + r + 3.0) as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt();
                let wob = (vnoise(x as f32 * 0.55, y as f32 * 0.55, seed + i as u32 * 13) - 0.5) * wobble;
                if d > r + wob {
                    continue;
                }
                let nx = dx / r;
                let ny = dy / r;
                let mut l = 0.62 - 0.45 * nx - 0.7 * ny;
                l += (vnoise(x as f32 * 0.42, y as f32 * 0.42, seed + 77) - 0.5) * texture;
                l += (hashf(x, y, seed + 5) - 0.5) * 0.14;
                let mut lvl = if l > 1.08 {
                    4
                } else if l > 0.72 {
                    3
                } else if l > 0.38 {
                    2
                } else if l > 0.05 {
                    1
                } else {
                    0
                };
                // bordo inferiore scuro: separa i ciuffi
                if d > r + wob - 1.4 && ny > 0.15 {
                    lvl = if ny > 0.55 { 0 } else { lvl.min(1) };
                }
                c.set(x, y, pal[lvl]);
            }
        }
    }
}

/// Foglia a goccia dalla base (x,y) in direzione `ang`.
fn leaf(c: &mut Canvas, x: f32, y: f32, ang: f32, len: f32, w: f32, pal: &[Rgba; 5]) {
    let (s, co) = ang.sin_cos();
    let steps = (len * 3.0) as i32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let hw = (t * PI).sin() * w * (1.0 - t * 0.3);
        let px = x + co * len * t;
        let py = y + s * len * t;
        let k = (hw * 3.0) as i32;
        for j in -k..=k {
            let o = j as f32 / 3.0;
            let qx = px - s * o;
            let qy = py + co * o;
            // lato in luce / in ombra
            let col = if j.abs() <= 1 && t > 0.15 && t < 0.85 {
                pal[3]
            } else if o < 0.0 {
                pal[3]
            } else {
                pal[2]
            };
            c.set(qx.floor() as i32, qy.floor() as i32, col);
        }
    }
    // nervatura
    for i in 0..(len as i32) {
        let t = i as f32 / len;
        if t > 0.1 && t < 0.8 {
            c.set((x + co * len * t) as i32, (y + s * len * t) as i32, pal[4]);
        }
    }
}

fn thick_line(c: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32, a: Rgba, b: Rgba) {
    c.line(x0, y0, x1, y1, a);
    c.line(x0 + 1, y0, x1 + 1, y1, b);
}

// ------------------------------------------------------------------ alberi

pub fn oak(var: u32) -> Canvas {
    let mut c = Canvas::new(48, 64);
    trunk(&mut c, 24, 36, 61, 8, var);
    let j = |a: f32, k: u32| a + (hashf(var as i32, k as i32, 9) - 0.5) * 3.0;
    let list = [
        (j(24.0, 1), j(13.0, 2), 11.5),
        (j(13.0, 3), j(21.0, 4), 10.0),
        (j(35.0, 5), j(21.0, 6), 10.0),
        (j(17.0, 7), j(31.0, 8), 10.5),
        (j(31.0, 9), j(31.0, 10), 10.5),
        (j(24.0, 11), j(24.0, 12), 11.0),
    ];
    blobs(&mut c, &list, &LEAF, 100 + var, 2.6, 0.6);
    leaf_marks(&mut c, &LEAF, 300 + var);
    c.outline();
    c
}

pub fn cherry(var: u32) -> Canvas {
    let mut c = Canvas::new(48, 64);
    trunk(&mut c, 24, 34, 61, 8, var + 40);
    // rami visibili
    thick_line(&mut c, 24, 40, 14, 30, WOOD[1], WOOD[2]);
    thick_line(&mut c, 25, 38, 34, 28, WOOD[1], WOOD[2]);
    let j = |a: f32, k: u32| a + (hashf(var as i32, k as i32, 19) - 0.5) * 3.0;
    let list = [
        (j(24.0, 1), j(12.0, 2), 11.0),
        (j(12.0, 3), j(21.0, 4), 9.5),
        (j(36.0, 5), j(21.0, 6), 9.5),
        (j(17.0, 7), j(29.0, 8), 9.0),
        (j(31.0, 9), j(29.0, 10), 9.0),
        (j(24.0, 11), j(22.0, 12), 10.0),
    ];
    blobs(&mut c, &list, &BLOSSOM, 200 + var, 3.0, 0.75);
    leaf_marks(&mut c, &BLOSSOM, 400 + var);
    c.outline();
    c
}

fn trunk(c: &mut Canvas, cx: i32, y0: i32, y1: i32, w: i32, var: u32) {
    let x0 = cx - w / 2;
    for y in y0..=y1 {
        let spread = if y > y1 - 5 { (y - (y1 - 5)) / 2 + 1 } else { 0 };
        for x in (x0 - spread)..(x0 + w + spread) {
            let rel = (x - (x0 - spread)) as f32 / (w + spread * 2) as f32;
            let mut col = if rel < 0.2 {
                WOOD[3]
            } else if rel < 0.55 {
                WOOD[2]
            } else if rel < 0.85 {
                WOOD[1]
            } else {
                WOOD[0]
            };
            if hashf(x, y / 3, var + 3) < 0.18 {
                col = shade(col, 0.78);
            }
            c.set(x, y, col);
        }
    }
    // radici
    c.ellipse(cx as f32 - 5.5, y1 as f32 - 1.0, 3.5, 2.0, WOOD[2]);
    c.ellipse(cx as f32 + 5.5, y1 as f32 - 1.0, 3.5, 2.0, WOOD[1]);
    c.hline(cx - 8, cx - 4, y1 - 2, WOOD[3]);
}

fn leaf_marks(c: &mut Canvas, pal: &[Rgba; 5], seed: u32) {
    let mut r = Rng::new(seed as u64);
    for _ in 0..40 {
        let x = r.range(2, c.w - 2);
        let y = r.range(2, c.h - 20);
        if c.get(x, y) == pal[3] && c.get(x + 1, y - 1) == pal[3] {
            c.set(x, y, pal[4]);
            c.set(x + 1, y - 1, pal[4]);
        } else if c.get(x, y) == pal[1] && c.get(x + 1, y + 1) == pal[1] {
            c.set(x, y, pal[2]);
            c.set(x + 1, y + 1, pal[0]);
        }
    }
}

pub fn pine(var: u32) -> Canvas {
    let mut c = Canvas::new(32, 64);
    // tronco
    for y in 48..62 {
        for x in 13..19 {
            let col = if x < 15 { WOOD[3] } else if x < 17 { WOOD[2] } else { WOOD[1] };
            c.set(x, y, col);
        }
    }
    c.hline(11, 20, 61, WOOD[1]);
    let cx = 16.0 + (hashf(var as i32, 1, 5) - 0.5) * 1.5;
    let tiers = 5;
    for i in (0..tiers).rev() {
        let ty = 3.0 + i as f32 * 9.0;
        let h = 15.0 + i as f32 * 0.6;
        let maxhw = 4.5 + i as f32 * 2.6;
        for yy in 0..(h as i32 + 2) {
            let y = ty as i32 + yy;
            let t = yy as f32 / h;
            let hw = 1.2 + t.min(1.0) * (maxhw - 1.2);
            for x in (cx - hw - 1.0) as i32..=(cx + hw + 1.0) as i32 {
                let dx = x as f32 + 0.5 - cx;
                if dx.abs() > hw {
                    continue;
                }
                let droop = (dx.abs() / maxhw) * 3.5;
                let tooth = if (x + i) % 3 == 0 { 1.5 } else { 0.0 };
                let bottom = ty + h - droop + tooth;
                if (y as f32) > bottom {
                    continue;
                }
                let nx = dx / maxhw;
                let mut l = 0.6 - 0.9 * nx - 0.25 * (t - 0.5);
                l += (vnoise(x as f32 * 0.6, y as f32 * 0.6, var + 21) - 0.5) * 0.4;
                if (x + y) % 3 == 0 && hashf(x, y, var + 2) < 0.5 {
                    l += 0.25;
                }
                let mut lvl = if l > 1.0 {
                    4
                } else if l > 0.68 {
                    3
                } else if l > 0.35 {
                    2
                } else if l > 0.0 {
                    1
                } else {
                    0
                };
                if (y as f32) > bottom - 1.6 {
                    lvl = lvl.min(1);
                }
                c.set(x, y, PINE[lvl]);
            }
        }
    }
    c.outline();
    c
}

pub fn stump() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for y in 7..14 {
        for x in 2..14 {
            let col = if x < 5 { WOOD[3] } else if x < 10 { WOOD[2] } else { WOOD[1] };
            c.set(x, y, col);
        }
    }
    c.ellipse(8.0, 13.5, 6.5, 2.0, WOOD[1]);
    c.ellipse(3.0, 13.5, 2.5, 1.6, WOOD[2]);
    c.ellipse(13.0, 13.5, 2.5, 1.6, WOOD[1]);
    // anelli
    for y in 4..11 {
        for x in 2..14 {
            let dx = (x as f32 + 0.5 - 8.0) / 6.0;
            let dy = (y as f32 + 0.5 - 7.0) / 3.0;
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                let ring = ((d.sqrt() * 3.5) as i32) % 2;
                c.set(x, y, if ring == 0 { WOOD[4] } else { WOOD[3] });
            }
        }
    }
    c.set(8, 7, WOOD[2]);
    c.outline();
    c
}

// ------------------------------------------------------------------ rocce, erbacce

pub fn rock_small(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 16);
    let ox = (hashf(var as i32, 2, 3) - 0.5) * 2.0;
    blobs(&mut c, &[(8.0 + ox, 10.0, 5.8), (6.0 + ox, 9.0, 3.5)], &STONE, 500 + var, 1.2, 0.25);
    // crepa
    if var % 2 == 0 {
        c.paint(9, 8, STONE[1]);
        c.paint(10, 9, STONE[1]);
        c.paint(10, 10, STONE[1]);
    }
    c.paint(5, 7, STONE[4]);
    c.paint(6, 6, STONE[4]);
    c.outline();
    c
}

pub fn rock_big(var: u32) -> Canvas {
    let mut c = Canvas::new(32, 32);
    blobs(
        &mut c,
        &[(16.0, 19.0, 12.0), (10.0, 14.0, 7.0), (22.0, 13.0, 7.5), (16.0, 21.0, 10.0)],
        &STONE,
        600 + var,
        1.6,
        0.3,
    );
    c.paint(14, 15, STONE[1]);
    c.paint(15, 16, STONE[1]);
    c.paint(15, 17, STONE[1]);
    c.paint(16, 18, STONE[1]);
    c.paint(20, 22, STONE[1]);
    c.paint(21, 23, STONE[1]);
    c.outline();
    c
}

pub fn weed(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 16);
    let blue: [Rgba; 5] = [hex(0x1c3e52), hex(0x2a5c74), hex(0x3a7c96), hex(0x55a0b8), hex(0x86c6d6)];
    let pal = if var % 3 == 2 { blue } else { LEAF };
    let n = 6;
    for i in 0..n {
        let a = -PI * 0.5 + (i as f32 - (n as f32 - 1.0) / 2.0) * 0.48 + (hashf(var as i32, i, 4) - 0.5) * 0.3;
        let len = 6.5 + hashf(var as i32, i, 8) * 2.5;
        leaf(&mut c, 8.0, 14.0, a, len, 2.0, &pal);
    }
    if var % 3 == 1 {
        c.set(5, 5, hex(0xe4a0e8));
        c.set(11, 4, hex(0xe4a0e8));
        c.set(8, 3, hex(0xfff0ff));
    }
    c.outline();
    c
}

pub fn stick(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 16);
    if var % 2 == 0 {
        thick_line(&mut c, 2, 12, 13, 7, WOOD[2], WOOD[1]);
        c.line(8, 9, 10, 5, WOOD[2]);
        c.set(13, 7, WOOD[4]);
    } else {
        thick_line(&mut c, 3, 7, 13, 12, WOOD[2], WOOD[1]);
        c.line(7, 9, 5, 12, WOOD[2]);
        c.set(3, 7, WOOD[4]);
    }
    c.outline();
    c
}

pub fn bush(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 24);
    blobs(&mut c, &[(8.0, 12.0, 6.5), (4.5, 16.0, 4.5), (11.5, 16.0, 4.5), (8.0, 17.0, 5.0)], &LEAF, 700 + var, 1.8, 0.5);
    let mut r = Rng::new(var as u64 + 9);
    let fl = if var % 2 == 0 { hex(0xf4b8d8) } else { hex(0xfff2b0) };
    for _ in 0..6 {
        let x = r.range(3, 13);
        let y = r.range(8, 20);
        if c.solid(x, y) {
            c.set(x, y, fl);
        }
    }
    c.outline();
    c
}

// ------------------------------------------------------------------ edifici

pub struct HouseSpec {
    pub w: i32,
    pub roof: [Rgba; 5],
    pub plaster: bool,
    pub door_cx: i32,
}

pub fn house(s: &HouseSpec) -> Canvas {
    let w = s.w;
    let mut c = Canvas::new(w, 96);
    let wx0 = 6;
    let wx1 = w - 7;
    // portico
    for y in 80..95 {
        for x in 3..w - 3 {
            let mut col = if (y - 80) % 3 == 0 { WOOD[2] } else { WOOD[3] };
            if (x + (y / 3) * 7) % 13 == 0 {
                col = WOOD[2];
            }
            c.set(x, y, col);
        }
    }
    c.hline(3, w - 4, 93, WOOD[1]);
    c.hline(3, w - 4, 94, WOOD[0]);
    // gradini
    for y in 87..96 {
        for x in s.door_cx - 9..s.door_cx + 9 {
            let col = if (y - 87) % 3 == 0 { WOOD[4] } else if (y - 87) % 3 == 1 { WOOD[3] } else { WOOD[1] };
            c.set(x, y, col);
        }
    }
    // muri
    let plaster = [hex(0xc9b48e), hex(0xe2d0aa), hex(0xf2e4c4)];
    for y in 40..80 {
        for x in wx0..=wx1 {
            let col = if s.plaster {
                let beam = (x - wx0) % 18 == 0 || y == 58;
                if beam {
                    WOOD[1]
                } else if hashf(x, y, 31) < 0.06 {
                    plaster[0]
                } else {
                    plaster[1]
                }
            } else {
                let plank = (x - wx0) / 5;
                let edge = (x - wx0) % 5 == 4;
                let v = hashf(plank, 0, 77);
                if edge {
                    WOOD[1]
                } else if v < 0.3 {
                    WOOD[2]
                } else if v < 0.85 {
                    WOOD[3]
                } else {
                    WOOD[4]
                }
            };
            c.set(x, y, col);
        }
    }
    // ombra sotto la gronda
    for x in wx0..=wx1 {
        c.set(x, 40, WOOD[0]);
        c.set(x, 41, shade(c.get(x, 41), 0.6));
        c.set(x, 42, shade(c.get(x, 42), 0.75));
    }
    // fondamenta in pietra
    for y in 75..80 {
        for x in wx0..=wx1 {
            let brick = ((x + if y % 4 < 2 { 0 } else { 3 }) % 6 == 0) || y == 75;
            c.set(x, y, if brick { STONE[1] } else if hashf(x / 6, y, 3) < 0.5 { STONE[2] } else { STONE[3] });
        }
    }
    // pilastri angolari
    for y in 42..80 {
        for x in [wx0, wx0 + 1, wx1 - 1, wx1] {
            c.set(x, y, if x == wx0 || x == wx1 - 1 { WOOD[1] } else { WOOD[0] });
        }
    }
    // porta
    let dx0 = s.door_cx - 6;
    for y in 53..80 {
        for x in dx0..dx0 + 12 {
            let col = if x == dx0 || x == dx0 + 11 || y == 53 {
                WOOD[0]
            } else if (x - dx0) % 4 == 0 {
                WOOD[1]
            } else {
                WOOD[2]
            };
            c.set(x, y, col);
        }
    }
    c.rect(dx0 + 3, 57, 6, 4, WAT[3]);
    c.set(dx0 + 3, 57, WAT[5]);
    c.set(dx0 + 4, 57, WAT[4]);
    c.set(dx0 + 9, 68, hex(0xf2c440));
    c.set(dx0 + 9, 69, hex(0xb08420));
    // finestre
    let mut wins = vec![(14, 50, 16, 14)];
    if w > 80 {
        wins.push((w - 26, 50, 13, 13));
    } else {
        wins.push((w - 24, 50, 12, 12));
    }
    for (i, &(x, y, ww, hh)) in wins.iter().enumerate() {
        let frame = hex(0xf1e2c2);
        c.rect(x - 1, y - 1, ww + 2, hh + 2, WOOD[0]);
        c.rect(x, y, ww, hh, frame);
        c.rect(x + 1, y + 1, ww - 2, hh - 2, WAT[3]);
        for k in 0..ww - 2 {
            // riflesso diagonale
            let yy = y + 1 + k;
            if yy < y + hh - 1 {
                c.set(x + ww - 2 - k, yy, WAT[2]);
            }
        }
        c.set(x + 2, y + 2, WAT[5]);
        c.set(x + 3, y + 2, WAT[4]);
        c.set(x + 2, y + 3, WAT[4]);
        c.vline(x + ww / 2, y + 1, y + hh - 2, frame);
        c.hline(x + 1, x + ww - 2, y + hh / 2, frame);
        // persiane verdi per la prima finestra
        if i == 0 {
            let sh = [hex(0x2e6a3a), hex(0x438a4a), hex(0x5ca85a)];
            for yy in y - 1..y + hh + 1 {
                for (k, xx) in [x - 5, x - 4, x - 3, x - 2].into_iter().enumerate() {
                    c.set(xx, yy, if yy % 2 == 0 { sh[1] } else { sh[k.min(2) % 3] });
                }
                for (k, xx) in [x + ww + 1, x + ww + 2, x + ww + 3, x + ww + 4].into_iter().enumerate() {
                    c.set(xx, yy, if yy % 2 == 0 { sh[1] } else { sh[(2 - k.min(2)) % 3] });
                }
            }
            // fioriera
            c.rect(x - 2, y + hh + 1, ww + 4, 4, WOOD[1]);
            c.hline(x - 2, x + ww + 1, y + hh + 1, WOOD[3]);
            let mut r = Rng::new(5);
            for k in 0..ww + 3 {
                let fc = [hex(0xf06a8a), hex(0xfff0f4), hex(0xf4b040), hex(0xb070e0)][r.range(0, 4) as usize];
                c.set(x - 1 + k, y + hh, if k % 2 == 0 { fc } else { G[3] });
                if k % 3 == 1 {
                    c.set(x - 1 + k, y + hh - 1, fc);
                }
            }
        }
    }
    // tetto
    for y in 4..45 {
        let inset = ((44 - y) as f32 * 0.14) as i32;
        for x in inset..w - inset {
            let row = (y - 4) / 5;
            let in_row = (y - 4) % 5;
            let sx = (x + row * 3) % 8;
            let mut col = if in_row == 4 {
                s.roof[1]
            } else if sx == 0 {
                s.roof[1]
            } else if in_row == 0 {
                s.roof[3]
            } else {
                s.roof[2]
            };
            if in_row < 3 && sx != 0 && hashf((x + row * 3) / 8, row, 55) < 0.2 {
                col = s.roof[3];
            }
            // luce sul lato sinistro, ombra sul destro
            let rel = x as f32 / w as f32;
            if rel > 0.82 && in_row != 0 {
                col = shade(col, 0.88);
            }
            if y <= 6 {
                col = if y == 4 { s.roof[1] } else { s.roof[4] };
            }
            c.set(x, y, col);
        }
    }
    for x in 0..w {
        c.set(x, 43, s.roof[0]);
        c.set(x, 44, s.roof[0]);
        if x % 8 == 3 {
            c.set(x, 42, s.roof[1]);
        }
    }
    // abbaino
    let ax = w / 2 - 6;
    c.rect(ax - 1, 15, 14, 13, WOOD[0]);
    c.rect(ax, 16, 12, 11, if s.plaster { plaster[2] } else { WOOD[3] });
    c.rect(ax + 3, 18, 6, 6, WAT[3]);
    c.set(ax + 3, 18, WAT[5]);
    c.vline(ax + 6, 18, 23, hex(0xf1e2c2));
    c.hline(ax + 3, ax + 8, 21, hex(0xf1e2c2));
    for k in 0..9 {
        c.hline(ax - 2 + k / 2, ax + 13 - k / 2, 14 - k / 2 + 1, s.roof[if k % 2 == 0 { 3 } else { 2 }]);
    }
    // camino
    if w > 80 {
        for y in 0..16 {
            for x in w - 26..w - 16 {
                let brick = (x + if y % 4 < 2 { 0 } else { 2 }) % 5 == 0 || y % 2 == 0 && y % 4 == 0;
                c.set(x, y, if brick { STONE[1] } else if x < w - 22 { STONE[3] } else { STONE[2] });
            }
        }
        c.rect(w - 27, 0, 12, 2, STONE[0]);
    }
    c.outline();
    c
}

pub fn bin() -> Canvas {
    let mut c = Canvas::new(32, 32);
    // corpo
    for y in 14..30 {
        for x in 2..30 {
            let col = if (y - 14) % 5 == 4 { WOOD[1] } else if x < 6 { WOOD[3] } else if x > 25 { WOOD[1] } else { WOOD[2] };
            c.set(x, y, col);
        }
    }
    // coperchio
    for y in 7..15 {
        for x in 1..31 {
            let col = if y < 9 { WOOD[4] } else if y == 14 { WOOD[0] } else if (x - 1) % 6 == 5 { WOOD[2] } else { WOOD[3] };
            c.set(x, y, col);
        }
    }
    // bordi metallici
    for (x0, x1) in [(2, 4), (28, 30)] {
        for y in 7..30 {
            for x in x0..x1 {
                c.set(x, y, if x == x0 { STONE[3] } else { STONE[1] });
            }
        }
    }
    c.rect(13, 16, 6, 4, STONE[2]);
    c.hline(13, 18, 16, STONE[4]);
    c.outline();
    c
}

pub fn well() -> Canvas {
    let mut c = Canvas::new(32, 48);
    // corpo in pietra
    for y in 30..45 {
        for x in 2..30 {
            let off = if (y / 4) % 2 == 0 { 0 } else { 3 };
            let mortar = (x + off) % 6 == 0 || y % 4 == 0;
            let rel = (x - 2) as f32 / 28.0;
            let base = if rel < 0.25 { STONE[3] } else if rel < 0.75 { STONE[2] } else { STONE[1] };
            c.set(x, y, if mortar { STONE[0] } else { base });
        }
    }
    c.ellipse(16.0, 44.0, 14.0, 3.0, STONE[1]);
    c.ellipse(16.0, 30.0, 14.0, 5.0, STONE[3]);
    c.ellipse(16.0, 30.0, 11.0, 3.4, WAT[0]);
    c.ellipse(16.0, 31.0, 9.0, 2.2, WAT[1]);
    c.set(13, 30, WAT[4]);
    // pali
    for y in 8..31 {
        for x in [4, 5, 26, 27] {
            c.set(x, y, if x % 2 == 0 { WOOD[3] } else { WOOD[1] });
        }
    }
    // trave + corda + secchio
    c.hline(4, 27, 12, WOOD[1]);
    c.hline(4, 27, 13, WOOD[2]);
    c.vline(16, 14, 20, hex(0xd8c090));
    c.rect(13, 21, 7, 5, STONE[2]);
    c.hline(13, 19, 21, STONE[4]);
    c.hline(13, 19, 25, STONE[0]);
    // tettoia
    for y in 0..11 {
        let inset = (10 - y) / 2;
        for x in inset..32 - inset {
            let col = if y == 10 { WOOD[0] } else if (x + y) % 5 == 0 { WOOD[1] } else if y < 3 { WOOD[3] } else { WOOD[2] };
            c.set(x, y, col);
        }
    }
    c.outline();
    c
}

pub fn stall() -> Canvas {
    let mut c = Canvas::new(48, 64);
    // pali
    for y in 12..62 {
        for x in [3, 4, 43, 44] {
            c.set(x, y, if x % 2 == 1 { WOOD[3] } else { WOOD[1] });
        }
    }
    // bancone
    for y in 46..62 {
        for x in 2..46 {
            let col = if (x - 2) % 7 == 6 { WOOD[1] } else if x < 6 { WOOD[3] } else { WOOD[2] };
            c.set(x, y, col);
        }
    }
    for y in 41..46 {
        for x in 1..47 {
            c.set(x, y, if y == 41 { WOOD[4] } else if y == 45 { WOOD[0] } else { WOOD[3] });
        }
    }
    // cassette di verdura
    let crates = [(6, hex(0xe8782c)), (19, hex(0xd8402c)), (32, hex(0x7cc04a))];
    for (x, col) in crates {
        c.rect(x, 36, 10, 6, WOOD[1]);
        c.hline(x, x + 9, 36, WOOD[3]);
        for k in 0..4 {
            c.ellipse(x as f32 + 2.0 + k as f32 * 2.2, 35.0, 1.4, 1.4, col);
            c.set(x + 1 + k * 2, 34, shade(col, 1.25));
        }
    }
    // tendone a strisce
    let cream = hex(0xf4e6c8);
    for y in 0..16 {
        for x in 0..48 {
            let stripe = (x / 6) % 2 == 0;
            let mut col = if stripe { ROOF[3] } else { cream };
            if y < 3 {
                col = shade(col, 1.08);
            }
            if y > 11 {
                col = shade(col, 0.85);
            }
            c.set(x, y, col);
        }
    }
    // smerlatura
    for k in 0..8 {
        let cx = k as f32 * 6.0 + 3.0;
        let col = if k % 2 == 0 { ROOF[2] } else { hex(0xd8c8a8) };
        c.ellipse(cx, 16.0, 3.0, 2.5, col);
    }
    c.outline();
    c
}

pub fn fence_post() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for y in 3..15 {
        for x in 6..10 {
            c.set(x, y, if x == 6 { WOOD[3] } else if x == 9 { WOOD[1] } else { WOOD[2] });
        }
    }
    c.ellipse(8.0, 3.5, 2.2, 1.4, WOOD[4]);
    c.set(8, 3, WOOD[3]);
    c.outline();
    c
}

pub fn fence_rail_h() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for x in 0..16 {
        c.set(x, 6, WOOD[3]);
        c.set(x, 7, WOOD[1]);
        c.set(x, 10, WOOD[3]);
        c.set(x, 11, WOOD[1]);
    }
    c.outline();
    c
}

pub fn fence_rail_v() -> Canvas {
    let mut c = Canvas::new(16, 16);
    for y in 0..16 {
        c.set(7, y, WOOD[3]);
        c.set(8, y, WOOD[1]);
    }
    c.outline();
    c
}

pub fn mailbox() -> Canvas {
    let mut c = Canvas::new(16, 32);
    c.rect(7, 16, 2, 15, WOOD[2]);
    c.vline(8, 16, 30, WOOD[1]);
    c.rect(3, 9, 10, 7, hex(0x5a6a8a));
    c.hline(3, 12, 9, hex(0x8a9aba));
    c.ellipse(8.0, 9.5, 5.0, 2.0, hex(0x7a8aaa));
    c.rect(4, 12, 2, 2, hex(0x2a3248));
    c.vline(13, 5, 10, hex(0xd8402c));
    c.rect(13, 5, 2, 2, hex(0xd8402c));
    c.outline();
    c
}

pub fn scarecrow() -> Canvas {
    let mut c = Canvas::new(16, 32);
    let burlap = [hex(0xa88a58), hex(0xc8aa70), hex(0xe0c68c)];
    let shirt = [hex(0x5a3a7a), hex(0x7a5aa0)];
    c.rect(7, 12, 2, 19, WOOD[2]);
    c.hline(1, 14, 15, WOOD[1]);
    // camicia
    c.rect(4, 14, 8, 8, shirt[1]);
    c.rect(4, 14, 2, 8, shirt[0]);
    c.rect(8, 17, 3, 3, hex(0xd8a040));
    c.set(2, 16, burlap[2]);
    c.set(13, 16, burlap[2]);
    // testa a sacco
    c.ellipse(8.0, 9.0, 4.2, 4.0, burlap[1]);
    c.ellipse(7.0, 8.0, 2.0, 1.8, burlap[2]);
    c.set(6, 9, OUTLINE);
    c.set(10, 9, OUTLINE);
    c.hline(6, 10, 11, burlap[0]);
    c.set(7, 11, OUTLINE);
    c.set(9, 11, OUTLINE);
    // cappello
    c.rect(2, 5, 12, 1, D[3]);
    c.rect(4, 2, 8, 3, D[2]);
    c.hline(4, 11, 4, ROOF[2]);
    c.outline();
    c
}

pub fn lamp() -> Canvas {
    let mut c = Canvas::new(16, 40);
    let iron = [hex(0x24242e), hex(0x3a3a48), hex(0x585868)];
    c.rect(7, 10, 2, 28, iron[1]);
    c.vline(7, 10, 37, iron[2]);
    c.rect(5, 36, 6, 3, iron[0]);
    c.rect(4, 4, 8, 7, iron[1]);
    c.rect(5, 5, 6, 5, hex(0xffe08a));
    c.rect(6, 6, 2, 2, hex(0xfff6d0));
    c.hline(3, 12, 3, iron[0]);
    c.hline(5, 10, 2, iron[1]);
    c.outline();
    c
}

// ------------------------------------------------------------------ terreno zappato

/// mask: bit0 N, bit1 E, bit2 S, bit3 W = vicino anch'esso zappato
pub fn tilled(mask: u8, wet: bool) -> Canvas {
    let p = if wet { SOILW } else { SOIL };
    let mut c = Canvas::new(16, 16);
    let n = mask & 1 != 0;
    let e = mask & 2 != 0;
    let s = mask & 4 != 0;
    let w = mask & 8 != 0;
    for y in 0..16 {
        for x in 0..16 {
            let top = !n && y == 0;
            let bot = !s && y == 15;
            let lef = !w && x == 0;
            let rig = !e && x == 15;
            // angoli arrotondati
            let corner = (!n && !w && x + y < 2) || (!n && !e && (15 - x) + y < 2) || (!s && !w && x + (15 - y) < 2) || (!s && !e && (15 - x) + (15 - y) < 2);
            if corner {
                continue;
            }
            let mut col = if (y + 1) % 4 == 0 { p[1] } else if (y % 4) == 0 { p[3] } else { p[2] };
            if hashf(x, y, 9) < 0.1 {
                col = p[1];
            }
            if top || (!n && y == 1) {
                col = p[0]; // bordo in ombra (il terreno è più basso)
            }
            if bot {
                col = p[3];
            }
            if lef || rig {
                col = p[1];
            }
            c.set(x, y, col);
        }
    }
    c
}

// ------------------------------------------------------------------ colture

pub fn crop(kind: Crop, stage: usize) -> Canvas {
    let mut c = Canvas::new(16, 32);
    let pal = LEAF;
    if stage == 0 {
        c.ellipse(8.0, 26.0, 4.0, 2.0, SOIL[3]);
        c.set(6, 26, D[4]);
        c.set(9, 25, D[4]);
        c.set(10, 27, D[4]);
        return c;
    }
    if stage == 1 {
        c.vline(8, 23, 27, pal[2]);
        leaf(&mut c, 8.0, 24.0, -PI * 0.85, 3.5, 1.4, &pal);
        leaf(&mut c, 8.5, 24.0, -PI * 0.15, 3.5, 1.4, &pal);
        c.outline();
        return c;
    }
    match kind {
        Crop::Carota => {
            let h = [0.0, 0.0, 5.0, 8.0, 10.0][stage];
            let n = if stage >= 3 { 5 } else { 3 };
            for i in 0..n {
                let a = -PI * 0.5 + (i as f32 - (n - 1) as f32 / 2.0) * 0.38;
                leaf(&mut c, 8.0, 27.0, a, h, 1.3, &pal);
            }
            if stage == 4 {
                c.ellipse(8.0, 27.5, 3.5, 2.0, hex(0xe8782c));
                c.hline(6, 9, 26, hex(0xf8a050));
            }
        }
        Crop::Pomodoro => {
            let top = [0, 0, 16, 9, 6][stage];
            for y in top..29 {
                c.set(7, y, WOOD[3]);
                c.set(8, y, WOOD[1]);
            }
            let mut y = 27;
            let mut side = 0;
            while y > top + 2 {
                let a = if side % 2 == 0 { -PI * 0.9 } else { -PI * 0.1 };
                leaf(&mut c, 7.5, y as f32, a, 4.5, 1.8, &pal);
                y -= 3;
                side += 1;
            }
            if stage == 3 {
                c.set(4, 14, hex(0xf8e050));
                c.set(11, 18, hex(0xf8e050));
            }
            if stage == 4 {
                for (x, yy) in [(4.0, 13.0), (11.0, 16.0), (5.0, 21.0), (11.0, 23.0)] {
                    c.ellipse(x, yy, 2.0, 2.0, hex(0xd83a2a));
                    c.set(x as i32 - 1, yy as i32 - 1, hex(0xff8a6a));
                }
            }
        }
        Crop::Zucca => {
            let s = [0.0, 0.0, 4.0, 6.0, 6.5][stage];
            leaf(&mut c, 8.0, 27.0, -PI * 0.95, s + 1.0, 2.6, &pal);
            leaf(&mut c, 8.0, 27.0, -PI * 0.05, s + 1.0, 2.6, &pal);
            leaf(&mut c, 8.0, 27.0, -PI * 0.5, s, 2.8, &pal);
            if stage == 3 {
                c.ellipse(11.0, 22.0, 1.6, 1.6, hex(0xf8d040));
            }
            if stage == 4 {
                let o = [hex(0xa84a14), hex(0xd8661c), hex(0xf08a2c), hex(0xffb050)];
                for y in 20..30 {
                    for x in 1..15 {
                        let dx = (x as f32 + 0.5 - 8.0) / 6.8;
                        let dy = (y as f32 + 0.5 - 25.0) / 4.8;
                        if dx * dx + dy * dy <= 1.0 {
                            let rib = (x - 1) % 4 == 0;
                            let col = if rib { o[0] } else if dx < -0.3 && dy < 0.2 { o[3] } else if dx > 0.4 || dy > 0.6 { o[1] } else { o[2] };
                            c.set(x, y, col);
                        }
                    }
                }
                c.rect(7, 18, 2, 3, WOOD[1]);
            }
        }
        Crop::Girasole => {
            let top = [0, 0, 18, 8, 7][stage];
            for y in top..29 {
                c.set(7, y, pal[2]);
                c.set(8, y, pal[1]);
            }
            leaf(&mut c, 7.5, 24.0, -PI * 0.88, 5.0, 2.0, &pal);
            leaf(&mut c, 8.0, 20.0, -PI * 0.12, 5.0, 2.0, &pal);
            if stage >= 3 {
                leaf(&mut c, 7.5, 15.0, -PI * 0.9, 4.5, 1.8, &pal);
            }
            if stage == 3 {
                c.ellipse(8.0, 8.0, 2.4, 2.4, pal[2]);
            }
            if stage == 4 {
                let y = [hex(0xc88a14), hex(0xf0c028), hex(0xffe060)];
                for k in 0..12 {
                    let a = k as f32 / 12.0 * PI * 2.0;
                    let px = 8.0 + a.cos() * 4.6;
                    let py = 7.0 + a.sin() * 4.2;
                    c.ellipse(px, py, 1.7, 1.7, if k % 3 == 0 { y[2] } else { y[1] });
                }
                c.ellipse(8.0, 7.0, 3.2, 3.0, hex(0x6a3a18));
                c.set(7, 6, hex(0x8a5428));
                c.set(9, 8, hex(0x4a2410));
            }
        }
    }
    c.outline();
    c
}

// ------------------------------------------------------------------ icone oggetti

pub fn icon(item: Item) -> Canvas {
    let mut c = Canvas::new(16, 16);
    let metal = [hex(0x4a4e5e), hex(0x7a8092), hex(0xaab0c0), hex(0xe0e6f0)];
    match item {
        Item::Hoe => {
            thick_line(&mut c, 3, 14, 10, 5, WOOD[3], WOOD[1]);
            c.rect(9, 2, 5, 3, metal[2]);
            c.rect(12, 2, 2, 6, metal[1]);
            c.hline(9, 12, 2, metal[3]);
        }
        Item::Can => {
            let t = [hex(0x2a5866), hex(0x3c7e90), hex(0x56a4b6), hex(0x8ccedc)];
            c.rect(3, 7, 8, 7, t[2]);
            c.rect(3, 7, 2, 7, t[3]);
            c.rect(9, 7, 2, 7, t[1]);
            c.ellipse(7.0, 7.0, 4.0, 1.5, t[3]);
            c.line(11, 10, 14, 6, t[1]);
            c.line(11, 11, 14, 7, t[2]);
            c.set(14, 5, t[3]);
            c.line(3, 7, 2, 5, t[0]);
            c.line(2, 5, 5, 3, t[0]);
            c.line(5, 3, 8, 5, t[0]);
        }
        Item::Axe => {
            thick_line(&mut c, 3, 14, 10, 4, WOOD[3], WOOD[1]);
            c.rect(8, 2, 6, 5, metal[2]);
            c.vline(14, 1, 7, metal[3]);
            c.rect(8, 2, 2, 5, metal[1]);
        }
        Item::Pick => {
            thick_line(&mut c, 4, 14, 9, 4, WOOD[3], WOOD[1]);
            for x in 2..15 {
                let y = 4 + ((x as f32 - 8.5).abs() * 0.5) as i32 - 2;
                c.set(x, y, metal[2]);
                c.set(x, y + 1, metal[1]);
            }
            c.set(2, 5, metal[3]);
            c.set(14, 5, metal[3]);
        }
        Item::Scythe => {
            thick_line(&mut c, 4, 14, 11, 3, WOOD[3], WOOD[1]);
            for x in 2..12 {
                let y = 3 + ((x as f32 - 2.0) * 0.25) as i32;
                c.set(x, y, metal[3]);
                c.set(x, y + 1, metal[1]);
            }
            c.set(1, 4, metal[2]);
        }
        Item::Seed(k) => {
            let paper = [hex(0xc8b088), hex(0xe8d6b0), hex(0xf8ecd0)];
            let band = match k {
                Crop::Carota => hex(0xe8782c),
                Crop::Pomodoro => hex(0xd83a2a),
                Crop::Zucca => hex(0xd8661c),
                Crop::Girasole => hex(0xf0c028),
            };
            c.rect(3, 2, 10, 13, paper[1]);
            c.rect(3, 2, 10, 2, paper[2]);
            c.vline(12, 2, 14, paper[0]);
            c.rect(3, 5, 10, 3, band);
            c.rect(5, 9, 6, 5, paper[2]);
            c.set(7, 10, OUTLINE);
            c.set(9, 11, OUTLINE);
            c.set(6, 12, OUTLINE);
            c.set(8, 13, G[2]);
        }
        Item::Produce(k) => match k {
            Crop::Carota => {
                let o = [hex(0xb8501c), hex(0xe8782c), hex(0xf8a050)];
                for i in 0..9 {
                    let w = (3.5 - i as f32 * 0.38).max(0.6);
                    c.ellipse(5.0 + i as f32, 6.0 + i as f32 * 0.9, w, w * 0.8, if i % 3 == 2 { o[0] } else { o[1] });
                }
                c.set(4, 5, o[2]);
                c.set(5, 5, o[2]);
                leaf(&mut c, 4.0, 5.0, -PI * 0.7, 4.0, 1.2, &LEAF);
                leaf(&mut c, 4.0, 5.0, -PI * 0.95, 4.0, 1.2, &LEAF);
            }
            Crop::Pomodoro => {
                c.ellipse(8.0, 9.0, 5.5, 5.0, hex(0xd83a2a));
                c.ellipse(7.0, 10.0, 4.0, 3.5, hex(0xe8503a));
                c.set(5, 7, hex(0xff9a80));
                c.set(6, 6, hex(0xff9a80));
                c.hline(6, 10, 4, G[2]);
                c.vline(8, 2, 5, G[1]);
            }
            Crop::Zucca => {
                let o = [hex(0xa84a14), hex(0xd8661c), hex(0xf08a2c), hex(0xffb050)];
                for y in 4..15 {
                    for x in 1..15 {
                        let dx = (x as f32 + 0.5 - 8.0) / 7.0;
                        let dy = (y as f32 + 0.5 - 9.5) / 5.5;
                        if dx * dx + dy * dy <= 1.0 {
                            let rib = (x - 2) % 4 == 0;
                            c.set(x, y, if rib { o[0] } else if dx < -0.2 && dy < 0.0 { o[3] } else { o[2] });
                        }
                    }
                }
                c.rect(7, 1, 2, 4, WOOD[1]);
            }
            Crop::Girasole => {
                for k in 0..10 {
                    let a = k as f32 / 10.0 * PI * 2.0;
                    c.ellipse(8.0 + a.cos() * 4.5, 8.0 + a.sin() * 4.5, 2.0, 2.0, if k % 2 == 0 { hex(0xffe060) } else { hex(0xf0c028) });
                }
                c.ellipse(8.0, 8.0, 3.2, 3.2, hex(0x6a3a18));
                c.set(7, 7, hex(0x9a6430));
            }
        },
        Item::Wood => {
            for (x, y) in [(4.0, 10.0), (11.0, 10.0), (7.5, 5.5)] {
                c.ellipse(x, y, 3.6, 3.6, WOOD[1]);
                c.ellipse(x - 0.3, y - 0.3, 2.8, 2.8, WOOD[3]);
                c.ellipse(x - 0.3, y - 0.3, 1.4, 1.4, WOOD[4]);
                c.set(x as i32, y as i32, WOOD[2]);
            }
        }
        Item::Stone => {
            blobs(&mut c, &[(8.0, 9.0, 5.5), (6.0, 8.0, 3.5)], &STONE, 901, 1.0, 0.2);
        }
        Item::Fiber => {
            for i in 0..5 {
                let x0 = 3 + i * 2;
                for y in 3..14 {
                    let x = x0 + (((y as f32) * 0.6 + i as f32).sin() * 1.2) as i32;
                    c.set(x, y, if i % 2 == 0 { G[3] } else { G[2] });
                }
            }
            c.hline(3, 12, 9, D[2]);
        }
    }
    c.outline();
    c
}

// ------------------------------------------------------------------ personaggi

#[derive(Clone, Copy)]
pub struct Look {
    pub hair: [Rgba; 2],
    pub shirt: [Rgba; 3],
    pub pants: [Rgba; 2],
    pub shoes: Rgba,
    pub hat: bool,
    pub scarf: Option<Rgba>,
    pub bun: bool,
}

pub const FARMER: Look = Look {
    hair: [hex(0x6a3a1c), hex(0x8e5428)],
    shirt: [hex(0x2e6a48), hex(0x3f8c5a), hex(0x5aac70)],
    pants: [hex(0x4a3226), hex(0x6a4a34)],
    shoes: hex(0x3a2418),
    hat: true,
    scarf: Some(hex(0xd8463a)),
    bun: false,
};

pub const MERCHANT: Look = Look {
    hair: [hex(0x2a2032), hex(0x46384e)],
    shirt: [hex(0x6a3a7a), hex(0x8c56a0), hex(0xb07cc4)],
    pants: [hex(0x3a3046), hex(0x544660)],
    shoes: hex(0x2a2020),
    hat: false,
    scarf: None,
    bun: true,
};

/// dir: 0 giù, 1 su, 2 lato (sinistra). frame 0..3 camminata, act = posa con attrezzo
pub fn character(l: &Look, dir: u8, frame: u8, act: bool) -> Canvas {
    let mut c = Canvas::new(16, 32);
    let bob = if !act && (frame == 1 || frame == 3) { -1 } else { 0 };
    let sk = SKIN;
    let step_l = frame == 1;
    let step_r = frame == 3;
    let ey = 0; // offset generico

    // --- gambe e scarpe
    if dir == 2 {
        // di lato: passo avanti/indietro
        let (fx, bx) = if step_l { (3, 9) } else if step_r { (9, 3) } else { (5, 8) };
        let wide = step_l || step_r;
        for (x0, darker) in [(bx, true), (fx, false)] {
            let pc = if darker { l.pants[0] } else { l.pants[1] };
            let w = if wide { 3 } else { 3 };
            c.rect(x0, 25, w, 4, pc);
            c.rect(x0 - if darker { 0 } else { 1 }, 29, w + 1, 2, l.shoes);
        }
    } else {
        let ly = if step_l { -1 } else { 0 };
        let ry = if step_r { -1 } else { 0 };
        c.rect(4, 25 + ly, 4, 4, l.pants[1]);
        c.rect(8, 25 + ry, 4, 4, l.pants[0]);
        c.rect(4, 29 + ly, 4, 2, l.shoes);
        c.rect(8, 29 + ry, 4, 2, l.shoes);
        c.set(7, 26, l.pants[0]);
    }

    let by = bob + ey;
    // --- busto
    let (bx0, bw) = if dir == 2 { (5, 7) } else { (4, 8) };
    for y in 16..25 {
        for x in bx0..bx0 + bw {
            let rel = (x - bx0) as f32 / bw as f32;
            let col = if rel < 0.25 { l.shirt[2] } else if rel > 0.75 { l.shirt[0] } else { l.shirt[1] };
            c.set(x, y + by, col);
        }
    }
    // cintura
    c.hline(bx0, bx0 + bw - 1, 23 + by, WOOD[1]);
    if dir == 0 {
        c.set(7, 23 + by, hex(0xe8c050));
        c.set(8, 23 + by, hex(0xe8c050));
    }
    if let Some(sc) = l.scarf {
        if dir != 1 {
            c.hline(bx0, bx0 + bw - 1, 16 + by, sc);
            if dir == 0 {
                c.set(6, 17 + by, sc);
                c.set(7, 17 + by, shade(sc, 0.8));
                c.set(7, 18 + by, sc);
            }
        } else {
            c.hline(bx0, bx0 + bw - 1, 16 + by, shade(sc, 0.8));
        }
    }

    // --- braccia
    let swing = if step_l { 1 } else if step_r { -1 } else { 0 };
    if dir == 2 {
        if act {
            c.rect(2, 18 + by, 5, 2, l.shirt[1]);
            c.rect(1, 18 + by, 2, 2, sk[1]);
        } else {
            c.rect(7, 17 + by + swing.max(0), 2, 5, l.shirt[2]);
            c.rect(7 - swing, 22 + by, 2, 2, sk[1]);
        }
    } else if act {
        if dir == 0 {
            c.rect(3, 17 + by, 2, 4, l.shirt[2]);
            c.rect(11, 17 + by, 2, 4, l.shirt[0]);
            c.rect(5, 20 + by, 6, 2, sk[1]);
        } else {
            c.rect(3, 15 + by, 2, 4, l.shirt[2]);
            c.rect(11, 15 + by, 2, 4, l.shirt[0]);
        }
    } else {
        c.rect(2, 17 + by + swing, 2, 5, l.shirt[2]);
        c.rect(12, 17 + by - swing, 2, 5, l.shirt[0]);
        c.rect(2, 22 + by + swing, 2, 2, sk[1]);
        c.rect(12, 22 + by - swing, 2, 2, sk[0]);
    }

    // --- testa
    let hy = 7 + by;
    for y in 0..9 {
        for x in 0..8 {
            let corner = (y == 0 || y == 8) && (x == 0 || x == 7);
            if corner {
                continue;
            }
            let col = if x == 0 { sk[2] } else if x >= 6 { sk[0] } else { sk[1] };
            c.set(4 + x, hy + y, col);
        }
    }
    match dir {
        0 => {
            // capelli + viso
            c.hline(4, 11, hy, l.hair[1]);
            c.hline(4, 11, hy + 1, l.hair[1]);
            c.hline(5, 10, hy + 2, l.hair[0]);
            c.vline(3, hy + 1, hy + 5, l.hair[0]);
            c.vline(12, hy + 1, hy + 5, l.hair[0]);
            c.set(4, hy + 2, l.hair[1]);
            c.set(11, hy + 2, l.hair[0]);
            c.rect(6, hy + 4, 1, 2, OUTLINE);
            c.rect(9, hy + 4, 1, 2, OUTLINE);
            c.set(6, hy + 4, hex(0x4a3a3a));
            c.set(5, hy + 6, hex(0xf08a8a));
            c.set(10, hy + 6, hex(0xf08a8a));
            c.set(7, hy + 7, sk[0]);
            c.set(8, hy + 7, sk[0]);
        }
        1 => {
            for y in 0..8 {
                for x in 0..10 {
                    if (y == 7) && (x == 0 || x == 9) {
                        continue;
                    }
                    c.set(3 + x, hy + y, if (x + y) % 4 == 0 { l.hair[1] } else { l.hair[0] });
                }
            }
        }
        _ => {
            // profilo a sinistra
            for y in 0..7 {
                for x in 4..10 {
                    let xx = 3 + x;
                    if y < 2 || x >= 6 {
                        c.set(xx, hy + y, if (x + y) % 3 == 0 { l.hair[1] } else { l.hair[0] });
                    }
                }
            }
            c.hline(4, 9, hy, l.hair[1]);
            c.set(3, hy + 4, sk[1]); // naso
            c.rect(5, hy + 3, 1, 2, OUTLINE);
            c.set(6, hy + 6, hex(0xf08a8a));
        }
    }
    if l.bun {
        let bx = if dir == 2 { 12 } else { 8 };
        c.ellipse(bx as f32, (hy - 1) as f32, 2.5, 2.2, l.hair[0]);
        c.set(bx - 1, hy - 2, l.hair[1]);
    }
    if l.hat {
        let straw = [hex(0xb88a38), hex(0xe2b45a), hex(0xf4d07c)];
        let ty = hy - 2;
        // tesa
        let (x0, x1) = if dir == 2 { (1, 12) } else { (1, 14) };
        c.hline(x0, x1, ty + 2, straw[1]);
        c.hline(x0 + 1, x1 - 1, ty + 3, straw[0]);
        if dir == 0 {
            c.hline(2, 13, ty + 2, straw[2]);
        }
        // cupola
        for y in ty - 3..ty + 2 {
            for x in 4..12 {
                let col = if x < 6 { straw[2] } else if x > 9 { straw[0] } else { straw[1] };
                c.set(x, y, col);
            }
        }
        c.hline(5, 10, ty - 4, straw[1]);
        c.hline(4, 11, ty + 1, hex(0xc8463a));
    }
    c.outline();
    c
}

pub fn chicken(frame: u8) -> Canvas {
    let mut c = Canvas::new(16, 16);
    let w = [hex(0xc8c0b4), hex(0xe8e2d8), hex(0xfffaf2)];
    c.ellipse(9.0, 10.0, 5.0, 3.8, w[1]);
    c.ellipse(8.0, 9.0, 3.0, 2.0, w[2]);
    c.ellipse(13.0, 7.5, 2.0, 2.5, w[0]);
    let (hx, hy) = if frame == 2 { (4.0, 9.5) } else { (5.0, 6.0) };
    c.ellipse(hx, hy, 2.4, 2.4, w[2]);
    c.set(hx as i32 - 1, hy as i32 - 3, hex(0xe03a2a));
    c.set(hx as i32, hy as i32 - 3, hex(0xe03a2a));
    c.set(hx as i32, hy as i32 - 2, hex(0xe03a2a));
    c.set(hx as i32 - 3, hy as i32, hex(0xf0b030));
    c.set(hx as i32 - 3, hy as i32 + 1, hex(0xd09020));
    c.set(hx as i32 - 1, hy as i32 - 1, OUTLINE);
    c.set(hx as i32 - 1, hy as i32 + 2, hex(0xe03a2a));
    c.set(10, 10, w[0]);
    c.set(11, 11, w[0]);
    let lo = if frame == 1 { 1 } else { 0 };
    c.vline(8, 13, 14, hex(0xe0902a));
    c.vline(10 + lo, 13, 14, hex(0xe0902a));
    c.outline();
    c
}

pub fn butterfly(frame: u8) -> Canvas {
    let mut c = Canvas::new(8, 8);
    let wcol = [255, 255, 255, 255];
    let edge = [200, 200, 200, 255];
    if frame == 0 {
        c.ellipse(2.0, 3.0, 2.0, 2.0, wcol);
        c.ellipse(6.0, 3.0, 2.0, 2.0, wcol);
        c.ellipse(2.5, 5.5, 1.3, 1.3, edge);
        c.ellipse(5.5, 5.5, 1.3, 1.3, edge);
    } else {
        c.ellipse(3.0, 3.0, 1.0, 2.0, wcol);
        c.ellipse(5.0, 3.0, 1.0, 2.0, wcol);
    }
    c.vline(4, 2, 6, [70, 50, 40, 255]);
    c
}

pub fn shadow(w: i32, h: i32) -> Canvas {
    let mut c = Canvas::new(w, h);
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    for y in 0..h {
        for x in 0..w {
            let dx = (x as f32 + 0.5 - cx) / cx;
            let dy = (y as f32 + 0.5 - cy) / cy;
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                let a = if d < 0.55 { 80 } else { 50 };
                c.set(x, y, [24, 36, 16, a]);
            }
        }
    }
    c
}

pub fn glow() -> Canvas {
    let s = 64;
    let mut c = Canvas::new(s, s);
    for y in 0..s {
        for x in 0..s {
            let dx = (x as f32 + 0.5 - 32.0) / 32.0;
            let dy = (y as f32 + 0.5 - 32.0) / 32.0;
            let d = (dx * dx + dy * dy).sqrt();
            if d < 1.0 {
                let a = (1.0 - d).powf(1.8);
                // gradini morbidi per un look pixel
                let q = (a * 6.0).ceil() / 6.0;
                c.set(x, y, [255, 255, 255, (q * 255.0) as u8]);
            }
        }
    }
    c
}

// ------------------------------------------------------------------ icone UI

pub fn coin() -> Canvas {
    let mut c = Canvas::new(9, 9);
    c.ellipse(4.5, 4.5, 4.0, 4.0, hex(0xd8a020));
    c.ellipse(4.5, 4.5, 3.0, 3.0, hex(0xf8d048));
    c.vline(4, 3, 5, hex(0xb07a10));
    c.set(3, 2, hex(0xfff0a0));
    c.outline_with(hex(0x6a4210));
    c
}

pub fn sun_icon() -> Canvas {
    let mut c = Canvas::new(11, 11);
    for k in 0..8 {
        let a = k as f32 / 8.0 * PI * 2.0;
        c.set((5.5 + a.cos() * 4.6) as i32, (5.5 + a.sin() * 4.6) as i32, hex(0xffc030));
    }
    c.ellipse(5.5, 5.5, 3.0, 3.0, hex(0xffd84a));
    c.set(4, 4, hex(0xfff4b0));
    c
}

pub fn rain_icon() -> Canvas {
    let mut c = Canvas::new(11, 11);
    c.ellipse(4.0, 4.0, 3.0, 2.5, hex(0xb8c4d8));
    c.ellipse(7.0, 3.5, 3.0, 2.8, hex(0xd8e0ec));
    c.hline(1, 9, 5, hex(0x98a4bc));
    for (x, y) in [(2, 8), (5, 9), (8, 8)] {
        c.set(x, y, WAT[3]);
        c.set(x, y - 1, WAT[4]);
    }
    c
}

pub fn heart() -> Canvas {
    let mut c = Canvas::new(9, 8);
    c.ellipse(2.5, 2.5, 2.3, 2.3, hex(0xe8463a));
    c.ellipse(6.5, 2.5, 2.3, 2.3, hex(0xe8463a));
    for y in 3..8 {
        let w = 7 - y;
        c.hline(4 - w, 4 + w, y, hex(0xe8463a));
    }
    c.set(2, 1, hex(0xffa0a0));
    c
}

// ------------------------------------------------------------------ arredo della piazza

pub fn fountain() -> Canvas {
    let mut c = Canvas::new(48, 46);
    // parete frontale della vasca
    for y in 30..44 {
        for x in 1..47 {
            let off = if (y / 4) % 2 == 0 { 0 } else { 4 };
            let mortar = (x + off) % 8 == 0 || y % 4 == 1;
            let rel = x as f32 / 48.0;
            let base = if rel < 0.25 { STONE[3] } else if rel < 0.75 { STONE[2] } else { STONE[1] };
            c.set(x, y, if mortar { STONE[1] } else { base });
        }
    }
    c.ellipse(24.0, 43.0, 23.0, 3.0, STONE[1]);
    // bordo superiore e acqua
    c.ellipse(24.0, 30.0, 23.0, 9.0, STONE[3]);
    c.ellipse(24.0, 29.5, 22.0, 8.0, STONE[4]);
    c.ellipse(24.0, 30.5, 19.0, 6.5, WAT[1]);
    c.ellipse(23.0, 31.0, 15.0, 4.5, WAT[2]);
    for (x, y) in [(12, 29), (30, 33), (20, 34), (34, 28)] {
        c.set(x, y, WAT[4]);
        c.set(x + 1, y, WAT[4]);
    }
    // colonna centrale
    for y in 12..31 {
        for x in 21..28 {
            c.set(x, y, if x < 23 { STONE[4] } else if x > 25 { STONE[1] } else { STONE[3] });
        }
    }
    c.ellipse(24.0, 30.0, 5.0, 2.0, STONE[2]);
    // coppa in alto
    c.ellipse(24.0, 12.0, 9.0, 3.5, STONE[3]);
    c.ellipse(24.0, 11.5, 7.5, 2.4, WAT[2]);
    c.ellipse(24.0, 13.5, 9.0, 1.6, STONE[1]);
    c.rect(23, 6, 3, 6, WAT[4]);
    c.set(24, 5, WAT[5]);
    c.outline();
    c
}

pub fn bench() -> Canvas {
    let mut c = Canvas::new(32, 24);
    let iron = [hex(0x24242e), hex(0x3a3a48)];
    for x in [3, 28] {
        c.vline(x, 4, 22, iron[1]);
        c.vline(x + 1, 4, 22, iron[0]);
    }
    for (y, col) in [(5, WOOD[3]), (6, WOOD[2]), (8, WOOD[3]), (9, WOOD[1])] {
        c.hline(2, 29, y, col);
    }
    for y in 13..17 {
        c.hline(1, 30, y, if y == 13 { WOOD[4] } else if y == 16 { WOOD[1] } else { WOOD[3] });
    }
    c.hline(1, 30, 17, WOOD[0]);
    c.outline();
    c
}

pub fn planter(var: u32) -> Canvas {
    let mut c = Canvas::new(16, 24);
    for y in 13..23 {
        for x in 2..14 {
            let col = if y == 15 || y == 20 { STONE[1] } else if x < 5 { WOOD[3] } else if x > 10 { WOOD[1] } else { WOOD[2] };
            c.set(x, y, col);
        }
    }
    c.ellipse(8.0, 13.0, 6.0, 1.6, WOOD[0]);
    blobs(&mut c, &[(8.0, 9.0, 5.5), (4.5, 11.0, 3.5), (11.5, 11.0, 3.5)], &LEAF, 800 + var, 1.4, 0.4);
    let fl = [[hex(0xf06a8a), hex(0xffd0e0)], [hex(0xf4b030), hex(0xfff0a0)], [hex(0x9a70e0), hex(0xe0d0ff)]][var as usize % 3];
    let mut r = Rng::new(var as u64 + 31);
    for _ in 0..7 {
        let x = r.range(3, 13);
        let y = r.range(4, 12);
        if c.solid(x, y) {
            c.set(x, y, fl[0]);
            c.set(x, y - 1, fl[1]);
        }
    }
    c.outline();
    c
}
