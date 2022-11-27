extern crate sdl2;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::{KeyboardState, Keycode, Scancode};
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::sys::KeyCode;
use sdl2::video::Window;
use std::collections::VecDeque;
use std::ops::Add;
use std::time::Duration;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
#[derive(Copy, Clone, PartialEq)]
enum ShapeType {
    Asteroid,
    Ship,
    Bullet,
}
struct Shape {
    pos: Point,
    rot: f64,
    s: f64,
    s_rot: f64,
    v: VecDeque<Point>,
    color: Color,
    scale: f64,
    kind: ShapeType,
}
impl Shape {
    fn new(x: i32, y: i32, rot: f64, scale: f64, color: Color, shape: ShapeType) -> Shape {
        Shape {
            pos: Point::new(x, y),
            rot: rot,
            s: 0.0,
            s_rot: 0.0,
            v: VecDeque::new(),
            color: color,
            scale: scale,
            kind: shape,
        }
    }
    fn fire(&self) -> Shape {
        Shape {
            pos: self.pos.add(Point::new(
                (self.scale * self.rot.cos()) as i32,
                (self.scale * self.rot.sin()) as i32,
            )),
            rot: self.rot,
            s: 0.0,
            s_rot: 0.0,
            v: VecDeque::new(),
            color: self.color,
            scale: 7.0,
            kind: ShapeType::Bullet,
        }
    }
    fn find_verticies(&mut self) {
        self.v.clear();
        if self.kind == ShapeType::Ship {
            self.v.push_front(self.pos.add(Point::new(
                (self.scale * self.rot.cos()) as i32,
                (self.scale * self.rot.sin()) as i32,
            )));

            self.v.push_front(self.pos.add(Point::new(
                (-(self.scale) * (self.rot + 0.53).cos()) as i32,
                (-(self.scale) * (self.rot + 0.53).sin()) as i32,
            )));
            self.v.push_front(self.pos.add(Point::new(
                (-(self.scale / 4.0) * self.rot.cos()) as i32,
                (-(self.scale / 4.0) * self.rot.sin()) as i32,
            )));
            self.v.push_front(self.pos.add(Point::new(
                (-(self.scale) * (self.rot - 0.53).cos()) as i32,
                (-(self.scale) * (self.rot - 0.53).sin()) as i32,
            )));
        } else if self.kind == ShapeType::Bullet {
            self.v.push_front(self.pos.add(Point::new(
                (self.scale * self.rot.cos()) as i32,
                (self.scale * self.rot.sin()) as i32,
            )));
            self.v.push_front(self.pos.add(Point::new(
                (-(self.scale) * (self.rot - 0.53).cos()) as i32,
                (-(self.scale) * (self.rot - 0.53).sin()) as i32,
            )));
            self.v.push_front(self.pos.add(Point::new(
                (-(self.scale) * (self.rot + 0.53).cos()) as i32,
                (-(self.scale) * (self.rot + 0.53).sin()) as i32,
            )));
        }
    }
    fn draw(&mut self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(self.color);
        self.find_verticies();
        for v in self.v.iter() {
            let i = self.v.iter().position(|i| i == v).unwrap();
            canvas.draw_line(self.v[i], self.v[(i + 1).rem_euclid(self.v.len())]);
        }
    }
    fn bound(&mut self, x: i32, y: i32, canvas: &mut Canvas<Window>) {
        let mut dummies: Vec<Shape> = Vec::new();
        if self.pos.x <= (self.scale) as i32 {
            dummies.push(Shape::new(
                self.pos.x + x,
                self.pos.y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            dummies.push(Shape::new(
                self.pos.x + x,
                self.pos.y + y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            if self.pos.x <= 0 {
                self.pos.x = self.pos.x + x;
            }
        }
        if self.pos.x >= x - (self.scale) as i32 {
            dummies.push(Shape::new(
                self.pos.x - x,
                self.pos.y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            dummies.push(Shape::new(
                self.pos.x - x,
                self.pos.y - y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            if self.pos.x >= x {
                self.pos.x = self.pos.x - x;
            }
        }
        if self.pos.y <= (self.scale) as i32 {
            dummies.push(Shape::new(
                self.pos.x,
                self.pos.y + y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            dummies.push(Shape::new(
                self.pos.x - x,
                self.pos.y + y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            if self.pos.y <= 0 {
                self.pos.y = self.pos.y + y;
            }
        }
        if self.pos.y >= y - (self.scale) as i32 {
            dummies.push(Shape::new(
                self.pos.x,
                self.pos.y - y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            dummies.push(Shape::new(
                self.pos.x + x,
                self.pos.y - y,
                self.rot,
                self.scale,
                self.color,
                self.kind,
            ));
            if self.pos.y >= y {
                self.pos.y = self.pos.y - y;
            }
        }
        for dummy in dummies.iter_mut() {
            dummy.scale = self.scale;
            for v in self.v.iter_mut() {
                dummy.v.push_front(*v)
            }
            dummy.draw(canvas);
        }
    }
    fn direct(&mut self, e: &sdl2::EventPump) {
        if e.keyboard_state().is_scancode_pressed(Scancode::A)
            && self.s_rot > -1.0
            && !e.keyboard_state().is_scancode_pressed(Scancode::D)
        {
            if self.s_rot > 0.0 {
                self.s_rot -= 0.0625;
            }
            self.s_rot -= 0.0625;
        } else if e.keyboard_state().is_scancode_pressed(Scancode::D)
            && self.s_rot < 1.0
            && !e.keyboard_state().is_scancode_pressed(Scancode::A)
        {
            if self.s_rot < 0.0 {
                self.s_rot += 0.0625;
            }
            self.s_rot += 0.0625;
        } else {
            if self.s_rot < 0.0 {
                self.s_rot += 0.0625;
            }
            if self.s_rot > 0.0 {
                self.s_rot -= 0.0625;
            }
        }
        if e.keyboard_state().is_scancode_pressed(Scancode::W)
            && self.s < 2.0
            && !e.keyboard_state().is_scancode_pressed(Scancode::S)
        {
            if self.s < 0.0 {
                self.s += 0.03125;
            }
            self.s += 0.03125;
        } else if e.keyboard_state().is_scancode_pressed(Scancode::S)
            && self.s > -1.0
            && !e.keyboard_state().is_scancode_pressed(Scancode::W)
        {
            if self.s > 0.0 {
                self.s -= 0.03125;
            }
            self.s -= 0.03125;
        } else {
            if self.s > 0.0 {
                self.s -= 0.03125;
            } else if self.s < 0.0 {
                self.s += 0.03125;
            }
        }
        self.rot = self.rot + self.s_rot * 0.1;
        self.pos = self.pos.add(Point::new(
            ((self.scale / 4.0 * self.rot.cos()) * self.s) as i32,
            (self.scale / 4.0 * self.rot.sin() * self.s) as i32,
        ))
    }
    fn color(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color::RGB(r, g, b);
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(60000),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };
    let shoot_sound = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.7,
                volume: 0.25,
            }
        })
        .unwrap();
    let res_x = 1200;
    let res_y = 600;

    let window = video_subsystem
        .window("L'asteroids", res_x, res_y)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut player = Shape::new(
        300,
        300,
        0.0,
        30.0,
        Color::RGB(255, 255, 255),
        ShapeType::Ship,
    );
    let mut bullets: Vec<Shape> = Vec::new();
    let mut fire_delay = 0;

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        player.direct(&event_pump);
        if event_pump
            .keyboard_state()
            .is_scancode_pressed(Scancode::Space)
        {
            if fire_delay == 0 {
                bullets.push(player.fire());
                player.color(255, 100, 0);
                shoot_sound.resume();
            } else {
                player.color(
                    255,
                    150 + 105 / (15 - fire_delay),
                    0 + 255 / (15 - fire_delay),
                );
            }
            fire_delay = (fire_delay + 1) % 15;
        } else if fire_delay > 0 {
            player.color(
                255,
                150 + 105 / (15 - fire_delay),
                0 + 255 / (15 - fire_delay),
            );
            fire_delay = (fire_delay + 1) % 15;
        }
        if fire_delay == 0 {
            shoot_sound.pause();
        }

        // The rest of the game loop goes here...
        player.bound(res_x as i32, res_y as i32, &mut canvas);
        player.draw(&mut canvas);
        if bullets.len() > 1000 {
            bullets.clear();
        }
        if !bullets.is_empty() {
            bullets[0].draw(&mut canvas);
            if bullets[0].pos.x < 0
                || bullets[0].pos.x > res_x as i32
                || bullets[0].pos.y < 0
                || bullets[0].pos.y > res_y as i32
            {
                bullets.remove(0);
            }
        }

        for i in 0..bullets.len() {
            bullets[i].s = 2.0 * player.scale;
            bullets[i].pos = Point::new(
                bullets[i].pos.x + (bullets[i].s * bullets[i].rot.cos()) as i32,
                bullets[i].pos.y + (bullets[i].s * bullets[i].rot.sin()) as i32,
            );
            bullets[i].draw(&mut canvas);
        }
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
