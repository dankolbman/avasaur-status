# Farm Multiplayer Game - Complete Specification

Build a multiplayer farming simulator in Rust inspired by Stardew Valley, featuring:
- Client-server multiplayer architecture
- Tile-based farming mechanics
- **Automatic state persistence** for hot-reloading during development
- Custom sprite support with procedural fallback
- Up to 8 simultaneous players

## Technology Stack

### Core Dependencies
```toml
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
rand = "0.8"

[client dependencies]
macroquad = "0.4"           # Game framework and rendering

[server dependencies]
ctrlc = "3.4"               # Graceful shutdown handling
```

## Project Structure

Create a Rust workspace with three crates:

```
farm-game/
├── Cargo.toml              # Workspace configuration
├── shared/                 # Common game logic (library)
│   ├── Cargo.toml
│   └── src/lib.rs
├── server/                 # Game server (binary)
│   ├── Cargo.toml
│   └── src/main.rs
├── client/                 # Game client (binary)
│   ├── Cargo.toml
│   └── src/main.rs
└── assets/                 # Optional custom sprites
    └── *.png
```

## Game Constants

```rust
MAX_PLAYERS: 8
WORLD_WIDTH: 32 tiles
WORLD_HEIGHT: 32 tiles
TILE_SIZE: 32.0 pixels
TICK_RATE: 20.0 ticks/second
CROP_GROWTH_TIME: 200 ticks (~10 seconds)
SERVER_ADDR: "127.0.0.1:7878"
STATE_FILE: "game_state.bin"
```

## Shared Data Structures (shared/src/lib.rs)

### Enums

```rust
TileType:
  - Grass (default state)
  - Soil (tilled, ready for planting)
  - WateredSoil (darker, enables crop growth)

CropType:
  - Wheat (golden when mature)
  - Carrot (orange when mature)
  - Tomato (red when mature)

GrowthStage:
  - Seed (just planted)
  - Sprout (small plant)
  - Growing (larger plant)
  - Mature (ready to harvest)
```

### Structures

```rust
Crop {
  crop_type: CropType
  growth_stage: GrowthStage
  ticks_in_stage: u32
}

Methods:
  - new(crop_type) -> Self
  - tick(&mut self, is_watered: bool)
    * Only grows if is_watered is true
    * Increments ticks_in_stage
    * Advances growth_stage every CROP_GROWTH_TIME/4 ticks
  - is_harvestable(&self) -> bool
    * Returns true if growth_stage == Mature

Tile {
  tile_type: TileType
  crop: Option<Crop>
}

Methods:
  - new(tile_type) -> Self
  - tick(&mut self)
    * Dry out watered soil if no crop
    * Tick crop growth if present

Player {
  id: u8
  name: String
  x: f32
  y: f32
  selected_crop: CropType
}

Methods:
  - new(id, name) -> Self
    * Spawns at center of world (WORLD_WIDTH/2, WORLD_HEIGHT/2)
    * Default selected_crop: Wheat

GameState {
  tiles: Vec<Vec<Tile>>  // [y][x] indexed
  players: Vec<Player>
}

Methods:
  - new() -> Self
    * Initialize all tiles as Grass
    * Empty players vec
  - tick(&mut self)
    * Tick all tiles
  - get_tile(&self, x, y) -> Option<&Tile>
  - get_tile_mut(&mut self, x, y) -> Option<&mut Tile>
```

### Network Messages

```rust
ClientMessage (sent from client to server):
  - Join { name: String }
  - Move { x: f32, y: f32 }
  - Till { x: usize, y: usize }
  - Water { x: usize, y: usize }
  - Plant { x: usize, y: usize, crop_type: CropType }
  - Harvest { x: usize, y: usize }
  - SelectCrop { crop_type: CropType }

ServerMessage (sent from server to client):
  - Welcome { player_id: u8 }
  - GameState(GameState)
  - PlayerJoined { player: Player }
  - PlayerLeft { player_id: u8 }
  - PlayerMoved { player_id: u8, x: f32, y: f32 }
  - TileUpdated { x: usize, y: usize, tile: Tile }
  - Error { message: String }
```

All enums and structs must derive `Debug, Clone, Serialize, Deserialize`.

## Server Implementation (server/src/main.rs)

### Architecture

Use TCP sockets with multi-threaded client handling. Use `Arc<Mutex<>>` for shared state.

### Server Structure

```rust
Server {
  game_state: Arc<Mutex<GameState>>
  clients: Arc<Mutex<HashMap<u8, TcpStream>>>
  next_player_id: Arc<Mutex<u8>>
}
```

### Core Features

**1. State Persistence**
- Load game state from `game_state.bin` on startup (if exists)
- Auto-save every 10 seconds
- Save on graceful shutdown (Ctrl+C)
- Use bincode for serialization

**2. Game Loop**
- Run in separate thread
- Tick at TICK_RATE (20 Hz)
- Call `game_state.tick()` every tick
- Track time with `Instant::now()` and `Duration`

**3. Client Connection Handling**
- Spawn thread for each client connection
- Set read timeout to 100ms to prevent blocking
- Assign unique player ID (0-7, wrapping)
- Add to clients HashMap on connect
- Remove from HashMap and game state on disconnect
- Broadcast PlayerLeft message on disconnect

**4. Message Protocol**
- Length-prefixed messages: `[4 bytes length (u32 LE)][message bytes]`
- Use bincode for serialization
- Non-blocking reads with timeout handling

**5. Message Handling Logic**

```
Join:
  - Check if players.len() >= MAX_PLAYERS, send Error if full
  - Create new Player with client_id and name
  - Add to game_state.players
  - Send Welcome { player_id }
  - Send GameState (full state)
  - Broadcast PlayerJoined to other clients

Move:
  - Find player by id
  - Update player x, y
  - Broadcast PlayerMoved to other clients

Till:
  - Get tile at (x, y)
  - If TileType::Grass, change to Soil
  - Broadcast TileUpdated

Water:
  - Get tile at (x, y)
  - If TileType::Soil, change to WateredSoil
  - Broadcast TileUpdated

Plant:
  - Get tile at (x, y)
  - If tile is Soil or WateredSoil AND has no crop
  - Set tile.crop = Some(Crop::new(crop_type))
  - Broadcast TileUpdated

Harvest:
  - Get tile at (x, y)
  - If tile has crop AND crop.is_harvestable()
  - Set tile.crop = None
  - Broadcast TileUpdated

SelectCrop:
  - Find player by id
  - Update player.selected_crop
```

**6. Broadcasting**
- `broadcast_message(message, except_id)` - send to all except one
- `send_message(client_id, message)` - send to specific client

**7. Graceful Shutdown**
- Use ctrlc crate to handle Ctrl+C
- Save game state before exit
- Print "Game state saved successfully!"

## Client Implementation (client/src/main.rs)

### Architecture

Use macroquad for rendering, TCP socket for networking, separate thread for receiving messages.

### Window Configuration

```rust
window_title: "Farm Multiplayer Game"
window_width: 1024
window_height: 768
```

### Client Structure

```rust
Client {
  stream: TcpStream
  player_id: Option<u8>
  game_state: Arc<Mutex<Option<GameState>>>
  sprites: HashMap<String, Texture2D>  // Can be unused, keep for future
}
```

### Sprite System

**1. Sprite Loading with Procedural Fallback**

Create a `Sprites` struct to hold all sprite images:

```rust
Sprites {
  grass: Image
  soil: Image
  watered_soil: Image
  wheat_seed: Image
  wheat_sprout: Image
  wheat_growing: Image
  wheat_mature: Image
  carrot_seed: Image
  carrot_sprout: Image
  carrot_growing: Image
  carrot_mature: Image
  tomato_seed: Image
  tomato_sprout: Image
  tomato_growing: Image
  tomato_mature: Image
  player: Image
}
```

**2. Load or Generate Method**

```rust
fn load_or_generate<F>(path: &str, color_fn: F) -> Image
where F: Fn(usize) -> Color
{
  // Try to load from file
  if let Ok(img) = Image::from_file_with_format(path.as_bytes(), None) {
    return img;
  }

  // Generate procedural sprite
  let size = 32u16;
  let mut img = Image::gen_image_color(size, size, transparent);

  // Hash path for deterministic RNG
  let hash = path.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
  let mut rng = ::rand::rngs::StdRng::seed_from_u64(hash as u64);

  // Fill pixels with base color + noise
  for y in 0..size {
    for x in 0..size {
      let color = color_fn((y * size + x) as usize);
      let noise = (rng.gen::<f32>() - 0.5) * 0.2;
      let r = (color.r + noise).clamp(0.0, 1.0);
      let g = (color.g + noise).clamp(0.0, 1.0);
      let b = (color.b + noise).clamp(0.0, 1.0);
      img.set_pixel(x as u32, y as u32, Color::new(r, g, b, color.a));
    }
  }

  img
}
```

**3. Sprite Colors for Procedural Generation**

```rust
grass: Color::new(0.2, 0.6, 0.2, 1.0)      // Green
soil: Color::new(0.4, 0.3, 0.2, 1.0)       // Brown
watered_soil: Color::new(0.3, 0.2, 0.1, 1.0)  // Dark brown
wheat_seed: Color::new(0.8, 0.7, 0.5, 1.0)
wheat_sprout: Color::new(0.5, 0.7, 0.3, 1.0)
wheat_growing: Color::new(0.7, 0.8, 0.4, 1.0)
wheat_mature: Color::new(0.9, 0.8, 0.3, 1.0)  // Golden
carrot_seed: Color::new(0.8, 0.7, 0.5, 1.0)
carrot_sprout: Color::new(0.3, 0.6, 0.3, 1.0)
carrot_growing: Color::new(0.3, 0.7, 0.3, 1.0)
carrot_mature: Color::new(0.9, 0.5, 0.2, 1.0)  // Orange
tomato_seed: Color::new(0.8, 0.7, 0.5, 1.0)
tomato_sprout: Color::new(0.3, 0.5, 0.3, 1.0)
tomato_growing: Color::new(0.4, 0.6, 0.3, 1.0)
tomato_mature: Color::new(0.8, 0.2, 0.2, 1.0)  // Red
player: Color::new(0.3, 0.3, 0.8, 1.0)  // Blue
```

**4. Sprite Paths**

```
assets/grass.png
assets/soil.png
assets/watered_soil.png
assets/wheat_seed.png
assets/wheat_sprout.png
assets/wheat_growing.png
assets/wheat_mature.png
assets/carrot_seed.png
assets/carrot_sprout.png
assets/carrot_growing.png
assets/carrot_mature.png
assets/tomato_seed.png
assets/tomato_sprout.png
assets/tomato_growing.png
assets/tomato_mature.png
assets/player.png
```

### Networking

**1. Connection**
- Connect to SERVER_ADDR on startup
- Set stream to non-blocking
- Send Join message immediately
- Wait for game state before rendering

**2. Receiver Thread**
- Clone stream with `try_clone()`
- Loop forever reading messages
- Handle WouldBlock errors gracefully
- Update shared game_state based on messages

**3. Message Handling**

```
Welcome: Print player ID
GameState: Set game_state = Some(state)
PlayerJoined: Add player to state.players
PlayerLeft: Remove player from state.players
PlayerMoved: Update player position
TileUpdated: Update tile at (x, y)
Error: Print error message
```

### Input Handling

**1. Movement (every frame)**
```
WASD or Arrow Keys:
  - Calculate dx, dy (-1, 0, or 1)
  - Normalize diagonal movement (multiply by 0.707)
  - new_position = current + direction * PLAYER_SPEED * dt
  - Clamp to world bounds [0, WORLD_WIDTH*TILE_SIZE]
  - Send Move { x, y }
```

Player speed: `PLAYER_SPEED = 150.0` pixels/second

**2. Crop Selection**
```
Key 1: Select Wheat, send SelectCrop { Wheat }
Key 2: Select Carrot, send SelectCrop { Carrot }
Key 3: Select Tomato, send SelectCrop { Tomato }
```

**3. Mouse Interactions**

Convert mouse position to tile coordinates:
```rust
let (mouse_x, mouse_y) = mouse_position();
let world_x = mouse_x + camera_x - screen_width() / 2.0;
let world_y = mouse_y + camera_y - screen_height() / 2.0;
let tile_x = (world_x / TILE_SIZE) as usize;
let tile_y = (world_y / TILE_SIZE) as usize;
```

```
Left Click:
  - Get tile at (tile_x, tile_y)
  - If has mature crop: Send Harvest { x, y }
  - Else if Soil/WateredSoil with no crop: Send Plant { x, y, selected_crop }
  - Else if Grass: Send Till { x, y }

Right Click:
  - Send Water { x, y }
```

### Rendering

**1. Camera**
- Follow player: `camera_x = player.x`, `camera_y = player.y`
- Calculate viewport offset: `offset_x = screen_width()/2 - camera_x`

**2. Tile Rendering**
```
For each tile in world:
  - Calculate screen position: screen_x = tile_x * TILE_SIZE + offset_x
  - Skip if off-screen (optimization)
  - Draw tile sprite based on tile_type
  - If has crop, draw crop sprite on top based on type + growth_stage
```

Use `draw_texture_ex()` with dest_size of TILE_SIZE x TILE_SIZE.

**3. Player Rendering**
```
For each player:
  - Calculate screen position from player.x, player.y
  - Draw player sprite (centered: subtract TILE_SIZE/2)
  - Draw player name above sprite
```

Use `measure_text()` to center player names.

**4. UI Overlay**
```
Top-left:
  - "Selected Crop: {crop_type} (1/2/3 to change)"
  - "Players: {count}/{MAX_PLAYERS}"

Bottom:
  - "WASD: Move | Left Click: Till/Plant/Harvest | Right Click: Water"
```

Use `draw_text()` with WHITE color.

**5. Texture Management**

Convert sprites to textures in initialization:
```rust
let mut sprite_textures = HashMap::new();
sprite_textures.insert("grass", Texture2D::from_image(&sprites.grass));
sprite_textures.insert("soil", Texture2D::from_image(&sprites.soil));
// etc...
```

For crops, create textures on-the-fly during rendering:
```rust
let crop_img = sprites.get_crop_sprite(&crop);
let crop_texture = Texture2D::from_image(crop_img);
```

### Main Game Loop

```rust
#[macroquad::main(window_conf)]
async fn main() {
  // Connect to server
  // Start receiver thread
  // Send Join message
  // Wait for game state

  // Generate player name: format!("Player{}", rand::random::<u8>() % 100)

  // Load sprites
  // Convert base sprites to textures

  // Initialize camera position
  // Initialize selected_crop = Wheat

  loop {
    let dt = get_frame_time();

    // Lock and clone game state
    // Find our player by name

    // Handle input (movement, crop selection, mouse)
    // Update camera

    // Clear screen (BLACK)
    // Render tiles
    // Render crops
    // Render players
    // Render UI

    next_frame().await;
  }
}
```

## Additional Features

### Player Name Generation
Generate random player names: `Player0` through `Player99`

### State Synchronization
- Server is authoritative
- Clients send actions, server validates and broadcasts
- Clients update local state from server messages

### Error Handling
- Use `.ok()` to ignore network errors during gameplay
- Use `.expect()` for critical startup errors
- Print server errors to stderr

## Game Mechanics Summary

1. **Tilling**: Click grass to convert to soil
2. **Watering**: Right-click soil to water it (becomes darker)
3. **Planting**: Left-click watered/dry soil to plant selected crop
4. **Growth**: Crops only grow on watered soil, 4 stages, ~10 seconds total
5. **Harvesting**: Left-click mature crops to harvest (removes crop)
6. **Multiplayer**: Up to 8 players can farm together in real-time

## Implementation Notes

### Important Details

1. Use `::rand` prefix to disambiguate from macroquad's rand
2. Convert u16 to u32 for Image::set_pixel()
3. Clone player name before moving player in Join handler
4. Set TCP stream timeouts to prevent blocking
5. Use length-prefixed messages for TCP framing
6. Lock game state briefly, clone, then drop lock
7. Handle WouldBlock errors in network code
8. Clear background BLACK each frame
9. Use `#[macroquad::main(window_conf)]` for client
10. Derive all necessary traits for serde

### Testing

Run server first, then multiple clients:
```bash
# Terminal 1
cargo run --release --bin server

# Terminal 2
cargo run --release --bin client

# Terminal 3
cargo run --release --bin client
```

### Performance

- Viewport culling for off-screen tiles
- Tick rate: 20 Hz (reasonable for farm game)
- Binary serialization for efficiency
- Non-blocking I/O to prevent freezing

## Success Criteria

The implementation is complete when:
1. ✅ Server starts and loads/saves game state
2. ✅ Multiple clients can connect simultaneously (up to 8)
3. ✅ Players can move around smoothly
4. ✅ Farming mechanics work (till, water, plant, harvest)
5. ✅ Crops grow visually through 4 stages
6. ✅ Sprites load from files or generate procedurally
7. ✅ State persists across server restarts
8. ✅ All players see each other in real-time
9. ✅ UI shows current state clearly
10. ✅ Game is playable and fun!

## Optional Enhancements

Consider adding (not required):
- Player inventory system
- More crop varieties
- Day/night cycle
- Weather affecting growth
- Tool system (hoe, watering can)
- Chat system
- Sound effects
- Animations for actions
- Minimap
- Player customization

This specification is complete and ready for implementation. Good luck!
