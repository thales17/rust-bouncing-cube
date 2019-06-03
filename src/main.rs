extern crate rand;
extern crate sdl2;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use specs::world::Entity;
use specs::{
    Builder, Component, Entities, Join, ReadStorage, RunNow, System, VecStorage, World,
    WriteStorage,
};
use std::time::Duration;

#[derive(Debug)]
struct Cube {
    x: i32,
    y: i32,
    size: u32,
}

impl Component for Cube {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Velocity {
    x: i32,
    y: i32,
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

struct Render {
    canvas: WindowCanvas,
}

impl<'a> System<'a> for Render {
    type SystemData = ReadStorage<'a, Cube>;

    fn run(&mut self, cube: Self::SystemData) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for cube in cube.join() {
            self.canvas
                .fill_rect(Rect::new(cube.x, cube.y, cube.size, cube.size))
                .unwrap();
        }
        self.canvas.present();
    }
}

struct Physics {
    width: u32,
    height: u32,
    entities: Vec<Entity>,
}

impl<'a> System<'a> for Physics {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Cube>,
        WriteStorage<'a, Velocity>,
    );

    fn run(&mut self, (entities, mut cube, mut velocity): Self::SystemData) {
        for (ent, cube, vel) in (&*entities, &mut cube, &mut velocity).join() {
            cube.x += vel.x;
            cube.y += vel.y;

            if (cube.x < 0) || (cube.x + cube.size as i32) > self.width as i32 {
                vel.x *= -1;
            }

            if (cube.y < 0) || (cube.y + cube.size as i32) > self.height as i32 {
                vel.y *= -1;
            }

            for e in &self.entities {
                if e == &ent {
                    continue;
                    // TODO: Not sure how to get a reference to the other Cubes
                }
            }
        }
    }
}

pub fn main() -> Result<(), String> {
    let mut world = World::new();
    world.register::<Cube>();
    world.register::<Velocity>();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Rust Bouncing Cube", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut render = Render {
        canvas: window.into_canvas().build().map_err(|e| e.to_string())?,
    };

    let mut physics = Physics {
        width: 800,
        height: 600,
        entities: Vec::new(),
    };

    let entity_count = 200;

    for _ in 0..entity_count {
        let size = rand::thread_rng().gen_range(0, 50);

        let e = world
            .create_entity()
            .with(Cube {
                x: rand::thread_rng().gen_range(0, 800 - size),
                y: rand::thread_rng().gen_range(0, 600 - size),
                size: size as u32,
            })
            .with(Velocity {
                x: rand::thread_rng().gen_range(1, 5),
                y: rand::thread_rng().gen_range(1, 5),
            })
            .build();

        physics.entities.push(e)
    }

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            };
        }
        physics.run_now(&world.res);
        render.run_now(&world.res);
        world.maintain();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
