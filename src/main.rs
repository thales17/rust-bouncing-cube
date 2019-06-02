extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::time::Duration;

use specs::{Builder, Component, ReadStorage, RunNow, System, VecStorage, World, WriteStorage};

#[derive(Debug)]
struct Position {
    x: i32,
    y: i32,
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

struct Render {
    canvas: WindowCanvas,
}

impl<'a> System<'a> for Render {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, position: Self::SystemData) {
        use specs::Join;
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for position in position.join() {
            self.canvas
                .fill_rect(Rect::new(position.x, position.y, 50, 50))
                .unwrap();
        }
        self.canvas.present();
    }
}

struct Physics {
    width: u32,
    height: u32,
}

impl<'a> System<'a> for Physics {
    type SystemData = WriteStorage<'a, Position>;

    fn run(&mut self, mut position: Self::SystemData) {
        use specs::Join;
        for position in (&mut position).join() {
            position.x += 1;
            position.y += 1;

            if (position.x + 50) > self.width as i32 {
                position.x -= 1;
            }

            if (position.y + 50) > self.height as i32 {
                position.y -= 1;
            }
        }
    }
}

pub fn main() -> Result<(), String> {
    let mut world = World::new();
    world.register::<Position>();

    world
        .create_entity()
        .with(Position { x: 100, y: 10 })
        .build();
    world
        .create_entity()
        .with(Position { x: 10, y: 50 })
        .build();

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
    };

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
