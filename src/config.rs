pub mod rs;

pub struct Config {
    pub insert_mode: i32,
    pub visual_mode: i32,
    pub normal_mode: i32,

    pub yank: i32,
    pub paste: i32,

    pub background: RgbColor,
    pub background1: RgbColor,
    pub foreground: RgbColor,
    pub foreground1: RgbColor,

    pub green: RgbColor,
    pub orange: RgbColor,
    pub yellow: RgbColor,
    pub quartz: RgbColor,
}

pub struct RgbColor {
    pub red: i16,
    pub green: i16,
    pub blue: i16,
}

impl Config {
    pub fn init() -> Config {
        return Config {
            insert_mode: 105,
            visual_mode: 118,
            normal_mode: 27,

            yank: 121,
            paste: 112,

            background: RgbColor {
                red: 24,
                green: 24,
                blue: 24,
            },

            background1: RgbColor {
                red: 40,
                green: 40,
                blue: 40,
            },

            foreground: RgbColor {
                red: 228,
                green: 228,
                blue: 239,
            },

            foreground1: RgbColor {
                red: 228,
                green: 228,
                blue: 239,
            },

            green: RgbColor {
                red: 115,
                green: 201,
                blue: 54,
            },

            orange: RgbColor {
                red: 204,
                green: 140,
                blue: 60,
            },

            yellow: RgbColor {
                red: 255,
                green: 221,
                blue: 51,
            },

            quartz: RgbColor {
                red: 149,
                green: 169,
                blue: 159,
            },
        };
    }
}


