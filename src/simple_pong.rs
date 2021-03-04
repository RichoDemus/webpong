use nalgebra::{Point2, Vector2, Isometry2};
use ncollide2d::shape::{Cuboid};
use ncollide2d::{shape, query};
use ncollide2d::query::Proximity;

pub struct SimplePong {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    paused: bool,
}

pub struct Ball {
    pub position: Point2<f64>,
    pub shape: shape::Ball<f64>,
    velocity: Vector2<f64>,
}

impl Ball {
    fn new() -> Self {
        Ball {
            position: Point2::new(500.,50.),
            shape: shape::Ball::new(5.),
            velocity: Vector2::new(-5.,5.),
        }
    }
}

pub struct Paddle {
    pub position: Point2<f64>,
    pub shape: Cuboid<f64>,
}

impl Paddle {
    fn left() -> Self {
        Paddle {
            position: Point2::new(20., 50.),
            shape: Cuboid::new(Vector2::new(10.,100.)),
        }
    }
    fn right() -> Self {
        Paddle {
            position: Point2::new(1570., 50.),
            shape: Cuboid::new(Vector2::new(10.,100.)),
        }
    }
}

impl SimplePong {
    pub fn new() -> Self {
        SimplePong {
            left_paddle: Paddle::left(),
            right_paddle: Paddle::right(),
            ball: Ball::new(),
            paused: true,
        }
    }

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        self.ball.position += self.ball.velocity.clone();
        if self.ball.position.x < 10. || self.ball.position.x > 1590. {
            self.ball.velocity = Vector2::new(0.,0.);
        }
        if self.ball.position.y < 10. || self.ball.position.y > 790. {
            self.ball.velocity.y *=-1.;
        }

        let ball_isometry = Isometry2::new(self.ball.position.clone().coords, nalgebra::zero());
        let left_paddle_isometry = Isometry2::new(self.left_paddle.position.clone().coords, nalgebra::zero());
        let right_paddle_isometry = Isometry2::new(self.right_paddle.position.clone().coords, nalgebra::zero());

        let proximity = query::proximity(&ball_isometry, &self.ball.shape, &left_paddle_isometry, &self.left_paddle.shape, 0.);
        if let Proximity::Intersecting = proximity {
            self.ball.velocity.x *=-1.;
        }

        let proximity = query::proximity(&ball_isometry, &self.ball.shape, &right_paddle_isometry, &self.right_paddle.shape, 0.);
        if let Proximity::Intersecting = proximity {
            self.ball.velocity.x *=-1.;
        }

    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn move_up(&mut self) {
        self.left_paddle.position.y -=8.;
    }

    pub fn move_down(&mut self) {
        self.left_paddle.position.y +=8.;
    }

    pub fn get_drawables(&self) -> (f64, f64, Point2<f64>) {
        (self.left_paddle.position.y, self.right_paddle.position.y, self.ball.position.clone())
    }
}
