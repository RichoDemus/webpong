use log::*;
use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d::query::Proximity;
use ncollide2d::shape::Cuboid;
use ncollide2d::{query, shape};
use crate::network::ws_server::WebsocketClient;
use crate::network::ws_event::WsEvent;
use crate::network::message::{Message, ClientMessage, PaddleState, GameState, ServerMessage};
use std::collections::VecDeque;
use retain_mut::RetainMut;

pub struct PongServer {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    paused: bool,
    left_player: Option<WebsocketClient>,
    right_player: Option<WebsocketClient>,
    observers: VecDeque<WebsocketClient>,
    send_gamestate:bool,
}

pub struct Ball {
    pub position: Point2<f64>,
    pub shape: shape::Ball<f64>,
    velocity: Vector2<f64>,
}

impl Ball {
    fn new() -> Self {
        Ball {
            position: Point2::new(500., 50.),
            shape: shape::Ball::new(5.),
            velocity: Vector2::new(-5., 5.),
        }
    }
}

#[derive(Debug)]
pub struct Paddle {
    pub position: Point2<f64>,
    pub shape: Cuboid<f64>,
    state: PaddleState,
    state_changed: bool,
}

impl Paddle {
    fn left() -> Self {
        Paddle {
            position: Point2::new(20., 50.),
            shape: Cuboid::new(Vector2::new(10., 100.)),
            state: PaddleState::Still,
            state_changed: false,
        }
    }
    fn right() -> Self {
        Paddle {
            position: Point2::new(770., 50.),
            shape: Cuboid::new(Vector2::new(10., 100.)),
            state: PaddleState::Still,
            state_changed: false,
        }
    }

    fn set_state(&mut self, state: PaddleState) {
        self.state = state;
        self.state_changed = true;
    }
}

impl PongServer {
    pub fn add_player(&mut self, player: WebsocketClient) {
        // if self.left_player.is_none() {
        //     info!("Added {} as left player", player.name.as_ref().unwrap_or(&String::from("N/A")));
        //     self.left_player = Some(player);
        //             self.send_gamestate = true;
        //     return;
        // }
        // if self.right_player.is_none() {
        //     info!("Added {} as right player", player.name.as_ref().unwrap_or(&String::from("N/A")));
        //     self.right_player = Some(player);
        //             self.send_gamestate = true;
        //     return;
        // }
        info!("Added {} as observer", player.name.as_ref().unwrap_or(&String::from("N/A")));
        self.observers.push_back(player);
        self.send_gamestate = true;
    }

    pub async fn tick(&mut self) {
        if !self.observers.is_empty() && (self.send_gamestate || self.left_player.is_none() || self.right_player.is_none()) {
            //also use this to check for reshuffles
            if self.left_player.is_none() {
                if let Some(new_player) = self.observers.pop_front() {
                    info!("Promoting observer {} to left player", new_player.name.as_ref().unwrap_or(&String::from("N/A")));
                    self.left_player = Some(new_player);
                } else {
                    // info!("No observer to promote to left player");
                }
            }
            if self.right_player.is_none() {
                if let Some(new_player) = self.observers.pop_front() {
                    info!("Promoting observer {} to right player", new_player.name.as_ref().unwrap_or(&String::from("N/A")));
                    self.right_player = Some(new_player);
                } else {
                    // info!("No observer to promote to right player");
                }
            }
        }
        // info!("Tick, left some: {}, send state: {}", self.left_player.is_some(), self.send_gamestate);
        if let Some(left_player) = self.left_player.as_mut() {
            if let Some(evt) = left_player.event_stream.next() {
                match evt {
                    WsEvent::Message(msg) => match msg {
                        Message::ClientMessage(msg) => match msg {
                            ClientMessage::EnterGame => {}
                            ClientMessage::PaddleUp => {
                                self.left_paddle.set_state(PaddleState::Up);
                                self.send_gamestate = true;
                            }
                            ClientMessage::PaddleDown => {
                                self.left_paddle.set_state(PaddleState::Down);
                                self.send_gamestate = true;
                            }
                            ClientMessage::PaddleStop => {
                                self.left_paddle.set_state(PaddleState::Still);
                                self.send_gamestate = true;
                            }
                        }
                        _ => {}
                    }
                    WsEvent::Closed => {
                        self.left_player = None;
                        self.send_gamestate = true;
                    }
                    _ => {},
                }
            }
        }

        if let Some(player) = self.right_player.as_mut() {
            if let Some(evt) = player.event_stream.next() {
                match evt {
                    WsEvent::Message(msg) => match msg {
                        Message::ClientMessage(msg) => match msg {
                            ClientMessage::EnterGame => {}
                            ClientMessage::PaddleUp => {
                                self.right_paddle.set_state(PaddleState::Up);
                                self.send_gamestate = true;
                            }
                            ClientMessage::PaddleDown => {
                                self.right_paddle.set_state(PaddleState::Down);
                                self.send_gamestate = true;
                            }
                            ClientMessage::PaddleStop => {
                                self.right_paddle.set_state(PaddleState::Still);
                                self.send_gamestate = true;
                            }
                        }
                        _ => {}
                    }
                    WsEvent::Closed => {
                        self.right_player = None;
                        self.send_gamestate = true;
                    }
                    _ => {},
                }
            }
        }

        self.observers.retain_mut(|observer|{
            if let Some(WsEvent::Closed) = observer.event_stream.next() {
                false
            } else {
                true
            }
        });

        // self.observers = self.observers.into_iter()
        //     .filter_map(|mut observer| {
        //         if let Some(WsEvent::Closed) = observer.event_stream.next() {
        //             return None
        //         }
        //         Some(observer)
        //     })
        //     .collect();

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

        if self.send_gamestate {
            let state = Message::ServerMessage(ServerMessage::GameState(GameState {
                left_paddle_y: self.left_paddle.position.y,
                left_paddle_state: self.left_paddle.state,
                right_paddle_y: self.right_paddle.position.y,
                right_paddle_state: self.right_paddle.state,
                left_player_name: self.left_player.as_ref().and_then(|p|p.name.clone()).unwrap_or(String::from("N/A")),
                right_player_name: self.right_player.as_ref().and_then(|p|p.name.clone()).unwrap_or(String::from("N/A")),
            }));

            let mut futures = vec![];
            if let Some(player) = self.left_player.as_mut() {
                futures.push(player.send(&state));
            }
            if let Some(player) = self.right_player.as_mut() {
                futures.push(player.send(&state));
            }
            for observer in &mut self.observers {
                futures.push(observer.send(&state));
            }
            let big_future = futures::future::join_all(futures);
            big_future.await;
            trace!("Sent some state");
            self.send_gamestate = false;
        }

        // if self.paused {
        //     return;
        // }
        //
        // self.ball.position += self.ball.velocity.clone();
        // if self.ball.position.x < 10. || self.ball.position.x > 790. {
        //     self.ball.velocity = Vector2::new(0., 0.);
        // }
        // if self.ball.position.y < 10. || self.ball.position.y > 590. {
        //     self.ball.velocity.y *= -1.;
        // }
        //
        // let ball_isometry = Isometry2::new(self.ball.position.clone().coords, nalgebra::zero());
        // let left_paddle_isometry =
        //     Isometry2::new(self.left_paddle.position.clone().coords, nalgebra::zero());
        // let right_paddle_isometry =
        //     Isometry2::new(self.right_paddle.position.clone().coords, nalgebra::zero());
        //
        // let proximity = query::proximity(
        //     &ball_isometry,
        //     &self.ball.shape,
        //     &left_paddle_isometry,
        //     &self.left_paddle.shape,
        //     0.,
        // );
        // if let Proximity::Intersecting = proximity {
        //     self.ball.velocity.x *= -1.;
        // }
        //
        // let proximity = query::proximity(
        //     &ball_isometry,
        //     &self.ball.shape,
        //     &right_paddle_isometry,
        //     &self.right_paddle.shape,
        //     0.,
        // );
        // if let Proximity::Intersecting = proximity {
        //     self.ball.velocity.x *= -1.;
        // }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn set_paddle_state(&mut self, left_paddle: bool, stop_moving: bool, up: bool) {
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
        info!(
            "Paddle states: {:?} {:?}",
            self.left_paddle.state, self.right_paddle.state
        );
    }

    pub fn get_drawables(&self) -> (f64, f64, Point2<f64>) {
        (
            self.left_paddle.position.y,
            self.right_paddle.position.y,
            self.ball.position.clone(),
        )
    }
}

impl Default for PongServer {
    fn default() -> Self {
        PongServer {
            left_paddle: Paddle::left(),
            right_paddle: Paddle::right(),
            ball: Ball::new(),
            paused: true,
            left_player: None,
            right_player: None,
            observers: VecDeque::new(),
            send_gamestate : false,
        }
    }
}