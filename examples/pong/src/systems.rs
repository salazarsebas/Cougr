use crate::components::{BallComponent, ECSWorldState, GameState};
#[cfg(test)]
use cougr_core::{GameApp, ScheduleStage, SimpleWorld, SystemConfig};
#[cfg(test)]
use soroban_sdk::Env;

pub(crate) const PADDLE_HEIGHT: i32 = 15;
pub(crate) const PADDLE_SPEED: i32 = 2;
pub(crate) const BALL_SPEED: i32 = 1;
pub(crate) const FIELD_WIDTH: i32 = 100;
pub(crate) const FIELD_HEIGHT: i32 = 60;
pub(crate) const WINNING_SCORE: u32 = 5;

pub(crate) fn physics_system(world: &mut ECSWorldState) {
    world.ball.x += world.ball.vx;
    world.ball.y += world.ball.vy;
}

pub(crate) fn collision_system(world: &mut ECSWorldState) {
    let paddle_half_height = PADDLE_HEIGHT / 2;

    if world.ball.y <= 0 || world.ball.y >= FIELD_HEIGHT {
        world.ball.vy = -world.ball.vy;
        world.ball.y = world.ball.y.clamp(0, FIELD_HEIGHT);
    }

    if world.ball.x <= 5 && world.ball.vx < 0 {
        let paddle_top = world.player1_paddle.y_position - paddle_half_height;
        let paddle_bottom = world.player1_paddle.y_position + paddle_half_height;
        if world.ball.y >= paddle_top && world.ball.y <= paddle_bottom {
            world.ball.vx = -world.ball.vx;
            world.ball.x = 5;
        }
    }

    if world.ball.x >= FIELD_WIDTH - 5 && world.ball.vx > 0 {
        let paddle_top = world.player2_paddle.y_position - paddle_half_height;
        let paddle_bottom = world.player2_paddle.y_position + paddle_half_height;
        if world.ball.y >= paddle_top && world.ball.y <= paddle_bottom {
            world.ball.vx = -world.ball.vx;
            world.ball.x = FIELD_WIDTH - 5;
        }
    }
}

pub(crate) fn scoring_system(world: &mut ECSWorldState) {
    if world.ball.x <= 0 {
        world.score.player2_score += 1;
        reset_ball(&mut world.ball);
    } else if world.ball.x >= FIELD_WIDTH {
        world.score.player1_score += 1;
        reset_ball(&mut world.ball);
    }

    if world.score.player1_score >= WINNING_SCORE || world.score.player2_score >= WINNING_SCORE {
        world.score.game_active = false;
    }
}

pub(crate) fn clamp_paddle_position(y: i32) -> i32 {
    let paddle_half_height = PADDLE_HEIGHT / 2;
    y.clamp(paddle_half_height, FIELD_HEIGHT - paddle_half_height)
}

pub(crate) fn reset_ball(ball: &mut BallComponent) {
    ball.x = FIELD_WIDTH / 2;
    ball.y = FIELD_HEIGHT / 2;
    ball.vx = -ball.vx;
}

pub(crate) fn world_to_game_state(world: &ECSWorldState) -> GameState {
    GameState {
        player1_paddle_y: world.player1_paddle.y_position,
        player2_paddle_y: world.player2_paddle.y_position,
        ball_x: world.ball.x,
        ball_y: world.ball.y,
        ball_vx: world.ball.vx,
        ball_vy: world.ball.vy,
        player1_score: world.score.player1_score,
        player2_score: world.score.player2_score,
        game_active: world.score.game_active,
    }
}

#[cfg(test)]
pub(crate) fn run_gameapp_tick(env: &Env) {
    let mut app = GameApp::new(env);
    app.add_system_with_config(
        "pong_tick_boundary",
        |_world: &mut SimpleWorld, _env: &Env| {},
        SystemConfig::new().in_stage(ScheduleStage::Update),
    );
    app.run(env).unwrap();
}
