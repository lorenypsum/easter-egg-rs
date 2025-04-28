// These lines disable certain warnings from Clippy, a Rust linter.
// Useful for focusing on core logic, but good to address these in larger projects.
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::manual_range_contains,
    clippy::too_many_lines
)]
use std::iter::once;

use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, PlaySoundParams, Sound,
};
use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::prelude::*;
use macroquad::rand::gen_range; // For generating random numbers and choices

// --- Physics Constants ---
// Defines how quickly objects fall downwards (pixels per second squared).
const GRAVITY: f32 = 1000.0;
// A small buffer zone below the player to detect ground slightly before touching.
const GROUND_DETECTION_BUFFER: f32 = 5.0;
// A small margin subtracted from entity bounds for collision checks, can help prevent sticking.
const COLLISION_MARGIN: f32 = 2.0;

// --- Player Constants ---
// The starting position (x, y) of the player character on the screen.
const PLAYER_START_POS: Vec2 = Vec2::new(243.0, 350.0);
// How fast the player moves horizontally (pixels per second).
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;
// The initial upward speed when the player jumps (pixels per second).
const PLAYER_JUMP_SPEED: f32 = 500.0;

// --- Entity Sizes ---
// Dimensions (width, height) for various game objects.
// Calculated by multiplying original pixel art size by a scaling factor.
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0); // Original: 10x16 pixels, Scaled by: 3.0
const PLATFORM_SIZE: Vec2 = Vec2::new(429.0, 141.0); // Original: 143x47 pixels, Scaled by: 3.0
const PLATFORM_BAR_SIZE: Vec2 = Vec2::new(214.5, 70.5); // Original: 143x47 pixels, Scaled by: 1.5
const CHICKEN_SIZE: Vec2 = Vec2::new(52.0, 48.0); // Original: 13x12 pixels, Scaled by: 4.0
const EGG_SIZE: Vec2 = Vec2::new(40.0, 40.0); // Original: 400x400 pixels, Scaled by: 0.1
const SPIKE_SIZE: Vec2 = Vec2::new(60.0, 52.0); // Original: 15x13 pixels, Scaled by: 4.0
const HOUSE_SIZE: Vec2 = Vec2::new(423.0, 624.0); // Original: 141x208 pixels, Scaled by: 3.0
const CLOUD_SIZE: Vec2 = Vec2::new(786.0, 150.0); // Original: 262x50 pixels, Scaled by: 3.0
const BACKGROUND_SIZE: Vec2 = Vec2::new(1024.0, 2304.0); // Original: 1024x2304 pixels, Scaled by: 1.0

// --- Game Goal Constants ---
// How many eggs the player needs to collect to trigger the "End" state (reaching the house).
const EGGS_NEEDED_FOR_HOUSE: u32 = 2;
// How many eggs the player needs to collect to trigger the "Win" state (reaching the house with enough eggs).
const EGGS_NEEDED_FOR_WIN: u32 = 5;

// --- Visual Constants ---
// The background color of the game window (a light beige).
const BACKGROUND_COLOR: Color = Color {
    r: 0.92, // Red component (0.0 to 1.0)
    g: 0.88, // Green component (0.0 to 1.0)
    b: 0.78, // Blue component (0.0 to 1.0)
    a: 1.0,  // Alpha (transparency) component (1.0 is fully opaque)
};

/// Represents a basic game object with a position and size (a rectangle).
struct GameEntity {
    /// The rectangle defining the entity's position (x, y) and dimensions (w, h).
    rect: Rect,
}

impl GameEntity {
    /// Calculates the collision bounding box, slightly smaller than the visual rectangle.
    /// This uses `COLLISION_MARGIN` to prevent overly sensitive collisions.
    fn get_collision_bounds(&self) -> Rect {
        Rect {
            x: self.rect.x + COLLISION_MARGIN,       // Move right edge inwards
            y: self.rect.y + COLLISION_MARGIN,       // Move top edge downwards
            w: self.rect.w - COLLISION_MARGIN * 2.0, // Reduce width
            h: self.rect.h - COLLISION_MARGIN * 2.0, // Reduce height
        }
    }
}

/// Represents a game entity that can move.
/// Contains a `GameEntity` for position/size and a `velocity` vector.
struct MovingGameEntity {
    /// The underlying entity with position and size.
    entity: GameEntity,
    /// The speed and direction of movement (pixels per second).
    velocity: Vec2,
}

impl MovingGameEntity {
    /// Updates the entity's position based on its velocity and the time elapsed since the last frame.
    /// `delta_time`: The time in seconds since the last frame update.
    fn apply_velocity(&mut self, delta_time: f32) {
        // Update position: position = position + velocity * time
        self.entity.rect.x += self.velocity.x * delta_time;
        self.entity.rect.y += self.velocity.y * delta_time;
    }
}

/// Represents the direction the player is currently facing. Used for drawing the correct sprite.
enum MoveDirection {
    Left,
    Right,
}

/// Represents the different reasons why the game might end.
enum DeathCause {
    Chicken,
    Spike,
    Fall,
}

/// Represents the different reasons why the game might end.
enum GameOverReason {
    /// Player died (hit enemy, spike, fell off screen). Includes the final score.
    Death { cause: DeathCause, score: u32 },
    /// Player reached the house but didn't have enough eggs to win.
    End { meme: usize },
    /// Player reached the house with enough eggs.
    Win,
}

enum Event {
    Jumped,
    Scored,
    GameOver(GameOverReason),
}

enum State {
    Start,
    Game {
        player: MovingGameEntity,
        player_direction: MoveDirection,
        score: u32,
        clouds: Vec<MovingGameEntity>,
        platforms: Vec<GameEntity>,
        eggs: Vec<GameEntity>,
        chickens: Vec<MovingGameEntity>,
        spikes: Vec<GameEntity>,
        house: GameEntity,
        background_entities: Vec<GameEntity>,
    },
    GameOver(GameOverReason),
}

impl State {
    fn new_game(&mut self) {
        // Create the player character as a moving entity.
        let player = MovingGameEntity {
            entity: GameEntity {
                rect: Rect {
                    // Center the player horizontally at the start position
                    x: PLAYER_START_POS.x - PLAYER_SIZE.x / 2.0,
                    // Center the player vertically at the start position
                    y: PLAYER_START_POS.y - PLAYER_SIZE.y / 2.0,
                    w: PLAYER_SIZE.x, // Use predefined player width
                    h: PLAYER_SIZE.y, // Use predefined player height
                },
            },
            velocity: Vec2::ZERO, // Start with no initial movement
        };
        // Create background images. They are placed side-by-side to create a long scrolling background.
        // `(0..=60)` creates a range from 0 to 60 (inclusive).
        // `.map()` transforms each number `i` in the range into a `GameEntity`.
        // `.collect()` gathers the results into a `Vec<GameEntity>`.
        let background_entities: Vec<GameEntity> = (0..=60)
            .map(|i| {
                // Calculate the x position for each background segment.
                let x = -1024.0 + i as f32 * 1024.0;
                GameEntity {
                    rect: Rect {
                        x: x - BACKGROUND_SIZE.x / 2.0,     // Center the background image
                        y: 336.0 - BACKGROUND_SIZE.y / 2.0, // Position vertically
                        w: BACKGROUND_SIZE.x,
                        h: BACKGROUND_SIZE.y,
                    },
                }
            })
            .collect();

        // Create clouds with random positions and horizontal movement speeds.
        let clouds: Vec<MovingGameEntity> = (0..=40)
            .map(|i| {
                // Distribute clouds horizontally.
                let x = -1024.0 + 500.0 * i as f32;
                // Place clouds at random heights.
                let y = gen_range(100.0, 500.0);
                MovingGameEntity {
                    entity: GameEntity {
                        rect: Rect {
                            x: x - CLOUD_SIZE.x / 2.0, // Center the cloud image
                            y: y - CLOUD_SIZE.y / 2.0, // Center the cloud image
                            w: CLOUD_SIZE.x,
                            h: CLOUD_SIZE.y,
                        },
                    },
                    // Give each cloud a random horizontal speed.
                    velocity: Vec2::new(gen_range(20.0, 60.0), 0.0), // No vertical velocity
                }
            })
            .collect();

        // Create platforms. Includes ground platforms and floating platforms.
        let platforms: Vec<GameEntity> = (-429..=2000) // Range for ground platform positions
            .step_by(400) // Place ground platforms 400 units apart
            .map(|x| GameEntity {
                // Create ground platforms
                rect: Rect {
                    x: x as f32 - PLATFORM_SIZE.x / 2.0,  // Center horizontally
                    y: screen_height() - PLATFORM_SIZE.y, // Place at the bottom of the screen
                    w: PLATFORM_SIZE.x,
                    h: PLATFORM_SIZE.y,
                },
            })
            // `.chain()` combines the ground platforms with the floating platforms.
            .chain((0..60).map(|i| {
                // Create 60 floating platforms
                // Calculate x position with some randomness.
                let x = i as f32 * 50.0 + gen_range(-200.0, 200.0);
                // Place at random heights within a range.
                let y = gen_range(150.0, 650.0);
                GameEntity {
                    // Use the smaller platform bar size
                    rect: Rect {
                        x: x - PLATFORM_BAR_SIZE.x / 2.0, // Center horizontally
                        y: y - PLATFORM_BAR_SIZE.y / 2.0, // Center vertically
                        w: PLATFORM_BAR_SIZE.x,
                        h: PLATFORM_BAR_SIZE.y,
                    },
                }
            }))
            .collect(); // Collect all platforms into a single Vec

        // Create eggs, placing them on top of some existing platforms.
        let eggs: Vec<GameEntity> = platforms
            .iter() // Iterate over the platforms
            .filter(|_| gen_range(0, 100) < 30) // Keep only about 30% of platforms to spawn an egg on
            .enumerate() // Get both the index (i) and the platform
            .map(|(i, platform)| {
                // Create an egg for each selected platform
                // Calculate a horizontal offset to spread eggs across the platform width
                let offset = (i as f32 - 0.5) * (platform.rect.w * 0.5);
                let x = platform.rect.center().x + offset; // Position egg horizontally on platform
                let y = platform.rect.y - EGG_SIZE.y + 5.0; // Position egg just above the platform surface

                GameEntity {
                    rect: Rect {
                        x: x - EGG_SIZE.x / 2.0, // Center the egg horizontally
                        y,                       // Use the calculated y position
                        w: EGG_SIZE.x,
                        h: EGG_SIZE.y,
                    },
                }
            })
            .collect(); // Collect the created eggs into a Vec

        // Create flying chickens with random starting positions and velocities.
        let chickens: Vec<MovingGameEntity> = (0..20) // Create 20 chickens
            .map(|_| {
                // The `_` means we don't need the loop counter value
                // Random horizontal position within a wide range of the game world.
                let x = gen_range(500.0, 4000.0);
                // Random vertical position within the typical play area.
                let y = gen_range(100.0, 600.0);

                // Random horizontal speed, can be left or right.
                let vx = gen_range(50.0, 150.0) * (if gen_range(0, 2) == 0 { 1.0 } else { -1.0 });
                // Random vertical speed, can be up or down.
                let vy = gen_range(30.0, 80.0) * (if gen_range(0, 2) == 0 { 1.0 } else { -1.0 });

                MovingGameEntity {
                    entity: GameEntity {
                        rect: Rect {
                            x: x - CHICKEN_SIZE.x / 2.0, // Center horizontally
                            y: y - CHICKEN_SIZE.y / 2.0, // Center vertically
                            w: CHICKEN_SIZE.x,
                            h: CHICKEN_SIZE.y,
                        },
                    },
                    velocity: Vec2::new(vx, vy), // Set the random velocity
                }
            })
            .collect();

        // Create spikes, placing them on top of some ground platforms.
        let spikes: Vec<GameEntity> = platforms
            .iter() // Iterate over platforms
            .filter(|platform| {
                // Select only ground platforms (check if their center is near the bottom)
                platform.rect.center().y > screen_height() - PLATFORM_SIZE.y
                // And only place spikes randomly (1 in 5 chance for selected platforms)
                && gen_range(0, 5) == 0
            })
            .map(|platform| GameEntity {
                // Create a spike for each selected platform
                rect: Rect {
                    // Position spike towards the right edge of the platform
                    x: platform.rect.right() - SPIKE_SIZE.x / 2.0,
                    // Position spike just above the platform surface
                    y: platform.rect.y - SPIKE_SIZE.y + 5.0,
                    w: SPIKE_SIZE.x,
                    h: SPIKE_SIZE.y,
                },
            })
            .collect();

        // Create the final house structure (the end goal).
        let house = GameEntity {
            rect: Rect {
                x: 3000.0 - HOUSE_SIZE.x / 2.0, // Position horizontally far into the level
                y: 292.0 - HOUSE_SIZE.y / 2.0,  // Position vertically
                w: HOUSE_SIZE.x,
                h: HOUSE_SIZE.y,
            },
        };

        *self = Self::Game {
            player,
            player_direction: MoveDirection::Right,
            score: 0,
            clouds,
            platforms,
            eggs,
            chickens,
            spikes,
            house,
            background_entities,
        };
    }

    fn process_input(&mut self) -> Vec<Event> {
        let mut events: Vec<Event> = vec![];
        match self {
            State::Start => {
                if is_key_pressed(KeyCode::P) {
                    self.new_game();
                }
            }
            State::Game {
                player,
                player_direction,
                ..
            } => {
                // Check left/right movement keys. `is_key_down` checks if held.
                match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
                    (true, false) => {
                        // Left key is down, Right key is up
                        *player_direction = MoveDirection::Left; // Set facing direction
                        player.velocity.x = -PLAYER_MOVEMENT_SPEED; // Set horizontal velocity leftwards
                    }
                    (false, true) => {
                        // Left key is up, Right key is down
                        *player_direction = MoveDirection::Right; // Set facing direction
                        player.velocity.x = PLAYER_MOVEMENT_SPEED; // Set horizontal velocity rightwards
                    }
                    _ => {
                        // Neither or both keys are pressed
                        player.velocity.x = 0.0; // Stop horizontal movement
                    }
                };
                // Check jump key. `is_key_pressed` checks if pressed *this frame*.
                // `player.velocity.y == 0.0` checks if the player is on the ground (or apex of jump).
                if is_key_pressed(KeyCode::Up) && player.velocity.y == 0.0 {
                    player.velocity.y = -PLAYER_JUMP_SPEED; // Set vertical velocity upwards (jump)
                    events.push(Event::Jumped);
                }
            }
            State::GameOver(_) => {
                // Handle input for the game over screen.
                if is_key_pressed(KeyCode::R) {
                    self.new_game();
                }
            }
        }
        events
    }

    fn update(&mut self, delta_time: f32) -> Vec<Event> {
        let mut events: Vec<Event> = vec![];
        let State::Game {
            player,
            score,
            clouds,
            platforms,
            eggs,
            chickens,
            spikes,
            house,
            ..
        } = self
        else {
            return events;
        };
        // --- Update Game State (Physics and Movement) ---
        {
            // Apply gravity to the player's vertical velocity.
            player.velocity.y += GRAVITY * delta_time;

            // --- Platform Collision Detection (Ground Check) ---
            // Find the first platform the player might land on.
            let ground_collision = platforms.iter().find_map(|platform| {
                // Check if player's horizontal range overlaps with the platform's horizontal range.
                let horizontally_overlapping = player.entity.rect.right() > platform.rect.x
                    && player.entity.rect.x < platform.rect.right();

                // Check if player is moving downwards or is stationary vertically.
                let falling_towards_platform = player.velocity.y >= 0.0;
                // Check if the player's bottom is slightly above or at the platform's top.
                let close_to_platform_top =
                    player.entity.rect.bottom() <= platform.rect.y + GROUND_DETECTION_BUFFER;
                // Predict if the player *will* be below the platform top in the next frame.
                let will_intersect_next_frame =
                    player.entity.rect.bottom() + player.velocity.y * delta_time >= platform.rect.y;

                // If all conditions are met, the player is about to land on this platform.
                if horizontally_overlapping
                    && falling_towards_platform
                    && close_to_platform_top
                    && will_intersect_next_frame
                {
                    // Return the Y-coordinate of the platform's top surface.
                    Some(platform.rect.y)
                } else {
                    // Otherwise, no collision with this platform.
                    None
                }
            });

            // Update player position based on velocity.
            player.apply_velocity(delta_time);

            // --- Handle Ground Collision Response ---
            // If `ground_collision` found a platform (`Some(platform_top)`)...
            if let Some(platform_top) = ground_collision {
                // Snap the player's bottom edge to the top of the platform.
                player.entity.rect.y = platform_top - player.entity.rect.h;
                // Stop vertical movement.
                player.velocity.y = 0.0;
            }

            // --- Update Chicken Movement ---
            for chicken in chickens.iter_mut() {
                // Apply velocity to update position.
                chicken.apply_velocity(delta_time);
                // Simple boundary check: reverse horizontal velocity if chicken hits world edges.
                if chicken.entity.rect.x > 5000.0 || chicken.entity.rect.x < 0.0 {
                    chicken.velocity.x = -chicken.velocity.x;
                }
                // Simple boundary check: reverse vertical velocity if chicken hits vertical limits.
                if chicken.entity.rect.y > 800.0 || chicken.entity.rect.y < 0.0 {
                    chicken.velocity.y = -chicken.velocity.y;
                }
            }

            // --- Update Cloud Movement ---
            for cloud in clouds {
                // Apply velocity to update position.
                cloud.apply_velocity(delta_time);
                // If cloud moves too far right, wrap it around to the left side.
                if cloud.entity.rect.x > 60000.0 {
                    // Use a large boundary for wrapping
                    cloud.entity.rect.x = -1024.0; // Reset position far left
                }
            }
        }

        // --- Check Collisions and Game Logic ---
        {
            // --- Check Player Falling Off Screen ---
            // If player falls too far below the screen...
            if player.entity.rect.bottom() > screen_height() + 100.0 {
                // End the game due to death.
                events.push(Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Fall,
                    score: *score,
                }));
                *self = State::GameOver(GameOverReason::Death {
                    cause: DeathCause::Fall,
                    score: *score,
                });
                return events;
            }

            // --- Egg Collection ---
            // `retain` keeps only the elements for which the closure returns true.
            eggs.retain(|egg| {
                // Check if the player's collision bounds overlap with the egg's bounds.
                let collided = player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&egg.get_collision_bounds());
                if collided {
                    *score += 1; // Increase score
                    events.push(Event::Scored);
                }
                // Return `!collided`: keep the egg if NOT collided, remove it if collided.
                !collided
            });

            // --- Chicken Collision ---
            // Check if the player collides with any chicken.
            if chickens.iter().any(|chicken| {
                // `any` returns true if the closure is true for at least one element
                player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&chicken.entity.get_collision_bounds())
            }) {
                events.push(Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Chicken,
                    score: *score,
                }));
                *self = State::GameOver(GameOverReason::Death {
                    cause: DeathCause::Chicken,
                    score: *score,
                });
                return events;
            }

            // --- Spike Collision ---
            // Check if the player collides with any spike.
            if spikes.iter().any(|spike| {
                player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&spike.get_collision_bounds())
            }) {
                events.push(Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Spike,
                    score: *score,
                }));
                *self = State::GameOver(GameOverReason::Death {
                    cause: DeathCause::Spike,
                    score: *score,
                });
                return events;
            }

            // --- House Collision (End/Win Condition) ---
            // Check if the player collides with the house.
            if player
                .entity
                .get_collision_bounds()
                .overlaps(&house.get_collision_bounds())
            {
                // Check if the player has enough eggs to win.
                if *score >= EGGS_NEEDED_FOR_WIN {
                    events.push(Event::GameOver(GameOverReason::Win));
                    *self = State::GameOver(GameOverReason::Win);
                } else if *score >= EGGS_NEEDED_FOR_HOUSE {
                    // Player reached the house but needs more eggs.
                    let meme = gen_range(0, 8);
                    events.push(Event::GameOver(GameOverReason::End { meme }));
                    *self = State::GameOver(GameOverReason::End { meme });
                }
                // If player has fewer eggs than needed for the house, nothing happens yet.
            }
        }
        events
    }
}

/// Holds all the textures (images) and sounds used in the game.
/// Loading these upfront helps prevent lag during gameplay.
struct Assets {
    // Player textures
    player_right: Texture2D,
    player_left: Texture2D,
    // Object textures
    platform: Texture2D,
    chicken: Texture2D,
    spike: Texture2D,
    egg: Texture2D,
    // UI / Screen textures
    game_over: Texture2D,
    win: Texture2D,
    game_start: Texture2D,
    score_panel: Texture2D,
    // Environment textures
    cloud: Texture2D,
    house: Texture2D, // The end goal structure
    background: Texture2D,
    // Fun extras
    meme_textures: [Texture2D; 8], // An array to hold multiple meme images
    // Sound effects
    jump: Sound,
    egg_collect: Sound,
    chicken_hit: Sound,
    spike_hit: Sound,
    magic: Sound, // Sound for reaching the house without enough eggs
    // Music
    background_music: Sound,
    game_over_sound: Sound,
    win_sound: Sound,
}

/// Loads a PNG image from embedded byte data into a Macroquad texture.
/// This allows including images directly in the executable.
/// `bytes`: A slice of bytes representing the PNG file data.
fn load_png_texture_from_bytes(bytes: &[u8]) -> Texture2D {
    // Load the texture from the raw byte data. `None` means Macroquad tries to auto-detect the format.
    let texture = Texture2D::from_file_with_format(bytes, None);
    // Set the texture filtering mode to Nearest. This prevents blurring in pixel art.
    texture.set_filter(FilterMode::Nearest);
    texture // Return the loaded texture
}

/// Asynchronously loads all game assets (textures and sounds).
/// Displays a simple "Loading..." message while assets are being loaded.
/// `async fn` means this function can perform operations (like file loading)
/// without blocking the main thread, important for responsiveness.
async fn load_assets() -> Assets {
    // Load all textures using the custom loader function.
    // `include_bytes!` embeds the file content directly into the compiled program.
    Assets {
        player_right: load_png_texture_from_bytes(include_bytes!(
            "../assets/character/c_right.png"
        )),
        player_left: load_png_texture_from_bytes(include_bytes!("../assets/character/c_left.png")),
        platform: load_png_texture_from_bytes(include_bytes!("../assets/platforms/platform.png")),
        chicken: load_png_texture_from_bytes(include_bytes!(
            "../assets/chickens/chicken_fly_1.png"
        )),
        spike: load_png_texture_from_bytes(include_bytes!("../assets/spikes/spike_1.png")),
        egg: load_png_texture_from_bytes(include_bytes!("../assets/eggs/easter_egg_1.png")),
        // Game state screens
        game_over: load_png_texture_from_bytes(include_bytes!("../assets/gui/game_over_cesta.png")),
        win: load_png_texture_from_bytes(include_bytes!("../assets/gui/end.png")),
        game_start: load_png_texture_from_bytes(include_bytes!("../assets/gui/game_start.png")),
        score_panel: load_png_texture_from_bytes(include_bytes!("../assets/gui/bar_panel.png")),
        // Environment
        cloud: load_png_texture_from_bytes(include_bytes!("../assets/clouds/clouds.png")),
        house: load_png_texture_from_bytes(include_bytes!("../assets/house/houseplat.png")),
        background: load_png_texture_from_bytes(include_bytes!(
            "../assets/background/chocobackground.png"
        )),
        // Load all meme textures into the array
        meme_textures: [
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme1.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme2.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme3.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme4.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme5.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme6.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme7.png")),
            load_png_texture_from_bytes(include_bytes!("../assets/gui/meme8.png")),
        ],
        // Load sounds using Macroquad's async loader.
        // `.await` pauses execution here until the sound is loaded.
        // `.unwrap()` handles potential loading errors (panics if loading fails).
        jump: load_sound_from_bytes(include_bytes!("../assets/sounds/ogg/jump.ogg"))
            .await
            .unwrap(),
        egg_collect: load_sound_from_bytes(include_bytes!("../assets/sounds/ogg/check.ogg"))
            .await
            .unwrap(),
        chicken_hit: load_sound_from_bytes(include_bytes!(
            "../assets/sounds/ogg/monster_scream.ogg"
        ))
        .await
        .unwrap(),
        spike_hit: load_sound_from_bytes(include_bytes!("../assets/sounds/ogg/bump.ogg"))
            .await
            .unwrap(),
        magic: load_sound_from_bytes(include_bytes!("../assets/sounds/ogg/magic.ogg"))
            .await
            .unwrap(),
        background_music: load_sound_from_bytes(include_bytes!(
            "../assets/sounds/ogg/music_theme.ogg"
        ))
        .await
        .unwrap(),
        game_over_sound: load_sound_from_bytes(include_bytes!(
            "../assets/sounds/ogg/water_splash.ogg"
        ))
        .await
        .unwrap(),
        win_sound: load_sound_from_bytes(include_bytes!("../assets/sounds/ogg/success.ogg"))
            .await
            .unwrap(),
    }
}

fn draw(state: &State, assets: &Assets) {
    // Clear the screen with the background color.
    clear_background(BACKGROUND_COLOR);
    match state {
        State::Start => {
            // Draw the start screen image, scaled to fit the window.
            draw_texture_ex(
                &assets.game_start,
                0.0,   // Draw at top-left corner (x=0)
                0.0,   // Draw at top-left corner (y=0)
                WHITE, // No tint
                DrawTextureParams {
                    // Scale the image to fill the entire screen width and height
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default() // Use defaults for other parameters
                },
            );
        }
        State::Game {
            player,
            player_direction,
            score,
            clouds,
            platforms,
            eggs,
            chickens,
            spikes,
            house,
            background_entities,
        } => {
            // --- Camera Setup ---
            // Calculate the camera's target X position to follow the player,
            // but don't let it go left of the starting area (x=0).
            let camera_x = (player.entity.rect.center().x - screen_width() / 2.0).max(0.0);

            // Create a 2D camera. `from_display_rect` sets up the view area.
            let mut camera = Camera2D::from_display_rect(Rect::new(
                camera_x,        // Camera's left edge follows player (or stays at 0)
                0.0,             // Camera's top edge stays at the top of the screen
                screen_width(),  // Camera's view width is the screen width
                screen_height(), // Camera's view height is the screen height
            ));
            // By default, Macroquad's Y-axis points down. Games often use Y-axis pointing up.
            // Flipping the camera's Y-zoom effectively inverts the Y-axis for drawing.
            camera.zoom.y = -camera.zoom.y;
            // Note: This also means positive Y velocity moves things *up* the screen,
            // and gravity needs to be positive to pull things *down*.

            // Apply the camera settings for drawing world elements.
            set_camera(&camera);

            // --- Draw World Elements (using camera coordinates) ---
            for (entity, texture) in (background_entities.iter().map(|e| (e, &assets.background)))
                .chain(clouds.iter().map(|c| (&c.entity, &assets.cloud)))
                .chain(platforms.iter().map(|p| (p, &assets.platform)))
                .chain(eggs.iter().map(|e| (e, &assets.egg)))
                .chain(chickens.iter().map(|c| (&c.entity, &assets.chicken)))
                .chain(spikes.iter().map(|s| (s, &assets.spike)))
                .chain(once((house, &assets.house)))
                .chain(once((
                    &player.entity,
                    match player_direction {
                        MoveDirection::Right => &assets.player_right,
                        MoveDirection::Left => &assets.player_left,
                    },
                )))
            {
                draw_texture_ex(
                    texture,       // The image to draw
                    entity.rect.x, // X position on screen
                    entity.rect.y, // Y position on screen
                    WHITE,         // Tint color (WHITE means no tint)
                    DrawTextureParams {
                        // Ensure the texture is drawn at the entity's width and height
                        dest_size: Some(Vec2::new(entity.rect.w, entity.rect.h)),
                        ..DrawTextureParams::default() // Use default values for other parameters
                    },
                );
            }

            // --- Draw UI Elements (using screen coordinates) ---
            // Switch back to the default camera (no scrolling, fixed to the screen).
            set_default_camera();

            // Draw the score panel background image in the top-right corner.
            // Positions and sizes are relative to screen dimensions.
            draw_texture_ex(
                &assets.score_panel,
                screen_width() * 0.7,   // X position (70% from left)
                screen_height() * 0.03, // Y position (3% from top)
                WHITE,                  // No tint
                DrawTextureParams {
                    // Scale panel size relative to screen size
                    dest_size: Some(Vec2::new(screen_width() * 0.25, screen_height() * 0.1)),
                    ..Default::default()
                },
            );
            // Draw the main score text (e.g., "Score: 3/5").
            draw_text(
                &format!("Score: {}/{}", score, EGGS_NEEDED_FOR_WIN), // Text content
                screen_width() * 0.75,                                // X position
                screen_height() * 0.07,                               // Y position
                0.03 * screen_height(), // Font size relative to screen height
                WHITE,                  // Text color
            );
            // Draw the secondary score text related to reaching the house (e.g., "🥚 + 3/2").
            draw_text(
                &format!("🥚 + {}/{}", score, EGGS_NEEDED_FOR_HOUSE), // Text content
                screen_width() * 0.75,                                // X position
                screen_height() * 0.10,                               // Y position
                0.03 * screen_height(),                               // Font size
                WHITE,                                                // Text color
            );
        }
        State::GameOver(reason) => {
            // Choose the appropriate game over image based on the reason.
            // Draw the chosen game over/win/end screen image, scaled to fit.
            draw_texture_ex(
                match reason {
                    GameOverReason::Death { .. } => &assets.game_over, // Standard game over screen
                    GameOverReason::End { meme: ending } => &assets.meme_textures[*ending], // Pick a random meme
                    GameOverReason::Win => &assets.win, // Winning screen
                },
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default()
                },
            );

            // If there's final score text to display (only on Death screen)...
            if let GameOverReason::Death { score, .. } = reason {
                // Calculate text position relative to screen size for consistent placement.
                let text_x = screen_width() * 0.415;
                let text_y = screen_height() * 0.227;
                let font_size = 0.04 * screen_height(); // Scale font size with screen height
                                                        // Draw the final score text.
                draw_text(
                    &format!("Final Score: {score}"),
                    text_x,
                    text_y,
                    font_size,
                    WHITE,
                );
            }
        }
    }
}

/// Configures the game window settings.
fn window_conf() -> Conf {
    Conf {
        window_title: "Easter Egg".to_owned(), // Title shown in the window bar
        window_width: 1024,                    // Initial width in pixels
        window_height: 768,                    // Initial height in pixels
        ..Default::default()                   // Use default values for other configuration options
    }
}

/// The main entry point of the application.
/// `#[macroquad::main(window_conf)]` sets up the Macroquad environment
/// using the configuration from `window_conf` and defines this `async fn`
/// as the starting point.
#[macroquad::main(window_conf)]
async fn main() {
    let assets = load_assets().await;
    let mut state = State::Start;
    // Start playing the background music on loop.
    play_sound(
        &assets.background_music,
        PlaySoundParams {
            looped: true, // Keep playing after it finishes
            volume: 0.5,  // Set volume to 50%
        },
    );
    loop {
        draw(&state, &assets); // Draw the current game state
        next_frame().await; // Wait for the next frame to start
        let input_events = state.process_input(); // Handle user input
        let update_events = state.update(get_frame_time()); // Update game state
        for event in input_events.into_iter().chain(update_events) {
            match event {
                Event::Jumped => play_sound_once(&assets.jump),
                Event::Scored => play_sound_once(&assets.egg_collect),
                Event::GameOver(GameOverReason::Win) => play_sound_once(&assets.win_sound),
                Event::GameOver(GameOverReason::End { .. }) => play_sound_once(&assets.magic),
                Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Chicken,
                    ..
                }) => play_sound_once(&assets.chicken_hit),
                Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Spike,
                    ..
                }) => play_sound_once(&assets.spike_hit),
                Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Fall,
                    ..
                }) => play_sound_once(&assets.game_over_sound),
            }
        }
    }
}
