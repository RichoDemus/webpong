use nalgebra::Point2;
use quicksilver::geom::{Circle, Rectangle, Vector};
use quicksilver::graphics::Color;
use quicksilver::Graphics;

pub fn draw(gfx: &mut Graphics, left_paddle: f64, right_paddle: f64, ball: Point2<f64>) {
    let left_paddle = Rectangle::new(
        Vector::new(20.0, left_paddle as f32 - 10.),
        Vector::new(10.0, 100.),
    );
    gfx.fill_rect(&left_paddle, Color::WHITE);

    let right_paddle = Rectangle::new(
        Vector::new(770.0, right_paddle as f32 - 10.),
        Vector::new(10.0, 100.),
    );
    gfx.fill_rect(&right_paddle, Color::WHITE);

    let position = Vector::new(ball.x as f32, ball.y as f32);
    let ball = Circle::new(position, 5.);
    gfx.fill_circle(&ball, Color::WHITE);
}
