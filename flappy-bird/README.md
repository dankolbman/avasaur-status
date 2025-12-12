# Flappy Bird in Rust

A simple Flappy Bird clone written in Rust using the Macroquad game library.

## Features

- Classic Flappy Bird gameplay
- Simple graphics using basic shapes
- Score tracking
- Game states (menu, playing, game over)
- Physics-based bird movement
- Randomly generated pipe obstacles

## How to Play

1. Press **SPACE** to start the game from the menu
2. Press **SPACE** to make the bird jump
3. Navigate through the pipes without hitting them
4. Try to get the highest score possible!
5. When you die, press **SPACE** to restart

## Building and Running

Make sure you have Rust installed. Then run:

```bash
# Build and run in debug mode
cargo run

# Build and run in release mode (better performance)
cargo run --release
```

## Controls

- **SPACE** - Jump / Start / Restart

## Game Mechanics

- The bird constantly falls due to gravity
- Each jump gives the bird an upward velocity
- Score increases by 1 for each pipe successfully passed
- The game ends if the bird hits a pipe or goes off-screen

## Dependencies

- **macroquad** - Simple and easy-to-use game library for Rust
