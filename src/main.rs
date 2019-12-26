mod domain;

use domain::{Game, Map, Object, Rect, Tile};
use rand::Rng;
use std::cmp;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
use tcod::map::{FovAlgorithm, Map as FovMap};

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

// size of the map
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;

// parameters for dungeon generator
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

// tile colors
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 50,
};

const FONT_PATH: &str = "src/assets/arial_10x10.png";
const LIMIT_FPS: i32 = 20; // 20 frames-per-second maximum

// fov constants
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic; // default FOV algorithm
const FOV_LIGHT_WALLS: bool = true; // light walls or not
const TORCH_RADIUS: i32 = 10;

// encapsulate all of our libtcod-related values
struct Tcod {
    root: Root,
    con: Offscreen,
    fov: FovMap,
}

fn make_map(player: &mut Object) -> Map {
    // fill map with "blocked" tiles
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];

    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        // random width and height
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

        // random position without going out of the boundaries of the map
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);

        let new_room = Rect::new(x, y, w, h);

        // run through the other rooms and see if they intersect with this one
        let has_intersections = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));

        if !has_intersections {
            // paint it to the map's tiles
            create_room(new_room, &mut map);

            // center coordinates of the new room, will be useful later
            let (new_x, new_y) = new_room.center();

            if rooms.is_empty() {
                // this is the first room, where the player starts at
                player.x = new_x;
                player.y = new_y;
            } else {
                // all rooms after the first: connect to previous with a tunnel

                // center coordinates of previous room
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();

                // flip a coin
                if rand::random() {
                    // first move horizontally, then vertically
                    create_horizontal_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_vertical_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    // first move vertically, then horizontally
                    create_vertical_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_horizontal_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }

            // append the new room to the list
            rooms.push(new_room);
        }
    }

    map
}

fn create_room(room: Rect, map: &mut Map) {
    // go through the tiles in the rectangle and make them passable
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_horizontal_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_vertical_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], should_compute_fov: bool) {
    if should_compute_fov {
        // compute FOV if needed (ie. player moved or other)
        let player = &objects[0];
        tcod.fov
            .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);
    }

    // go through all tiles and set their background color
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let explored = &mut game.map[x as usize][y as usize].explored;
            let visible = tcod.fov.is_in_fov(x, y);

            if visible {
                // since it's visible, it has been explored
                *explored = true;
            }

            if *explored {
                let wall = game.map[x as usize][y as usize].block_sight;
                let color = match (visible, wall) {
                    // outside FOV
                    (false, true) => COLOR_DARK_WALL,
                    (false, false) => COLOR_DARK_GROUND,
                    // inside FOV
                    (true, true) => COLOR_LIGHT_WALL,
                    (true, false) => COLOR_LIGHT_GROUND,
                };

                tcod.con
                    .set_char_background(x, y, color, BackgroundFlag::Set)
            }
        }
    }

    // draw all objects in the list
    for object in objects {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
    }

    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );
}

fn handle_keys(tcod: &mut Tcod, game: &Game, player: &mut Object) -> bool {
    match tcod.root.wait_for_keypress(true) {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key { code: Up, .. } => player.move_by(0, -1, game),
        Key { code: Down, .. } => player.move_by(0, 1, game),
        Key { code: Left, .. } => player.move_by(-1, 0, game),
        Key { code: Right, .. } => player.move_by(1, 0, game),
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

    let mut tcod = Tcod {
        root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
    };

    // create object representing the player
    let player = Object::new(0, 0, '@', WHITE);

    // create an NPC
    let npc = Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);

    // the list of objects with those two
    let mut objects = [player, npc];

    let mut game = Game {
        // generate map (at this point it's not drawn to the screen)
        map: make_map(&mut objects[0]),
    };

    // force FOV recompute first time through the game loop
    let mut previous_player_position = (-1, -1);

    // populate the FOV map according to the generated map
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                // invert our domain's `block_sight` to match tcod's `transparent`
                !game.map[x as usize][y as usize].block_sight,
                // invert our domain's `blocked` to match tcod's `walkable`
                !game.map[x as usize][y as usize].blocked,
            )
        }
    }

    // game loop
    while !tcod.root.window_closed() {
        // clear the screen of the previous frame
        tcod.con.clear();

        // render the screen
        let should_compute_fov = previous_player_position != (objects[0].x, objects[0].y);
        render_all(&mut tcod, &mut game, &objects, should_compute_fov);

        tcod.root.flush();

        // handle keys and exit game if needed
        let player = &mut objects[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut tcod, &game, player);
        if exit {
            break;
        }
    }
}
