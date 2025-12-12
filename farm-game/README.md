# Farm Multiplayer Game

A multiplayer farming simulator built in Rust, inspired by Stardew Valley.

## Features

- **Client-server multiplayer** architecture supporting up to 8 simultaneous players
- **Tile-based farming mechanics** with tilling, planting, watering, and harvesting
- **Automatic state persistence** - game state is saved every 10 seconds and on server shutdown for hot-reloading during development
- **Custom sprite support** with procedural fallback generation
- **Three crop types** (Wheat, Carrot, Tomato) with four growth stages each
- **Real-time multiplayer** synchronization of player movements and farm changes

## Architecture

The project is organized as a Rust workspace with three crates:

- **`shared/`** - Common data structures and game logic
- **`server/`** - Multiplayer server with TCP networking and state persistence
- **`client/`** - Game client with rendering using macroquad

## Prerequisites

- Rust (latest stable version)
- Cargo

## Building

Build all crates:

```bash
cd farm-game
cargo build --release
```

## Running

### Start the Server

In one terminal:

```bash
cargo run --release --bin server
```

The server will:
- Start listening on `127.0.0.1:7878`
- Load game state from `game_state.bin` if it exists
- Auto-save every 10 seconds
- Save state on graceful shutdown (Ctrl+C)

### Start the Client(s)

In separate terminal(s):

```bash
cargo run --release --bin client
```

You can run up to 8 clients simultaneously to test multiplayer functionality.

## Controls

### Movement
- **WASD** or **Arrow Keys** - Move your character

### Farming Actions
- **Left Click** on grass - Till the soil
- **Left Click** on soil - Plant selected crop
- **Left Click** on mature crop - Harvest
- **Right Click** on soil - Water the soil

### Crop Selection
- **1** - Select Wheat
- **2** - Select Carrot
- **3** - Select Tomato

## Gameplay

1. **Till the soil** - Click on grass tiles to convert them to soil
2. **Water the soil** - Right-click soil to make it watered (darker brown)
3. **Plant crops** - Select a crop type (1/2/3) and click on watered or dry soil
4. **Wait for growth** - Crops grow through 4 stages (Seed → Sprout → Growing → Mature)
5. **Harvest** - Click on mature crops to harvest them

**Note:** Crops only grow on watered soil! Make sure to water your plants.

## Game Mechanics

- **World Size:** 32x32 tiles
- **Crop Growth:** ~10 seconds per crop (4 stages, each ~2.5 seconds when watered)
- **Auto-save:** Every 10 seconds
- **Max Players:** 8 simultaneous players

### Tile Types
- **Grass** (green) - Default state, needs tilling
- **Soil** (brown) - Ready for planting, but crops won't grow
- **Watered Soil** (dark brown) - Crops grow here

### Crop Types
- **Wheat** (yellow when mature)
- **Carrot** (orange when mature)
- **Tomato** (red when mature)

## Development Features

### State Persistence

The server automatically saves the game state to `game_state.bin`:
- Every 10 seconds (auto-save)
- On graceful shutdown (Ctrl+C)

This enables **hot-reloading**: You can restart the server and all farm progress is preserved, making development much faster.

### Custom Sprites

Place custom sprites in `farm-game/assets/` with these filenames:
- `grass.png`
- `soil.png`
- `watered_soil.png`
- `wheat_seed.png`, `wheat_sprout.png`, `wheat_growing.png`, `wheat_mature.png`
- `carrot_seed.png`, `carrot_sprout.png`, `carrot_growing.png`, `carrot_mature.png`
- `tomato_seed.png`, `tomato_sprout.png`, `tomato_growing.png`, `tomato_mature.png`
- `player.png`

If sprites are not found, the game will automatically generate **procedural sprites** with appropriate colors, so the game works out-of-the-box without any assets.

## Project Structure

```
farm-game/
├── Cargo.toml          # Workspace configuration
├── shared/             # Shared game logic
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs      # Data structures, game state, messages
├── server/             # Game server
│   ├── Cargo.toml
│   └── src/
│       └── main.rs     # TCP server, state persistence, game loop
├── client/             # Game client
│   ├── Cargo.toml
│   └── src/
│       └── main.rs     # Rendering, input, networking
└── README.md
```

## Technical Details

### Networking

- Protocol: TCP with length-prefixed binary messages
- Serialization: bincode (fast binary serialization)
- Server Address: `127.0.0.1:7878`

### Server Features

- Multi-threaded client handling
- Game loop running at 20 ticks/second
- Automatic state serialization/deserialization
- Graceful shutdown with Ctrl+C handler
- Player limit enforcement (max 8)

### Client Features

- Asynchronous rendering with macroquad
- Camera following player
- Procedural sprite generation as fallback
- Real-time multiplayer synchronization
- Player name tags

## Dependencies

- **macroquad** (0.4) - Game framework and rendering
- **serde** (1.0) - Serialization framework
- **bincode** (1.3) - Binary serialization
- **rand** (0.8) - Random number generation
- **ctrlc** (3.4) - Graceful shutdown handling

## Future Enhancements

Possible additions:
- Player inventory system
- More crop varieties
- Day/night cycle
- Weather system
- Tool upgrades
- Multiplayer chat
- Sound effects and music

## License

This project is open source and available under the MIT License.

## Credits

Created as a Rust multiplayer farming game inspired by Stardew Valley.
