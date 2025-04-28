#![allow(clippy::pedantic, clippy::nursery, clippy::manual_range_contains)]
use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, PlaySoundParams, Sound,
};
use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::prelude::*;
use macroquad::rand::{gen_range, ChooseRandom};

// Physics constants
const GRAVITY: f32 = 1000.0; // Increased gravity for fixed timestep
const GROUND_DETECTION_BUFFER: f32 = 5.0;
const COLLISION_MARGIN: f32 = 2.0;

// Player constants
const PLAYER_START_POS: Vec2 = Vec2::new(243.0, 350.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0; // Increased for fixed timestep (pixels per second)
const PLAYER_JUMP_SPEED: f32 = 500.0; // Increased for fixed timestep (pixels per second)

// Entity sizes (pre-calculated based on texture dimensions and previous scale)
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0); // 10x16 * 3.0
const PLATFORM_SIZE: Vec2 = Vec2::new(429.0, 141.0); // 143x47 * 3.0
const PLATFORM_BAR_SIZE: Vec2 = Vec2::new(214.5, 70.5); // 143x47 * 1.5
const CHICKEN_SIZE: Vec2 = Vec2::new(52.0, 48.0); // 13x12 * 4.0
const EGG_SIZE: Vec2 = Vec2::new(40.0, 40.0); // 400x400 * 0.1
const SPIKE_SIZE: Vec2 = Vec2::new(60.0, 52.0); // 15x13 * 4.0
const HOUSE_SIZE: Vec2 = Vec2::new(423.0, 624.0); // 141x208 * 3.0
const CLOUD_SIZE: Vec2 = Vec2::new(786.0, 150.0); // 262x50 * 3.0
const BACKGROUND_SIZE: Vec2 = Vec2::new(1024.0, 2304.0); // 1024x2304 * 1.0

const EGGS_NEEDED_FOR_HOUSE: u32 = 2; // Number of eggs needed to reach the house
const EGGS_NEEDED_FOR_WIN: u32 = 5; // Number of eggs needed to win

// Visual constants
const BACKGROUND_COLOR: Color = Color {
    r: 0.92,
    g: 0.88,
    b: 0.78,
    a: 1.0,
};

// Struct to store all game textures
struct Assets {
    player_right: Texture2D,
    player_left: Texture2D,
    platform: Texture2D,
    chicken: Texture2D,
    spike: Texture2D,
    egg: Texture2D,
    game_over: Texture2D,
    win: Texture2D,
    game_start: Texture2D,
    score_panel: Texture2D,
    cloud: Texture2D,
    house: Texture2D,
    background: Texture2D,
    meme_textures: [Texture2D; 8],
    jump: Sound,
    egg_collect: Sound,
    chicken_hit: Sound,
    spike_hit: Sound,
    magic: Sound,
    background_music: Sound,
    game_over_sound: Sound,
    win_sound: Sound,
}

// Custom PNG loader that loads from bytes instead of files
fn load_png_texture_from_bytes(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_file_with_format(bytes, None);
    // Set proper filtering mode for pixel art
    texture.set_filter(FilterMode::Nearest);
    texture
}

// Load textures with a simple loading screen
async fn load_assets() -> Assets {
    // Now load all textures
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
        cloud: load_png_texture_from_bytes(include_bytes!("../assets/clouds/clouds.png")),
        house: load_png_texture_from_bytes(include_bytes!("../assets/house/houseplat.png")),
        background: load_png_texture_from_bytes(include_bytes!(
            "../assets/background/chocobackground.png"
        )),
        // Load all meme textures for random selection
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

// Game entity struct
struct GameEntity {
    rect: Rect,
}

impl GameEntity {
    // Helper method to get collision bounds with margin
    fn get_collision_bounds(&self) -> Rect {
        Rect {
            x: self.rect.x + COLLISION_MARGIN,
            y: self.rect.y + COLLISION_MARGIN,
            w: self.rect.w - COLLISION_MARGIN,
            h: self.rect.h - COLLISION_MARGIN,
        }
    }

    // Helper function to draw an entity (no need for offset with Camera2D)
    fn draw(&self, texture: &Texture2D) {
        draw_texture_ex(
            texture,
            self.rect.x,
            self.rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(self.rect.w, self.rect.h)),
                ..DrawTextureParams::default()
            },
        );
    }
}

// Game entity struct
struct MovingGameEntity {
    entity: GameEntity,
    velocity: Vec2,
}

impl MovingGameEntity {
    fn apply_velocity(&mut self, delta_time: f32) {
        self.entity.rect.x += self.velocity.x * delta_time;
        self.entity.rect.y += self.velocity.y * delta_time;
    }
}

enum MoveDirection {
    Left,
    Right,
}

// Reasons for game over
enum GameOverReason {
    Death { score: u32 },
    End, // Player reached the end (house)
    Win, // Player collected enough eggs
}

async fn start_screen(texture_assets: &Assets) {
    // Play background music when the game starts
    play_sound(
        &texture_assets.background_music,
        PlaySoundParams {
            looped: true,
            volume: 0.5,
        },
    );

    loop {
        next_frame().await;
        if is_key_pressed(KeyCode::P) {
            break;
        }
        clear_background(BACKGROUND_COLOR);
        draw_texture_ex(
            &texture_assets.game_start,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                ..Default::default()
            },
        );
    }
}

async fn game_over_screen(assets: &Assets, reason: GameOverReason) {
    // Play appropriate sound based on game over reason
    match reason {
        GameOverReason::Death { .. } => play_sound_once(&assets.game_over_sound),
        GameOverReason::End => play_sound_once(&assets.magic),
        GameOverReason::Win => play_sound_once(&assets.win_sound),
    }

    let texture = match reason {
        GameOverReason::Death { .. } => &assets.game_over,
        GameOverReason::End => assets.meme_textures.choose().unwrap(),
        GameOverReason::Win => &assets.win,
    };
    let final_score_text = if let GameOverReason::Death { score, .. } = reason {
        Some(format!("Final Score: {score}"))
    } else {
        None
    };
    loop {
        next_frame().await;
        if is_key_pressed(KeyCode::R) {
            break;
        }
        clear_background(BACKGROUND_COLOR);
        // Draw game over screen based on reason
        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        // Display score on game over screen (except for End and Win)
        if let Some(final_score_text) = &final_score_text {
            // Position score text properly in the game over box using scaled text
            draw_text(
                final_score_text,
                screen_width() * 0.415,
                screen_height() * 0.227,
                0.04 * screen_height(),
                WHITE,
            );
        }
    }
}

async fn game_screen(assets: &Assets) -> GameOverReason {
    // Create player
    let mut player = MovingGameEntity {
        entity: GameEntity {
            rect: Rect {
                x: PLAYER_START_POS.x - PLAYER_SIZE.x / 2.0,
                y: PLAYER_START_POS.y - PLAYER_SIZE.y / 2.0,
                w: PLAYER_SIZE.x,
                h: PLAYER_SIZE.y,
            },
        },
        velocity: Vec2::ZERO,
    };
    let mut player_direction = MoveDirection::Right;
    let mut score = 0;

    // Create background entities (based on Haskell implementation)
    let background_entities: Vec<GameEntity> = (0..=60)
        .map(|i| {
            let x = -1024.0 + i as f32 * 1024.0;
            GameEntity {
                rect: Rect {
                    x: x - BACKGROUND_SIZE.x / 2.0,
                    y: 336.0 - BACKGROUND_SIZE.y / 2.0,
                    w: BACKGROUND_SIZE.x,
                    h: BACKGROUND_SIZE.y,
                },
            }
        })
        .collect();

    // Create clouds with random positions and velocities
    let mut clouds: Vec<MovingGameEntity> = (0..=40)
        .map(|i| {
            let x = -1024.0 + 500.0 * i as f32;
            let y = gen_range(100.0, 500.0); // Random height
            MovingGameEntity {
                entity: GameEntity {
                    rect: Rect {
                        x: x - CLOUD_SIZE.x / 2.0,
                        y: y - CLOUD_SIZE.y / 2.0,
                        w: CLOUD_SIZE.x,
                        h: CLOUD_SIZE.y,
                    },
                },
                velocity: Vec2::new(gen_range(20.0, 60.0), 0.0), // Random horizontal velocity
            }
        })
        .collect();

    // Adjust platforms to be more accessible and spaced out
    let platforms: Vec<GameEntity> = (-429..=2000)
        .step_by(400) // Increase spacing between platforms to 400 units
        .map(|x| GameEntity {
            rect: Rect {
                x: x as f32 - PLATFORM_SIZE.x / 2.0,
                y: screen_height() - PLATFORM_SIZE.y,
                w: PLATFORM_SIZE.x,
                h: PLATFORM_SIZE.y,
            },
        })
        .chain((0..60).map(|i| {
            let x = i as f32 * 50.0 + gen_range(-200.0, 200.0); // Adjust random offset to be smaller
            let y = gen_range(150.0, 650.0);
            GameEntity {
                rect: Rect {
                    x: x - PLATFORM_BAR_SIZE.x / 2.0,
                    y: y - PLATFORM_BAR_SIZE.y / 2.0,
                    w: PLATFORM_BAR_SIZE.x,
                    h: PLATFORM_BAR_SIZE.y,
                },
            }
        }))
        .collect();

    // Create eggs by mapping from platforms - this ensures eggs are always on platforms
    let mut eggs: Vec<GameEntity> = platforms
        .iter()
        .filter(|_| gen_range(0, 100) < 30) // 30% chance to spawn an egg
        .enumerate()
        .map(|(i, platform)| {
            let offset = (i as f32 - 0.5) * (platform.rect.w * 0.5);
            let x = platform.rect.center().x + offset;
            let y = platform.rect.y - EGG_SIZE.y + 5.0;

            GameEntity {
                rect: Rect {
                    x: x - EGG_SIZE.x / 2.0,
                    y,
                    w: EGG_SIZE.x,
                    h: EGG_SIZE.y,
                },
            }
        })
        .collect();

    // Create chickens with completely random positioning, independent of platforms
    let mut chickens: Vec<MovingGameEntity> = (0..20)
        .map(|_| {
            // Random x position across the game world
            // Distribute chickens throughout the game world with some spacing
            let x = gen_range(500.0, 4000.0);
            // Random y position within playable area
            let y = gen_range(100.0, 600.0);

            // Random velocity in both directions
            let vx = gen_range(50.0, 150.0) * (if gen_range(0, 2) == 0 { 1.0 } else { -1.0 });
            let vy = gen_range(30.0, 80.0) * (if gen_range(0, 2) == 0 { 1.0 } else { -1.0 });

            MovingGameEntity {
                entity: GameEntity {
                    rect: Rect {
                        x: x - CHICKEN_SIZE.x / 2.0,
                        y: y - CHICKEN_SIZE.y / 2.0,
                        w: CHICKEN_SIZE.x,
                        h: CHICKEN_SIZE.y,
                    },
                },
                velocity: Vec2::new(vx, vy),
            }
        })
        .collect();

    // Create spikes by mapping from platforms - this ensures spikes are always on platforms
    let spikes: Vec<GameEntity> = platforms
        .iter()
        .filter(|platform| {
            platform.rect.center().y > screen_height() - PLATFORM_SIZE.y && gen_range(0, 5) == 0
        })
        .map(|platform| GameEntity {
            rect: Rect {
                x: platform.rect.right() - SPIKE_SIZE.x / 2.0,
                y: platform.rect.y - SPIKE_SIZE.y + 5.0,
                w: SPIKE_SIZE.x,
                h: SPIKE_SIZE.y,
            },
        })
        .collect();

    // Create houses (end goals) with similar positioning to Haskell version
    let house = GameEntity {
        rect: Rect {
            x: 3000.0 - HOUSE_SIZE.x / 2.0,
            y: 292.0 - HOUSE_SIZE.y / 2.0,
            w: HOUSE_SIZE.x,
            h: HOUSE_SIZE.y,
        },
    };

    loop {
        // Clear the screen
        next_frame().await;

        // Handle input
        {
            match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
                (true, false) => {
                    player_direction = MoveDirection::Left;
                    player.velocity.x = -PLAYER_MOVEMENT_SPEED;
                }
                (false, true) => {
                    player_direction = MoveDirection::Right;
                    player.velocity.x = PLAYER_MOVEMENT_SPEED;
                }
                _ => {
                    player.velocity.x = 0.0;
                }
            };
            if is_key_pressed(KeyCode::Up) && player.velocity.y == 0.0 {
                player.velocity.y = -PLAYER_JUMP_SPEED;
                play_sound_once(&assets.jump);
            }
        }

        // Update game state
        {
            let delta_time = get_frame_time();
            player.velocity.y += GRAVITY * delta_time;

            let ground_collision = platforms.iter().find_map(|platform| {
                // Check horizontal overlap using Rect methods
                let horizontally_overlapping = player.entity.rect.right() > platform.rect.x
                    && player.entity.rect.x < platform.rect.right();

                // Check vertical conditions for ground detection
                let falling_towards_platform = player.velocity.y >= 0.0;
                let close_to_platform_top =
                    player.entity.rect.bottom() <= platform.rect.y + GROUND_DETECTION_BUFFER;
                let will_intersect_next_frame =
                    player.entity.rect.bottom() + player.velocity.y * delta_time >= platform.rect.y;

                if horizontally_overlapping
                    && falling_towards_platform
                    && close_to_platform_top
                    && will_intersect_next_frame
                {
                    Some(platform.rect.y)
                } else {
                    None
                }
            });

            player.apply_velocity(delta_time);

            if let Some(platform_top) = ground_collision {
                player.entity.rect.y = platform_top - player.entity.rect.h;
                player.velocity.y = 0.0;
            }

            if player.entity.rect.bottom() > screen_height() + 100.0 {
                return GameOverReason::Death { score };
            }

            // Update chickens with fixed timestep
            for chicken in &mut chickens {
                chicken.apply_velocity(delta_time);
                if chicken.entity.rect.x > 5000.0 || chicken.entity.rect.x < 0.0 {
                    chicken.velocity.x = -chicken.velocity.x;
                }
                if chicken.entity.rect.y > 800.0 || chicken.entity.rect.y < 0.0 {
                    chicken.velocity.y = -chicken.velocity.y;
                }
            }

            // Update clouds with fixed timestep
            for cloud in &mut clouds {
                cloud.apply_velocity(delta_time);
                if cloud.entity.rect.x > 60000.0 {
                    cloud.entity.rect.x = -1024.0;
                }
            }
        }

        // Check collisions and game state updates (non-physics but game logic)
        {
            eggs.retain(|egg| {
                let collided = player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&egg.get_collision_bounds());
                if collided {
                    score += 1;
                    play_sound_once(&assets.egg_collect);
                }
                !collided
            });

            if chickens.iter().any(|chicken| {
                player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&chicken.entity.get_collision_bounds())
            }) {
                play_sound_once(&assets.chicken_hit);
                return GameOverReason::Death { score };
            }

            if spikes.iter().any(|spike| {
                player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&spike.get_collision_bounds())
            }) {
                play_sound_once(&assets.spike_hit);
                return GameOverReason::Death { score };
            }

            if player
                .entity
                .get_collision_bounds()
                .overlaps(&house.get_collision_bounds())
            {
                if score >= EGGS_NEEDED_FOR_WIN {
                    return GameOverReason::Win;
                } else if score >= EGGS_NEEDED_FOR_HOUSE {
                    return GameOverReason::End;
                }
            }
        }

        // Draw game entities
        {
            clear_background(BACKGROUND_COLOR);

            // Calculate camera position based on player
            let camera_x = (player.entity.rect.center().x - screen_width() / 2.0).max(0.0);

            // Setup Camera2D with explicit control over the coordinate system
            let mut camera = Camera2D::from_display_rect(Rect::new(
                camera_x,
                0.0,
                screen_width(),
                screen_height(),
            ));
            // Invert the Y-axis to fix the upside-down rendering
            camera.zoom.y = -camera.zoom.y;

            // Set camera for world elements
            set_camera(&camera);

            // Draw all world elements with their actual coordinates
            for background in &background_entities {
                background.draw(&assets.background);
            }
            for cloud in &clouds {
                cloud.entity.draw(&assets.cloud);
            }
            for platform in &platforms {
                platform.draw(&assets.platform);
            }
            house.draw(&assets.house);
            for egg in &eggs {
                egg.draw(&assets.egg);
            }
            for spike in &spikes {
                spike.draw(&assets.spike);
            }
            for chicken in &chickens {
                chicken.entity.draw(&assets.chicken);
            }
            match player_direction {
                MoveDirection::Right => player.entity.draw(&assets.player_right),
                MoveDirection::Left => player.entity.draw(&assets.player_left),
            }

            // Switch back to default camera for UI elements
            set_default_camera();

            // Draw UI elements (score panel, text)
            draw_texture_ex(
                &assets.score_panel,
                screen_width() * 0.7,
                screen_height() * 0.03,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width() * 0.25, screen_height() * 0.1)),
                    ..Default::default()
                },
            );
            draw_text(
                &format!("Score: {}/5", score),
                screen_width() * 0.75,
                screen_height() * 0.07,
                0.03 * screen_height(),
                WHITE,
            );
            draw_text(
                &format!("🥚 + {}/2", score),
                screen_width() * 0.75,
                screen_height() * 0.10,
                0.03 * screen_height(),
                WHITE,
            );
        }
    }
}

// Main game configuration
fn window_conf() -> Conf {
    Conf {
        window_title: "Easter Egg".to_owned(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let textures = load_assets().await;

    start_screen(&textures).await;
    loop {
        let game_over_reason = game_screen(&textures).await;
        game_over_screen(&textures, game_over_reason).await;
    }
}
