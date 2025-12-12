use macroquad::prelude::*;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const GRAVITY: f32 = 0.5;
const JUMP_FORCE: f32 = -10.0;
const BIRD_SIZE: f32 = 30.0;
const PIPE_WIDTH: f32 = 80.0;
const PIPE_GAP: f32 = 200.0;
const PIPE_SPEED: f32 = 3.0;
const PIPE_SPAWN_INTERVAL: f32 = 2.0;

#[derive(PartialEq)]
enum GameState {
    Menu,
    Playing,
    GameOver,
}

struct Bird {
    x: f32,
    y: f32,
    velocity: f32,
}

impl Bird {
    fn new() -> Self {
        Bird {
            x: 150.0,
            y: WINDOW_HEIGHT / 2.0,
            velocity: 0.0,
        }
    }

    fn update(&mut self) {
        self.velocity += GRAVITY;
        self.y += self.velocity;
    }

    fn jump(&mut self) {
        self.velocity = JUMP_FORCE;
    }

    fn draw(&self) {
        draw_circle(self.x, self.y, BIRD_SIZE / 2.0, YELLOW);
        draw_circle(self.x + 8.0, self.y - 5.0, 5.0, BLACK);
    }

    fn get_bounds(&self) -> (f32, f32, f32, f32) {
        (
            self.x - BIRD_SIZE / 2.0,
            self.y - BIRD_SIZE / 2.0,
            BIRD_SIZE,
            BIRD_SIZE,
        )
    }
}

struct Pipe {
    x: f32,
    gap_y: f32,
    passed: bool,
}

impl Pipe {
    fn new(x: f32) -> Self {
        let gap_y = rand::gen_range(150.0, WINDOW_HEIGHT - 150.0);
        Pipe {
            x,
            gap_y,
            passed: false,
        }
    }

    fn update(&mut self) {
        self.x -= PIPE_SPEED;
    }

    fn draw(&self) {
        draw_rectangle(
            self.x,
            0.0,
            PIPE_WIDTH,
            self.gap_y - PIPE_GAP / 2.0,
            GREEN,
        );
        draw_rectangle(
            self.x,
            self.gap_y + PIPE_GAP / 2.0,
            PIPE_WIDTH,
            WINDOW_HEIGHT - (self.gap_y + PIPE_GAP / 2.0),
            GREEN,
        );

        draw_rectangle(
            self.x - 5.0,
            self.gap_y - PIPE_GAP / 2.0 - 20.0,
            PIPE_WIDTH + 10.0,
            20.0,
            DARKGREEN,
        );
        draw_rectangle(
            self.x - 5.0,
            self.gap_y + PIPE_GAP / 2.0,
            PIPE_WIDTH + 10.0,
            20.0,
            DARKGREEN,
        );
    }

    fn is_off_screen(&self) -> bool {
        self.x + PIPE_WIDTH < 0.0
    }

    fn collides_with_bird(&self, bird: &Bird) -> bool {
        let (bird_x, bird_y, bird_w, bird_h) = bird.get_bounds();

        if bird_x + bird_w > self.x && bird_x < self.x + PIPE_WIDTH {
            if bird_y < self.gap_y - PIPE_GAP / 2.0 || bird_y + bird_h > self.gap_y + PIPE_GAP / 2.0 {
                return true;
            }
        }
        false
    }
}

struct Game {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: i32,
    state: GameState,
    time_since_last_pipe: f32,
}

impl Game {
    fn new() -> Self {
        Game {
            bird: Bird::new(),
            pipes: vec![Pipe::new(WINDOW_WIDTH)],
            score: 0,
            state: GameState::Menu,
            time_since_last_pipe: 0.0,
        }
    }

    fn reset(&mut self) {
        self.bird = Bird::new();
        self.pipes = vec![Pipe::new(WINDOW_WIDTH)];
        self.score = 0;
        self.time_since_last_pipe = 0.0;
    }

    fn update(&mut self, delta_time: f32) {
        if self.state != GameState::Playing {
            return;
        }

        self.bird.update();

        if self.bird.y > WINDOW_HEIGHT || self.bird.y < 0.0 {
            self.state = GameState::GameOver;
            return;
        }

        for pipe in &mut self.pipes {
            pipe.update();

            if pipe.collides_with_bird(&self.bird) {
                self.state = GameState::GameOver;
                return;
            }

            if !pipe.passed && pipe.x + PIPE_WIDTH < self.bird.x {
                pipe.passed = true;
                self.score += 1;
            }
        }

        self.pipes.retain(|pipe| !pipe.is_off_screen());

        self.time_since_last_pipe += delta_time;
        if self.time_since_last_pipe >= PIPE_SPAWN_INTERVAL {
            self.pipes.push(Pipe::new(WINDOW_WIDTH));
            self.time_since_last_pipe = 0.0;
        }
    }

    fn draw(&self) {
        clear_background(SKYBLUE);

        for pipe in &self.pipes {
            pipe.draw();
        }

        self.bird.draw();

        let score_text = format!("Score: {}", self.score);
        draw_text(&score_text, 20.0, 40.0, 40.0, WHITE);

        match self.state {
            GameState::Menu => {
                draw_text(
                    "FLAPPY BIRD",
                    WINDOW_WIDTH / 2.0 - 150.0,
                    WINDOW_HEIGHT / 2.0 - 60.0,
                    60.0,
                    WHITE,
                );
                draw_text(
                    "Press SPACE to start",
                    WINDOW_WIDTH / 2.0 - 140.0,
                    WINDOW_HEIGHT / 2.0 + 20.0,
                    30.0,
                    WHITE,
                );
                draw_text(
                    "Press SPACE to jump",
                    WINDOW_WIDTH / 2.0 - 140.0,
                    WINDOW_HEIGHT / 2.0 + 60.0,
                    25.0,
                    LIGHTGRAY,
                );
            }
            GameState::GameOver => {
                draw_text(
                    "GAME OVER",
                    WINDOW_WIDTH / 2.0 - 120.0,
                    WINDOW_HEIGHT / 2.0 - 60.0,
                    60.0,
                    RED,
                );
                let final_score = format!("Final Score: {}", self.score);
                draw_text(
                    &final_score,
                    WINDOW_WIDTH / 2.0 - 100.0,
                    WINDOW_HEIGHT / 2.0 + 20.0,
                    35.0,
                    WHITE,
                );
                draw_text(
                    "Press SPACE to restart",
                    WINDOW_WIDTH / 2.0 - 140.0,
                    WINDOW_HEIGHT / 2.0 + 70.0,
                    30.0,
                    WHITE,
                );
            }
            GameState::Playing => {}
        }
    }

    fn handle_input(&mut self) {
        if is_key_pressed(KeyCode::Space) {
            match self.state {
                GameState::Menu => {
                    self.state = GameState::Playing;
                }
                GameState::Playing => {
                    self.bird.jump();
                }
                GameState::GameOver => {
                    self.reset();
                    self.state = GameState::Playing;
                }
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Flappy Bird".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    let mut last_time = get_time();

    loop {
        let current_time = get_time();
        let delta_time = (current_time - last_time) as f32;
        last_time = current_time;

        game.handle_input();
        game.update(delta_time);
        game.draw();

        next_frame().await;
    }
}
