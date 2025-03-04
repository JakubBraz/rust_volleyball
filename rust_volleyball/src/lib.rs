use rapier2d::na::SMatrix;
use rapier2d::prelude::*;

pub fn fn1(a: i32, b: i32) -> i32 {
    a + b
}

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
    event_handler: (),

    player1_handle: RigidBodyHandle,
    ground_handle: ColliderHandle,
}

impl GameState {
    pub fn new() -> GameState {
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
            event_handler: (),

            player1_handle: RigidBodyHandle::default(),
            ground_handle: ColliderHandle::default(),
        };

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(10.0, 0.1).build();
        game_state.ground_handle = game_state.collider_set.insert(collider);

        /* Create the bouncing ball. */
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 10.0])
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        game_state.player1_handle = game_state.rigid_body_set.insert(rigid_body);
        game_state.collider_set.insert_with_parent(collider, game_state.player1_handle, &mut game_state.rigid_body_set);

        game_state
    }

    pub fn player1(&self) -> (f32, f32, f32) {
        let body = &self.rigid_body_set[self.player1_handle];
        let r = self.collider_set[body.colliders()[0]].shape().as_ball().unwrap().radius;
        (body.translation().x, body.translation().y, r)
    }

    pub fn ground(&self) -> (f32, f32, f32, f32) {
        let size = &self.collider_set[self.ground_handle].shape().as_cuboid().unwrap().half_extents;
        let pos = self.collider_set[self.ground_handle].translation();
        (pos.x, pos.y, size.x, size. y)
    }

    pub fn step(&mut self) {
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
        )
    }

    pub fn apply_impulse(&mut self, is_strong: bool) {
        let body = &mut self.rigid_body_set[self.player1_handle];
        body.apply_impulse(vector![0.0, if is_strong { 10.0 } else { 5.0 }], true);
    }

    pub fn add_force(&mut self, right_force: bool) {
        let body = &mut self.rigid_body_set[self.player1_handle];
        body.add_force(vector![1.0 * (if right_force { 1.0 } else { -1.0 }), 0.0], true);
    }

    pub fn reset_force(&mut self) {
        let body = &mut self.rigid_body_set[self.player1_handle];
        body.reset_forces(true);
    }
}
