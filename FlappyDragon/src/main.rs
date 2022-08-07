use bracket_lib::prelude::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;


struct Obstacle{
    x: i32,    // x value position of the gap
    gap_y: i32, // center of the y value of the gap
    size: i32,  // size of the gap, it's length
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle{
            x, 
            gap_y: random.range(10, 40), // random values between 10 and 39
            size: i32::max(2, 20-score), // max of these 2 values is the gap size, walls close in as the player progresses
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x; // transform world space to screen space, player is always 0 at the screen space, but player_x in world space
        let half_size = self.size / 2;

        // Drawing of top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(
                screen_x, 
                y, 
                RED, 
                BLACK, 
                to_cp437('|'),
            );
        }

        // Draw the bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(
                screen_x, 
                y, 
                RED, 
                BLACK, 
                to_cp437('|'),
            );
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool { // no need for mut self, since this method is const, meaning does not mutate it's members
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x; // player's x coordinate and obstacle's x coordinate
        let is_player_above_gap = player.y < self.gap_y - half_size;
        let is_player_below_gap = player.y > self.gap_y + half_size;
        does_x_match && (is_player_above_gap || is_player_below_gap)
    }
}

struct Player{ // needs to be part of the Game State
    x: i32, 
    y: i32, 
    velocity: f32,
}

impl Player {
    fn new(x: i32, y:i32) -> Self {
        Player {
            x, 
            y, 
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set( // sets a single character on the screen
            0, // x coordinate on which to render the character
            self.y, // y coordinate on which to render character
            YELLOW, // RGB::from_u8() or RGB::from_hex() can be also used to represent HTML colors
            BLACK, 
            to_cp437('@') //char to render, convert it to Codepage-437 unicode
        );
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 { // check for terminal velocity, apply gravity only if less than 2.0
            self.velocity += 0.2; // apply gravity
        }
        self.y += self.velocity as i32; // add the velocity to y position
        self.x += 1;
        if self.y < 0 {
            self.y = 0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0; // will move player upward
    }
}

struct State{
    player: Player, 
    frame_time: f32, // tracks the time between frames to control the game speed
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> State {
        State{
            player: Player::new(5, 25),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY); // specify backgroung color
        self.frame_time += ctx.frame_time_ms; // time elapsed since the last tick called, not to be too fast
        if self.frame_time > FRAME_DURATION { // only when it reaches this variable do the physics and move
            self.frame_time = 0.0; // reset the counter of the frame_time for the next one
            self.player.gravity_and_move();
        }

        // it is not restircted to frame time, otherwise keybaord will be
        // unresponsive during wait frames.
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);
        ctx.print(0, 0, "Press Space to flap.");

        ctx.print(0, 1, &format!("Score: {}", self.score));
        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score); // make a new obstacle at the end of a screen (x-coord)
        }

        // if player has fallen to the bottom of the screen
        if self.player.y > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, 25);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            // the value of the key is there
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,  // instructs bracket lib we are ready to terminate our program
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned: {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(), 
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

enum GameMode {// <callout id="co.flappy_state.enum" />
    Menu,// <callout id="co.flappy_state.enums" />
    Playing,
    End,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}


fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Dragon")
        .build()?;

    // link the engine with our State so that the bracket-lib
    // knows where the tick() function is located.
    main_loop(context, State::new())
}
