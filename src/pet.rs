/// Pet ASCII art with animation frames
/// Each pet has: idle frames (2 for animation), happy, alert

pub struct Pet {
    #[allow(dead_code)]
    pub name: &'static str,
    pub frames: &'static [&'static str],
    pub happy: &'static str,
    pub alert: &'static str,
}

pub enum PetMood {
    Idle(usize), // frame index
    Happy,
    #[allow(dead_code)]
    Alert,
}

impl Pet {
    pub fn render(&self, mood: &PetMood) -> Vec<String> {
        let art = match mood {
            PetMood::Idle(frame) => self.frames[frame % self.frames.len()],
            PetMood::Happy => self.happy,
            PetMood::Alert => self.alert,
        };
        art.lines().map(|l| l.to_string()).collect()
    }
}

pub fn get_pet(name: &str) -> &'static Pet {
    match name {
        "dino" => &DINO,
        "cat" => &CAT,
        "ghost" => &GHOST,
        "dragon" => &DRAGON,
        _ => &ROBOT,
    }
}

static ROBOT: Pet = Pet {
    name: "robot",
    frames: &[
        " ┌───┐ \n │◉ ◉│ \n │ ▽ │ \n/└─┬─┘\\\n  │ │  \n  ┘ └  ",
        " ┌───┐ \n │◉ ◉│ \n │ ▽ │ \n/└─┬─┘\\\n  │ │  \n  └ ┘  ",
    ],
    happy: " ┌───┐ \n │^ ^│ \n │ ω │ \n\\└─┬─┘/\n  │ │  \n  ┘ └  ",
    alert: " ┌───┐ \n │◉ △│ \n │ ！│ \n/└─┬─┘\\\n  │ │  \n  ┘ └  ",
};

static DINO: Pet = Pet {
    name: "dino",
    frames: &[
        "  __   \n / _)  \n/ /    \n| |__  \n|____) \n ||    \n ^^    ",
        "  __   \n / _)  \n/ /    \n| |__  \n|____) \n ||    \n ^ ^   ",
    ],
    happy: "  __   \n / ^)  \n/ /    \n| |__  \n|____) \n ||    \n ^^    ",
    alert: "  __   \n / o)  \n/ /  ! \n| |__  \n|____) \n ||    \n ^^    ",
};

static CAT: Pet = Pet {
    name: "cat",
    frames: &[
        " /\\_/\\ \n( ◉.◉ )\n > ^ < \n  ||   \n  ^^   ",
        " /\\_/\\ \n( ◉.◉ )\n > ^ < \n  ||   \n  ^ ^  ",
    ],
    happy: " /\\_/\\ \n( ^.^ )\n > ω < \n  ||   \n  ^^   ",
    alert: " /\\_/\\ \n( ◉.△ )\n > ！< \n  ||   \n  ^^   ",
};

static GHOST: Pet = Pet {
    name: "ghost",
    frames: &[
        "  ___  \n /   \\ \n| ◉ ◉ |\n|  ▽  |\n \\~~~/ \n  ~~~  ",
        "  ___  \n /   \\ \n| ◉ ◉ |\n|  ▽  |\n \\~~~/ \n   ~~  ",
    ],
    happy: "  ___  \n /   \\ \n| ^ ^ |\n|  ω  |\n \\~~~/ \n  ~~~  ",
    alert: "  ___  \n /   \\ \n| ◉ △ |\n|  ！ |\n \\~~~/ \n  ~~~  ",
};

static DRAGON: Pet = Pet {
    name: "dragon",
    frames: &[
        "  /\\__ \n ( ◉  )\n/|    |\\\n \\|  |/\n  |  | \n  ^^   ",
        "  /\\__ \n ( ◉  )\n/|    |\\\n \\|  |/\n  |  | \n  ^ ^  ",
    ],
    happy: "  /\\__ \n ( ^  )\n/| ω  |\\\n \\|  |/\n  |  | \n  ^^   ",
    alert: "  /\\__ \n ( ◉! )\n/|    |\\\n \\|  |/\n  |  | \n  ^^   ",
};
