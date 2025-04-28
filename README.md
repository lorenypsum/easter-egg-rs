# EasterEgg: Haskell Platformer Game
A Haskell platformer game inspired by the tutorial found at [Arcade Academy Platform Tutorial](https://api.arcade.academy/en/latest/examples/platform_tutorial/index.html), which is part of the Python Arcade 2.6.17 Tutorial. In the game, players control a rabbit named Cueio Lalao who steals eggs from hens. Once discovered, Cueio Lalao must evade the furious hens while safeguarding the stolen eggs throughout the journey.

<p align="center">
  <img src="gif-easter-egg.gif" alt="EasterEgg">
</p>

# Asset's Artists
- [Cup Nooble](https://cupnooble.itch.io/)
- [Gat de Sucre](https://gatdesucre.itch.io/sugarland-tileset)
- [Qunoot Art](https://qunootart.itch.io/candy-land)
- [Cania East](https://caniaeast.itch.io/simple-sky-pixel-backgrounds)

# Building a Platformer Game from Scratch

This tutorial guides you through creating a simple platformer game in Rust using the Macroquad library, starting from a basic "Hello World" and building up to the full game.

## Step 1: Hello World

Start with a basic Macroquad application that displays text on screen:

```rust
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Hello World".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(WHITE);
        
        draw_text("Hello, World!", 
                 screen_width() / 2.0 - 100.0, 
                 screen_height() / 2.0, 
                 40.0, 
                 DARKGRAY);
        
        next_frame().await;
    }
}
```

## Step 2: Drawing a Simple Platform

Next, add a basic platform to the screen:

```rust
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(SKYBLUE);
        
        // Draw a platform at the bottom of the screen
        draw_rectangle(
            100.0,             // x position
            screen_height() - 50.0, // y position (near bottom)
            600.0,             // width
            20.0,              // height
            DARKBROWN          // color
        );
        
        next_frame().await;
    }
}
```

## Step 3: Adding a Player Character

Now, add a simple player character using a rectangle:

```rust
use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Player".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Player position
    let mut player_pos = Vec2::new(
        screen_width() / 2.0, 
        screen_height() - 100.0
    );
    
    loop {
        clear_background(SKYBLUE);
        
        // Draw platform
        draw_rectangle(
            100.0,
            screen_height() - 50.0,
            600.0,
            20.0,
            DARKBROWN
        );
        
        // Draw player
        draw_rectangle(
            player_pos.x - PLAYER_SIZE.x / 2.0,
            player_pos.y - PLAYER_SIZE.y,
            PLAYER_SIZE.x,
            PLAYER_SIZE.y,
            RED
        );
        
        next_frame().await;
    }
}
```

## Step 4: Player Movement (Left and Right)

Add horizontal movement controls using the arrow keys:

```rust
use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Moving Player".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player_pos = Vec2::new(
        screen_width() / 2.0, 
        screen_height() - 100.0
    );
    
    loop {
        let delta_time = get_frame_time();
        
        // Handle player movement
        if is_key_down(KeyCode::Left) {
            player_pos.x -= PLAYER_MOVEMENT_SPEED * delta_time;
        }
        if is_key_down(KeyCode::Right) {
            player_pos.x += PLAYER_MOVEMENT_SPEED * delta_time;
        }
        
        clear_background(SKYBLUE);
        
        // Draw platform
        draw_rectangle(
            100.0,
            screen_height() - 50.0,
            600.0,
            20.0,
            DARKBROWN
        );
        
        // Draw player
        draw_rectangle(
            player_pos.x - PLAYER_SIZE.x / 2.0,
            player_pos.y - PLAYER_SIZE.y,
            PLAYER_SIZE.x,
            PLAYER_SIZE.y,
            RED
        );
        
        next_frame().await;
    }
}
```

## Step 5: Adding Gravity and Jumping

Implement gravity and jumping mechanics:

```rust
use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;
const PLAYER_JUMP_SPEED: f32 = 500.0;
const GRAVITY: f32 = 1000.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Jumping Player".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut player_pos = Vec2::new(
        screen_width() / 2.0, 
        screen_height() - 100.0
    );
    let mut player_velocity = Vec2::new(0.0, 0.0);
    let mut is_grounded = false;
    
    loop {
        let delta_time = get_frame_time();
        
        // Handle player movement
        if is_key_down(KeyCode::Left) {
            player_velocity.x = -PLAYER_MOVEMENT_SPEED;
        } else if is_key_down(KeyCode::Right) {
            player_velocity.x = PLAYER_MOVEMENT_SPEED;
        } else {
            player_velocity.x = 0.0;
        }
        
        // Handle jumping
        if is_key_pressed(KeyCode::Up) && is_grounded {
            player_velocity.y = -PLAYER_JUMP_SPEED;
            is_grounded = false;
        }
        
        // Apply gravity
        player_velocity.y += GRAVITY * delta_time;
        
        // Update position
        player_pos += player_velocity * delta_time;
        
        // Platform collision (basic)
        let platform_top = screen_height() - 50.0;
        if player_pos.y >= platform_top && player_velocity.y > 0.0 &&
           player_pos.x >= 100.0 && player_pos.x <= 700.0 {
            player_pos.y = platform_top;
            player_velocity.y = 0.0;
            is_grounded = true;
        } else {
            is_grounded = false;
        }
        
        clear_background(SKYBLUE);
        
        // Draw platform
        draw_rectangle(
            100.0,
            platform_top,
            600.0,
            20.0,
            DARKBROWN
        );
        
        // Draw player
        draw_rectangle(
            player_pos.x - PLAYER_SIZE.x / 2.0,
            player_pos.y - PLAYER_SIZE.y,
            PLAYER_SIZE.x,
            PLAYER_SIZE.y,
            RED
        );
        
        next_frame().await;
    }
}
```

## Step 6: Creating Proper Game Entities

Organize code with proper structures for game entities:

```rust
use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;
const PLAYER_JUMP_SPEED: f32 = 500.0;
const GRAVITY: f32 = 1000.0;

struct GameEntity {
    rect: Rect,
}

struct MovingGameEntity {
    entity: GameEntity,
    velocity: Vec2,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Game Entities".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Create player
    let mut player = MovingGameEntity {
        entity: GameEntity {
            rect: Rect::new(
                screen_width() / 2.0 - PLAYER_SIZE.x / 2.0,
                screen_height() - 100.0 - PLAYER_SIZE.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y
            ),
        },
        velocity: Vec2::ZERO,
    };
    
    // Create platform
    let platform = GameEntity {
        rect: Rect::new(
            100.0,
            screen_height() - 50.0,
            600.0,
            20.0
        ),
    };
    
    let mut is_grounded = false;
    
    loop {
        let delta_time = get_frame_time();
        
        // Handle player movement
        if is_key_down(KeyCode::Left) {
            player.velocity.x = -PLAYER_MOVEMENT_SPEED;
        } else if is_key_down(KeyCode::Right) {
            player.velocity.x = PLAYER_MOVEMENT_SPEED;
        } else {
            player.velocity.x = 0.0;
        }
        
        // Handle jumping
        if is_key_pressed(KeyCode::Up) && is_grounded {
            player.velocity.y = -PLAYER_JUMP_SPEED;
            is_grounded = false;
        }
        
        // Apply gravity
        player.velocity.y += GRAVITY * delta_time;
        
        // Update position
        player.entity.rect.x += player.velocity.x * delta_time;
        player.entity.rect.y += player.velocity.y * delta_time;
        
        // Platform collision
        if player.entity.rect.bottom() >= platform.rect.top() && 
           player.velocity.y > 0.0 &&
           player.entity.rect.right() >= platform.rect.left() && 
           player.entity.rect.left() <= platform.rect.right() {
            player.entity.rect.y = platform.rect.top() - player.entity.rect.h;
            player.velocity.y = 0.0;
            is_grounded = true;
        } else {
            is_grounded = false;
        }
        
        clear_background(SKYBLUE);
        
        // Draw platform
        draw_rectangle(
            platform.rect.x,
            platform.rect.y,
            platform.rect.w,
            platform.rect.h,
            DARKBROWN
        );
        
        // Draw player
        draw_rectangle(
            player.entity.rect.x,
            player.entity.rect.y,
            player.entity.rect.w,
            player.entity.rect.h,
            RED
        );
        
        next_frame().await;
    }
}
```

## Step 7: Adding Multiple Platforms

Create multiple platforms for a more interesting level:

```rust
use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;
const PLAYER_JUMP_SPEED: f32 = 500.0;
const GRAVITY: f32 = 1000.0;

struct GameEntity {
    rect: Rect,
}

struct MovingGameEntity {
    entity: GameEntity,
    velocity: Vec2,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Multiple Platforms".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Create player
    let mut player = MovingGameEntity {
        entity: GameEntity {
            rect: Rect::new(
                screen_width() / 2.0 - PLAYER_SIZE.x / 2.0,
                screen_height() - 100.0 - PLAYER_SIZE.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y
            ),
        },
        velocity: Vec2::ZERO,
    };
    
    // Create multiple platforms
    let platforms = vec![
        // Ground platform
        GameEntity {
            rect: Rect::new(0.0, screen_height() - 50.0, screen_width(), 50.0),
        },
        // Floating platforms
        GameEntity {
            rect: Rect::new(100.0, 400.0, 200.0, 20.0),
        },
        GameEntity {
            rect: Rect::new(400.0, 300.0, 200.0, 20.0),
        },
        GameEntity {
            rect: Rect::new(200.0, 200.0, 200.0, 20.0),
        },
    ];
    
    let mut is_grounded = false;
    
    loop {
        let delta_time = get_frame_time();
        
        // Handle player movement
        if is_key_down(KeyCode::Left) {
            player.velocity.x = -PLAYER_MOVEMENT_SPEED;
        } else if is_key_down(KeyCode::Right) {
            player.velocity.x = PLAYER_MOVEMENT_SPEED;
        } else {
            player.velocity.x = 0.0;
        }
        
        // Handle jumping
        if is_key_pressed(KeyCode::Up) && is_grounded {
            player.velocity.y = -PLAYER_JUMP_SPEED;
            is_grounded = false;
        }
        
        // Apply gravity
        player.velocity.y += GRAVITY * delta_time;
        
        // Update position
        player.entity.rect.x += player.velocity.x * delta_time;
        player.entity.rect.y += player.velocity.y * delta_time;
        
        // Platform collision
        is_grounded = false;
        for platform in &platforms {
            if player.entity.rect.bottom() >= platform.rect.top() && 
               player.entity.rect.bottom() <= platform.rect.top() + 10.0 &&
               player.velocity.y > 0.0 &&
               player.entity.rect.right() >= platform.rect.left() && 
               player.entity.rect.left() <= platform.rect.right() {
                player.entity.rect.y = platform.rect.top() - player.entity.rect.h;
                player.velocity.y = 0.0;
                is_grounded = true;
                break;
            }
        }
        
        clear_background(SKYBLUE);
        
        // Draw platforms
        for platform in &platforms {
            draw_rectangle(
                platform.rect.x,
                platform.rect.y,
                platform.rect.w,
                platform.rect.h,
                DARKBROWN
            );
        }
        
        // Draw player
        draw_rectangle(
            player.entity.rect.x,
            player.entity.rect.y,
            player.entity.rect.w,
            player.entity.rect.h,
            RED
        );
        
        next_frame().await;
    }
}
```

## Step 8: Adding Collectible Eggs

Add collectible eggs that increase the player's score:

```rust
use macroquad::prelude::*;

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;
const PLAYER_JUMP_SPEED: f32 = 500.0;
const GRAVITY: f32 = 1000.0;
const EGG_SIZE: Vec2 = Vec2::new(20.0, 20.0);

struct GameEntity {
    rect: Rect,
}

impl GameEntity {
    fn overlaps(&self, other: &GameEntity) -> bool {
        self.rect.overlaps(&other.rect)
    }
}

struct MovingGameEntity {
    entity: GameEntity,
    velocity: Vec2,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Collectible Eggs".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Create player
    let mut player = MovingGameEntity {
        entity: GameEntity {
            rect: Rect::new(
                screen_width() / 2.0 - PLAYER_SIZE.x / 2.0,
                screen_height() - 100.0 - PLAYER_SIZE.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y
            ),
        },
        velocity: Vec2::ZERO,
    };
    
    // Create platforms
    let platforms = vec![
        GameEntity {
            rect: Rect::new(0.0, screen_height() - 50.0, screen_width(), 50.0),
        },
        GameEntity {
            rect: Rect::new(100.0, 400.0, 200.0, 20.0),
        },
        GameEntity {
            rect: Rect::new(400.0, 300.0, 200.0, 20.0),
        },
        GameEntity {
            rect: Rect::new(200.0, 200.0, 200.0, 20.0),
        },
    ];
    
    // Create eggs on platforms
    let mut eggs = vec![
        GameEntity {
            rect: Rect::new(150.0, 380.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
        },
        GameEntity {
            rect: Rect::new(450.0, 280.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
        },
        GameEntity {
            rect: Rect::new(250.0, 180.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
        },
    ];
    
    let mut is_grounded = false;
    let mut score = 0;
    
    loop {
        let delta_time = get_frame_time();
        
        // Handle player movement
        if is_key_down(KeyCode::Left) {
            player.velocity.x = -PLAYER_MOVEMENT_SPEED;
        } else if is_key_down(KeyCode::Right) {
            player.velocity.x = PLAYER_MOVEMENT_SPEED;
        } else {
            player.velocity.x = 0.0;
        }
        
        // Handle jumping
        if is_key_pressed(KeyCode::Up) && is_grounded {
            player.velocity.y = -PLAYER_JUMP_SPEED;
            is_grounded = false;
        }
        
        // Apply gravity
        player.velocity.y += GRAVITY * delta_time;
        
        // Update position
        player.entity.rect.x += player.velocity.x * delta_time;
        player.entity.rect.y += player.velocity.y * delta_time;
        
        // Platform collision
        is_grounded = false;
        for platform in &platforms {
            if player.entity.rect.bottom() >= platform.rect.top() && 
               player.entity.rect.bottom() <= platform.rect.top() + 10.0 &&
               player.velocity.y > 0.0 &&
               player.entity.rect.right() >= platform.rect.left() && 
               player.entity.rect.left() <= platform.rect.right() {
                player.entity.rect.y = platform.rect.top() - player.entity.rect.h;
                player.velocity.y = 0.0;
                is_grounded = true;
                break;
            }
        }
        
        // Egg collision and collection
        eggs.retain(|egg| {
            let collided = player.entity.overlaps(egg);
            if collided {
                score += 1;
            }
            !collided
        });
        
        clear_background(SKYBLUE);
        
        // Draw platforms
        for platform in &platforms {
            draw_rectangle(
                platform.rect.x,
                platform.rect.y,
                platform.rect.w,
                platform.rect.h,
                DARKBROWN
            );
        }
        
        // Draw eggs
        for egg in &eggs {
            draw_rectangle(
                egg.rect.x,
                egg.rect.y,
                egg.rect.w,
                egg.rect.h,
                YELLOW
            );
        }
        
        // Draw player
        draw_rectangle(
            player.entity.rect.x,
            player.entity.rect.y,
            player.entity.rect.w,
            player.entity.rect.h,
            RED
        );
        
        // Draw score
        draw_text(
            &format!("Score: {}", score),
            20.0,
            30.0,
            30.0,
            WHITE
        );
        
        next_frame().await;
    }
}
```

## Step 9: Adding Enemies (Chickens)

Add moving enemies (chickens) that cause game over when touched:

```rust
use macroquad::prelude::*;
use macroquad::rand::{gen_range};

const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 48.0);
const PLAYER_MOVEMENT_SPEED: f32 = 300.0;
const PLAYER_JUMP_SPEED: f32 = 500.0;
const GRAVITY: f32 = 1000.0;
const EGG_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const CHICKEN_SIZE: Vec2 = Vec2::new(40.0, 40.0);

struct GameEntity {
    rect: Rect,
}

impl GameEntity {
    fn overlaps(&self, other: &GameEntity) -> bool {
        self.rect.overlaps(&other.rect)
    }
}

struct MovingGameEntity {
    entity: GameEntity,
    velocity: Vec2,
}

enum GameState {
    Playing,
    GameOver,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Chickens".to_owned(),
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game_state = GameState::Playing;
    
    // Create player
    let mut player = MovingGameEntity {
        entity: GameEntity {
            rect: Rect::new(
                screen_width() / 2.0 - PLAYER_SIZE.x / 2.0,
                screen_height() - 100.0 - PLAYER_SIZE.y,
                PLAYER_SIZE.x,
                PLAYER_SIZE.y
            ),
        },
        velocity: Vec2::ZERO,
    };
    
    // Create platforms
    let platforms = vec![
        GameEntity {
            rect: Rect::new(0.0, screen_height() - 50.0, screen_width(), 50.0),
        },
        GameEntity {
            rect: Rect::new(100.0, 400.0, 200.0, 20.0),
        },
        GameEntity {
            rect: Rect::new(400.0, 300.0, 200.0, 20.0),
        },
        GameEntity {
            rect: Rect::new(200.0, 200.0, 200.0, 20.0),
        },
    ];
    
    // Create eggs
    let mut eggs = vec![
        GameEntity {
            rect: Rect::new(150.0, 380.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
        },
        GameEntity {
            rect: Rect::new(450.0, 280.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
        },
        GameEntity {
            rect: Rect::new(250.0, 180.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
        },
    ];
    
    // Create chickens (enemies)
    let mut chickens = vec![
        MovingGameEntity {
            entity: GameEntity {
                rect: Rect::new(600.0, 200.0, CHICKEN_SIZE.x, CHICKEN_SIZE.y),
            },
            velocity: Vec2::new(-50.0, 0.0),
        },
        MovingGameEntity {
            entity: GameEntity {
                rect: Rect::new(100.0, 100.0, CHICKEN_SIZE.x, CHICKEN_SIZE.y),
            },
            velocity: Vec2::new(40.0, 0.0),
        },
    ];
    
    let mut is_grounded = false;
    let mut score = 0;
    
    loop {
        match game_state {
            GameState::Playing => {
                let delta_time = get_frame_time();
                
                // Handle player movement
                if is_key_down(KeyCode::Left) {
                    player.velocity.x = -PLAYER_MOVEMENT_SPEED;
                } else if is_key_down(KeyCode::Right) {
                    player.velocity.x = PLAYER_MOVEMENT_SPEED;
                } else {
                    player.velocity.x = 0.0;
                }
                
                // Handle jumping
                if is_key_pressed(KeyCode::Up) && is_grounded {
                    player.velocity.y = -PLAYER_JUMP_SPEED;
                    is_grounded = false;
                }
                
                // Apply gravity
                player.velocity.y += GRAVITY * delta_time;
                
                // Update player position
                player.entity.rect.x += player.velocity.x * delta_time;
                player.entity.rect.y += player.velocity.y * delta_time;
                
                // Update chicken positions
                for chicken in &mut chickens {
                    // Move chickens
                    chicken.entity.rect.x += chicken.velocity.x * delta_time;
                    
                    // Reverse direction at screen edges
                    if chicken.entity.rect.right() > screen_width() || chicken.entity.rect.left() < 0.0 {
                        chicken.velocity.x = -chicken.velocity.x;
                    }
                }
                
                // Platform collision
                is_grounded = false;
                for platform in &platforms {
                    if player.entity.rect.bottom() >= platform.rect.top() && 
                       player.entity.rect.bottom() <= platform.rect.top() + 10.0 &&
                       player.velocity.y > 0.0 &&
                       player.entity.rect.right() >= platform.rect.left() && 
                       player.entity.rect.left() <= platform.rect.right() {
                        player.entity.rect.y = platform.rect.top() - player.entity.rect.h;
                        player.velocity.y = 0.0;
                        is_grounded = true;
                        break;
                    }
                }
                
                // Egg collision
                eggs.retain(|egg| {
                    let collided = player.entity.overlaps(egg);
                    if collided {
                        score += 1;
                    }
                    !collided
                });
                
                // Check chicken collision
                if chickens.iter().any(|chicken| player.entity.overlaps(&chicken.entity)) {
                    game_state = GameState::GameOver;
                }
                
                // Check falling off screen
                if player.entity.rect.top() > screen_height() {
                    game_state = GameState::GameOver;
                }
                
                clear_background(SKYBLUE);
                
                // Draw platforms
                for platform in &platforms {
                    draw_rectangle(
                        platform.rect.x,
                        platform.rect.y,
                        platform.rect.w,
                        platform.rect.h,
                        DARKBROWN
                    );
                }
                
                // Draw eggs
                for egg in &eggs {
                    draw_rectangle(
                        egg.rect.x,
                        egg.rect.y,
                        egg.rect.w,
                        egg.rect.h,
                        YELLOW
                    );
                }
                
                // Draw chickens
                for chicken in &chickens {
                    draw_rectangle(
                        chicken.entity.rect.x,
                        chicken.entity.rect.y,
                        chicken.entity.rect.w,
                        chicken.entity.rect.h,
                        MAGENTA
                    );
                }
                
                // Draw player
                draw_rectangle(
                    player.entity.rect.x,
                    player.entity.rect.y,
                    player.entity.rect.w,
                    player.entity.rect.h,
                    RED
                );
                
                // Draw score
                draw_text(
                    &format!("Score: {}", score),
                    20.0,
                    30.0,
                    30.0,
                    WHITE
                );
            },
            GameState::GameOver => {
                clear_background(DARKGRAY);
                
                draw_text(
                    &format!("Game Over! Final Score: {}", score),
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0,
                    40.0,
                    WHITE
                );
                
                draw_text(
                    "Press R to restart",
                    screen_width() / 2.0 - 120.0,
                    screen_height() / 2.0 + 50.0,
                    30.0,
                    WHITE
                );
                
                // Restart game
                if is_key_pressed(KeyCode::R) {
                    // Reset player
                    player.entity.rect.x = screen_width() / 2.0 - PLAYER_SIZE.x / 2.0;
                    player.entity.rect.y = screen_height() - 100.0 - PLAYER_SIZE.y;
                    player.velocity = Vec2::ZERO;
                    
                    // Reset game state
                    is_grounded = false;
                    score = 0;
                    game_state = GameState::Playing;
                    
                    // Reset eggs
                    eggs = vec![
                        GameEntity {
                            rect: Rect::new(150.0, 380.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
                        },
                        GameEntity {
                            rect: Rect::new(450.0, 280.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
                        },
                        GameEntity {
                            rect: Rect::new(250.0, 180.0 - EGG_SIZE.y, EGG_SIZE.x, EGG_SIZE.y),
                        },
                    ];
                }
            }
        }
        
        next_frame().await;
    }
}
```

## Step 10: Expanding to Current Game

From here, you can continue expanding the game by:

1. **Adding Textures**: Replace rectangles with proper sprites
2. **Adding Sound Effects**: For jumping, collecting eggs, collisions
3. **Adding a Camera**: To follow the player in larger levels
4. **Creating a Start Screen**: With instructions
5. **Adding Spikes**: As additional hazards
6. **Creating the House**: As the end goal
7. **Improving Game States**: For different ending conditions (win vs lose)

The final game in the repository builds on all these concepts to create a complete experience with:
- Multiple game states (start, playing, game over, win)
- Scrolling camera following the player
- Sound effects and music
- Animated sprites
- Score tracking
- Multiple hazards and obstacles
- Win condition (collecting enough eggs and reaching the house)

To reach the current implementation, continue expanding the code structure, refining collision detection, adding visual effects, and enhancing game mechanics to create an engaging platformer experience.

