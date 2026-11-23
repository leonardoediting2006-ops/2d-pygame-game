//! Piccoli strumenti condivisi: colori, rumore deterministico, generatore casuale.

pub type Rgba = [u8; 4];
pub const CLEAR: Rgba = [0, 0, 0, 0];

pub const fn hex(v: u32) -> Rgba {
    [(v >> 16) as u8, (v >> 8) as u8, v as u8, 255]
}

pub fn shade(c: Rgba, f: f32) -> Rgba {
    [
        (c[0] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[1] as f32 * f).clamp(0.0, 255.0) as u8,
        (c[2] as f32 * f).clamp(0.0, 255.0) as u8,
        c[3],
    ]
}

pub fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).clamp(0.0, 255.0) as u8;
    [l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]), l(a[3], b[3])]
}

pub fn alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

pub fn hash(x: i32, y: i32, s: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ s.wrapping_mul(2_246_822_519).wrapping_add(0x9E37_79B9);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

pub fn hashf(x: i32, y: i32, s: u32) -> f32 {
    (hash(x, y, s) & 0x00ff_ffff) as f32 / 16_777_216.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn vnoise(x: f32, y: f32, s: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let fx = smooth(x - xi as f32);
    let fy = smooth(y - yi as f32);
    let a = hashf(xi, yi, s);
    let b = hashf(xi + 1, yi, s);
    let c = hashf(xi, yi + 1, s);
    let d = hashf(xi + 1, yi + 1, s);
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

pub fn fbm(x: f32, y: f32, s: u32) -> f32 {
    vnoise(x, y, s) * 0.55 + vnoise(x * 2.1, y * 2.1, s + 1) * 0.3 + vnoise(x * 4.3, y * 4.3, s + 2) * 0.15
}

#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407) | 1)
    }
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// Numero in [0,1)
    pub fn f(&mut self) -> f32 {
        (self.next() >> 40) as f32 / 16_777_216.0
    }
    /// Intero in [lo, hi)
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next() % (hi - lo) as u64) as i32
    }
    pub fn rf(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f()
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
}
