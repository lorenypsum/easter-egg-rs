// Import needed types from macroquad
// The 'use' keyword in Rust brings external items into scope
use macroquad::prelude::{is_key_down, is_key_pressed, screen_height, KeyCode, Rect, Vec2};
use macroquad::rand::gen_range;

// --- Physics Constants ---
/// Defines how quickly objects fall downwards (pixels per second squared).
/// This positive value means objects accelerate downward, simulating gravity.
const GRAVITY: f32 = 1000.0;

/// A small buffer zone below the player to detect ground slightly before touching.
/// This helps make jumping feel more responsive by detecting ground a bit early.
const GROUND_DETECTION_BUFFER: f32 = 5.0;

/// A small margin subtracted from entity bounds for collision checks.
/// This prevents collision detection from being too sensitive at the edges.
const COLLISION_MARGIN: f32 = 2.0;

// --- Player Constants ---
/// The starting position (x, y) of the player character on the screen.
/// Vec2 is a 2D vector type that holds two f32 values (x and y).
const PLAYER_START_POS: Vec2 = Vec2::new(243.0, 350.0);

/// How fast the player moves horizontally (pixels per second).
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;

/// The initial upward speed when the player jumps (pixels per second).
/// This is negative because in our coordinate system, negative Y means upward.
const PLAYER_JUMP_SPEED: f32 = 500.0;

// --- Entity Sizes ---
/// Dimensions (width, height) for the player character.
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);

/// Size for the main platform (ground) objects.
/// Original size was 10x16 pixels, scaled by 3.0.
const PLATFORM_SIZE: Vec2 = Vec2::new(429.0, 141.0);

/// Size for the floating platform (bar) objects.
/// Original size was 143x47 pixels, scaled by 3.0.
const PLATFORM_BAR_SIZE: Vec2 = Vec2::new(214.5, 70.5);

/// Size for the chicken enemy objects.
/// Original size was 143x47 pixels, scaled by 1.5.
const CHICKEN_SIZE: Vec2 = Vec2::new(52.0, 48.0);

/// Size for the collectible egg objects.
/// Original size was 13x12 pixels, scaled by 4.0.
const EGG_SIZE: Vec2 = Vec2::new(40.0, 40.0);

/// Size for the dangerous spike objects.
/// Original size was 400x400 pixels, scaled by 0.1.
const SPIKE_SIZE: Vec2 = Vec2::new(60.0, 52.0);

/// Size for the house (end goal) object.
/// Original size was 141x208 pixels, scaled by 3.0.
const HOUSE_SIZE: Vec2 = Vec2::new(423.0, 624.0);

/// Size for the decorative cloud objects.
/// Original size was 262x50 pixels, scaled by 3.0.
const CLOUD_SIZE: Vec2 = Vec2::new(786.0, 150.0);

/// Size for the background image objects.
/// Original size was 1024x2304 pixels, no scaling.
const BACKGROUND_SIZE: Vec2 = Vec2::new(1024.0, 2304.0);

// --- Game Goal Constants ---
/// How many eggs the player needs to collect to trigger the "End" state.
/// This allows the player to reach the house and see a meme ending.
pub const EGGS_NEEDED_FOR_HOUSE: u32 = 2;

/// How many eggs the player needs to collect to trigger the "Win" state.
/// This is the goal for a complete victory.
pub const EGGS_NEEDED_FOR_WIN: u32 = 5;

/// Represents a basic game object with a position and size.
/// 
/// In Rust, structs are defined using the 'struct' keyword and contain fields.
/// This uses 'pub' to make the struct and its field accessible outside this module.
pub struct GameEntity {
    /// The rectangle defining the entity's position (x, y) and dimensions (w, h).
    /// Rect is a type from macroquad that stores x, y, width, and height.
    pub rect: Rect,
}

/// Implementation block for GameEntity.
/// 
/// In Rust, the 'impl' keyword is used to define methods for a type.
/// Methods are functions associated with a struct or enum.
impl GameEntity {
    /// Calculates a slightly smaller collision box for more forgiving collision detection.
    /// 
    /// Methods in Rust take 'self' as their first parameter, which refers to the instance.
    /// The '&self' means we're borrowing the instance (not taking ownership).
    pub fn get_collision_bounds(&self) -> Rect {
        // Create a new Rect that's slightly smaller than the entity's visual rectangle
        Rect {
            x: self.rect.x + COLLISION_MARGIN,       // Move right edge inwards
            y: self.rect.y + COLLISION_MARGIN,       // Move top edge downwards
            w: self.rect.w - COLLISION_MARGIN * 2.0, // Reduce width
            h: self.rect.h - COLLISION_MARGIN * 2.0, // Reduce height
        }
    }
}

/// Represents a game entity that can move.
/// 
/// This struct demonstrates composition - it contains a GameEntity plus velocity.
/// Composition is a way to build complex types from simpler ones.
pub struct MovingGameEntity {
    /// The underlying entity with position and size.
    pub entity: GameEntity,
    
    /// The speed and direction of movement (pixels per second).
    /// Vec2 has x and y components for horizontal and vertical velocity.
    pub velocity: Vec2,
}

impl MovingGameEntity {
    /// Updates the entity's position based on its velocity and elapsed time.
    /// 
    /// This method shows how to modify a struct through a mutable reference (&mut self).
    /// In Rust, you need to explicitly mark variables as mutable to change them.
    /// 
    /// Parameters:
    ///   - delta_time: The time in seconds since the last frame update.
    pub fn apply_velocity(&mut self, delta_time: f32) {
        // Update position: new_position = old_position + velocity * time
        // This is basic physics for motion (distance = speed × time)
        self.entity.rect.x += self.velocity.x * delta_time; // Update x position
        self.entity.rect.y += self.velocity.y * delta_time; // Update y position
    }
}

/// Represents the direction the player is facing.
/// 
/// This is an enum (enumeration) - a type that can be one of several variants.
/// Enums in Rust are more powerful than in many other languages.
pub enum MoveDirection {
    Left,  // Player is facing left
    Right, // Player is facing right
}

/// Represents the different ways a player can die.
/// 
/// This enum shows how to create a simple enumeration with variants.
pub enum DeathCause {
    Chicken, // Player touched a chicken enemy
    Spike,   // Player touched a spike
    Fall,    // Player fell off the bottom of the screen
}

/// Represents the different reasons why the game might end.
/// 
/// This enum demonstrates how variants can carry additional data.
pub enum GameOverReason {
    /// Player died. Contains the cause of death and the final score.
    /// This variant holds a DeathCause enum and a u32 (unsigned 32-bit integer).
    Death { cause: DeathCause, score: u32 },
    
    /// Player reached the house without enough eggs for a full win.
    /// Contains an index for which meme ending to show.
    End { meme: usize },
    
    /// Player won by reaching the house with enough eggs.
    Win,
}

/// Represents game events that can trigger sounds or state changes.
/// 
/// This enum shows how an enum can hold complex data in its variants.
pub enum Event {
    /// Player jumped
    Jumped,
    
    /// Player collected an egg
    Scored,
    
    /// Game ended for some reason (contains the specific reason)
    GameOver(GameOverReason),
}

/// Represents the possible states of the game.
/// 
/// This enum demonstrates Rust's ability to have variants with different data structures.
/// Each variant can hold completely different types of data.
pub enum State {
    /// Title screen, waiting for player to start
    Start,
    
    /// Active gameplay - contains all game entities and state
    /// This uses a struct-like enum variant that holds multiple fields
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
    
    /// Game has ended - contains the reason why
    GameOver(GameOverReason),
}

impl State {
    /// Initializes a new game by creating all game entities.
    /// 
    /// This method modifies the current state to become a new Game state.
    /// It demonstrates:
    /// 1. Creating and initializing complex data structures
    /// 2. Using iterators and functional programming approaches
    /// 3. Using Rust's random number generation
    fn new_game(&mut self) {
        // Create the player character
        let player = MovingGameEntity {
            entity: GameEntity {
                rect: Rect {
                    // Center player at the start position
                    x: PLAYER_START_POS.x - PLAYER_SIZE.x / 2.0,
                    y: PLAYER_START_POS.y - PLAYER_SIZE.y / 2.0,
                    w: PLAYER_SIZE.x,
                    h: PLAYER_SIZE.y,
                },
            },
            velocity: Vec2::ZERO, // Start with no velocity
        };

        // Create background images
        // This demonstrates how to use iterators and functional programming in Rust:
        // 1. range 0..=60 creates numbers from 0 to 60 inclusive
        // 2. map transforms each number into a GameEntity
        // 3. collect() gathers all items into a Vec (vector/array)
        let background_entities: Vec<GameEntity> = (0..=60)
            .map(|i| {
                // Calculate position for this background segment
                let x = -1024.0 + i as f32 * 1024.0;
                
                // Create a new GameEntity for this background
                GameEntity {
                    rect: Rect {
                        x: x - BACKGROUND_SIZE.x / 2.0,     // Center horizontally
                        y: 336.0 - BACKGROUND_SIZE.y / 2.0, // Position vertically
                        w: BACKGROUND_SIZE.x,
                        h: BACKGROUND_SIZE.y,
                    },
                }
            })
            .collect(); // Gather all entities into a Vec

        // Create clouds with random positions and movement speeds
        let clouds: Vec<MovingGameEntity> = (0..=40)
            .map(|i| {
                // Distribute clouds horizontally
                let x = -1024.0 + 500.0 * i as f32;
                // Random height for each cloud
                let y = gen_range(100.0, 500.0);
                
                // Create a moving entity for this cloud
                MovingGameEntity {
                    entity: GameEntity {
                        rect: Rect {
                            x: x - CLOUD_SIZE.x / 2.0, // Center horizontally
                            y: y - CLOUD_SIZE.y / 2.0, // Center vertically
                            w: CLOUD_SIZE.x,
                            h: CLOUD_SIZE.y,
                        },
                    },
                    // Random horizontal speed for each cloud
                    velocity: Vec2::new(gen_range(20.0, 60.0), 0.0),
                }
            })
            .collect();

        // Create platforms (ground and floating)
        // This uses the 'chain' method to combine two collections
        let platforms: Vec<GameEntity> = (-429..=2000) // Range for ground platform positions
            .step_by(400) // Place ground platforms every 400 units
            .map(|x| GameEntity {
                // Create ground platforms
                rect: Rect {
                    x: x as f32 - PLATFORM_SIZE.x / 2.0,  // Center horizontally
                    y: screen_height() - PLATFORM_SIZE.y, // Place at bottom of screen
                    w: PLATFORM_SIZE.x,
                    h: PLATFORM_SIZE.y,
                },
            })
            // Combine ground platforms with floating platforms
            .chain((0..60).map(|i| {
                // Create floating platforms with some randomness
                let x = i as f32 * 50.0 + gen_range(-200.0, 200.0); // Random x offset
                let y = gen_range(150.0, 650.0);                    // Random height
                
                GameEntity {
                    rect: Rect {
                        x: x - PLATFORM_BAR_SIZE.x / 2.0, // Center horizontally
                        y: y - PLATFORM_BAR_SIZE.y / 2.0, // Center vertically
                        w: PLATFORM_BAR_SIZE.x,
                        h: PLATFORM_BAR_SIZE.y,
                    },
                }
            }))
            .collect();

        // Create eggs on top of some platforms
        // This demonstrates filter, enumerate, and map in a chain
        let eggs: Vec<GameEntity> = platforms
            .iter()                           // Iterate through all platforms
            .filter(|_| gen_range(0, 100) < 30) // Randomly select ~30% of platforms
            .enumerate()                      // Get both index and item
            .map(|(i, platform)| {
                // Create an egg on this platform
                // Calculate offset to distribute eggs across platform
                let offset = (i as f32 - 0.5) * (platform.rect.w * 0.5);
                // X position is platform center + offset
                let x = platform.rect.center().x + offset;
                // Y position is just above platform top
                let y = platform.rect.y - EGG_SIZE.y + 5.0;

                GameEntity {
                    rect: Rect {
                        x: x - EGG_SIZE.x / 2.0, // Center horizontally
                        y,                       // Use calculated Y position
                        w: EGG_SIZE.x,
                        h: EGG_SIZE.y,
                    },
                }
            })
            .collect();

        // Create flying chickens with random positions and velocities
        let chickens: Vec<MovingGameEntity> = (0..20) // Create 20 chickens
            .map(|_| {
                // Generate random positions
                let x = gen_range(500.0, 4000.0);
                let y = gen_range(100.0, 600.0);

                // Generate random velocities
                // This uses a ternary-like expression with if/else
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

        // Create spikes on some ground platforms
        // This demonstrates how to chain filter and map on an iterator
        let spikes: Vec<GameEntity> = platforms
            .iter()
            .filter(|platform| {
                // Select only ground platforms (checking Y position)
                platform.rect.center().y > screen_height() - PLATFORM_SIZE.y
                    // And only place spikes randomly (1 in 5 chance)
                    && gen_range(0, 5) == 0
            })
            .map(|platform| GameEntity {
                rect: Rect {
                    // Position spike towards right edge of platform
                    x: platform.rect.right() - SPIKE_SIZE.x / 2.0,
                    // Position spike on platform surface
                    y: platform.rect.y - SPIKE_SIZE.y + 5.0,
                    w: SPIKE_SIZE.x,
                    h: SPIKE_SIZE.y,
                },
            })
            .collect();

        // Create the house (end goal)
        let house = GameEntity {
            rect: Rect {
                x: 3000.0 - HOUSE_SIZE.x / 2.0, // Position far right in level
                y: 292.0 - HOUSE_SIZE.y / 2.0,  // Position vertically
                w: HOUSE_SIZE.x,
                h: HOUSE_SIZE.y,
            },
        };

        // Replace the current state with our new Game state
        // The * dereferences 'self' so we can assign the new value
        // This is using Rust's pattern matching to create a new enum variant
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

    /// Processes user input and updates the game state accordingly.
    /// 
    /// This method:
    /// 1. Handles different inputs based on the current game state
    /// 2. Returns a vector of events triggered by input
    /// 3. Demonstrates Rust's pattern matching with match expressions
    /// 
    /// Returns:
    ///   A Vec<Event> containing any events triggered by input (like jumping)
    pub fn process_input(&mut self) -> Vec<Event> {
        // Create an empty vector to hold events
        // The 'vec![]' macro creates a new vector
        let mut events: Vec<Event> = vec![];
        
        // Handle input differently based on game state
        match self {
            // On the title screen
            State::Start => {
                // Check if 'P' key was pressed this frame to start the game
                if is_key_pressed(KeyCode::P) {
                    self.new_game(); // Initialize a new game
                }
            }
            
            // During active gameplay
            State::Game {
                player,
                player_direction,
                ..
            } => {
                // Check left/right movement keys
                // is_key_down returns true if the key is being held
                match (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right)) {
                    (true, false) => {
                        // Only Left key is down
                        *player_direction = MoveDirection::Left; // Set facing direction
                        player.velocity.x = -PLAYER_MOVEMENT_SPEED; // Move left
                    }
                    (false, true) => {
                        // Only Right key is down
                        *player_direction = MoveDirection::Right; // Set facing direction
                        player.velocity.x = PLAYER_MOVEMENT_SPEED; // Move right
                    }
                    _ => {
                        // Neither key or both keys are down (underscore is a catch-all pattern)
                        player.velocity.x = 0.0; // Stop horizontal movement
                    }
                };
                
                // Check for jump (Up key)
                // Only allow jumping if on ground (vertical velocity = 0)
                if is_key_pressed(KeyCode::Up) && player.velocity.y == 0.0 {
                    // Set upward velocity to jump
                    player.velocity.y = -PLAYER_JUMP_SPEED;
                    // Add jump event to trigger sound effect
                    events.push(Event::Jumped);
                }
            }
            
            // On the game over screen
            State::GameOver(_) => {
                // Check for restart (R key)
                if is_key_pressed(KeyCode::R) {
                    self.new_game(); // Start a new game
                }
            }
        }
        
        // Return any events that were triggered
        events
    }

    /// Updates the game state for a single frame.
    /// 
    /// This method:
    /// 1. Handles physics (gravity, movement, collisions)
    /// 2. Checks win/lose conditions
    /// 3. Returns events triggered during the update
    /// 
    /// Parameters:
    ///   - delta_time: Time in seconds since the last frame
    /// 
    /// Returns:
    ///   A Vec<Event> containing any events triggered during the update
    pub fn update(&mut self, delta_time: f32) -> Vec<Event> {
        // Create an empty vector for events
        let mut events: Vec<Event> = vec![];
        
        // Use pattern matching to extract all the game entities
        // The 'else' clause returns early if not in the Game state
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
            return events; // Return empty events if not in Game state
        };
        
        // --- Update Physics and Movement ---
        {
            // Apply gravity to player's vertical velocity
            player.velocity.y += GRAVITY * delta_time;

            // --- Platform Collision Detection (Ground Check) ---
            // Find the first platform the player might land on
            // find_map transforms items and returns the first non-None result
            let ground_collision = platforms.iter().find_map(|platform| {
                // Check horizontal overlap between player and platform
                let horizontally_overlapping = player.entity.rect.right() > platform.rect.x
                    && player.entity.rect.x < platform.rect.right();

                // Check if player is moving downward or stationary
                let falling_towards_platform = player.velocity.y >= 0.0;
                
                // Check if player's bottom is near platform's top
                let close_to_platform_top =
                    player.entity.rect.bottom() <= platform.rect.y + GROUND_DETECTION_BUFFER;
                    
                // Predict if player will intersect platform next frame
                let will_intersect_next_frame =
                    player.entity.rect.bottom() + player.velocity.y * delta_time >= platform.rect.y;

                // If all conditions met, return platform's top Y position
                if horizontally_overlapping
                    && falling_towards_platform
                    && close_to_platform_top
                    && will_intersect_next_frame
                {
                    Some(platform.rect.y) // Return the platform's top Y-coordinate
                } else {
                    None // No collision with this platform
                }
            });

            // Update player position based on velocity and time
            player.apply_velocity(delta_time);

            // --- Handle Ground Collision Response ---
            // If we found a platform the player landed on...
            if let Some(platform_top) = ground_collision {
                // Place player on top of platform
                player.entity.rect.y = platform_top - player.entity.rect.h;
                // Stop vertical movement
                player.velocity.y = 0.0;
            }

            // --- Update Chicken Movement ---
            for chicken in chickens.iter_mut() {
                // Move chickens according to their velocity
                chicken.apply_velocity(delta_time);
                
                // Bounce chickens off horizontal boundaries
                if chicken.entity.rect.x > 5000.0 || chicken.entity.rect.x < 0.0 {
                    // Reverse horizontal velocity to bounce
                    chicken.velocity.x = -chicken.velocity.x;
                }
                
                // Bounce chickens off vertical boundaries
                if chicken.entity.rect.y > 800.0 || chicken.entity.rect.y < 0.0 {
                    // Reverse vertical velocity to bounce
                    chicken.velocity.y = -chicken.velocity.y;
                }
            }

            // --- Update Cloud Movement ---
            for cloud in clouds {
                // Move clouds horizontally
                cloud.apply_velocity(delta_time);
                
                // Wrap clouds that move too far right back to the left
                if cloud.entity.rect.x > 60000.0 {
                    cloud.entity.rect.x = -1024.0; // Reset to far left
                }
            }
        }

        // --- Check Collisions and Game Logic ---
        {
            // --- Check Player Falling Off Screen ---
            if player.entity.rect.bottom() > screen_height() + 100.0 {
                // Player fell too far - end game with death
                events.push(Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Fall,
                    score: *score,
                }));
                
                // Update game state to GameOver
                *self = State::GameOver(GameOverReason::Death {
                    cause: DeathCause::Fall,
                    score: *score,
                });
                
                return events; // Exit early
            }

            // --- Egg Collection ---
            // retain keeps elements for which the closure returns true
            eggs.retain(|egg| {
                // Check collision between player and egg
                let collided = player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&egg.get_collision_bounds());
                    
                if collided {
                    *score += 1; // Increase score
                    events.push(Event::Scored); // Add score event
                }
                
                // Keep egg if NOT collided, remove if collided
                !collided
            });

            // --- Chicken Collision ---
            // Check if player hit any chicken
            if chickens.iter().any(|chicken| {
                // 'any' returns true if the closure is true for any element
                player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&chicken.entity.get_collision_bounds())
            }) {
                // Game over due to chicken collision
                events.push(Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Chicken,
                    score: *score,
                }));
                
                // Update game state
                *self = State::GameOver(GameOverReason::Death {
                    cause: DeathCause::Chicken,
                    score: *score,
                });
                
                return events; // Exit early
            }

            // --- Spike Collision ---
            // Check if player hit any spike
            if spikes.iter().any(|spike| {
                player
                    .entity
                    .get_collision_bounds()
                    .overlaps(&spike.get_collision_bounds())
            }) {
                // Game over due to spike collision
                events.push(Event::GameOver(GameOverReason::Death {
                    cause: DeathCause::Spike,
                    score: *score,
                }));
                
                // Update game state
                *self = State::GameOver(GameOverReason::Death {
                    cause: DeathCause::Spike,
                    score: *score,
                });
                
                return events; // Exit early
            }

            // --- House Collision (End/Win Condition) ---
            // Check if player reached the house
            if player
                .entity
                .get_collision_bounds()
                .overlaps(&house.get_collision_bounds())
            {
                // Complete win if player has enough eggs
                if *score >= EGGS_NEEDED_FOR_WIN {
                    events.push(Event::GameOver(GameOverReason::Win));
                    *self = State::GameOver(GameOverReason::Win);
                } 
                // Partial win (meme ending) if some eggs but not enough for full win
                else if *score >= EGGS_NEEDED_FOR_HOUSE {
                    // Choose a random meme ending (0-7)
                    let meme = gen_range(0, 8);
                    events.push(Event::GameOver(GameOverReason::End { meme }));
                    *self = State::GameOver(GameOverReason::End { meme });
                }
                // If player has fewer eggs than needed for house, nothing happens
            }
        }
        
        // Return all events that occurred during update
        events
    }
}