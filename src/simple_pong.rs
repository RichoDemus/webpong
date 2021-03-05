use nalgebra::{Point2, Vector2, Isometry2};
use ncollide2d::shape::{Cuboid};
use ncollide2d::{shape, query};
use ncollide2d::query::Proximity;
use log::*;

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

#[derive(Debug)]
pub struct Paddle {
    pub position: Point2<f64>,
    pub shape: Cuboid<f64>,
    state: PaddleState,
}

#[derive(Debug)]
enum PaddleState {
    Up, Down, Still,
}

impl Paddle {
    fn left() -> Self {
        Paddle {
            position: Point2::new(20., 50.),
            shape: Cuboid::new(Vector2::new(10.,100.)),
            state: PaddleState::Still,
        }
    }
    fn right() -> Self {
        Paddle {
            position: Point2::new(1570., 50.),
            shape: Cuboid::new(Vector2::new(10.,100.)),
            state: PaddleState::Still,
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


        let mov = match self.right_paddle.state {
            PaddleState::Up => -8.,
            PaddleState::Down => 8.,
            PaddleState::Still => 0.,
        };
        self.right_paddle.position.y += mov;

        let mov = match self.left_paddle.state {
            PaddleState::Up => -8.,
            PaddleState::Down => 8.,
            PaddleState::Still => 0.,
        };
        self.left_paddle.position.y += mov;

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

    pub fn set_paddle_state(&mut self, left_paddle: bool, stop_moving:bool, up: bool) {
        if left_paddle {
            if stop_moving {
                self.left_paddle.state = PaddleState::Still;
            } else if up {
                self.left_paddle.state = PaddleState::Up;
            } else {
                self.left_paddle.state = PaddleState::Down;
            }
        } else {
            if stop_moving {
                self.right_paddle.state = PaddleState::Still;
            } else if up {
                self.right_paddle.state = PaddleState::Up;
            } else {
                self.right_paddle.state = PaddleState::Down;
            }
        }
        info!("Paddle states: {:?} {:?}", self.left_paddle.state, self.right_paddle.state);
    }

    pub fn get_drawables(&self) -> (f64, f64, Point2<f64>) {
        (self.left_paddle.position.y, self.right_paddle.position.y, self.ball.position.clone())
    }
}