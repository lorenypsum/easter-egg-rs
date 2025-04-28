// These lines disable certain warnings from Clippy, which is Rust's additional linter.
// We're ignoring some style warnings to focus on the game's core functionality.
// In larger projects, you'd want to address these warnings for better code quality.
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::manual_range_contains,
    clippy::too_many_lines
)]

// Import our game engine module defined in engine.rs
// The 'mod' keyword declares that we're using a module
mod engine;

// Import specific items from our engine module using the 'use' keyword
// This brings these types into our current scope so we can use them directly
use engine::{
    DeathCause, Event, GameOverReason, MoveDirection, State, EGGS_NEEDED_FOR_HOUSE,
    EGGS_NEEDED_FOR_WIN,
};

// Import functions and types from the macroquad crate
// Macroquad is a simple game library for Rust that handles graphics, input, etc.
use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, PlaySoundParams, Sound,
};
use macroquad::camera::{set_camera, set_default_camera, Camera2D};
use macroquad::prelude::{
    clear_background, draw_text, draw_texture_ex, get_frame_time, next_frame, screen_height,
    screen_width, Color, Conf, DrawTextureParams, FilterMode, Rect, Texture2D, Vec2, WHITE,
};

// Import iterator tools from the standard library
// The 'once' function creates an iterator that yields a single value
use std::iter::once;

// --- Visual Constants ---
// Define a constant for the background color using Rust's const keyword
// In Rust, constants are always uppercase by convention
pub const BACKGROUND_COLOR: Color = Color {
    r: 0.92, // Red component (0.0 to 1.0)
    g: 0.88, // Green component (0.0 to 1.0)
    b: 0.78, // Blue component (0.0 to 1.0)
    a: 1.0,  // Alpha (transparency) component (1.0 is fully opaque)
};

/// This struct holds all the game's assets (images and sounds).
/// 
/// In Rust, structs are defined with the 'struct' keyword and contain fields.
/// The /// comments create documentation that can be generated with 'cargo doc'.
struct Assets {
    // Player textures
    player_right: Texture2D, // The player sprite facing right
    player_left: Texture2D,  // The player sprite facing left
    
    // Object textures
    platform: Texture2D,  // Platforms the player can stand on
    chicken: Texture2D,   // Enemy chicken sprites
    spike: Texture2D,     // Deadly spike obstacles
    egg: Texture2D,       // Collectible eggs
    
    // UI / Screen textures
    game_over: Texture2D,   // Shown when the game ends in failure
    win: Texture2D,         // Shown when the player wins
    game_start: Texture2D,  // The initial title screen
    score_panel: Texture2D, // Panel showing the player's score
    
    // Environment textures
    cloud: Texture2D,      // Decorative clouds in the background
    house: Texture2D,      // The goal structure to reach
    background: Texture2D, // The game's background image
    
    // Fun extras
    meme_textures: [Texture2D; 8], // An array of 8 meme images shown in different end states
    
    // Sound effects
    jump: Sound,        // Played when the player jumps
    egg_collect: Sound, // Played when collecting an egg
    chicken_hit: Sound, // Played when hitting a chicken
    spike_hit: Sound,   // Played when hitting a spike
    magic: Sound,       // Played when reaching the house without enough eggs
    
    // Music
    background_music: Sound, // Looping background music
    game_over_sound: Sound,  // Played on game over
    win_sound: Sound,        // Played when winning
}

/// Loads a PNG image from embedded byte data into a Macroquad texture.
/// 
/// This function demonstrates how to:
/// 1. Take a parameter (&[u8] is a slice of bytes)
/// 2. Convert raw data into a texture
/// 3. Configure the texture's properties
/// 4. Return the texture (Rust functions return the last expression without 'return' keyword)
/// 
/// Parameters:
///   - bytes: A slice of bytes representing the PNG file data.
fn load_png_texture_from_bytes(bytes: &[u8]) -> Texture2D {
    // Load texture from bytes. 'None' means auto-detect the image format.
    let texture = Texture2D::from_file_with_format(bytes, None);
    
    // Set the texture filtering mode to Nearest (for pixel-perfect rendering).
    // This prevents the texture from looking blurry when scaled.
    texture.set_filter(FilterMode::Nearest);
    
    // In Rust, the last expression without a semicolon is implicitly returned
    texture
}

/// Asynchronously loads all game assets (textures and sounds).
/// 
/// This function is marked 'async', which means it can perform operations
/// that might take time (like loading files) without blocking the main thread.
/// The 'await' keyword is used to pause execution until an async operation completes.
/// 
/// In Rust, functions that can be paused and resumed are called 'async functions'.
async fn load_assets() -> Assets {
    // Use the 'include_bytes!' macro to embed files directly into the executable.
    // This macro reads the file at compile time and includes its contents in the binary.
    // The '!' indicates this is a macro, not a function.
    
    Assets {
        // Load player textures
        player_right: load_png_texture_from_bytes(include_bytes!(
            "../assets/character/c_right.png"
        )),
        player_left: load_png_texture_from_bytes(include_bytes!("../assets/character/c_left.png")),
        
        // Load environment and object textures
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
        
        // Load meme textures into an array
        // In Rust, arrays have a fixed size determined at compile time [Type; size]
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
        
        // Load sounds using Macroquad's async loader
        // The '.await' keyword pauses execution until the sound is loaded
        // '.unwrap()' is a way to handle errors in Rust - it gets the value or crashes if there's an error
        // In a production app, you'd handle errors more gracefully
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

/// Draws the current game state to the screen.
/// 
/// This function shows how to handle different states using Rust's pattern matching.
/// The 'match' expression is similar to switch/case in other languages but more powerful.
/// 
/// Parameters:
///   - state: The current game state to draw
///   - assets: The loaded game assets to use for drawing
fn draw(state: &State, assets: &Assets) {
    // Clear the screen with our background color
    clear_background(BACKGROUND_COLOR);
    
    // Use 'match' to handle different game states
    // This is Rust's pattern matching - it helps ensure we handle all possible cases
    match state {
        // When the game is in the Start state, draw the title screen
        State::Start => {
            // Draw the start screen image, scaled to fit the window
            draw_texture_ex(
                &assets.game_start,      // The texture to draw
                0.0,                     // X position (top-left corner)
                0.0,                     // Y position (top-left corner)
                WHITE,                   // Color tint (WHITE means no tint)
                DrawTextureParams {
                    // Scale the image to fill the entire screen
                    // Vec2 is a 2D vector with x and y components
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default() // Use defaults for other parameters
                },
            );
        }
        
        // When the game is active, draw all game elements
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
            // Calculate camera X position to follow player, but don't go left of screen edge
            // The '.max()' method returns the larger of two values
            let camera_x = (player.entity.rect.center().x - screen_width() / 2.0).max(0.0);
            
            // Create a 2D camera that shows a specific rectangle of the world
            let mut camera = Camera2D::from_display_rect(Rect::new(
                camera_x,        // Left edge of camera view
                0.0,             // Top edge of camera view
                screen_width(),  // Width of camera view
                screen_height(), // Height of camera view
            ));
            
            // Flip the Y-axis because game coordinates typically have (0,0) at top-left
            // with Y increasing downward, but we want Y to increase upward
            camera.zoom.y = -camera.zoom.y;
            
            // Apply the camera settings for drawing world elements
            set_camera(&camera);
            
            // --- Draw World Elements (using camera coordinates) ---
            // This complex expression chains multiple iterators together to draw different types of objects
            // It creates pairs of (entity, texture) for each game object
            for (entity, texture) in (background_entities.iter().map(|e| (e, &assets.background)))
                // Chain combines multiple iterators into one
                .chain(clouds.iter().map(|c| (&c.entity, &assets.cloud)))
                .chain(platforms.iter().map(|p| (p, &assets.platform)))
                .chain(eggs.iter().map(|e| (e, &assets.egg)))
                .chain(chickens.iter().map(|c| (&c.entity, &assets.chicken)))
                .chain(spikes.iter().map(|s| (s, &assets.spike)))
                .chain(once((house, &assets.house)))
                .chain(once((
                    &player.entity,
                    // Choose player texture based on direction
                    match player_direction {
                        MoveDirection::Right => &assets.player_right,
                        MoveDirection::Left => &assets.player_left,
                    },
                )))
            {
                // Draw each entity with its corresponding texture
                draw_texture_ex(
                    texture,       // The texture to draw
                    entity.rect.x, // X position
                    entity.rect.y, // Y position
                    WHITE,         // No color tint
                    DrawTextureParams {
                        // Set the size to match the entity's dimensions
                        dest_size: Some(Vec2::new(entity.rect.w, entity.rect.h)),
                        ..DrawTextureParams::default() // Default values for other parameters
                    },
                );
            }
            
            // --- Draw UI Elements (using screen coordinates) ---
            // Switch back to the default camera for UI elements that don't move with the world
            set_default_camera();
            
            // Draw score panel in top-right corner
            draw_texture_ex(
                &assets.score_panel,
                screen_width() * 0.7,   // X position (70% from left edge)
                screen_height() * 0.03, // Y position (3% from top edge)
                WHITE,
                DrawTextureParams {
                    // Size relative to screen dimensions
                    dest_size: Some(Vec2::new(screen_width() * 0.25, screen_height() * 0.1)),
                    ..Default::default()
                },
            );
            
            // Draw the main score text
            // format! is a macro that creates a String using printf-like syntax
            draw_text(
                &format!("Score: {}/{}", score, EGGS_NEEDED_FOR_WIN), // Text to display
                screen_width() * 0.75,                                // X position 
                screen_height() * 0.07,                               // Y position
                0.03 * screen_height(),                               // Font size (scales with screen)
                WHITE,                                                // Text color
            );
            
            // Draw secondary score text showing progress toward house goal
            draw_text(
                &format!("🥚 + {}/{}", score, EGGS_NEEDED_FOR_HOUSE), // Text with emoji
                screen_width() * 0.75,
                screen_height() * 0.10,
                0.03 * screen_height(),
                WHITE,
            );
        }
        
        // When the game is over, show the appropriate end screen
        State::GameOver(reason) => {
            // Choose which end screen to show based on the reason for game over
            // This uses pattern matching to select the right texture
            draw_texture_ex(
                match reason {
                    // Different screens for different game end reasons
                    GameOverReason::Death { .. } => &assets.game_over, // Death screen
                    GameOverReason::End { meme: ending } => &assets.meme_textures[*ending], // Random meme
                    GameOverReason::Win => &assets.win,                // Win screen
                },
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
            
            // If it's a death game-over, show the final score
            // This uses pattern matching with the 'if let' syntax to extract values
            if let GameOverReason::Death { score, .. } = reason {
                // Calculate positions to place text relative to screen size
                let text_x = screen_width() * 0.415;
                let text_y = screen_height() * 0.227;
                let font_size = 0.04 * screen_height();
                
                // Draw the final score text
                draw_text(
                    &format!("Final Score: {score}"), // Using the destructured score value
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
/// 
/// This function returns a Conf struct that Macroquad uses to set up the window.
/// In Rust, functions can return values without using a 'return' keyword.
fn window_conf() -> Conf {
    Conf {
        window_title: "Easter Egg".to_owned(), // Convert &str to String using to_owned()
        window_width: 1024,                    // Initial window width in pixels
        window_height: 768,                    // Initial window height in pixels
        ..Default::default()                   // Use default values for other settings
    }
}

/// The main entry point of the application.
/// 
/// The #[macroquad::main] attribute is a macro that sets up the Macroquad environment
/// using our window_conf function. The 'async' keyword makes this function asynchronous,
/// allowing us to use .await to handle asset loading and frame timing.
#[macroquad::main(window_conf)]
async fn main() {
    // Load all game assets - we wait (await) until loading completes
    let assets = load_assets().await;
    
    // Initialize the game state to the Start screen
    let mut state = State::Start;
    
    // Start playing background music in a loop
    play_sound(
        &assets.background_music,
        PlaySoundParams {
            looped: true, // Keep repeating when done
            volume: 0.5,  // 50% volume
        },
    );
    
    // Main game loop - runs until the program exits
    loop {
        // Draw the current game state
        draw(&state, &assets);
        
        // Wait for the next frame to start
        // This helps maintain a consistent frame rate
        next_frame().await;
        
        // Process user input and get any resulting events
        let input_events = state.process_input();
        
        // Update the game state for this frame and get resulting events
        // get_frame_time() returns the time elapsed since last frame in seconds
        let update_events = state.update(get_frame_time());
        
        // Combine input and update events and process them
        // chain() combines both iterators, into_iter() converts to an iterator
        for event in input_events.into_iter().chain(update_events) {
            // Handle each event with pattern matching
            match event {
                Event::Jumped => play_sound_once(&assets.jump),
                Event::Scored => play_sound_once(&assets.egg_collect),
                Event::GameOver(GameOverReason::Win) => play_sound_once(&assets.win_sound),
                Event::GameOver(GameOverReason::End { .. }) => play_sound_once(&assets.magic),
                // Nested pattern matching to handle different death causes
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