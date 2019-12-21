use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const FONT_PATH: &str = "src/assets/arial_10x10.png";
const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

// encapsulate all of our libtcod-related values. Having all the tcod stuff in one
// place makes it easier to pass it around to functions. And doing it now will save
// us from gnarly refactors later.
struct Tcod {
    root: Root,
}

fn handle_keys(tcod: &mut Tcod, player_x: &mut i32, player_y: &mut i32) -> bool {
    let key = tcod.root.wait_for_keypress(true);

    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Up, .. } => *player_y -= 1,
        Key { code: Down, .. } => *player_y += 1,
        Key { code: Left, .. } => *player_x -= 1,
        Key { code: Right, .. } => *player_x += 1,
        _ => {}
    }

    // true: exit game, false: keep running game loop
    false
}

fn main() {
    tcod::system::set_fps(LIMIT_FPS);

    let root = Root::initializer()
        .font(FONT_PATH, FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Rust/libtcod tutorial")
        .init();

    let mut tcod = Tcod { root };

    // initial player location
    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT / 2;

    // game loop
    while !tcod.root.window_closed() {
        tcod.root.set_default_foreground(WHITE);
        tcod.root.clear();
        tcod.root
            .put_char(player_x, player_y, '@', BackgroundFlag::None);
        tcod.root.flush();
        
        // handle keys and exit game if needed
        let exit = handle_keys(&mut tcod, &mut player_x, &mut player_y);
        if exit {
            break;
        }
    }
}
