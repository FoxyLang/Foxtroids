extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::collections::VecDeque;
use std::ops::Add;
use std::time::Duration;
#[derive(Copy, Clone, PartialEq)]
enum ShapeType {
    Asteroid,
    Ship,
}
struct Shape {
    pos: Point,
    rot: f64,
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
            v: VecDeque::new(),
            color: color,
            scale: scale,
            kind: shape,
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
            println!("{}", self.rot.sin())
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
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let res_x = 600;
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
                Event::KeyDown {
                    keycode: Some(s), ..
                } => {
                    if s == Keycode::Right {
                        player.rot = player.rot + 0.15;
                    }
                    if s == Keycode::Left {
                        player.rot = player.rot - 0.15;
                    }
                    if s == Keycode::Up {
                        player.pos = player.pos.add(Point::new(
                            (player.scale / 2.0 * player.rot.cos()) as i32,
                            (player.scale / 2.0 * player.rot.sin()) as i32,
                        ))
                    }
                    if s == Keycode::Comma {
                        player.scale = player.scale - 5.0;
                    }
                    if s == Keycode::Period {
                        player.scale = player.scale + 5.0;
                    }
                }
                Event::MouseButtonDown {
                    mouse_btn: s, x, y, ..
                } => {}

                _ => {}
            }
        }
        // The rest of the game loop goes here...
        player.bound(res_x as i32, res_y as i32, &mut canvas);
        player.draw(&mut canvas);
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
