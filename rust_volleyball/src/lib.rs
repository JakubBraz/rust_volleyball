pub mod udp_server;
pub mod tcp_server;
pub mod server_logic;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use rapier2d::prelude::*;

const TIME_STEP: f32 = 1.0 / 60.0;

const MAX_SPEED: f32 = 3.0;
const ALMOST_ZERO: f32 = 0.001;
const MOVE_FORCE: f32 = 10.0;
const START_BALL_1: f32 = 5.5;
const START_BALL_2: f32 = 2.5;
const START_BALL_HEIGHT: f32 = 2.0;
const START_PLAYER_1: f32 = 6.0;
const START_PLAYER_2: f32 = 2.0;
const START_PLAYER_HEIGHT: f32 = 0.6;
const POINT_LIMIT: u32 = 10;

// scoring, reset ball after that number of frames
const POINT_RESET: u64 = 180;
const GRAVITY_AFTER: u64 = 180;

pub struct GameState {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vector<f32>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multi_body_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: MyEventHandler,

    frame_counter: u64,
    time: f32,
    game_time: f32,
    player1_handle: RigidBodyHandle,
    player2_handle: RigidBodyHandle,
    ball_handle: RigidBodyHandle,
    ground_handle: ColliderHandle,
    net_handle: ColliderHandle,
    player1_collider_handle: ColliderHandle,
    player2_collider_handle: ColliderHandle,
    ball_collider_handle: ColliderHandle,
    left_wall_handle: ColliderHandle,
    right_wall_handle: ColliderHandle,
    middle_wall_handle: ColliderHandle,

    points1: u32,
    points2: u32,
    points_added: bool,
    ball_for_1: bool,
    ball_touch: bool,
    reset_frame: u64,
    enable_gravity_frame: u64,
    game_over: bool,
    event_handler_receiver: Receiver<CollisionEvent>,

    player_input: HashMap<RigidBodyHandle, [bool; 2]>,
}

impl GameState {
    pub fn new() -> GameState {
        let (sender, receiver) = channel();
        let ball_touch_1 = rand::random();
        let mut game_state = GameState {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, -9.81],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multi_body_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_hooks: (),
            event_handler: MyEventHandler{sender},

            frame_counter: 0,
            time: 0.0,
            game_time: 0.0,
            player1_handle: Default::default(),
            player2_handle: Default::default(),
            ball_handle: Default::default(),
            ground_handle: Default::default(),
            net_handle: Default::default(),
            player1_collider_handle: Default::default(),
            player2_collider_handle: Default::default(),
            ball_collider_handle: Default::default(),
            left_wall_handle: Default::default(),
            right_wall_handle: Default::default(),
            middle_wall_handle: Default::default(),

            points1: 0,
            points2: 0,
            points_added: false,
            ball_for_1: ball_touch_1,
            ball_touch: ball_touch_1,
            reset_frame: 0,
            enable_gravity_frame: GRAVITY_AFTER,
            game_over: false,
            event_handler_receiver: receiver,

            player_input: Default::default(),
        };

        // Create walls.
        let collider = ColliderBuilder::cuboid(4.0, 0.1)
            .translation(vector![4.0, 0.0])
            .build();
        game_state.ground_handle = game_state.collider_set.insert(collider);
        let collider = ColliderBuilder::cuboid(0.1, 200.0)
            .translation(vector![8.0, 200.0])
            .friction(0.0)
            .friction_combine_rule(CoefficientCombineRule::Min)
            .build();
        game_state.right_wall_handle = game_state.collider_set.insert(collider);
        let collider = ColliderBuilder::cuboid(0.1, 200.0)
            .translation(vector![0.0, 200.0])
            .friction(0.0)
            .friction_combine_rule(CoefficientCombineRule::Min)
            .build();
        game_state.left_wall_handle = game_state.collider_set.insert(collider);
        let collider = ColliderBuilder::cuboid(0.1, 200.0)
            .translation(vector![4.0, 200.0])
            .collision_groups(InteractionGroups::new(Group::GROUP_2, Group::GROUP_2))
            .friction(0.0)
            .friction_combine_rule(CoefficientCombineRule::Min)
            .build();
        game_state.middle_wall_handle = game_state.collider_set.insert(collider);

        // Create net
        let collider = ColliderBuilder::cuboid(0.05, 1.0)
            .translation(vector![4.0, 1.0])
            .build();
        game_state.net_handle = game_state.collider_set.insert(collider);

        // Create players
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![START_PLAYER_1, START_PLAYER_HEIGHT])
            .build();
        let collider = ColliderBuilder::ball(0.5)
            .restitution(0.7)
            .restitution_combine_rule(CoefficientCombineRule::Min)
            .build();
        game_state.player1_handle = game_state.rigid_body_set.insert(rigid_body);
        let handle = game_state.collider_set.insert_with_parent(collider, game_state.player1_handle, &mut game_state.rigid_body_set);
        game_state.player1_collider_handle = handle;

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![START_PLAYER_2, START_PLAYER_HEIGHT])
            .build();
        let collider = ColliderBuilder::ball(0.5)
            .restitution(0.7)
            .restitution_combine_rule(CoefficientCombineRule::Min)
            .build();
        game_state.player2_handle = game_state.rigid_body_set.insert(rigid_body);
        let handle = game_state.collider_set.insert_with_parent(collider, game_state.player2_handle, &mut game_state.rigid_body_set);
        game_state.player2_collider_handle = handle;

        game_state.player_input.insert(game_state.player1_handle, [false; 2]);
        game_state.player_input.insert(game_state.player2_handle, [false; 2]);

        let ball_x = if game_state.ball_for_1 { START_BALL_1 } else { START_BALL_2 };
        // Create ball
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![ball_x, START_BALL_HEIGHT])
            .gravity_scale(0.0)
            .build();
        let collider = ColliderBuilder::ball(0.25)
            .restitution(0.8)
            .density(0.9)
            .restitution_combine_rule(CoefficientCombineRule::Max)
            .collision_groups(InteractionGroups::new(Group::GROUP_1, Group::GROUP_1))
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();
        game_state.ball_handle = game_state.rigid_body_set.insert(rigid_body);
        let handle = game_state.collider_set.insert_with_parent(collider, game_state.ball_handle, &mut game_state.rigid_body_set);
        game_state.ball_collider_handle = handle;

        game_state
    }

    pub fn players(&self) -> (f32, f32, f32, f32, f32, f32) {
        let body1 = &self.rigid_body_set[self.player1_handle];
        let t1 = body1.translation();
        let r1 = self.collider_set[body1.colliders()[0]].shape().as_ball().unwrap().radius;
        let body2 = &self.rigid_body_set[self.player2_handle];
        let t2 = body2.translation();
        let r2 = self.collider_set[body2.colliders()[0]].shape().as_ball().unwrap().radius;
        (t1.x, t1.y, r1, t2.x, t2.y, r2)
    }

    pub fn ball(&self) -> (f32, f32, f32) {
        let body = &self.rigid_body_set[self.ball_handle];
        let r = self.collider_set[body.colliders()[0]].shape().as_ball().unwrap().radius;
        (body.translation().x, body.translation().y, r)
    }

    pub fn ground(&self) -> (f32, f32, f32, f32) {
        let pos = self.collider_set[self.ground_handle].translation();
        let size = &self.collider_set[self.ground_handle].shape().as_cuboid().unwrap().half_extents;
        (pos.x, pos.y, size.x, size. y)
    }

    pub fn net(&self) -> (f32, f32, f32, f32) {
        let pos = self.collider_set[self.net_handle].translation();
        let size = &self.collider_set[self.net_handle].shape().as_cuboid().unwrap().half_extents;
        (pos.x, pos.y, size.x, size. y)
    }

    pub fn points(&self) -> (u32, u32, bool) {
        (self.points1, self.points2, self.game_over)
    }

    pub fn step(&mut self, frame_time: f32) -> bool {
        self.time += frame_time;
        self.game_time += frame_time;
        let mut update_done = false;
        while self.time - TIME_STEP > 0.0 {
            self.frame_counter += 1;
            self.time -= TIME_STEP;

            self.control_player(self.player1_handle);
            self.control_player(self.player2_handle);

            self.physics_pipeline.step(
                &self.gravity,
                &self.integration_parameters,
                &mut self.island_manager,
                &mut self.broad_phase,
                &mut self.narrow_phase,
                &mut self.rigid_body_set,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multi_body_joint_set,
                &mut self.ccd_solver,
                Some(&mut self.query_pipeline),
                &self.physics_hooks,
                &self.event_handler,
            );

            while let Ok(event) = self.event_handler_receiver.try_recv() {
                match event {
                    CollisionEvent::Started(handle1, handle2, _) => {
                        // if [handle1, handle2].contains(&self.player1_collider_handle) || [handle1, handle2].contains(&self.player2_collider_handle) {
                        //     self.ball_touch = BallTouch::None;
                        // } else
                        if [handle1, handle2].contains(&self.player1_collider_handle) {
                            self.ball_touch = true;
                            self.rigid_body_set[self.ball_handle].set_gravity_scale(1.0,true);
                        }
                        else if [handle1, handle2].contains(&self.player2_collider_handle) {
                            self.ball_touch = false;
                            self.rigid_body_set[self.ball_handle].set_gravity_scale(1.0,true);
                        }
                        else if [handle1, handle2].contains(&self.ground_handle) && !self.points_added {
                            let add_point =
                                if self.ball().0 < self.net().0 {
                                    if self.ball_touch == false {
                                        self.points1 += 1;
                                        self.ball_for_1 = true;
                                        true
                                    }
                                    else {
                                        self.ball_touch = false;
                                        false
                                    }
                                } else {
                                    if self.ball_touch == true {
                                        self.points2 += 1;
                                        self.ball_for_1 = false;
                                        true
                                    }
                                    else {
                                        self.ball_touch = true;
                                        false
                                    }
                                };
                            if add_point {
                                self.points_added = true;
                                self.game_over = self.points1 >= POINT_LIMIT || self.points2 >= POINT_LIMIT;
                                if !self.game_over {
                                    self.reset_frame = self.frame_counter + POINT_RESET;
                                }
                            }
                        }
                    }
                    CollisionEvent::Stopped(_, _, _) => {}
                }
            }

            // if contact(&self.narrow_phase, self.ball_collider_handle, self.player1_collider_handle) ||
            //     contact(&self.narrow_phase, self.ball_collider_handle, self.player2_collider_handle) {
            //     println!("1 {:?} {}", self.ball_touch, self.frame_counter);
            //     self.ball_touch = BallTouch::None;
            // }
            //
            // if contact(&self.narrow_phase, self.ball_collider_handle, self.ground_handle) && !self.points_added {
            //     println!("200 {:?} {}", self.ball_touch, self.frame_counter);
            //     let add_point =
            //         if self.ball().0 < self.net().0 {
            //             if self.ball_touch == BallTouch::Left {
            //                 self.points1 += 1;
            //                 self.ball_for_1 = true;
            //                 true
            //             }
            //             else {
            //                 self.ball_touch = BallTouch::Left;
            //                 false
            //             }
            //         } else {
            //             if self.ball_touch == BallTouch::Right {
            //                 self.points2 += 1;
            //                 self.ball_for_1 = false;
            //                 true
            //             }
            //             else {
            //                 self.ball_touch = BallTouch::Right;
            //                 false
            //             }
            //         };
            //     if add_point {
            //         self.points_added = true;
            //         self.game_over = self.points1 >= POINT_LIMIT || self.points2 >= POINT_LIMIT;
            //         if !self.game_over {
            //             self.reset_frame = self.frame_counter + POINT_RESET;
            //         }
            //     }
            // }

            if self.frame_counter == self.reset_frame {
                self.points_added = false;
                self.ball_touch = self.ball_for_1;
                self.player_input.insert(self.player1_handle, [false, false]);
                self.player_input.insert(self.player2_handle, [false, false]);
                self.rigid_body_set[self.player1_handle].set_translation(vector![START_PLAYER_1, START_PLAYER_HEIGHT], true);
                self.rigid_body_set[self.player2_handle].set_translation(vector![START_PLAYER_2, START_PLAYER_HEIGHT], true);
                self.rigid_body_set[self.player1_handle].set_linvel(vector![0.0, 0.0], true);
                self.rigid_body_set[self.player2_handle].set_linvel(vector![0.0, 0.0], true);
                self.rigid_body_set[self.player1_handle].reset_forces(true);
                self.rigid_body_set[self.player2_handle].reset_forces(true);
                let ball = &mut self.rigid_body_set[self.ball_handle];
                ball.set_gravity_scale(0.0, true);
                let ball_x = if self.ball_for_1 {START_BALL_1} else {START_BALL_2};
                ball.set_translation(vector![ball_x, START_BALL_HEIGHT], true);
                ball.reset_forces(true);
                ball.set_linvel(vector![0.0, 0.0], true);
                ball.set_angvel(0.0, true);
                self.enable_gravity_frame = self.frame_counter + GRAVITY_AFTER;
            }
            else if self.frame_counter == self.enable_gravity_frame {
                self.rigid_body_set[self.ball_handle].set_gravity_scale(1.0,true);
                self.rigid_body_set[self.ball_handle].apply_impulse(vector![0.0, -0.1], true);
            }

            update_done = true;
        }

        update_done
    }

    fn control_player(&mut self, handle: RigidBodyHandle) {
        let player_body = &mut self.rigid_body_set[handle];
        let pressing_left = self.player_input[&handle][0];
        let pressing_right = self.player_input[&handle][1];
        player_body.set_angvel(0.0, true);
        let v = *player_body.linvel();
        let f = player_body.user_force().x;

        if pressing_right && f == 0.0 {
            player_body.add_force(vector![MOVE_FORCE, 0.0], true);
        }
        else if pressing_left && f == 0.0 {
            player_body.add_force(vector![-MOVE_FORCE, 0.0], true);
        }

        if v.x.abs() > MAX_SPEED {
            player_body.set_linvel(vector![if v.x > 0.0 { MAX_SPEED } else { -MAX_SPEED }, v.y], true);
            player_body.reset_forces(true);
        }
        if !pressing_left && !pressing_right {
            player_body.reset_forces(true);
        }

        if v.x.abs() < ALMOST_ZERO {
            player_body.set_linvel(vector![0.0, v.y], true);
        }
    }

    pub fn apply_impulse(&mut self, is_strong: bool, is_player1: bool) {
        if !is_strong {
            let (handler, coll_handler) = if is_player1 {
                (self.player1_handle, self.player1_collider_handle)
            } else {
                (self.player2_handle, self.player2_collider_handle)
            };
            if contact(&self.narrow_phase, coll_handler, self.ground_handle) {
                let body = &mut self.rigid_body_set[handler];
                body.apply_impulse(vector![0.0, 5.0], true);
            }
        }
        else {
            let body = &mut self.rigid_body_set[self.ball_handle];
            body.apply_impulse(vector![0.0, 1.0], true);
        }
    }

    pub fn add_force(&mut self, right_force: bool, is_player1: bool) {
        // let body = &mut self.rigid_body_set[self.player1_handle];
        // body.add_force(vector![MOVE_FORCE * (if right_force { 1.0 } else { -1.0 }), 0.0], true);
        let handle = if is_player1 { self.player1_handle } else { self.player2_handle };
        let [left, right] = self.player_input[&handle];
        if right_force {
            self.player_input.insert(handle, [left, true]);
        }
        else {
            self.player_input.insert(handle, [true, right]);
        }
    }

    pub fn reset_force(&mut self, right_force: bool, is_player1: bool) {
        // let body = &mut self.rigid_body_set[self.player1_handle];
        // body.reset_forces(true);
        let handle = if is_player1 {self.player1_handle} else {self.player2_handle};
        let [left, right] = self.player_input[&handle];
        if right_force {
            self.player_input.insert(handle, [left, false]);
        }
        else {
            self.player_input.insert(handle, [false, right]);
        }
    }
}

struct MyEventHandler {
    sender: Sender<CollisionEvent>
}

impl EventHandler for MyEventHandler {
    fn handle_collision_event(&self, bodies: &RigidBodySet, colliders: &ColliderSet, event: CollisionEvent, contact_pair: Option<&ContactPair>) {
        let _ = self.sender.send(event);
    }

    fn handle_contact_force_event(&self, dt: Real, bodies: &RigidBodySet, colliders: &ColliderSet, contact_pair: &ContactPair, total_force_magnitude: Real) {
        log::error!("Force event handler not implemented");
    }
}

fn contact(narrow_phase: &NarrowPhase, handle1: ColliderHandle, handle2: ColliderHandle) -> bool {
    if let Some(pair) = narrow_phase.contact_pair(handle1, handle2) {
        return pair.has_any_active_contact;
    }
    false
}
