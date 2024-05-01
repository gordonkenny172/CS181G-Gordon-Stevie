use assets_manager::{asset::Png, AssetCache};
use frenderer::{
    input::{Input, Key},
    sprites::{Camera2D, SheetRegion, Transform},
    wgpu, Immediate,
};
use image::error::ParameterErrorKind;
use rand::Rng;
mod geom;
mod grid;
use geom::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum EntityType {
    Player1,
    Player2,
    Enemy,
    P_Projectile,
    E_Projectile,
    // which level, grid x in dest level, grid y in dest level
    #[allow(dead_code)]
    Door(String, u16, u16),
}

#[derive(Clone, Copy, Debug)]
struct TileData {
    solid: bool,
    sheet_region: SheetRegion,
}

// todo!("take out of engine");

const PLAYER: SheetRegion = SheetRegion::rect(315, 100, 16, 16);

const PLAYER2: SheetRegion = SheetRegion::rect(315, 100, 16, 16);

const ENEMY: SheetRegion = SheetRegion::rect(533 + 16, 39, 16, 16);

const P_PROJECTILE: SheetRegion = SheetRegion::rect(525, 19, 7, 7);

const E_PROJECTILE: SheetRegion = SheetRegion::rect(525, 43, 7, 7);

const HEART: SheetRegion = SheetRegion::rect(525, 35, 8, 8);

#[derive(Clone, Debug)]
struct Entity {
    alive: bool,
    pos: Vec2,
    dir: f32,
    etype: EntityType,
}

fn dir_to_vec2(dir: f32) -> Vec2 {
    Vec2 {
        x: f32::cos(dir),
        y: f32::sin(dir),
    }
}
fn vec2_to_dir(vec2: Vec2) -> f32 {
    vec2.y.atan2(vec2.x)
}

impl Entity {
    pub fn rect(&self) -> Rect {
        Rect {
            x: self.pos.x - TILE_SZ as f32 / 2.0 + 2.0,
            y: self.pos.y - TILE_SZ as f32 / 2.0 + 2.0,
            w: TILE_SZ as u16 - 4,
            h: TILE_SZ as u16 - 4,
        }
    }
    pub fn circle(&self) -> Circle {
        Circle {
            x: self.pos.x,
            y: self.pos.y,
            r: TILE_SZ as f32 / 2.0,
        }
    }
    pub fn shape_rect(&self) -> Shape {
        Shape::Rect(Rect {
            x: self.pos.x - TILE_SZ as f32 / 2.0 + 2.0,
            y: self.pos.y - TILE_SZ as f32 / 2.0 + 2.0,
            w: TILE_SZ as u16 - 4,
            h: TILE_SZ as u16 - 4,
        })
    }
    pub fn shape_circle(&self) -> Shape {
        Shape::Circle(Circle {
            x: self.pos.x,
            y: self.pos.y,
            r: TILE_SZ as f32 / 2.0,
        })
    }
    pub fn transform(&self) -> Transform {
        if self.etype == EntityType::P_Projectile || self.etype == EntityType::E_Projectile {
            Transform {
                x: self.pos.x,
                y: self.pos.y,
                w: 4,
                h: 4,
                rot: self.dir,
            }
        } else {
            Transform {
                x: self.pos.x,
                y: self.pos.y,
                w: TILE_SZ as u16,
                h: TILE_SZ as u16,
                rot: self.dir,
            }
        }
    }
    pub fn uv(&self) -> SheetRegion {
        match self.etype {
            EntityType::Player1 => PLAYER,
            EntityType::Player2 => PLAYER2,
            EntityType::Enemy => ENEMY,
            EntityType::P_Projectile => P_PROJECTILE,
            EntityType::E_Projectile => E_PROJECTILE,
            _ => panic!("can't draw doors"),
        }
        .with_depth(1)
    }
}
mod level;
use level::Level;
struct Game {
    assets: AssetCache,
    current_level: usize,
    levels: Vec<Level>,
    players: Vec<Entity>,
    enemies: Vec<Entity>,
    bounce: Vec<usize>,
    p_projectiles: Vec<Entity>,
    e_projectiles: Vec<Entity>,
    e_attack_timer: f32,
    p1_attack_timer: f32,
    p2_attack_timer: f32,
    p_health: Vec<u8>,
    e_health: Vec<u8>,
}

// Feel free to change this if you use a different tilesheet
const TILE_SZ: usize = 16;
const W: usize = 320;
const H: usize = 240;

// pixels per second

// label useful constants

// const ROTATE_SPEED: f32 = 0.1;
// const ENEMY_SPEED: f32 = 32.0;

// const ATTACK_MAX_TIME: f32 = 0.3;
// const ATTACK_COOLDOWN_TIME: f32 = 0.1;
// const ENEMY_ATTACK_COOLDOWN_TIME: f32 = 10.0;

const DT: f32 = 1.0 / 60.0;

//necessary structs and functions for collision detection
struct Contact {
    a_i: usize,
    a_r: Rect,
    b_i: usize,
    b_r: Rect,
    displacement: Vec2,
}

fn gather_contacts(objs_a: &Vec<Rect>, objs_b: &Vec<Rect>) -> Vec<Contact> {
    let mut contacts: Vec<Contact> = Vec::new();

    for (a_idx, a_rect) in objs_a.iter().enumerate() {
        for (b_idx, b_rect) in objs_b.iter().enumerate() {
            if let Some(overlap) = a_rect.overlap(*b_rect) {
                contacts.push(Contact {
                    a_i: a_idx,
                    a_r: *a_rect,
                    b_i: b_idx,
                    b_r: *b_rect,
                    displacement: overlap,
                })
            }
        }
    }
    contacts
}

fn gather_level_contacts(objs: &Vec<Rect>, level: &Level) -> Vec<Contact> {
    let mut contacts: Vec<Contact> = Vec::new();

    for (a_idx, a_rect) in objs.iter().enumerate() {
        for (b_idx, (b_rect, tile_data)) in level.tiles_within(*a_rect).enumerate() {
            if tile_data.solid {
                if let Some(overlap) = a_rect.overlap(b_rect) {
                    contacts.push(Contact {
                        a_i: a_idx,
                        a_r: *a_rect,
                        b_i: b_idx,
                        b_r: b_rect,
                        displacement: overlap,
                    });
                }
            }
        }
    }
    contacts
}

impl Game {
    fn player_level_collision_response(&mut self, contacts: &mut Vec<Contact>) {
        for contact in contacts.iter_mut() {
            if contact.displacement.x < contact.displacement.y {
                contact.displacement.y = 0.0;
            } else {
                contact.displacement.x = 0.0;
            }

            let b_pos = contact.b_r.rect_to_pos();

            if let Some(entity) = self.players.get_mut(contact.a_i) {
                if entity.pos.x < b_pos.x {
                    contact.displacement.x *= -1.0;
                }
                if entity.pos.y < b_pos.y {
                    contact.displacement.y *= -1.0;
                }

                entity.pos += contact.displacement;
            }
        }
    }
    fn enemy_level_collision_response(&mut self, contacts: &mut Vec<Contact>) {
        for contact in contacts.iter_mut() {
            if contact.displacement.x < contact.displacement.y {
                contact.displacement.y = 0.0;
            } else {
                contact.displacement.x = 0.0;
            }

            let b_pos = contact.b_r.rect_to_pos();

            if let Some(entity) = self.enemies.get_mut(contact.a_i) {
                if entity.pos.x < b_pos.x {
                    contact.displacement.x *= -1.0;
                }
                if entity.pos.y < b_pos.y {
                    contact.displacement.y *= -1.0;
                }

                entity.pos += contact.displacement;
            }
        }
    }

    fn p_projectile_level_response(&mut self, contacts: &mut Vec<Contact>) {
        for contact in contacts.iter_mut() {
            if contact.displacement.x < contact.displacement.y {
                contact.displacement.y = 0.0;
            } else {
                contact.displacement.x = 0.0;
            }

            let b_pos: Vec2 = contact.b_r.rect_to_pos();

            if let Some(projectile) = self.p_projectiles.get_mut(contact.a_i) {
                let mut t_vec2 = dir_to_vec2(projectile.dir);

                if projectile.pos.x < b_pos.x {
                    contact.displacement.x *= -1.0;
                }
                if projectile.pos.y < b_pos.y {
                    contact.displacement.y *= -1.0;
                }

                //now bounce

                if contact.displacement.x != 0.0 {
                    t_vec2.x *= -1.0;
                } else if contact.displacement.y != 0.0 {
                    t_vec2.y *= -1.0;
                }

                projectile.pos += contact.displacement;
                projectile.dir = vec2_to_dir(t_vec2);
            }
        }
    }

    fn e_projectile_level_response(&mut self, contacts: &mut Vec<Contact>) {
        for contact in contacts.iter_mut() {
            if contact.displacement.x < contact.displacement.y {
                contact.displacement.y = 0.0;
            } else {
                contact.displacement.x = 0.0;
            }

            let b_pos: Vec2 = contact.b_r.rect_to_pos();

            if let Some(projectile) = self.e_projectiles.get_mut(contact.a_i) {
                let mut t_vec2 = dir_to_vec2(projectile.dir);

                if projectile.pos.x < b_pos.x {
                    contact.displacement.x *= -1.0;
                }
                if projectile.pos.y < b_pos.y {
                    contact.displacement.y *= -1.0;
                }

                //now bounce

                if contact.displacement.x != 0.0 {
                    t_vec2.x *= -1.0;
                } else if contact.displacement.y != 0.0 {
                    t_vec2.y *= -1.0;
                }

                projectile.pos += contact.displacement;
                projectile.dir = vec2_to_dir(t_vec2);
            }
        }
    }

    fn damage_player(&mut self, entity_contacts: &mut Vec<Contact>) {
        for contact in entity_contacts.iter_mut() {
            if self.p_health[contact.b_i] > 0 {
                self.p_health[contact.b_i] -= 1;
            }

            self.e_projectiles[contact.a_i].alive = false;
        }
    }
    fn damage_enemy(&mut self, entity_contacts: &mut Vec<Contact>) {
        for contact in entity_contacts.iter_mut() {
            if self.e_health[contact.b_i] > 0 {
                self.e_health[contact.b_i] -= 1;
            }
            self.p_projectiles[contact.a_i].alive = false;
        }
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let source = assets_manager::source::FileSystem::new("adventure/content")
        .expect("Couldn't load resources");
    #[cfg(target_arch = "wasm32")]
    let source = assets_manager::source::Embedded::from(assets_manager::source::embed!("content"));
    let cache = assets_manager::AssetCache::with_source(source);

    let drv = frenderer::Driver::new(
        winit::window::WindowBuilder::new()
            .with_title("test")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0)),
        Some((W as u32 * 4, H as u32 * 4)),
    );

    let mut input = Input::default();

    let mut now = frenderer::clock::Instant::now();
    let mut acc = 0.0;
    drv.run_event_loop::<(), _>(
        move |window, frend| {
            let mut frend = Immediate::new(frend);
            let game = Game::new(&mut frend, cache);
            (window, game, frend)
        },
        move |event, target, (window, ref mut game, ref mut frend)| {
            use winit::event::{Event, WindowEvent};
            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    target.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    if !frend.gpu().is_web() {
                        frend.resize_surface(size.width, size.height);
                    }
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let elapsed = now.elapsed().as_secs_f32();
                    // You can add the time snapping/death spiral prevention stuff here if you want.
                    // I'm not using it here to keep the starter code small.
                    acc += elapsed;
                    now = std::time::Instant::now();
                    // While we have time to spend
                    while acc >= DT {
                        // simulate a frame
                        acc -= DT;
                        game.simulate(&input, DT);
                        input.next_frame();
                    }
                    game.render(frend);
                    frend.render();
                    window.request_redraw();
                }
                event => {
                    input.process_input_event(&event);
                }
            }
        },
    )
    .expect("event loop error");
}

impl Game {
    fn new(renderer: &mut Immediate, cache: AssetCache) -> Self {
        let tile_handle = cache
            .load::<Png>("texture")
            .expect("Couldn't load tilesheet img");
        let tile_img = tile_handle.read().0.to_rgba8();
        let tile_tex = renderer.create_array_texture(
            &[&tile_img],
            wgpu::TextureFormat::Rgba8UnormSrgb,
            tile_img.dimensions(),
            Some("tiles-sprites"),
        );
        let levels = vec![Level::from_str(
            &cache
                .load::<String>("level1")
                .expect("Couldn't access level1.txt")
                .read(),
        )];
        let current_level = 0;
        let camera = Camera2D {
            screen_pos: [0.0, 0.0],
            screen_size: [W as f32, H as f32],
        };
        let sprite_estimate =
            levels[current_level].sprite_count() + levels[current_level].starts().len();
        renderer.sprite_group_add(
            &tile_tex,
            vec![Transform::ZERO; sprite_estimate],
            vec![SheetRegion::ZERO; sprite_estimate],
            camera,
        );
        let player_start = *levels[current_level]
            .starts()
            .iter()
            .find(|(t, _)| *t == EntityType::Player1)
            .map(|(_, ploc)| ploc)
            .expect("Start level doesn't put the player anywhere");

        let player2_start = *levels[current_level]
            .starts()
            .iter()
            .find(|(t, _)| *t == EntityType::Player2)
            .map(|(_, ploc)| ploc)
            .expect("Start level doesn't put the player anywhere");

        let mut game = Game {
            assets: cache,
            current_level,
            p1_attack_timer: 0.0,
            p2_attack_timer: 0.0,
            e_attack_timer: 0.0,
            bounce: Vec::new(),
            levels,
            p_health: vec![1, 1],
            players: vec![
                Entity {
                    alive: true,
                    etype: EntityType::Player1,
                    pos: player_start,
                    dir: 0.0,
                },
                Entity {
                    alive: true,
                    etype: EntityType::Player2,
                    pos: player2_start,
                    dir: 0.0,
                },
            ],
            e_health: Vec::new(),
            enemies: Vec::new(),
            p_projectiles: Vec::new(),
            e_projectiles: Vec::new(),
        };
        game.enter_level(player_start, player2_start);
        game
    }
    fn level(&self) -> &Level {
        &self.levels[self.current_level]
    }
    fn enter_level(&mut self, player_pos: Vec2, player2_pos: Vec2) {
        self.players.truncate(2);
        self.players[0].pos = player_pos;
        self.players[1].pos = player2_pos;
        for (etype, pos) in self.levels[self.current_level].starts().iter() {
            match etype {
                EntityType::Player1 => {}
                EntityType::Player2 => {}
                EntityType::Door(_rm, _x, _y) => todo!("doors not supported"),
                EntityType::Enemy => {
                    // spawn enemies based on level data
                }
                EntityType::P_Projectile => {}
                EntityType::E_Projectile => {}
            }
        }
    }
    fn render(&mut self, frend: &mut Immediate) {
        self.level().render_immediate(frend);
        //render

    }
    fn simulate(&mut self, input: &Input, dt: f32) {
        //simulate
    }
}
