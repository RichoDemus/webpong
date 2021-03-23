use nalgebra::Point2;
use quicksilver::geom::{Circle, Rectangle, Vector};
use quicksilver::graphics::Color;
use quicksilver::Graphics;
use quicksilver::graphics::FontRenderer;
use crate::simple_pong::Paddle;

pub fn draw(gfx: &mut Graphics, font: &mut FontRenderer, left_paddle: Paddle, right_paddle: Paddle, ball: Point2<f64>) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let left_paddle_vec = Rectangle::new(
        Vector::new(20.0, left_paddle.position.y as f32 - 10.),
        Vector::new(10.0, 100.),
    );
    gfx.fill_rect(&left_paddle_vec, Color::WHITE);

    let right_paddle_vec = Rectangle::new(
        Vector::new(770.0, right_paddle.position.y as f32 - 10.),
        Vector::new(10.0, 100.),
    );
    gfx.fill_rect(&right_paddle_vec, Color::WHITE);

    let position = Vector::new(ball.x as f32, ball.y as f32);
    let ball = Circle::new(position, 5.);
    gfx.fill_circle(&ball, Color::WHITE);

    font.draw(
        gfx,
        format!("{}", left_paddle.player_name).as_str(),
        Color::GREEN,
        Vector::new(30.0, 30.0),
    )?;
    font.draw(
        gfx,
        format!("{}", right_paddle.player_name).as_str(),
        Color::GREEN,
        Vector::new(700., 30.),
    )?;
    Ok(())
}
