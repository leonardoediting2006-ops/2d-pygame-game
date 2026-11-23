//! Tela per dipingere pixel art via codice + tavolozza del gioco.

use crate::util::*;
use macroquad::prelude::*;

#[derive(Clone)]
pub struct Canvas {
    pub w: i32,
    pub h: i32,
    pub px: Vec<Rgba>,
}

impl Canvas {
    pub fn new(w: i32, h: i32) -> Self {
        Canvas { w, h, px: vec![CLEAR; (w * h) as usize] }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgba) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> Rgba {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize]
        } else {
            CLEAR
        }
    }

    #[inline]
    pub fn solid(&self, x: i32, y: i32) -> bool {
        self.get(x, y)[3] > 0
    }

    /// Cambia colore solo se il pixel è già pieno
    pub fn paint(&mut self, x: i32, y: i32, c: Rgba) {
        if self.solid(x, y) {
            self.set(x, y, c);
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgba) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: Rgba) {
        for x in x0..=x1 {
            self.set(x, y, c);
        }
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: Rgba) {
        for y in y0..=y1 {
            self.set(x, y, c);
        }
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: Rgba) {
        let y0 = (cy - ry).floor() as i32;
        let y1 = (cy + ry).ceil() as i32;
        let x0 = (cx - rx).floor() as i32;
        let x1 = (cx + rx).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.set(x, y, c);
                }
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgba) {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Contorno "selettivo": ogni pixel trasparente vicino a uno pieno
    /// prende una versione scura del colore vicino (stile pixel art moderno).
    pub fn outline(&mut self) {
        let src = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                if src.solid(x, y) {
                    continue;
                }
                let n = [(0, 1), (1, 0), (-1, 0), (0, -1)];
                for (dx, dy) in n {
                    let c = src.get(x + dx, y + dy);
                    if c[3] > 200 {
                        self.set(x, y, mix(shade(c, 0.42), OUTLINE, 0.45));
                        break;
                    }
                }
            }
        }
    }

    pub fn outline_with(&mut self, col: Rgba) {
        let src = self.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                if src.solid(x, y) {
                    continue;
                }
                if src.solid(x + 1, y) || src.solid(x - 1, y) || src.solid(x, y + 1) || src.solid(x, y - 1) {
                    self.set(x, y, col);
                }
            }
        }
    }

    pub fn blit(&mut self, o: &Canvas, x: i32, y: i32) {
        for yy in 0..o.h {
            for xx in 0..o.w {
                let c = o.get(xx, yy);
                if c[3] > 0 {
                    self.set(x + xx, y + yy, c);
                }
            }
        }
    }

    pub fn flipped_h(&self) -> Canvas {
        let mut c = Canvas::new(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                c.set(self.w - 1 - x, y, self.get(x, y));
            }
        }
        c
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(self.px.len() * 4);
        for p in &self.px {
            b.extend_from_slice(p);
        }
        b
    }

    pub fn tex(&self) -> Texture2D {
        let t = Texture2D::from_rgba8(self.w as u16, self.h as u16, &self.bytes());
        t.set_filter(FilterMode::Nearest);
        t
    }

    pub fn save_png(&self, path: &str) {
        // export_png di macroquad capovolge l'immagine (pensata per lo schermo):
        // la capovolgiamo prima così il risultato è dritto.
        let mut bytes = vec![0u8; self.px.len() * 4];
        for y in 0..self.h {
            for x in 0..self.w {
                let c = self.get(x, self.h - 1 - y);
                let i = ((y * self.w + x) * 4) as usize;
                bytes[i..i + 4].copy_from_slice(&c);
            }
        }
        let img = Image { bytes, width: self.w as u16, height: self.h as u16 };
        img.export_png(path);
    }
}

// ---------------------------------------------------------------- tavolozza
pub const OUTLINE: Rgba = hex(0x2a1a16);

pub const G: [Rgba; 5] = [hex(0x2c6a2e), hex(0x3d8a34), hex(0x55a83c), hex(0x6cbe45), hex(0x90d65a)];
pub const D: [Rgba; 5] = [hex(0x9c6428), hex(0xc0823a), hex(0xd99c47), hex(0xe9b35c), hex(0xf5cc7a)];
pub const SOIL: [Rgba; 4] = [hex(0x5e331b), hex(0x7a4526), hex(0x925a30), hex(0xa96d3a)];
pub const SOILW: [Rgba; 4] = [hex(0x371d0f), hex(0x4c2a16), hex(0x5f381f), hex(0x724629)];
pub const WAT: [Rgba; 6] = [hex(0x2a58a2), hex(0x346dc0), hex(0x4384d8), hex(0x60a2ea), hex(0xa4d2f8), hex(0xe4f5ff)];
pub const WOOD: [Rgba; 5] = [hex(0x55301a), hex(0x7a4824), hex(0xa4642f), hex(0xc7843f), hex(0xe4a85a)];
pub const STONE: [Rgba; 5] = [hex(0x484858), hex(0x6a6a7c), hex(0x8c8c9c), hex(0xb0b0be), hex(0xd6d6e0)];
pub const LEAF: [Rgba; 5] = [hex(0x1d4c26), hex(0x2c6b2f), hex(0x3e8b36), hex(0x58aa40), hex(0x80c955)];
pub const PINE: [Rgba; 5] = [hex(0x143e3a), hex(0x1d5848), hex(0x2a7256), hex(0x3c8e63), hex(0x5aab74)];
pub const BLOSSOM: [Rgba; 5] = [hex(0x8a3c6c), hex(0xb65890), hex(0xd97db2), hex(0xefa7cd), hex(0xffd6ea)];
pub const ROOF: [Rgba; 5] = [hex(0x641c16), hex(0x8e2b1e), hex(0xb83d28), hex(0xd95934), hex(0xf07c48)];
pub const ROOFB: [Rgba; 5] = [hex(0x1e2c50), hex(0x2c4274), hex(0x3c5c9c), hex(0x5478c0), hex(0x78a0dc)];
pub const CLIFF: [Rgba; 5] = [hex(0x4a3620), hex(0x664c2e), hex(0x82653a), hex(0x9e7f4a), hex(0xbb9c5f)];
pub const COBBLE: [Rgba; 4] = [hex(0x6a5a4c), hex(0x8e7c68), hex(0xa8947c), hex(0xc4b096)];
pub const SKIN: [Rgba; 3] = [hex(0xc78058), hex(0xeaa77a), hex(0xf8c89a)];
