use assets_manager::{asset::Png, AssetCache};
use frenderer::{
    input::{Input, Key},
    sprites::{Camera2D, SheetRegion, Transform},
    wgpu, Immediate,
};
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

const PLAYER_ATK: SheetRegion = SheetRegion::rect(428, 0, 16, 8);

const ENEMY: SheetRegion = SheetRegion::rect(533 + 16, 39, 16, 16);

const PROJECTILE: SheetRegion = SheetRegion::rect(525, 19, 7, 7);

const HEART: SheetRegion = SheetRegion::rect(525, 35, 8, 8);

#[derive(Clone, Debug)]
struct Entity {
    pos: Vec2,
    dir: f32,
    etype: EntityType,
}

fn dir_to_vec2(dir: f32) -> Vec2 {
    Vec2 { x: f32::cos(dir), y: f32::sin(dir) }
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
    pub fn transform(&self) -> Transform {
        Transform {
            x: self.pos.x,
            y: self.pos.y,
            w: TILE_SZ as u16,
            h: TILE_SZ as u16,
            rot: self.dir,
        }
    }
    pub fn uv(&self) -> SheetRegion {
        match self.etype {
            EntityType::Player => PLAYER,
            EntityType::Enemy => ENEMY,
            EntityType::Projectile => PROJECTILE,
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
    attack_area: Rect,
    attack_timer: f32,
    knockback_timer: f32,
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

fn gather_contacts(objs: &Vec<Rect>, level: &Level) -> Vec<Contact> {
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
    fn do_collision_response(&mut self, contacts: &mut Vec<Contact>) {
        for (contact) in contacts.iter_mut() {
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
        Some((W as u32, H as u32)),
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
            attack_area: Rect {
                x: 0.0,
                y: 0.0,
                w: 0,
                h: 0,
            },
            knockback_timer: 0.0,
            attack_timer: 0.0,
            levels,
            health: 3,
            entities: vec![
                Entity {
                    etype: EntityType::Player,
                    pos: player_start,
                    dir: 0.0,
                },
                Entity {
                    etype: EntityType::Player,
                    pos: player2_start,
                    dir: 0.0,
                },
            ],
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
        for entity in self.entities.iter() {
            frend.draw_sprite(0, entity.transform(), entity.uv());
        }
        if !self.attack_area.is_empty() {
            let (w, h) = match self.entities[0].dir {
                90.0 | 270.0 => (16, 8),
                _ => (8, 16),
            };
            let delta = dir_to_vec2(self.entities[0].dir) * 7.0;
            let delta2 = dir_to_vec2(self.entities[1].dir) * 7.0;

            let pos = self.entities[0].pos + delta;
            let pos2 = self.entities[1].pos + delta;
            frend.draw_sprite(
                0,
                Transform {
                    w,
                    h,
                    x: pos.x,
                    y: pos.y,
                    rot: 0.0,
                },
                PLAYER_ATK.with_depth(0),
            );
            frend.draw_sprite(
                0,
                Transform {
                    w,
                    h,
                    x: pos.x,
                    y: pos.y,
                    rot: self.entities[1].dir,
                },
                PLAYER_ATK.with_depth(0),
            );
            
        }
        // TODO POINT: draw hearts
    }
    fn simulate(&mut self, input: &Input, dt: f32) {
        if self.attack_timer > 0.0 {
            self.attack_timer -= dt;
        }
        if self.knockback_timer > 0.0 {
            self.knockback_timer -= dt;
        }

        let mut d_angle: f32 = 0.0;
        let mut d_angle2: f32 = 0.0;

        if input.is_key_down(Key::ArrowLeft) {
            d_angle += ROTATE_SPEED;
        }
        else if input.is_key_down(Key::ArrowRight) {
            d_angle -= ROTATE_SPEED;
        }

        if input.is_key_down(Key::KeyA) {
            d_angle2 += ROTATE_SPEED;
        }
        else if input.is_key_down(Key::KeyD) {
            d_angle2 -= ROTATE_SPEED;
        }

        let attacking = !self.attack_area.is_empty();
        let knockback = self.knockback_timer > 0.0;

        self.entities[0].dir += d_angle;
        self.entities[1].dir += d_angle2;

        if self.attack_timer <= 0.0 && input.is_key_pressed(Key::Space) {
            // TODO POINT: compute the attack area's center based on the player's position and facing and some offset
            // For the spritesheet provided, the attack is placed 8px "forwards" from the player.
            self.attack_timer = ATTACK_MAX_TIME;
        } else if self.attack_timer <= ATTACK_COOLDOWN_TIME {
            self.attack_area = Rect {
                x: 0.0,
                y: 0.0,
                w: 0,
                h: 0,
            };
        }
        
        let mut dest = self.entities[0].pos;
        let mut dest2 = self.entities[1].pos;

        if input.is_key_down(Key::ArrowUp) {
            dest += dir_to_vec2(self.entities[0].dir);
        }
        else if input.is_key_down(Key::ArrowDown) {
            dest += dir_to_vec2(self.entities[0].dir) * -1.0;
        }

        if input.is_key_down(Key::KeyW) {
            dest2 += dir_to_vec2(self.entities[1].dir);
        }
        else if input.is_key_down(Key::KeyS) {
            dest2 += dir_to_vec2(self.entities[1].dir) * -1.0;
        }


        self.entities[0].pos = dest;
        self.entities[1].pos = dest2;

        let mut rng = rand::thread_rng();
        for enemy in self.entities[1..].iter_mut() {
            if rng.gen_bool(0.05) {
                enemy.dir = match rng.gen_range(0..4) {
                    0 => 90.0,
                    1 => 0.0,
                    2 => 270.0,
                    3 => 90.0,
                    _ => panic!(),
                };
            }
            enemy.pos += dir_to_vec2(enemy.dir) * ENEMY_SPEED * DT;
        }

        //Collision Detection & Response:
        let rects: Vec<Rect> = self.entities.iter().map(|entity| entity.rect()).collect();
        let mut contacts: Vec<Contact> = gather_contacts(&rects, self.level());
        contacts.sort_by(|a, b| {
            b.displacement
                .mag_sq()
                .partial_cmp(&a.displacement.mag_sq())
                .unwrap()
        });
        self.do_collision_response(&mut contacts);
    }
}
