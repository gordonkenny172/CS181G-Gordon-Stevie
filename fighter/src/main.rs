use assets_manager::{asset::Png, AssetCache};
use frenderer::{
    input::{Input, Key},
    sprites::{Camera2D, SheetRegion, Transform},
    wgpu, Renderer,
};

mod grid;
mod geom;
use geom::*;

mod level;
use level::Level;

const TILE_SZ: usize = 16;
const W: usize = 320;
const H: usize = 240;
const DT: f32 = 1.0 / 60.0;

fn main() {
    println!("Hello, world!");
}
