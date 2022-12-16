extern crate sdl2;

use rand::Rng;
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
use std::f64::consts::*;
use std::ops::Add;
use std::time::Duration;
#[derive(PartialEq)]
struct Old {
    pos: FloatPoint,
    rot: f64,
}
#[derive(Copy, Clone, PartialEq)]
struct FloatPoint {
    x: f64,
    y: f64,
}
impl FloatPoint {
    fn new(x: f64, y: f64) -> FloatPoint {
        FloatPoint { x: x, y: y }
    }
    fn to_sdl(&self) -> Point {
        Point::new(self.x as i32, self.y as i32)
    }
}
#[derive(PartialEq)]
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
#[derive(PartialEq)]
enum ShapeType {
    Asteroid(f64),
    Ship,
    Bullet,
    Dummy,
}
impl ShapeType {
    fn unwrap(&self) -> f64 {
        match self {
            ShapeType::Asteroid(s) => *s,
            _ => 0.0,
        }
    }
}
#[derive(PartialEq)]
struct Shape {
    old: Old,
    pos: FloatPoint,
    rot: f64,
    s: f64,
    s_rot: f64,
    v: VecDeque<FloatPoint>,
    color: Color,
    scale: f64,
    kind: ShapeType,
    bound: bool,
}
impl Shape {
    fn new(x: f64, y: f64, rot: f64, scale: f64, color: Color, shape: ShapeType) -> Shape {
        Shape {
            old: Old {
                pos: FloatPoint::new(x, y),
                rot: rot,
            },
            pos: FloatPoint::new(x, y),
            rot: rot,
            s: 0.0,
            s_rot: 0.0,
            v: VecDeque::new(),
            color: color,
            scale: scale,
            kind: shape,
            bound: false,
        }
    }
    fn set_bound(&mut self) {
        self.bound = true;
    }
    fn fire(&self) -> Shape {
        Shape {
            old: Old {
                pos: FloatPoint::new(
                    self.pos.x + self.scale * self.rot.cos(),
                    self.pos.y + self.scale * self.rot.sin(),
                ),
                rot: self.rot,
            },
            pos: FloatPoint::new(
                self.pos.x + self.scale * self.rot.cos(),
                self.pos.y + self.scale * self.rot.sin(),
            ),
            rot: self.rot,
            s: self.scale,
            s_rot: 0.0,
            v: VecDeque::new(),
            color: self.color,
            scale: self.scale / 4.0,
            kind: ShapeType::Bullet,
            bound: false,
        }
    }
    fn find_verticies(&mut self) {
        match self.kind {
            ShapeType::Ship => {
                self.v.clear();
                self.v.push_front(FloatPoint::new(
                    (self.pos.x + self.scale * self.rot.cos()),
                    (self.pos.y + self.scale * self.rot.sin()),
                ));

                self.v.push_front(FloatPoint::new(
                    (self.pos.x - (self.scale) * (self.rot + 0.53).cos()),
                    (self.pos.y - (self.scale) * (self.rot + 0.53).sin()),
                ));
                self.v.push_front(FloatPoint::new(
                    (self.pos.x - (self.scale / 4.0) * self.rot.cos()),
                    (self.pos.y - (self.scale / 4.0) * self.rot.sin()),
                ));
                self.v.push_front(FloatPoint::new(
                    (self.pos.x - (self.scale) * (self.rot - 0.53).cos()),
                    (self.pos.y - (self.scale) * (self.rot - 0.53).sin()),
                ));
            }
            ShapeType::Bullet => {
                self.v.clear();
                self.v.push_front(FloatPoint::new(
                    (self.pos.x + self.scale * self.rot.cos()),
                    (self.pos.y + self.scale * self.rot.sin()),
                ));
                self.v.push_front(FloatPoint::new(
                    (self.pos.x - (self.scale) * (self.rot - 0.53).cos()),
                    (self.pos.y - (self.scale) * (self.rot - 0.53).sin()),
                ));
                self.v.push_front(FloatPoint::new(
                    (self.pos.x - (self.scale) * (self.rot + 0.53).cos()),
                    (self.pos.y - (self.scale) * (self.rot + 0.53).sin()),
                ));
            }
            ShapeType::Asteroid(s) => {
                if s == 0.0 {
                    self.v.clear();
                    let mut angle: f64 = 0.0;
                    while angle < TAU {
                        let mut rng = rand::thread_rng();
                        let distance = rng.gen_range((self.scale * 0.7)..=self.scale);
                        self.v.push_front(FloatPoint::new(
                            self.pos.x + distance * angle.cos(),
                            self.pos.y + distance * angle.sin(),
                        ));

                        angle += rng.gen_range(PI / 32.0..=PI / 8.0);
                    }
                    let mut rng = rand::thread_rng();
                    let neg_rng: i32 = rand::thread_rng().gen_range(0..=1);
                    if neg_rng == 1 {
                        self.kind = ShapeType::Asteroid(-rng.gen_range(0.0..=PI / 64.0));
                    } else if neg_rng == 0 {
                        self.kind = ShapeType::Asteroid(rng.gen_range(0.0..=PI / 64.0));
                    }
                } else if s != 0.0 {
                    for i in 0..self.v.len() {
                        let distance = ((self.v[i].x - self.old.pos.x).powf(2.0)
                            + (self.v[i].y - self.old.pos.y).powf(2.0))
                        .sqrt();
                        let mut angle = -((self.v[i].x - self.old.pos.x) / distance).acos();
                        if (self.v[i].y - self.old.pos.y) > 0.0 {
                            angle = (PI as f64 + (PI as f64 - angle));
                        }
                        let new_angle = angle + (self.rot - self.old.rot) + self.kind.unwrap();
                        self.v[i].x = self.pos.x + distance * (new_angle).cos();
                        self.v[i].y = self.pos.y + distance * (new_angle).sin();
                    }
                }
            }
            ShapeType::Dummy => {}
        }
    }
    fn draw(&mut self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(self.color);
        self.find_verticies();

        for i in 0..self.v.len() {
            canvas
                .draw_line(
                    self.v[i].to_sdl(),
                    self.v[(i + 1).rem_euclid(self.v.len())].to_sdl(),
                )
                .unwrap();
        }
    }
    fn bound(&mut self, x: f64, y: f64, canvas: &mut Canvas<Window>) {
        if self.bound == true {
            let mut dummies: Vec<Shape> = Vec::new();
            if self.pos.x <= self.scale {
                dummies.push(Shape::new(
                    self.pos.x + x,
                    self.pos.y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                dummies.push(Shape::new(
                    self.pos.x + x,
                    self.pos.y + y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                if self.pos.x <= 0.0 {
                    self.pos.x = self.pos.x + x;
                }
            }
            if self.pos.x >= x - self.scale {
                dummies.push(Shape::new(
                    self.pos.x - x,
                    self.pos.y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                dummies.push(Shape::new(
                    self.pos.x - x,
                    self.pos.y - y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                if self.pos.x >= x {
                    self.pos.x = self.pos.x - x;
                }
            }
            if self.pos.y <= self.scale {
                dummies.push(Shape::new(
                    self.pos.x,
                    self.pos.y + y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                dummies.push(Shape::new(
                    self.pos.x - x,
                    self.pos.y + y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                if self.pos.y <= 0.0 {
                    self.pos.y = self.pos.y + y;
                }
            }
            if self.pos.y >= y - self.scale {
                dummies.push(Shape::new(
                    self.pos.x,
                    self.pos.y - y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                dummies.push(Shape::new(
                    self.pos.x + x,
                    self.pos.y - y,
                    self.rot,
                    self.scale,
                    self.color,
                    ShapeType::Dummy,
                ));
                if self.pos.y >= y {
                    self.pos.y = self.pos.y - y;
                }
            }
            for dummy in dummies.iter_mut() {
                for i in 0..self.v.len() {
                    let difference =
                        FloatPoint::new(dummy.pos.x - self.pos.x, dummy.pos.y - self.pos.y);
                    dummy.v.push_front(FloatPoint::new(
                        self.v[i].x + difference.x,
                        self.v[i].y + difference.y,
                    ));
                }
                dummy.draw(canvas);
            }
        }
    }
    fn direct(&mut self, e: &sdl2::EventPump) {
        self.old.pos = self.pos;
        self.old.rot = self.rot;
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
            && self.s < 1.0
            && !e.keyboard_state().is_scancode_pressed(Scancode::S)
        {
            if self.s < 0.0 {
                self.s += 0.03125;
            }
            self.s += 0.03125;
        } else if e.keyboard_state().is_scancode_pressed(Scancode::S)
            && self.s > -0.5
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
        self.pos = FloatPoint::new(
            self.pos.x + (self.scale / 4.0 * self.rot.cos()) * self.s,
            self.pos.y + (self.scale / 4.0 * self.rot.sin()) * self.s,
        )
    }
    fn color(&mut self, r: u8, g: u8, b: u8) {
        self.color = Color::RGB(r, g, b);
    }
}

fn rand_f64(x: f64, y: f64) -> f64 {
    let rng = rand::thread_rng().gen_range(x..=y);
    rng
}
fn create_asteroid(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Shape {
    let enter = rand::thread_rng().gen_range(0..=3);
    let mut rpoint = FloatPoint::new(0.0, 0.0);
    let mut rot = 0.0;
    if enter == 0 {
        rpoint = FloatPoint::new(rand_f64(-x_max / 2.0, x_min), rand_f64(-y_max / 2.0, y_min));
        rot = ((x_max / 2.0 - rpoint.x) / (y_max / 2.0 - rpoint.y)).atan();
    } else if enter == 1 {
        rpoint = FloatPoint::new(
            rand_f64(x_max, x_max + x_max / 2.0),
            rand_f64(-y_max / 2.0, y_min),
        );
        rot = PI + ((x_max / 2.0 - rpoint.x) / (y_max / 2.0 - rpoint.y)).atan();
    } else if enter == 2 {
        rpoint = FloatPoint::new(
            rand_f64(x_max, x_max + x_max / 2.0),
            rand_f64(y_max, y_max + y_max / 2.0),
        );
        rot = PI + ((x_max / 2.0 - rpoint.x) / (y_max / 2.0 - rpoint.y)).atan();
    } else if enter == 3 {
        rpoint = FloatPoint::new(
            rand_f64(-x_max / 2.0, x_min),
            rand_f64(y_max, y_max + y_max / 2.0),
        );
        rot = ((x_max / 2.0 - rpoint.x) / (y_max / 2.0 - rpoint.y)).atan();
    }
    let rrot = rot;
    let rscale = rand_f64(10.0, 80.0);
    Shape {
        old: Old {
            pos: rpoint,
            rot: rrot,
        },
        pos: rpoint,
        rot: rrot,
        s: rand_f64(0.25, 0.75),
        s_rot: 0.0,
        v: VecDeque::new(),
        color: Color::RGB(255, 255, 255),
        scale: rscale,
        kind: ShapeType::Asteroid(0.0),
        bound: false,
    }
}
fn split_asteroid(vector: &mut Vec<Shape>, index: usize) {
    let angle = rand_f64(PI / 4.0, PI / 2.0);
    let speed_diff = rand_f64(vector[index].s * 0.25, vector[index].s * 0.75);
    let scale_diff = rand_f64(vector[index].scale * 0.40, vector[index].scale * 0.60);
    let new1 = Shape {
        old: Old {
            pos: vector[index].old.pos,
            rot: vector[index].old.rot + angle,
        },
        pos: vector[index].pos,
        rot: vector[index].rot + angle,
        s: rand_f64(vector[index].s, vector[index].s * 2.0),
        s_rot: 2.0 * vector[index].s - speed_diff,
        v: VecDeque::new(),
        color: vector[index].color,
        scale: vector[index].scale - scale_diff,
        kind: ShapeType::Asteroid(0.0),
        bound: true,
    };
    let new2 = Shape {
        old: Old {
            pos: vector[index].old.pos,
            rot: vector[index].old.rot - angle,
        },
        pos: vector[index].pos,
        rot: vector[index].rot - angle,
        s: vector[index].s + speed_diff,
        s_rot: vector[index].s_rot,
        v: VecDeque::new(),
        color: vector[index].color,
        scale: scale_diff,
        kind: ShapeType::Asteroid(0.0),
        bound: true,
    };
    vector.push(new1);
    vector.push(new2);
}
fn destroy(tester: &mut Vec<Shape>, to_destroy: &Shape) -> bool {
    let mut destroy = false;
    for i in 0..tester.len() {
        if (tester[i].pos.x - to_destroy.pos.x).powf(2.0)
            + (tester[i].pos.y - to_destroy.pos.y).powf(2.0)
            <= ((tester[i].scale * 0.8) + (to_destroy.scale * 0.8)).powf(2.0)
        {
            destroy = true;
        }
    }
    destroy
}
fn collide(tester: &mut Vec<Shape>, testee: &mut Vec<Shape>, score: u32) -> u32 {
    let mut score = score;
    if tester.len() > 0 && testee.len() > 0 {
        'remove: loop {
            for i in 0..tester.len() {
                for j in 0..testee.len() {
                    if (tester[i].pos.x - testee[j].pos.x).powf(2.0)
                        + (tester[i].pos.y - testee[j].pos.y).powf(2.0)
                        <= tester[i].scale.powf(2.0)
                    {
                        score = add_score(&mut tester[i], score);
                        split_asteroid(tester, i);
                        tester.remove(i);
                        testee.remove(j);
                        break 'remove;
                    }
                }
                if i == tester.len() - 1 {
                    break 'remove;
                }
            }
        }
    }
    score
}
fn add_score(shape: &mut Shape, score: u32) -> u32 {
    let score = score + shape.scale as u32;
    score
}
pub fn main() {
    let mut to_create = 0;
    let mut loop_iter = 0;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut score: u32 = 0;
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
    let res_x = 600.0;
    let res_y = 600.0;

    let window = video_subsystem
        .window("L'asteroids", res_x as u32, res_y as u32)
        .allow_highdpi()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut player = Shape::new(
        300.0,
        300.0,
        -PI / 2.0,
        10.0,
        Color::RGB(255, 255, 255),
        ShapeType::Ship,
    );
    player.set_bound();
    let mut alive = true;
    let mut bullets: Vec<Shape> = Vec::new();
    let mut asteroids: Vec<Shape> = Vec::new();
    let mut fire_delay = 0;

    'running: loop {
        let start = ::std::time::Instant::now();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => player.kind = ShapeType::Asteroid(0.0),
                Event::KeyDown {
                    keycode: Some(Keycode::T),
                    ..
                } => player.kind = ShapeType::Ship,
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                } => asteroids.push(create_asteroid(0.0, res_x, 0.0, res_y)),
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => asteroids.clear(),
                Event::KeyDown {
                    keycode: Some(Keycode::Period),
                    ..
                } => player.scale += 1.0,
                Event::KeyDown {
                    keycode: Some(Keycode::Comma),
                    ..
                } => player.scale -= 1.0,
                Event::KeyDown {
                    keycode: Some(Keycode::Equals),
                    ..
                } => alive = true,

                _ => {}
            }
        }
        if alive == true {
            player.direct(&event_pump);
            if event_pump
                .keyboard_state()
                .is_scancode_pressed(Scancode::Space)
            {
                if fire_delay == 0 {
                    bullets.push(player.fire());
                    player.color(255, 100, 0);
                    //shoot_sound.resume();
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
                //shoot_sound.pause();
                fire_delay = (fire_delay + 1) % 15;
            }
            player.bound(res_x, res_y, &mut canvas);
            player.draw(&mut canvas);
        }

        // The rest of the game loop goes here...

        if !bullets.is_empty() {
            bullets[0].draw(&mut canvas);
            if bullets[0].pos.x < 0.0
                || bullets[0].pos.x > res_x
                || bullets[0].pos.y < 0.0
                || bullets[0].pos.y > res_y
            {
                bullets.remove(0);
            }
        }

        if loop_iter == 0 {
            to_create += 1;
            if to_create == 4 {
                asteroids.push(create_asteroid(0.0, res_x, 0.0, res_y));
                to_create = 0;
            }
        }

        for asteroid in asteroids.iter_mut() {
            if asteroid.pos.x <= res_x - asteroid.scale
                && asteroid.pos.x >= asteroid.scale
                && asteroid.pos.y <= res_y - asteroid.scale
                && asteroid.pos.y >= asteroid.scale
            {
                asteroid.bound = true;
            }
            asteroid.pos.x =
                asteroid.pos.x + (asteroid.scale / 16.0 * (asteroid.rot).cos()) * asteroid.s;
            asteroid.pos.y =
                asteroid.pos.y + (asteroid.scale / 16.0 * (asteroid.rot).sin()) * asteroid.s;
            asteroid.bound(res_x, res_y, &mut canvas);
            asteroid.draw(&mut canvas);
            asteroid.old.pos = asteroid.pos;
        }
        for i in 0..bullets.len() {
            bullets[i].s = player.scale * 0.6;
            bullets[i].pos = FloatPoint::new(
                bullets[i].pos.x + (bullets[i].s * bullets[i].rot.cos()),
                bullets[i].pos.y + (bullets[i].s * bullets[i].rot.sin()),
            );
            bullets[i].draw(&mut canvas);
        }
        score = collide(&mut asteroids, &mut bullets, score);
        println!("score: {}", score);
        if destroy(&mut asteroids, &player) {
            alive = false;
        }
        let mut ast_indx: Vec<usize> = Vec::new();
        if asteroids.len() > 0 {
            'remove: loop {
                for i in 0..asteroids.len() {
                    if asteroids[i].scale <= player.scale / 2.0 {
                        asteroids.remove(i);
                        break;
                    }
                    if i == asteroids.len() - 1 {
                        break 'remove;
                    }
                }
                if asteroids.len() == 0 {
                    break 'remove;
                }
            }
        }

        canvas.present();
        println!("{loop_iter}");
        loop_iter += 1;
        if loop_iter >= 60 {
            loop_iter -= 60;
        }
        let end = ::std::time::Instant::now();
        let delta_time = end - start;
        let frame_dur = Duration::new(0, 1_000_000_000u32 / 60);
        if delta_time <= frame_dur {
            let sleep = frame_dur - delta_time;
            ::std::thread::sleep(sleep);
        }
    }
}
