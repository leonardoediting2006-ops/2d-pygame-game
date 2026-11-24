//! Detailed buildings and props: farmhouse, cottage, well, shipping chest.
//! Every building is assembled from reusable pieces (roof, siding, stones,
//! windows, doors...). Light always comes from the top left.

use crate::canvas::*;
use crate::util::*;

/// Where the lit parts of a building are, in sprite coordinates.
#[derive(Clone, Default)]
pub struct HouseInfo {
    pub w: i32,
    pub h: i32,
    /// sprite y of the bottom of the footprint (where the porch starts)
    pub base_y: i32,
    pub windows: Vec<(i32, i32, i32, i32)>,
    pub lanterns: Vec<(i32, i32)>,
    pub chimney: Option<(i32, i32)>,
}

const CREAM: [Rgba; 4] = [hex(0xb8a07a), hex(0xd8c49c), hex(0xf0e2c2), hex(0xfffaea)];
const IRON: [Rgba; 3] = [hex(0x22222c), hex(0x3a3a48), hex(0x5c5c6e)];
const BRASS: [Rgba; 2] = [hex(0xb08420), hex(0xf2c440)];

// ------------------------------------------------------------------ pieces

/// Front-facing roof with rows of rounded shingles (style 0) or flat slates (style 1).
pub(crate) fn roof(c: &mut Canvas, y0: i32, y1: i32, slope: f32, pal: &[Rgba; 5], style: u8, seed: u32) {
    let w = c.w;
    let (rh, sw) = if style == 0 { (5, 6) } else { (4, 7) };
    let rows = (y1 - y0 - 3) / rh;
    for y in y0..=y1 + 2 {
        let inset = ((y1 - y.min(y1)) as f32 * slope) as i32;
        for x in inset..w - inset {
            let mut col;
            if y < y0 + 3 {
                // ridge cap
                col = match y - y0 {
                    0 => pal[1],
                    1 => pal[4],
                    _ => pal[3],
                };
                if (x - inset) % 8 == 7 {
                    col = pal[1];
                }
            } else if y > y1 {
                // eave edge
                col = if y == y1 + 1 { pal[1] } else { pal[0] };
            } else {
                let ly = y - y0 - 3;
                let row = ly / rh;
                let ry = ly % rh;
                let sx = (x + row * (sw / 2)).rem_euclid(sw);
                let id = (x + row * (sw / 2)).div_euclid(sw);
                let v = hashf(id, row, seed);
                let body = if v < 0.18 {
                    shade(pal[2], 0.88)
                } else if v > 0.9 {
                    pal[3]
                } else {
                    pal[2]
                };
                col = if style == 0 {
                    if ry == rh - 1 || (ry == rh - 2 && (sx == 0 || sx == sw - 1)) {
                        pal[1]
                    } else if sx == 0 {
                        pal[1]
                    } else if ry == 0 {
                        pal[3]
                    } else if ry == 1 && sx == 1 {
                        pal[4]
                    } else {
                        body
                    }
                } else if ry == rh - 1 || sx == 0 {
                    pal[1]
                } else if ry == 0 && sx < sw - 2 {
                    if v > 0.6 { pal[4] } else { pal[3] }
                } else {
                    body
                };
                // moss on the lowest rows
                let rel_x = x as f32 / w as f32;
                let edge_zone = rel_x < 0.14 || rel_x > 0.86;
                if (row >= rows - 2 || (row >= rows - 5 && edge_zone)) && ry < rh - 1 && sx != 0 {
                    let m = vnoise(x as f32 * 0.22, y as f32 * 0.35, seed + 9);
                    if m > 0.7 {
                        col = if hashf(x, y, seed) < 0.5 { G[1] } else { G[2] };
                    }
                }
                // global light: lighter top-left, darker right side
                let rel = x as f32 / w as f32;
                if rel > 0.8 {
                    col = shade(col, 0.9);
                } else if rel < 0.18 && y < y0 + rows * rh / 2 {
                    col = mix(col, pal[4], 0.15);
                }
            }
            // barge boards along the sloped sides
            if y >= y0 && y <= y1 && (x - inset < 2 || w - inset - 1 - x < 2) {
                col = if x - inset == 0 || w - inset - 1 - x == 0 { WOOD[0] } else { WOOD[2] };
            }
            c.set(x, y, col);
        }
    }
}

/// Horizontal clapboard siding with grain, knots and joints.
fn siding(c: &mut Canvas, x0: i32, x1: i32, y0: i32, y1: i32, seed: u32) {
    for y in y0..y1 {
        let board = (y - y0) / 4;
        let r = (y - y0) % 4;
        let joint = 8 + (hash(board, 0, seed) % 22) as i32;
        let joint2 = joint + 26 + (hash(board, 1, seed) % 14) as i32;
        for x in x0..x1 {
            let lx = x - x0;
            let mut col = match r {
                0 => WOOD[4],
                3 => WOOD[1],
                _ => {
                    let g = vnoise(x as f32 * 0.35, board as f32 * 3.1, seed + 4);
                    if g > 0.72 {
                        WOOD[2]
                    } else if g < 0.18 {
                        mix(WOOD[3], WOOD[4], 0.4)
                    } else {
                        WOOD[3]
                    }
                }
            };
            if (lx == joint || lx == joint2 || lx == joint2 + 30) && r < 3 {
                col = WOOD[1];
            }
            if r == 1 && hashf(x, board, seed + 2) < 0.012 {
                col = WOOD[1];
            }
            // weathering towards the ground
            let k = (y - y0) as f32 / (y1 - y0) as f32;
            if k > 0.7 {
                col = shade(col, 1.0 - (k - 0.7) * 0.25);
            }
            c.set(x, y, col);
        }
    }
}

/// Cream plaster with mottling and a few hairline cracks.
fn plaster(c: &mut Canvas, x0: i32, x1: i32, y0: i32, y1: i32, seed: u32) {
    for y in y0..y1 {
        for x in x0..x1 {
            let n = vnoise(x as f32 * 0.18, y as f32 * 0.18, seed) + (hashf(x, y, seed + 1) - 0.5) * 0.25;
            let mut col = if n > 0.72 {
                CREAM[3]
            } else if n < 0.3 {
                CREAM[1]
            } else {
                CREAM[2]
            };
            let k = (y - y0) as f32 / (y1 - y0) as f32;
            if k > 0.75 && hashf(x, y, seed + 3) < (k - 0.75) * 2.0 {
                col = CREAM[1];
            }
            c.set(x, y, col);
        }
    }
    let mut r = Rng::new(seed as u64);
    for _ in 0..4 {
        let mut x = r.range(x0 + 3, x1 - 3);
        let mut y = r.range(y0 + 3, y1 - 6);
        for _ in 0..r.range(3, 6) {
            c.set(x, y, CREAM[0]);
            y += 1;
            x += r.range(-1, 2);
        }
    }
}

/// A dark timber beam (vertical if w < h).
fn beam(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32) {
    for yy in y..y + h {
        for xx in x..x + w {
            let edge_lo = if w < h { xx == x } else { yy == y };
            let edge_hi = if w < h { xx == x + w - 1 } else { yy == y + h - 1 };
            let col = if edge_lo {
                WOOD[2]
            } else if edge_hi {
                WOOD[0]
            } else {
                WOOD[1]
            };
            c.set(xx, yy, col);
        }
    }
}

fn diagonal_beam(c: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32) {
    c.line(x0, y0, x1, y1, WOOD[1]);
    c.line(x0 + 1, y0, x1 + 1, y1, WOOD[1]);
    c.line(x0 + 2, y0, x1 + 2, y1, WOOD[0]);
}

/// Foundation of individual stones with highlights, mortar and moss.
pub(crate) fn stones(c: &mut Canvas, x0: i32, x1: i32, y0: i32, y1: i32, seed: u32) {
    let mut r = Rng::new(seed as u64);
    let mut y = y0;
    let mut row = 0;
    while y < y1 {
        let h = if row % 2 == 0 { 4 } else { 3 }.min(y1 - y);
        let mut x = x0 - r.range(0, 4);
        while x < x1 {
            let w = r.range(4, 9);
            let tone = r.f();
            let base = if tone < 0.3 {
                STONE[2]
            } else if tone < 0.85 {
                mix(STONE[3], D[2], 0.12)
            } else {
                mix(STONE[2], D[1], 0.25)
            };
            let mossy = row > 0 && r.chance(0.35);
            for yy in y..y + h {
                for xx in x.max(x0)..(x + w).min(x1) {
                    let lx = xx - x;
                    let ly = yy - y;
                    let mut col = if lx == 0 || ly == h - 1 {
                        STONE[0]
                    } else if ly == 0 || lx == 1 {
                        STONE[4]
                    } else if lx == w - 1 || ly == h - 2 {
                        STONE[1]
                    } else {
                        base
                    };
                    if mossy && ly >= h - 2 && lx > 0 && hashf(xx, yy, seed) < 0.6 {
                        col = if hashf(xx, yy, seed + 1) < 0.5 { G[1] } else { G[2] };
                    }
                    c.set(xx, yy, col);
                }
            }
            x += w;
        }
        y += h;
        row += 1;
    }
}

pub struct WinOpts {
    pub shutters: Option<[Rgba; 3]>,
    pub curtain: Option<Rgba>,
    pub flowers: Option<[Rgba; 2]>,
    pub arch: bool,
}

/// Window with trim, glass reflections, curtains, sill, optional shutters and flower box.
/// Returns the glass rectangle (used for the night glow).
fn window(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, o: &WinOpts, seed: u32) -> (i32, i32, i32, i32) {
    // lintel
    for xx in x - 3..x + w + 3 {
        c.set(xx, y - 4, WOOD[0]);
        c.set(xx, y - 3, WOOD[3]);
        c.set(xx, y - 2, WOOD[1]);
    }
    // frame
    for yy in y - 1..y + h + 1 {
        for xx in x - 2..x + w + 2 {
            let col = if xx == x - 2 || xx == x + w + 1 {
                WOOD[0]
            } else if yy == y - 1 || xx == x - 1 {
                CREAM[3]
            } else if yy == y + h || xx == x + w {
                CREAM[1]
            } else {
                CREAM[2]
            };
            c.set(xx, yy, col);
        }
    }
    // glass: dark interior with sky reflection
    for yy in y..y + h {
        for xx in x..x + w {
            let k = (yy - y) as f32 / h as f32;
            let mut col = if k < 0.3 {
                WAT[3]
            } else if k < 0.7 {
                WAT[2]
            } else {
                WAT[1]
            };
            let d = (xx - x) + (yy - y);
            if d == 3 || d == 4 || d == 8 {
                col = if d == 8 { WAT[3] } else { WAT[4] };
            }
            c.set(xx, yy, col);
        }
    }
    if o.arch {
        // rounded top: cut the corners into the frame colour
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (w - 1, 0), (w - 2, 0), (w - 1, 1)] {
            c.set(x + dx, y + dy, CREAM[2]);
        }
    }
    // curtains, gathered at the bottom
    if let Some(cc) = o.curtain {
        let dark = shade(cc, 0.78);
        for yy in y..y + h {
            let k = (yy - y) as f32 / h as f32;
            let cw = (3.0 - k * 2.2).round().max(1.0) as i32;
            for i in 0..cw {
                let col = if i == cw - 1 { dark } else { cc };
                c.set(x + i, yy, col);
                c.set(x + w - 1 - i, yy, if i == cw - 1 { dark } else { cc });
            }
        }
        c.hline(x, x + w - 1, y, shade(cc, 0.9));
    }
    // mullions
    c.vline(x + w / 2, y, y + h - 1, CREAM[2]);
    c.hline(x, x + w - 1, y + h / 2, CREAM[2]);
    c.set(x + w / 2, y + h / 2, CREAM[1]);
    // sill
    for xx in x - 3..x + w + 3 {
        c.set(xx, y + h + 1, CREAM[3]);
        c.set(xx, y + h + 2, CREAM[1]);
        let below = c.get(xx, y + h + 3);
        if below[3] > 0 {
            c.set(xx, y + h + 3, shade(below, 0.7));
        }
    }
    // shutters with slats and hinges
    if let Some(sp) = o.shutters {
        for (sx, flip) in [(x - 8, false), (x + w + 3, true)] {
            for yy in y - 1..y + h + 1 {
                for i in 0..5 {
                    let xx = sx + i;
                    let edge = i == 0 || i == 4 || yy == y - 1 || yy == y + h;
                    let col = if edge {
                        sp[0]
                    } else if (yy - y) % 2 == 0 {
                        sp[2]
                    } else {
                        sp[1]
                    };
                    c.set(xx, yy, col);
                }
            }
            let hx = if flip { sx } else { sx + 4 };
            c.set(hx, y + 1, IRON[2]);
            c.set(hx, y + h - 2, IRON[2]);
        }
    }
    // flower box with trailing leaves
    if let Some(fl) = o.flowers {
        let by = y + h + 3;
        for yy in by..by + 4 {
            for xx in x - 2..x + w + 2 {
                let col = if yy == by {
                    WOOD[3]
                } else if xx == x - 2 || xx == x + w + 1 || yy == by + 3 {
                    WOOD[0]
                } else if (xx - x) % 5 == 4 {
                    WOOD[1]
                } else {
                    WOOD[2]
                };
                c.set(xx, yy, col);
            }
        }
        let mut r = Rng::new(seed as u64 + 3);
        for xx in x - 2..x + w + 2 {
            let hgt = r.range(1, 4);
            for k in 0..hgt {
                c.set(xx, by - 1 - k, if k == hgt - 1 { G[3] } else { G[2] });
            }
            if r.chance(0.45) {
                let fy = by - hgt - 1;
                c.set(xx, fy, fl[0]);
                c.set(xx, fy - 1, fl[1]);
                if r.chance(0.5) {
                    c.set(xx + 1, fy, fl[0]);
                }
            }
            if r.chance(0.18) {
                let len = r.range(2, 5);
                for k in 0..len {
                    c.set(xx, by + 1 + k, if k % 2 == 0 { G[2] } else { G[1] });
                }
            }
        }
    }
    (x, y, w, h)
}

/// Panelled door with frame, brass handle and small window. `arch` rounds the top.
fn door(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, pal: [Rgba; 3], arch: bool) {
    for yy in y - 2..y + h {
        for xx in x - 2..x + w + 2 {
            let col = if xx == x - 2 || xx == x + w + 1 || yy == y - 2 {
                WOOD[0]
            } else {
                WOOD[1]
            };
            c.set(xx, yy, col);
        }
    }
    for yy in y..y + h {
        for xx in x..x + w {
            let lx = xx - x;
            let ly = yy - y;
            let mut col = if lx == 0 { pal[2] } else if lx == w - 1 { pal[0] } else { pal[1] };
            // two recessed panels
            for (py0, py1) in [(h * 4 / 10, h * 6 / 10 + 1), (h * 7 / 10, h - 3)] {
                if ly >= py0 && ly <= py1 && lx >= 2 && lx <= w - 3 {
                    col = if ly == py0 || lx == 2 {
                        pal[0]
                    } else if ly == py1 || lx == w - 3 {
                        pal[2]
                    } else {
                        shade(pal[1], 0.93)
                    };
                }
            }
            c.set(xx, yy, col);
        }
    }
    if arch {
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (w - 1, 0), (w - 2, 0), (w - 1, 1)] {
            c.set(x + dx, y + dy, WOOD[1]);
        }
        c.set(x - 1, y - 1, WOOD[0]);
        c.set(x + w, y - 1, WOOD[0]);
    }
    // little window
    let wx = x + 2;
    let wy = y + 2;
    let ww = w - 4;
    let wh = (h / 4).max(4);
    c.rect(wx, wy, ww, wh, WAT[2]);
    c.hline(wx, wx + ww - 1, wy, WAT[3]);
    c.set(wx, wy + 1, WAT[4]);
    c.set(wx + 1, wy, WAT[5]);
    c.vline(wx + ww / 2, wy, wy + wh - 1, pal[0]);
    // handle and threshold
    c.set(x + w - 3, y + h / 2 + 2, BRASS[1]);
    c.set(x + w - 3, y + h / 2 + 3, BRASS[0]);
    c.hline(x - 2, x + w + 1, y + h, STONE[3]);
}

fn lantern(c: &mut Canvas, x: i32, y: i32) {
    c.hline(x - 1, x + 2, y, IRON[0]);
    c.set(x + 3, y + 1, IRON[1]);
    c.rect(x - 1, y + 1, 4, 5, IRON[1]);
    c.rect(x, y + 2, 2, 3, hex(0xffe08a));
    c.set(x, y + 2, hex(0xfff6d0));
    c.hline(x - 1, x + 2, y + 6, IRON[0]);
}

fn log_pile(c: &mut Canvas, x0: i32, y0: i32, cols: i32, rows: i32) {
    c.rect(x0, y0, cols * 5 + 1, rows * 5, WOOD[0]);
    for r in 0..rows {
        let n = cols - if r % 2 == 1 { 1 } else { 0 };
        let off = if r % 2 == 1 { 2.5 } else { 0.0 };
        for i in 0..n {
            let cx = x0 as f32 + 3.0 + off + i as f32 * 5.0;
            let cy = y0 as f32 + 2.5 + (rows - 1 - r) as f32 * 4.6;
            c.ellipse(cx, cy, 2.6, 2.4, WOOD[1]);
            c.ellipse(cx - 0.3, cy - 0.3, 1.9, 1.8, WOOD[4]);
            c.ellipse(cx - 0.3, cy - 0.3, 0.9, 0.9, WOOD[3]);
        }
    }
}

pub(crate) fn barrel(c: &mut Canvas, x: i32, y: i32) {
    for yy in y + 2..y + 13 {
        let bulge = if (yy - y - 7).abs() < 3 { 1 } else { 0 };
        for xx in x - bulge..x + 10 + bulge {
            let rel = (xx - x) as f32 / 10.0;
            let mut col = if rel < 0.25 { WOOD[3] } else if rel < 0.7 { WOOD[2] } else { WOOD[1] };
            if (xx - x) % 3 == 0 {
                col = shade(col, 0.85);
            }
            c.set(xx, yy, col);
        }
    }
    for hy in [y + 4, y + 10] {
        for xx in x - 1..x + 11 {
            if c.solid(xx, hy) {
                c.set(xx, hy, if xx < x + 4 { STONE[3] } else { STONE[1] });
            }
        }
    }
    c.ellipse(x as f32 + 5.0, y as f32 + 2.5, 5.0, 2.0, WOOD[3]);
    c.ellipse(x as f32 + 5.0, y as f32 + 2.5, 3.6, 1.2, WOOD[4]);
}

fn potted_plant(c: &mut Canvas, x: i32, y: i32, flower: Rgba) {
    let pot = [hex(0x8a3c22), hex(0xb85a34), hex(0xd8804e)];
    for yy in y + 6..y + 11 {
        let inset = (yy - y - 6) / 3;
        for xx in x + inset..x + 8 - inset {
            c.set(xx, yy, if xx < x + 3 { pot[2] } else if xx > x + 5 { pot[0] } else { pot[1] });
        }
    }
    c.hline(x - 1, x + 8, y + 5, pot[2]);
    c.hline(x - 1, x + 8, y + 6, pot[0]);
    c.ellipse(x as f32 + 4.0, y as f32 + 2.5, 4.5, 3.5, G[2]);
    c.ellipse(x as f32 + 3.0, y as f32 + 1.5, 2.5, 2.0, G[3]);
    c.set(x + 2, y + 1, flower);
    c.set(x + 5, y, flower);
    c.set(x + 6, y + 3, flower);
    c.set(x + 1, y + 4, G[1]);
}

/// Ivy climbing up from (x, y0) to y1.
fn ivy(c: &mut Canvas, x: i32, y0: i32, y1: i32, seed: u32) {
    let mut r = Rng::new(seed as u64);
    let mut xx = x;
    let mut y = y0;
    while y > y1 {
        c.set(xx, y, G[1]);
        if r.chance(0.55) {
            let s = if r.chance(0.5) { -1 } else { 1 };
            c.set(xx + s, y, G[3]);
            c.set(xx + s, y - 1, G[2]);
            if r.chance(0.4) {
                c.set(xx + s * 2, y, G[2]);
            }
        }
        y -= 1;
        if r.chance(0.3) {
            xx += r.range(-1, 2);
        }
    }
}

fn porch(c: &mut Canvas, x0: i32, x1: i32, y0: i32, y1: i32, step_cx: i32) {
    for y in y0..y1 {
        let r = (y - y0) % 3;
        let plank = (y - y0) / 3;
        for x in x0..x1 {
            let seam = (x + plank * 11).rem_euclid(23) == 0;
            let mut col = if r == 2 || seam { WOOD[1] } else if r == 0 { WOOD[4] } else { WOOD[3] };
            if r == 1 && hashf(x, plank, 71) < 0.04 {
                col = WOOD[2];
            }
            c.set(x, y, col);
        }
    }
    // front fascia and the shadow it casts on the ground
    for x in x0..x1 {
        c.set(x, y1, WOOD[2]);
        c.set(x, y1 + 1, WOOD[1]);
        c.set(x, y1 + 2, WOOD[0]);
        c.set(x, y1 + 3, [20, 30, 14, 90]);
    }
    // corner posts
    for px in [x0, x1 - 3] {
        for y in y0 - 4..y1 + 2 {
            for x in px..px + 3 {
                c.set(x, y, if x == px { WOOD[4] } else if x == px + 2 { WOOD[1] } else { WOOD[2] });
            }
        }
        c.hline(px - 1, px + 3, y0 - 5, WOOD[3]);
        c.hline(px - 1, px + 3, y0 - 4, WOOD[1]);
    }
    // two wide steps down to the ground
    for (sy, hw) in [(y1 - 1, step_cx.min(10)), (y1 + 2, 12)] {
        for y in sy..sy + 3 {
            for x in step_cx - hw..step_cx + hw {
                let col = if x == step_cx - hw || x == step_cx + hw - 1 {
                    WOOD[0]
                } else if y == sy {
                    WOOD[4]
                } else if y == sy + 2 {
                    WOOD[1]
                } else {
                    WOOD[3]
                };
                c.set(x, y, col);
            }
        }
    }
}

// ------------------------------------------------------------------ farmhouse

pub fn farmhouse() -> (Canvas, HouseInfo) {
    let (w, h) = (96, 104);
    let mut c = Canvas::new(w, h);
    let mut info = HouseInfo { w, h, base_y: 88, ..Default::default() };

    // walls, posts, foundation
    siding(&mut c, 6, 90, 48, 84, 11);
    stones(&mut c, 5, 91, 82, 89, 12);
    for (px, light) in [(6, true), (86, false)] {
        for y in 48..83 {
            for x in px..px + 4 {
                let col = if x == px { if light { WOOD[4] } else { WOOD[3] } } else if x == px + 3 { WOOD[0] } else { WOOD[2] };
                c.set(x, y, col);
            }
        }
    }
    // porch, door and its little awning
    porch(&mut c, 2, 94, 88, 99, 56);
    c.rect(50, 89, 13, 3, hex(0xa8402c));
    c.hline(50, 62, 89, hex(0xc8603c));
    for x in (51..62).step_by(2) {
        c.set(x, 90, hex(0x7a2a1c));
    }
    door(&mut c, 50, 60, 13, 28, [WOOD[1], WOOD[2], WOOD[3]], false);
    info.windows.push((52, 62, 9, 7));
    for y in 52..58 {
        let inset = (y - 52) / 2;
        for x in 44 - (5 - inset)..69 + (5 - inset) {
            let col = if y == 57 { ROOF[0] } else if y == 56 { ROOF[1] } else if (x + y) % 5 == 0 { ROOF[1] } else if y < 54 { ROOF[3] } else { ROOF[2] };
            c.set(x, y, col);
        }
    }
    for x in [45, 67] {
        c.line(x, 58, x + if x < 56 { 2 } else { -2 }, 61, WOOD[1]);
    }
    lantern(&mut c, 67, 63);
    info.lanterns.push((68, 66));

    // windows
    let pink = hex(0xf0a0b8);
    let left = window(
        &mut c,
        18,
        59,
        15,
        13,
        &WinOpts { shutters: Some([hex(0x24502e), hex(0x3a7a44), hex(0x58a060)]), curtain: Some(pink), flowers: Some([hex(0xf0607a), hex(0xffc8d4)]), arch: false },
        21,
    );
    info.windows.push(left);
    let right = window(
        &mut c,
        73,
        59,
        11,
        11,
        &WinOpts { shutters: None, curtain: Some(hex(0xf4ead0)), flowers: Some([hex(0xf4b030), hex(0xfff0a0)]), arch: false },
        22,
    );
    info.windows.push(right);
    ivy(&mut c, 89, 86, 54, 5);
    ivy(&mut c, 7, 86, 66, 6);

    // roof, dormer, chimney
    roof(&mut c, 6, 48, 0.16, &ROOF, 0, 31);
    // shadow of the eaves on the wall
    for x in 6..90 {
        for (k, f) in [(51, 0.55), (52, 0.7), (53, 0.85)] {
            let p = c.get(x, k);
            if p[3] > 0 {
                c.set(x, k, shade(p, f));
            }
        }
    }
    // dormer
    let dx = 38;
    for y in 22..38 {
        for x in dx..dx + 20 {
            c.set(x, y, if (y - 22) % 4 == 3 { WOOD[1] } else if (y - 22) % 4 == 0 { WOOD[4] } else { WOOD[3] });
        }
    }
    for x in dx..dx + 20 {
        c.set(x, 38, shade(c.get(x, 38), 0.6));
        c.set(x, 39, shade(c.get(x, 39), 0.75));
    }
    let dw = window(&mut c, dx + 6, 26, 8, 8, &WinOpts { shutters: None, curtain: None, flowers: None, arch: true }, 23);
    info.windows.push(dw);
    for k in 0..13 {
        let y = 22 - k;
        let x0 = dx - 3 + k;
        let x1 = dx + 22 - k;
        for x in x0..x1 {
            let edge = x < x0 + 2 || x >= x1 - 2;
            let col = if edge { WOOD[1] } else if k % 2 == 0 { ROOF[3] } else { ROOF[2] };
            c.set(x, y, col);
        }
        c.set(x0, y, WOOD[0]);
        c.set(x1 - 1, y, WOOD[0]);
    }
    c.hline(dx - 3, dx + 22, 23, ROOF[0]);
    // chimney with stone bricks and a cap
    stones(&mut c, 68, 80, 2, 24, 41);
    for y in 2..24 {
        c.set(79, y, shade(c.get(79, y), 0.75));
    }
    c.rect(67, 0, 14, 3, STONE[1]);
    c.hline(67, 80, 0, STONE[3]);
    c.rect(70, 1, 8, 1, IRON[0]);
    for x in 80..84 {
        for y in 14..26 {
            let p = c.get(x, y);
            if p[3] > 0 {
                c.set(x, y, shade(p, 0.8));
            }
        }
    }
    info.chimney = Some((74, 0));

    // porch props
    log_pile(&mut c, 3, 74, 3, 3);
    barrel(&mut c, 77, 78);
    potted_plant(&mut c, 40, 80, hex(0xf06a8a));
    c.outline();
    (c, info)
}

// ------------------------------------------------------------------ cottage

pub fn cottage() -> (Canvas, HouseInfo) {
    let (w, h) = (80, 104);
    let mut c = Canvas::new(w, h);
    let mut info = HouseInfo { w, h, base_y: 88, ..Default::default() };
    plaster(&mut c, 5, 75, 48, 84, 51);
    stones(&mut c, 4, 76, 82, 89, 52);
    // half timbering
    beam(&mut c, 5, 48, 3, 35);
    beam(&mut c, 72, 48, 3, 35);
    beam(&mut c, 5, 66, 70, 3);
    beam(&mut c, 28, 48, 3, 35);
    beam(&mut c, 49, 48, 3, 35);
    diagonal_beam(&mut c, 8, 82, 18, 69);
    diagonal_beam(&mut c, 62, 69, 71, 82);
    // stone path and steps instead of a wooden porch
    for y in 89..100 {
        for x in 24..57 {
            let id = (x / 6 + (y / 4) * 3) as i32;
            let lx = x % 6;
            let ly = y % 4;
            let col = if lx == 0 || ly == 0 { COBBLE[0] } else if hashf(id, 0, 3) < 0.5 { COBBLE[2] } else { COBBLE[3] };
            c.set(x, y, col);
        }
    }
    for x in 24..57 {
        c.set(x, 100, COBBLE[0]);
        c.set(x, 101, [20, 30, 14, 80]);
    }
    // arched teal door
    let teal = [hex(0x23504e), hex(0x337470), hex(0x52a098)];
    door(&mut c, 34, 60, 12, 28, teal, true);
    for (sx, sy) in [(36, 72), (43, 72), (36, 80), (43, 80)] {
        c.set(sx, sy, IRON[2]);
    }
    info.windows.push((36, 62, 8, 7));
    lantern(&mut c, 51, 61);
    info.lanterns.push((52, 64));
    // windows with blue-grey shutters
    let shut = [hex(0x2c3e5c), hex(0x45608a), hex(0x6a88b4)];
    let l = window(&mut c, 14, 54, 9, 9, &WinOpts { shutters: Some(shut), curtain: Some(hex(0xe0c0f0)), flowers: Some([hex(0xa070e0), hex(0xe8d8ff)]), arch: false }, 61);
    info.windows.push(l);
    let r = window(&mut c, 58, 54, 9, 9, &WinOpts { shutters: Some(shut), curtain: Some(hex(0xe0c0f0)), flowers: Some([hex(0xf06a8a), hex(0xffd0e0)]), arch: false }, 62);
    info.windows.push(r);
    // lower windows between beams
    let lw = window(&mut c, 15, 72, 7, 6, &WinOpts { shutters: None, curtain: None, flowers: None, arch: false }, 63);
    info.windows.push(lw);
    // slate roof with chimney on the left
    roof(&mut c, 8, 48, 0.2, &ROOFB, 1, 71);
    for x in 5..75 {
        for (k, f) in [(51, 0.55), (52, 0.72), (53, 0.88)] {
            let p = c.get(x, k);
            if p[3] > 0 {
                c.set(x, k, shade(p, f));
            }
        }
    }
    // round attic window
    c.ellipse(40.0, 30.0, 6.5, 6.5, WOOD[0]);
    c.ellipse(40.0, 30.0, 5.5, 5.5, CREAM[2]);
    c.ellipse(40.0, 30.0, 4.0, 4.0, WAT[2]);
    c.ellipse(39.0, 29.0, 2.0, 2.0, WAT[3]);
    c.vline(40, 26, 33, CREAM[2]);
    c.hline(36, 43, 30, CREAM[2]);
    c.hline(33, 47, 37, ROOFB[0]);
    info.windows.push((37, 27, 7, 7));
    stones(&mut c, 12, 22, 2, 22, 81);
    c.rect(11, 0, 12, 3, STONE[1]);
    c.hline(11, 22, 0, STONE[3]);
    c.rect(14, 1, 6, 1, IRON[0]);
    info.chimney = Some((17, 0));
    // little herb garden and a sleeping cat on the step
    potted_plant(&mut c, 25, 82, hex(0xfff0f4));
    potted_plant(&mut c, 59, 84, hex(0xa070e0));
    let cat = [hex(0xb8602a), hex(0xe08a3c), hex(0xf8b060)];
    c.ellipse(52.0, 95.0, 4.0, 2.4, cat[1]);
    c.ellipse(51.0, 94.0, 2.5, 1.4, cat[2]);
    c.ellipse(48.0, 94.5, 2.0, 1.8, cat[1]);
    c.set(47, 93, cat[0]);
    c.set(49, 93, cat[0]);
    c.hline(53, 57, 97, cat[0]);
    c.set(47, 95, OUTLINE);
    c.outline();
    (c, info)
}

// ------------------------------------------------------------------ well & chest

pub fn well() -> Canvas {
    let mut c = Canvas::new(32, 50);
    // posts with grain
    for (px, l) in [(4, true), (25, false)] {
        for y in 8..36 {
            for x in px..px + 3 {
                let col = if x == px { if l { WOOD[4] } else { WOOD[3] } } else if x == px + 2 { WOOD[0] } else { WOOD[2] };
                c.set(x, y, col);
            }
        }
    }
    // axle with rope and crank
    c.hline(5, 27, 15, WOOD[1]);
    c.hline(5, 27, 16, WOOD[3]);
    for x in 12..20 {
        c.set(x, 15, if x % 2 == 0 { hex(0xe0c890) } else { hex(0xb89a60) });
        c.set(x, 16, if x % 2 == 0 { hex(0xc8aa70) } else { hex(0xe0c890) });
    }
    c.line(28, 16, 30, 19, IRON[1]);
    c.rect(30, 19, 2, 2, WOOD[2]);
    c.vline(16, 17, 23, hex(0xd0b47c));
    // bucket
    for y in 23..29 {
        let inset = (y - 23) / 3;
        for x in 13 + inset..20 - inset {
            c.set(x, y, if x < 15 { WOOD[3] } else if x > 17 { WOOD[1] } else { WOOD[2] });
        }
    }
    c.hline(13, 19, 24, IRON[2]);
    c.hline(14, 18, 27, IRON[1]);
    c.line(13, 23, 16, 21, IRON[1]);
    c.line(16, 21, 19, 23, IRON[1]);
    // stone ring: back rim, water, front wall of individual stones
    c.ellipse(16.0, 34.0, 15.0, 6.0, STONE[2]);
    c.ellipse(16.0, 34.0, 11.5, 3.8, hex(0x1a2a40));
    c.ellipse(16.0, 35.0, 10.0, 2.6, WAT[0]);
    c.set(12, 35, WAT[3]);
    c.set(13, 35, WAT[4]);
    c.set(19, 36, WAT[2]);
    let mut r = Rng::new(17);
    let mut y = 35;
    let mut row = 0;
    while y < 48 {
        let mut x = 1 - r.range(0, 3);
        while x < 31 {
            let w = r.range(4, 7);
            let tone = r.f();
            let base = if tone < 0.4 { STONE[2] } else { mix(STONE[3], D[2], 0.1) };
            let mossy = row >= 2 && r.chance(0.5);
            for yy in y..y + 4 {
                for xx in x.max(1)..(x + w).min(31) {
                    let dx = (xx as f32 + 0.5 - 16.0) / 15.0;
                    // keep the round silhouette at the bottom
                    let bottom = 46.0 + (1.0 - dx * dx).max(0.0).sqrt() * 2.5;
                    if yy as f32 > bottom {
                        continue;
                    }
                    let lx = xx - x;
                    let ly = yy - y;
                    let rel = (xx - 1) as f32 / 30.0;
                    let mut col = if lx == 0 || ly == 3 {
                        STONE[0]
                    } else if ly == 0 {
                        STONE[4]
                    } else {
                        base
                    };
                    if rel > 0.72 && col != STONE[0] {
                        col = shade(col, 0.82);
                    }
                    if mossy && ly >= 2 && lx > 0 && hashf(xx, yy, 5) < 0.65 {
                        col = if hashf(xx, yy, 6) < 0.5 { G[1] } else { G[2] };
                    }
                    c.set(xx, yy, col);
                }
            }
            x += w;
        }
        y += 4;
        row += 1;
    }
    // rim stones on top
    for k in 0..9 {
        let a = std::f32::consts::PI * (k as f32 / 8.0);
        let x = 16.0 - a.cos() * 14.0;
        let yy = 34.0 + a.sin() * 5.2;
        c.ellipse(x, yy, 2.6, 1.7, if k < 4 { STONE[4] } else { STONE[3] });
    }
    // roof: front gable with shingles
    for y in 0..12 {
        let inset = (11 - y) / 2;
        for x in inset..32 - inset {
            let edge = x - inset < 1 || 32 - inset - 1 - x < 1;
            let col = if y == 11 {
                WOOD[0]
            } else if edge {
                WOOD[1]
            } else if y % 3 == 2 {
                ROOF[1]
            } else if (x + (y / 3) * 2) % 5 == 0 {
                ROOF[1]
            } else if y % 3 == 0 {
                ROOF[3]
            } else {
                ROOF[2]
            };
            c.set(x, y, col);
        }
    }
    for x in 3..29 {
        c.set(x, 12, [20, 20, 20, 110]);
    }
    c.outline();
    c
}

pub fn chest() -> Canvas {
    let mut c = Canvas::new(32, 32);
    // body planks
    for y in 15..30 {
        for x in 2..30 {
            let plank = (y - 15) / 5;
            let r = (y - 15) % 5;
            let mut col = if r == 4 {
                WOOD[0]
            } else if r == 0 {
                WOOD[3]
            } else {
                let g = vnoise(x as f32 * 0.4, plank as f32 * 2.3, 9);
                if g > 0.7 { WOOD[1] } else { WOOD[2] }
            };
            if x < 5 {
                col = mix(col, WOOD[4], 0.25);
            }
            if x > 26 {
                col = shade(col, 0.82);
            }
            c.set(x, y, col);
        }
    }
    // lid, slightly domed
    for y in 6..16 {
        let inset = if y < 8 { 2 } else if y < 9 { 1 } else { 0 };
        for x in 1 + inset..31 - inset {
            let col = if y == 15 {
                WOOD[0]
            } else if y == 14 {
                WOOD[1]
            } else if y < 9 {
                WOOD[4]
            } else if (x - 1) % 7 == 6 {
                WOOD[1]
            } else {
                WOOD[3]
            };
            c.set(x, y, col);
        }
    }
    // iron bands and corner brackets with rivets
    for bx in [7, 23] {
        for y in 6..30 {
            if c.solid(bx, y) {
                c.set(bx, y, IRON[2]);
                c.set(bx + 1, y, IRON[1]);
            }
        }
        for ry in [10, 19, 26] {
            c.set(bx, ry, STONE[4]);
        }
    }
    for (x0, y0) in [(1, 13), (27, 13), (2, 26), (27, 26)] {
        c.rect(x0, y0, 3, 3, IRON[1]);
        c.set(x0, y0, IRON[2]);
    }
    // lock
    c.rect(14, 13, 4, 5, BRASS[0]);
    c.rect(14, 13, 3, 4, BRASS[1]);
    c.set(15, 15, IRON[0]);
    // paper label
    c.rect(4, 19, 7, 5, hex(0xf2e6c6));
    c.hline(5, 9, 20, hex(0xa89070));
    c.hline(5, 8, 22, hex(0xa89070));
    c.set(10, 19, hex(0xc8b894));
    c.outline();
    c
}
