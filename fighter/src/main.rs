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
    Player,
    Enemy,
    Projectile,
    // which level, grid x in dest level, grid y in dest level
    #[allow(dead_code)]
    Door(String, u16, u16),
}

#[derive(Clone, Copy, Debug)]
struct TileData {
    solid: bool,
    sheet_region: SheetRegion,
}

const PLAYER: SheetRegion = SheetRegion::rect(296, 119, 25, 20);

const PLAYER2: SheetRegion = SheetRegion::rect(328, 151, 25, 20);

const ENEMY: SheetRegion = SheetRegion::rect(533 + 16, 39, 16, 16);

const P1_PROJECTILE: SheetRegion = SheetRegion::rect(525, 19, 7, 7);

const P2_PROJECTILE: SheetRegion = SheetRegion::rect(525, 43, 7, 7);

const HEART: SheetRegion = SheetRegion::rect(525, 35, 8, 8);

#[derive(Clone, Debug)]
struct Entity {
    alive: bool,
    pos: Vec2,
    dir: f32,
    etype: EntityType,
}

// struct Projectile {
//     alive: bool,
//     pos: Vec2,
//     dir: f32,
//     shooter: ProjectileType,
// }

// enum

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
    pub fn transform(&self) -> Transform {
        if self.etype == EntityType::Projectile {
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
            EntityType::Player => PLAYER,
            EntityType::Enemy => ENEMY,
            EntityType::Projectile => P1_PROJECTILE,
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
    entities: Vec<Entity>,
    bounce: Vec<usize>,
    projectiles: Vec<Entity>,
    p1_attack_timer: f32,
    p2_attack_timer: f32,
    health: u8,
}

// Feel free to change this if you use a different tilesheet
const TILE_SZ: usize = 16;
const W: usize = 240;
const H: usize = 160;

// pixels per second
const PLAYER_SPEED: f32 = 64.0;
const ROTATE_SPEED: f32 = 0.1;
const ENEMY_SPEED: f32 = 32.0;
const KNOCKBACK_SPEED: f32 = 128.0;

const ATTACK_MAX_TIME: f32 = 0.3;
const ATTACK_COOLDOWN_TIME: f32 = 0.1;
const KNOCKBACK_TIME: f32 = 0.25;

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

struct Contact2 {
    a_i: usize,
    a_r: Shape,
    b_i: usize,
    b_r: Shape,
    displacement: Vec2,
}

fn gather_contacts_2(objs_a: &Vec<Shape>, objs_b: &Vec<Shape>) -> Vec<Contact2> {
    let mut contacts: Vec<Contact2> = Vec::new();

    for (a_idx, a_shape) in objs_a.iter().enumerate() {
        for (b_idx, b_shape) in objs_b.iter().enumerate() {
            if let Some(overlap) = a_shape.overlap(*b_shape) {
                contacts.push(Contact2 {
                    a_i: a_idx,
                    a_r: *a_shape,
                    b_i: b_idx,
                    b_r: *b_shape,
                    displacement: overlap,
                })
            }
        }
    }
    contacts
}

fn gather_level_contacts_2(objs: &Vec<Shape>, level: &Level) -> Vec<Contact2> {
    let mut contacts: Vec<Contact2> = Vec::new();

    //edit tiles_within
    for (a_idx, a_shape) in objs.iter().enumerate() {
        for (b_idx, (b_rect, tile_data)) in level.tiles_within(*a_shape).enumerate() {
            let b_shape = Shape::Rect(b_rect);

            if tile_data.solid {
                if let Some(overlap) = a_shape.overlap(b_shape) {
                    contacts.push(Contact2 {
                        a_i: a_idx,
                        a_r: *a_shape,
                        b_i: b_idx,
                        b_r: b_shape,
                        displacement: overlap,
                    });
                }
            }
        }
    }
    contacts
}

impl Game {
    fn do_collision_response(&mut self, contacts: &mut Vec<Contact>) {
        for contact in contacts.iter_mut() {
            if contact.displacement.x < contact.displacement.y {
                contact.displacement.y = 0.0;
            } else {
                contact.displacement.x = 0.0;
            }

            let b_pos = contact.b_r.rect_to_pos();

            if let Some(entity) = self.entities.get_mut(contact.a_i) {
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

    //todo! Separate projectiles from entities
    fn projectile_level_response(&mut self, contacts: &mut Vec<Contact>) {
        for contact in contacts.iter_mut() {
            if contact.displacement.x < contact.displacement.y {
                contact.displacement.y = 0.0;
            } else {
                contact.displacement.x = 0.0;
            }

            let b_pos = contact.b_r.rect_to_pos();

            if let Some(projectile) = self.projectiles.get_mut(contact.a_i) {
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

    fn kill_player(&mut self, player_contacts: &mut Vec<Contact>) {
        for contact in player_contacts.iter_mut() {
            if contact.b_i == 0 {
                self.entities[0].alive = false;
            }
            if contact.b_i == 1 {
                self.entities[1].alive = false;
            }
        }
    }
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let source = assets_manager::source::FileSystem::new("fighter/content")
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
                .load::<String>("level3")
                .expect("Couldn't access level3.txt")
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
            .find(|(t, _)| *t == EntityType::Player)
            .map(|(_, ploc)| ploc)
            .expect("Start level doesn't put the player anywhere");
        let player2_start = *levels[current_level]
            .starts()
            .iter()
            .find(|(t, _)| *t == EntityType::Player)
            .map(|(_, ploc)| ploc)
            .expect("Start level doesn't put the player anywhere");

        let mut game = Game {
            assets: cache,
            current_level,
            p1_attack_timer: 0.0,
            p2_attack_timer: 0.0,
            bounce: Vec::new(),
            levels,
            health: 3,
            entities: vec![
                Entity {
                    alive: true,
                    etype: EntityType::Player,
                    pos: player_start,
                    dir: 0.0,
                },
                Entity {
                    alive: true,
                    etype: EntityType::Player,
                    pos: player2_start,
                    dir: 0.0,
                },
            ],
            projectiles: Vec::new(),
        };
        game.enter_level(player_start, player2_start);
        game
    }
    fn level(&self) -> &Level {
        &self.levels[self.current_level]
    }
    fn enter_level(&mut self, player_pos: Vec2, player2_pos: Vec2) {
        self.entities.truncate(2);
        self.entities[0].pos = player_pos;
        self.entities[1].pos = player2_pos;
        for (etype, pos) in self.levels[self.current_level].starts().iter() {
            match etype {
                EntityType::Player => {}
                EntityType::Door(_rm, _x, _y) => todo!("doors not supported"),
                EntityType::Enemy => self.entities.push(Entity {
                    alive: true,
                    pos: *pos,
                    dir: 270.0,
                    etype: etype.clone(),
                }),
                EntityType::Projectile => {}
            }
        }
    }
    fn render(&mut self, frend: &mut Immediate) {
        self.level().render_immediate(frend);

        if self.entities[0].alive {
            frend.draw_sprite(0, self.entities[0].transform(), PLAYER);
        }
        if self.entities[1].alive {
            frend.draw_sprite(0, self.entities[1].transform(), PLAYER2);
        }

        for entity in self.entities[2..].iter() {
            if entity.alive {
                frend.draw_sprite(0, entity.transform(), entity.uv());
            }
        }

        for (p_i, projectile) in self.projectiles.iter().enumerate() {
            if projectile.alive {
                frend.draw_sprite(0, projectile.transform(), projectile.uv());
            }
        }

        // do we need this? what is this for?

        // let (w, h) = match self.entities[0].dir {
        //     90.0 | 270.0 => (16, 8),
        //     _ => (8, 16),
        // };

        // let delta = dir_to_vec2(self.entities[0].dir) * 7.0;
        // let delta2 = dir_to_vec2(self.entities[1].dir) * 7.0;

        // let pos = self.entities[0].pos + delta;
        // let pos2 = self.entities[1].pos + delta;
    }
    fn simulate(&mut self, input: &Input, dt: f32) {
        if self.p1_attack_timer > 0.0 {
            self.p1_attack_timer -= dt;
        }
        if self.p2_attack_timer > 0.0 {
            self.p2_attack_timer -= dt;
        }

        let mut d_angle: f32 = 0.0;
        let mut d_angle2: f32 = 0.0;

        if input.is_key_down(Key::ArrowLeft) {
            d_angle += ROTATE_SPEED;
        } else if input.is_key_down(Key::ArrowRight) {
            d_angle -= ROTATE_SPEED;
        }

        if input.is_key_down(Key::KeyA) {
            d_angle2 += ROTATE_SPEED;
        } else if input.is_key_down(Key::KeyD) {
            d_angle2 -= ROTATE_SPEED;
        }

        self.entities[0].dir += d_angle;
        self.entities[1].dir += d_angle2;

        if self.p1_attack_timer <= 0.0 && input.is_key_pressed(Key::Space) {
            // TODO POINT: compute the attack area's center based on the player's position and facing and some offset
            // For the spritesheet provided, the attack is placed 8px "forwards" from the player.
            self.projectiles.push(Entity {
                alive: true,

                // how to put the bullet at the top of the tank so it doesnt kill itself
                pos: self.entities[0].pos + dir_to_vec2(self.entities[0].dir) * 15.0,
                dir: self.entities[0].dir,
                etype: EntityType::Projectile,
            });

            self.bounce.push(3);

            self.p1_attack_timer = ATTACK_MAX_TIME;
        }

        if self.p2_attack_timer <= 0.0 && input.is_key_pressed(Key::KeyQ) {
            // TODO POINT: compute the attack area's center based on the player's position and facing and some offset
            // For the spritesheet provided, the attack is placed 8px "forwards" from the player.
            self.projectiles.push(Entity {
                alive: true,
                pos: self.entities[1].pos + dir_to_vec2(self.entities[1].dir) * 15.0,
                dir: self.entities[1].dir,
                etype: EntityType::Projectile,
            });

            self.bounce.push(3);

            self.p2_attack_timer = ATTACK_MAX_TIME;
        }

        let mut dest = self.entities[0].pos;
        let mut dest2 = self.entities[1].pos;

        if input.is_key_down(Key::ArrowUp) {
            dest += dir_to_vec2(self.entities[0].dir);
        } else if input.is_key_down(Key::ArrowDown) {
            dest += dir_to_vec2(self.entities[0].dir) * -1.0;
        }

        if input.is_key_down(Key::KeyW) {
            dest2 += dir_to_vec2(self.entities[1].dir);
        } else if input.is_key_down(Key::KeyS) {
            dest2 += dir_to_vec2(self.entities[1].dir) * -1.0;
        }

        self.entities[0].pos = dest;
        self.entities[1].pos = dest2;

        let mut rng = rand::thread_rng();
        for enemy in self.entities[2..5].iter_mut() {
            if rng.gen_bool(0.05) {
                enemy.dir = match rng.gen_range(0..4) {
                    0 => 180.0,
                    1 => 0.0,
                    2 => 270.0,
                    3 => 90.0,
                    _ => panic!(),
                };
            }
            enemy.pos += dir_to_vec2(enemy.dir) * ENEMY_SPEED * DT;
        }

        for projectile in self.projectiles.iter_mut() {
            projectile.pos += dir_to_vec2(projectile.dir);
        }

        //Collision Detection & Response:
        let player_rects: Vec<Rect> = self.entities.iter().map(|entity| entity.rect()).collect();

        let projectile_circles: Vec<Rect> = self
            .projectiles
            .iter()
            .map(|projectile| projectile.rect())
            .collect();

        let mut player_level_contacts: Vec<Contact> =
            gather_level_contacts(&player_rects, self.level());

        let mut projectile_player_contacts: Vec<Contact> =
            gather_contacts(&projectile_circles, &player_rects);

        let mut projectile_level_contacts: Vec<Contact> =
            gather_level_contacts(&projectile_circles, self.level());

        player_level_contacts.sort_by(|a, b| {
            b.displacement
                .mag_sq()
                .partial_cmp(&a.displacement.mag_sq())
                .unwrap()
        });

        self.do_collision_response(&mut player_level_contacts);
        self.kill_player(&mut projectile_player_contacts);
        self.projectile_level_response(&mut projectile_level_contacts);
    }
}
