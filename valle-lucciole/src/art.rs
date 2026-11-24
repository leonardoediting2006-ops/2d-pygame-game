//! Raccolta di tutta la grafica: genera i canvas, crea le texture, esporta i PNG.

use crate::canvas::*;
use crate::items::*;
use crate::buildings::*;
use crate::decor::*;
use crate::sprites::*;
use crate::util::*;
use macroquad::prelude::*;

/// Tutti gli sprite generati, con un nome (usato anche per l'export PNG).
pub fn all_canvases() -> Vec<(String, Canvas)> {
    let mut v: Vec<(String, Canvas)> = Vec::new();
    let mut push = |n: String, c: Canvas| v.push((n, c));
    for i in 0..3 {
        push(format!("albero_quercia_{i}"), oak(i));
        push(format!("albero_pino_{i}"), pine(i));
        push(format!("albero_ciliegio_{i}"), cherry(i));
    }
    for i in 0..3 {
        push(format!("albero_acero_{i}"), leafy(i, &MAPLE, None, 500));
        push(format!("albero_melo_{i}"), leafy(i, &LEAF, Some([hex(0xd8302a), hex(0xff8a70)]), 600));
        push(format!("siepe_{i}"), hedge(i));
        push(format!("erba_alta_{i}"), tallgrass(i, 0));
    }
    for i in 0..6 {
        push(format!("aiuola_{i}"), flowerbed(i));
    }
    push("botte".into(), keg(1));
    push("cassetta".into(), crate_box(0));
    push("balla_fieno".into(), haybale(0));
    push("braciere".into(), brazier());
    push("fiamma_0".into(), flame(0));
    push("pollaio".into(), coop());
    push("funghi_0".into(), mushrooms(0));
    push("funghi_1".into(), mushrooms(1));
    push("cartello".into(), sign());
    push("ceppo".into(), stump());
    for i in 0..3 {
        push(format!("sasso_{i}"), rock_small(i));
        push(format!("erbaccia_{i}"), weed(i));
    }
    push("macigno".into(), rock_big(0));
    push("ramo_0".into(), stick(0));
    push("ramo_1".into(), stick(1));
    push("cespuglio_0".into(), bush(0));
    push("cespuglio_1".into(), bush(1));
    push("casa_fattoria".into(), farmhouse().0);
    push("casa_mercante".into(), cottage().0);
    push("cassa_spedizioni".into(), chest());
    push("pozzo".into(), well());
    for i in 0..8 {
        push(format!("ciuffo_erba_{i}"), tuft(i));
    }
    for i in 0..4 {
        push(format!("sassolini_{i}"), pebbles(i));
    }
    push("bancarella".into(), stall());
    push("staccionata_palo".into(), fence_post());
    push("staccionata_h".into(), fence_rail_h());
    push("staccionata_v".into(), fence_rail_v());
    push("cassetta_posta".into(), mailbox());
    push("spaventapasseri".into(), scarecrow());
    push("lampione".into(), lamp());
    push("fontana".into(), fountain());
    push("panchina".into(), bench());
    for i in 0..3 {
        push(format!("fioriera_{i}"), planter(i));
    }
    for m in 0..16u8 {
        push(format!("zappato_{m:02}"), tilled(m, false));
        push(format!("zappato_bagnato_{m:02}"), tilled(m, true));
    }
    for k in Crop::ALL {
        for s in 0..5 {
            push(format!("coltura_{}_{s}", k.name().to_lowercase()), crop(k, s));
        }
    }
    for it in icon_items() {
        push(format!("icona_{}", it.code()), icon(it));
    }
    for (name, look) in [("contadino", FARMER), ("mercante", MERCHANT)] {
        for dir in 0..3u8 {
            for f in 0..4u8 {
                push(format!("{name}_d{dir}_f{f}"), character(&look, dir, f, false));
            }
            push(format!("{name}_d{dir}_azione"), character(&look, dir, 0, true));
        }
    }
    for f in 0..3 {
        push(format!("gallina_{f}"), chicken(f));
    }
    push("farfalla_0".into(), butterfly(0));
    push("farfalla_1".into(), butterfly(1));
    push("moneta".into(), coin());
    push("sole".into(), sun_icon());
    push("pioggia".into(), rain_icon());
    push("cuore".into(), heart());
    v
}

pub fn icon_items() -> Vec<Item> {
    let mut v = vec![Item::Hoe, Item::Can, Item::Axe, Item::Pick, Item::Scythe, Item::Wood, Item::Stone, Item::Fiber];
    for k in Crop::ALL {
        v.push(Item::Seed(k));
        v.push(Item::Produce(k));
    }
    v
}

/// Foglio riassuntivo con tutti gli sprite, ingrandito, su sfondo d'erba.
pub fn contact_sheet(scale: i32) -> Canvas {
    let list = all_canvases();
    let maxw = 600;
    let pad = 4;
    // impaginazione a righe
    let mut placed = Vec::new();
    let (mut x, mut y, mut row_h) = (pad, pad, 0);
    for (_, c) in &list {
        if x + c.w + pad > maxw {
            x = pad;
            y += row_h + pad;
            row_h = 0;
        }
        placed.push((x, y));
        x += c.w + pad;
        row_h = row_h.max(c.h);
    }
    let h = y + row_h + pad;
    let mut sheet = Canvas::new(maxw * scale, h * scale);
    for yy in 0..h {
        for xx in 0..maxw {
            let checker = ((xx / 8) + (yy / 8)) % 2 == 0;
            let col = if checker { hex(0x7cb84e) } else { hex(0x74b048) };
            for sy in 0..scale {
                for sx in 0..scale {
                    sheet.set(xx * scale + sx, yy * scale + sy, col);
                }
            }
        }
    }
    for ((_, c), (px, py)) in list.iter().zip(placed) {
        for yy in 0..c.h {
            for xx in 0..c.w {
                let p = c.get(xx, yy);
                if p[3] == 0 {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        let tx = (px + xx) * scale + sx;
                        let ty = (py + yy) * scale + sy;
                        let base = sheet.get(tx, ty);
                        let a = p[3] as f32 / 255.0;
                        sheet.set(tx, ty, alpha(mix(base, p, a), 255));
                    }
                }
            }
        }
    }
    sheet
}

pub fn export_all(dir: &str) {
    let _ = std::fs::create_dir_all(dir);
    for (name, c) in all_canvases() {
        c.save_png(&format!("{dir}/{name}.png"));
    }
    contact_sheet(3).save_png(&format!("{dir}/_tutti_gli_sprite.png"));
}

// ------------------------------------------------------------------ texture pronte per il gioco

pub struct CharSet {
    /// [dir][frame], dir 0 giù, 1 su, 2 sinistra, 3 destra
    pub walk: Vec<Vec<Texture2D>>,
    pub act: Vec<Texture2D>,
}

impl CharSet {
    fn new(l: &Look) -> Self {
        let mut walk = Vec::new();
        let mut act = Vec::new();
        for dir in 0..4u8 {
            let d = if dir == 3 { 2 } else { dir };
            let mut fr = Vec::new();
            for f in 0..4u8 {
                let c = character(l, d, f, false);
                fr.push(if dir == 3 { c.flipped_h().tex() } else { c.tex() });
            }
            walk.push(fr);
            let a = character(l, d, 0, true);
            act.push(if dir == 3 { a.flipped_h().tex() } else { a.tex() });
        }
        CharSet { walk, act }
    }
}

pub struct Art {
    pub oak: Vec<Texture2D>,
    pub pine: Vec<Texture2D>,
    pub cherry: Vec<Texture2D>,
    pub stump: Texture2D,
    pub rock: Vec<Texture2D>,
    pub boulder: Texture2D,
    pub weed: Vec<Texture2D>,
    pub stick: Vec<Texture2D>,
    pub bush: Vec<Texture2D>,
    pub house: Texture2D,
    pub cottage: Texture2D,
    pub bin: Texture2D,
    pub well: Texture2D,
    pub stall: Texture2D,
    pub fence_post: Texture2D,
    pub fence_h: Texture2D,
    pub fence_v: Texture2D,
    pub mailbox: Texture2D,
    pub scarecrow: Texture2D,
    pub lamp: Texture2D,
    pub fountain: Texture2D,
    pub maple: Vec<Texture2D>,
    pub apple: Vec<Texture2D>,
    pub hedge: Vec<Texture2D>,
    pub keg: Vec<Texture2D>,
    pub crate_box: Vec<Texture2D>,
    pub hay: Vec<Texture2D>,
    pub brazier: Texture2D,
    pub flame: Vec<Texture2D>,
    pub coop: Texture2D,
    pub flowerbed: Vec<Texture2D>,
    pub mushroom: Vec<Texture2D>,
    pub tallgrass: Vec<[Texture2D; 2]>,
    pub sign: Texture2D,
    pub bench: Texture2D,
    pub planter: Vec<Texture2D>,
    pub tilled: Vec<Texture2D>,
    pub tilled_wet: Vec<Texture2D>,
    pub crops: Vec<Vec<Texture2D>>,
    icons: Vec<(Item, Texture2D)>,
    pub farmer: CharSet,
    pub merchant: CharSet,
    pub chicken: Vec<Texture2D>,
    pub chicken_r: Vec<Texture2D>,
    pub butterfly: Vec<Texture2D>,
    pub shadow_s: Texture2D,
    pub shadow_m: Texture2D,
    pub shadow_l: Texture2D,
    pub shadow_xl: Texture2D,
    pub contact: Texture2D,
    pub tufts: Vec<Texture2D>,
    pub clouds: Vec<Texture2D>,
    pub beam: Texture2D,
    pub pebbles: Vec<Texture2D>,
    pub house_info: HouseInfo,
    pub cottage_info: HouseInfo,
    pub glow: Texture2D,
    pub coin: Texture2D,
    pub sun: Texture2D,
    pub rain: Texture2D,
    pub heart: Texture2D,
    pub white: Texture2D,
}

impl Art {
    pub fn new() -> Self {
        let t = |c: Canvas| c.tex();
        let fh = farmhouse();
        let ct = cottage();
        let mut white = Canvas::new(1, 1);
        white.set(0, 0, [255, 255, 255, 255]);
        Art {
            oak: (0..3).map(|i| t(oak(i))).collect(),
            pine: (0..3).map(|i| t(pine(i))).collect(),
            cherry: (0..3).map(|i| t(cherry(i))).collect(),
            stump: t(stump()),
            rock: (0..3).map(|i| t(rock_small(i))).collect(),
            boulder: t(rock_big(0)),
            weed: (0..3).map(|i| t(weed(i))).collect(),
            stick: (0..2).map(|i| t(stick(i))).collect(),
            bush: (0..2).map(|i| t(bush(i))).collect(),
            house: t(fh.0),
            cottage: t(ct.0),
            bin: t(chest()),
            well: t(well()),
            stall: t(stall()),
            fence_post: t(fence_post()),
            fence_h: t(fence_rail_h()),
            fence_v: t(fence_rail_v()),
            mailbox: t(mailbox()),
            scarecrow: t(scarecrow()),
            lamp: t(lamp()),
            fountain: t(fountain()),
            maple: (0..3).map(|i| t(leafy(i, &MAPLE, None, 500))).collect(),
            apple: (0..3).map(|i| t(leafy(i, &LEAF, Some([hex(0xd8302a), hex(0xff8a70)]), 600))).collect(),
            hedge: (0..3).map(|i| t(hedge(i))).collect(),
            keg: (0..2).map(|i| t(keg(i))).collect(),
            crate_box: (0..2).map(|i| t(crate_box(i))).collect(),
            hay: (0..3).map(|i| t(haybale(i))).collect(),
            brazier: t(brazier()),
            flame: (0..4).map(|i| t(flame(i))).collect(),
            coop: t(coop()),
            flowerbed: (0..6).map(|i| t(flowerbed(i))).collect(),
            mushroom: (0..2).map(|i| t(mushrooms(i))).collect(),
            tallgrass: (0..3).map(|i| [t(tallgrass(i, 0)), t(tallgrass(i, 1))]).collect(),
            sign: t(sign()),
            bench: t(bench()),
            planter: (0..3).map(|i| t(planter(i))).collect(),
            tilled: (0..16u8).map(|m| t(tilled(m, false))).collect(),
            tilled_wet: (0..16u8).map(|m| t(tilled(m, true))).collect(),
            crops: Crop::ALL.iter().map(|&k| (0..5).map(|s| t(crop(k, s))).collect()).collect(),
            icons: icon_items().into_iter().map(|i| (i, t(icon(i)))).collect(),
            farmer: CharSet::new(&FARMER),
            merchant: CharSet::new(&MERCHANT),
            chicken: (0..3).map(|f| t(chicken(f))).collect(),
            chicken_r: (0..3).map(|f| t(chicken(f).flipped_h())).collect(),
            butterfly: (0..2).map(|f| t(butterfly(f))).collect(),
            shadow_s: t(shadow(12, 5, 0.9)),
            shadow_m: t(shadow(20, 7, 0.9)),
            shadow_l: t(shadow(34, 11, 0.85)),
            shadow_xl: t(shadow(52, 16, 0.6)),
            contact: t(shadow(16, 5, 1.3)),
            tufts: (0..8).map(|i| t(tuft(i))).collect(),
            clouds: (0..2).map(|i| t(cloud(i))).collect(),
            beam: t(beam()),
            pebbles: (0..4).map(|i| t(pebbles(i))).collect(),
            house_info: fh.1,
            cottage_info: ct.1,
            glow: t(glow()),
            coin: t(coin()),
            sun: t(sun_icon()),
            rain: t(rain_icon()),
            heart: t(heart()),
            white: t(white),
        }
    }

    pub fn icon(&self, it: Item) -> &Texture2D {
        &self.icons.iter().find(|(i, _)| *i == it).unwrap().1
    }
}
