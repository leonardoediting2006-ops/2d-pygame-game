//! Oggetti, colture e relative statistiche.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Crop {
    Carota,
    Pomodoro,
    Zucca,
    Girasole,
}

impl Crop {
    pub const ALL: [Crop; 4] = [Crop::Carota, Crop::Pomodoro, Crop::Zucca, Crop::Girasole];

    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn from_idx(i: usize) -> Crop {
        Crop::ALL[i.min(3)]
    }
    /// Giorni d'acqua necessari per maturare
    pub fn days(self) -> u8 {
        match self {
            Crop::Carota => 4,
            Crop::Pomodoro => 6,
            Crop::Girasole => 7,
            Crop::Zucca => 9,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Crop::Carota => "Carota",
            Crop::Pomodoro => "Pomodoro",
            Crop::Zucca => "Zucca",
            Crop::Girasole => "Girasole",
        }
    }
    pub fn seed_price(self) -> i32 {
        match self {
            Crop::Carota => 20,
            Crop::Pomodoro => 45,
            Crop::Girasole => 60,
            Crop::Zucca => 110,
        }
    }
    pub fn sell(self) -> i32 {
        match self {
            Crop::Carota => 38,
            Crop::Pomodoro => 30,
            Crop::Girasole => 130,
            Crop::Zucca => 340,
        }
    }
    /// Quanti frutti si raccolgono (min, max)
    pub fn yield_range(self) -> (i32, i32) {
        match self {
            Crop::Pomodoro => (2, 4),
            _ => (1, 2),
        }
    }
    pub fn energy(self) -> i32 {
        match self {
            Crop::Carota => 25,
            Crop::Pomodoro => 18,
            Crop::Girasole => 0,
            Crop::Zucca => 70,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Item {
    Hoe,
    Can,
    Axe,
    Pick,
    Scythe,
    Seed(Crop),
    Produce(Crop),
    Wood,
    Stone,
    Fiber,
}

impl Item {
    pub fn name(self) -> String {
        match self {
            Item::Hoe => "Zappa".into(),
            Item::Can => "Annaffiatoio".into(),
            Item::Axe => "Ascia".into(),
            Item::Pick => "Piccone".into(),
            Item::Scythe => "Falce".into(),
            Item::Seed(c) => format!("Semi di {}", c.name()),
            Item::Produce(c) => c.name().into(),
            Item::Wood => "Legna".into(),
            Item::Stone => "Pietra".into(),
            Item::Fiber => "Fibra".into(),
        }
    }
    pub fn is_tool(self) -> bool {
        matches!(self, Item::Hoe | Item::Can | Item::Axe | Item::Pick | Item::Scythe)
    }
    pub fn sell_price(self) -> i32 {
        match self {
            Item::Produce(c) => c.sell(),
            Item::Seed(c) => c.seed_price() / 2,
            Item::Wood => 3,
            Item::Stone => 3,
            Item::Fiber => 1,
            _ => 0,
        }
    }
    pub fn energy(self) -> i32 {
        match self {
            Item::Produce(c) => c.energy(),
            _ => 0,
        }
    }
    pub fn code(self) -> String {
        match self {
            Item::Hoe => "hoe".into(),
            Item::Can => "can".into(),
            Item::Axe => "axe".into(),
            Item::Pick => "pick".into(),
            Item::Scythe => "scythe".into(),
            Item::Seed(c) => format!("seed{}", c.idx()),
            Item::Produce(c) => format!("prod{}", c.idx()),
            Item::Wood => "wood".into(),
            Item::Stone => "stone".into(),
            Item::Fiber => "fiber".into(),
        }
    }
    pub fn from_code(s: &str) -> Option<Item> {
        Some(match s {
            "hoe" => Item::Hoe,
            "can" => Item::Can,
            "axe" => Item::Axe,
            "pick" => Item::Pick,
            "scythe" => Item::Scythe,
            "wood" => Item::Wood,
            "stone" => Item::Stone,
            "fiber" => Item::Fiber,
            _ => {
                if let Some(n) = s.strip_prefix("seed") {
                    Item::Seed(Crop::from_idx(n.parse().ok()?))
                } else if let Some(n) = s.strip_prefix("prod") {
                    Item::Produce(Crop::from_idx(n.parse().ok()?))
                } else {
                    return None;
                }
            }
        })
    }
}
