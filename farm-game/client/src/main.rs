use macroquad::prelude::*;
use shared::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:7878";
const PLAYER_SPEED: f32 = 150.0;

struct Sprites {
    grass: Image,
    soil: Image,
    watered_soil: Image,
    wheat_seed: Image,
    wheat_sprout: Image,
    wheat_growing: Image,
    wheat_mature: Image,
    carrot_seed: Image,
    carrot_sprout: Image,
    carrot_growing: Image,
    carrot_mature: Image,
    tomato_seed: Image,
    tomato_sprout: Image,
    tomato_growing: Image,
    tomato_mature: Image,
    player: Image,
}

impl Sprites {
    fn new() -> Self {
        Self {
            grass: Self::load_or_generate("assets/grass.png", |_| Color::new(0.2, 0.6, 0.2, 1.0)),
            soil: Self::load_or_generate("assets/soil.png", |_| Color::new(0.4, 0.3, 0.2, 1.0)),
            watered_soil: Self::load_or_generate("assets/watered_soil.png", |_| {
                Color::new(0.3, 0.2, 0.1, 1.0)
            }),
            wheat_seed: Self::load_or_generate("assets/wheat_seed.png", |_| {
                Color::new(0.8, 0.7, 0.5, 1.0)
            }),
            wheat_sprout: Self::load_or_generate("assets/wheat_sprout.png", |_| {
                Color::new(0.5, 0.7, 0.3, 1.0)
            }),
            wheat_growing: Self::load_or_generate("assets/wheat_growing.png", |_| {
                Color::new(0.7, 0.8, 0.4, 1.0)
            }),
            wheat_mature: Self::load_or_generate("assets/wheat_mature.png", |_| {
                Color::new(0.9, 0.8, 0.3, 1.0)
            }),
            carrot_seed: Self::load_or_generate("assets/carrot_seed.png", |_| {
                Color::new(0.8, 0.7, 0.5, 1.0)
            }),
            carrot_sprout: Self::load_or_generate("assets/carrot_sprout.png", |_| {
                Color::new(0.3, 0.6, 0.3, 1.0)
            }),
            carrot_growing: Self::load_or_generate("assets/carrot_growing.png", |_| {
                Color::new(0.3, 0.7, 0.3, 1.0)
            }),
            carrot_mature: Self::load_or_generate("assets/carrot_mature.png", |_| {
                Color::new(0.9, 0.5, 0.2, 1.0)
            }),
            tomato_seed: Self::load_or_generate("assets/tomato_seed.png", |_| {
                Color::new(0.8, 0.7, 0.5, 1.0)
            }),
            tomato_sprout: Self::load_or_generate("assets/tomato_sprout.png", |_| {
                Color::new(0.3, 0.5, 0.3, 1.0)
            }),
            tomato_growing: Self::load_or_generate("assets/tomato_growing.png", |_| {
                Color::new(0.4, 0.6, 0.3, 1.0)
            }),
            tomato_mature: Self::load_or_generate("assets/tomato_mature.png", |_| {
                Color::new(0.8, 0.2, 0.2, 1.0)
            }),
            player: Self::load_or_generate("assets/player.png", |_| Color::new(0.3, 0.3, 0.8, 1.0)),
        }
    }

    fn load_or_generate<F>(path: &str, color_fn: F) -> Image
    where
        F: Fn(usize) -> Color,
    {
        // Try to load sprite from file
        if let Ok(img) = Image::from_file_with_format(path.as_bytes(), None) {
            return img;
        }

        // Generate procedural sprite
        let size = 32;
        let mut img = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));

        // Create a simple procedural pattern based on filename
        let hash = path.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32));
        let mut rng = ::rand::rngs::StdRng::seed_from_u64(hash as u64);

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) as usize;
                let color = color_fn(idx);

                // Add some procedural noise
                let noise = (rng.gen::<f32>() - 0.5) * 0.2;
                let r = (color.r + noise).clamp(0.0, 1.0);
                let g = (color.g + noise).clamp(0.0, 1.0);
                let b = (color.b + noise).clamp(0.0, 1.0);

                img.set_pixel(x as u32, y as u32, Color::new(r, g, b, color.a));
            }
        }

        img
    }

    fn get_crop_sprite(&self, crop: &Crop) -> &Image {
        match (crop.crop_type, crop.growth_stage) {
            (CropType::Wheat, GrowthStage::Seed) => &self.wheat_seed,
            (CropType::Wheat, GrowthStage::Sprout) => &self.wheat_sprout,
            (CropType::Wheat, GrowthStage::Growing) => &self.wheat_growing,
            (CropType::Wheat, GrowthStage::Mature) => &self.wheat_mature,
            (CropType::Carrot, GrowthStage::Seed) => &self.carrot_seed,
            (CropType::Carrot, GrowthStage::Sprout) => &self.carrot_sprout,
            (CropType::Carrot, GrowthStage::Growing) => &self.carrot_growing,
            (CropType::Carrot, GrowthStage::Mature) => &self.carrot_mature,
            (CropType::Tomato, GrowthStage::Seed) => &self.tomato_seed,
            (CropType::Tomato, GrowthStage::Sprout) => &self.tomato_sprout,
            (CropType::Tomato, GrowthStage::Growing) => &self.tomato_growing,
            (CropType::Tomato, GrowthStage::Mature) => &self.tomato_mature,
        }
    }
}

use ::rand::{Rng, SeedableRng};

struct Client {
    stream: TcpStream,
    player_id: Option<u8>,
    game_state: Arc<Mutex<Option<GameState>>>,
    sprites: HashMap<String, Texture2D>,
}

impl Client {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(SERVER_ADDR)?;
        stream.set_nonblocking(true)?;

        Ok(Self {
            stream,
            player_id: None,
            game_state: Arc::new(Mutex::new(None)),
            sprites: HashMap::new(),
        })
    }

    fn send_message(&mut self, message: &ClientMessage) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(message)?;
        let len = (data.len() as u32).to_le_bytes();
        self.stream.write_all(&len)?;
        self.stream.write_all(&data)?;
        Ok(())
    }

    fn start_receiver(&self) {
        let mut stream = self.stream.try_clone().unwrap();
        let game_state = Arc::clone(&self.game_state);

        thread::spawn(move || loop {
            // Read message length
            let mut len_buf = [0u8; 4];
            match stream.read_exact(&mut len_buf) {
                Ok(_) => {
                    let len = u32::from_le_bytes(len_buf) as usize;
                    let mut buf = vec![0u8; len];

                    if stream.read_exact(&mut buf).is_ok() {
                        if let Ok(message) = bincode::deserialize::<ServerMessage>(&buf) {
                            Self::handle_server_message(message, &game_state);
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });
    }

    fn handle_server_message(message: ServerMessage, game_state: &Arc<Mutex<Option<GameState>>>) {
        match message {
            ServerMessage::Welcome { player_id } => {
                println!("Connected! Player ID: {}", player_id);
            }
            ServerMessage::GameState(state) => {
                let mut gs = game_state.lock().unwrap();
                *gs = Some(state);
            }
            ServerMessage::PlayerJoined { player } => {
                let mut gs = game_state.lock().unwrap();
                if let Some(ref mut state) = *gs {
                    state.players.push(player);
                }
            }
            ServerMessage::PlayerLeft { player_id } => {
                let mut gs = game_state.lock().unwrap();
                if let Some(ref mut state) = *gs {
                    state.players.retain(|p| p.id != player_id);
                }
            }
            ServerMessage::PlayerMoved { player_id, x, y } => {
                let mut gs = game_state.lock().unwrap();
                if let Some(ref mut state) = *gs {
                    if let Some(player) = state.players.iter_mut().find(|p| p.id == player_id) {
                        player.x = x;
                        player.y = y;
                    }
                }
            }
            ServerMessage::TileUpdated { x, y, tile } => {
                let mut gs = game_state.lock().unwrap();
                if let Some(ref mut state) = *gs {
                    if let Some(t) = state.get_tile_mut(x, y) {
                        *t = tile;
                    }
                }
            }
            ServerMessage::Error { message } => {
                eprintln!("Server error: {}", message);
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Farm Multiplayer Game".to_string(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut client = Client::new().expect("Failed to connect to server");
    client.start_receiver();

    // Get player name
    let player_name = format!("Player{}", ::rand::random::<u8>() % 100);
    client
        .send_message(&ClientMessage::Join {
            name: player_name.clone(),
        })
        .expect("Failed to send join message");

    // Wait for game state
    while client.game_state.lock().unwrap().is_none() {
        next_frame().await;
    }

    client.player_id = Some(0); // Will be updated by server

    // Load sprites
    let sprites = Sprites::new();
    let mut sprite_textures = HashMap::new();

    // Convert images to textures
    sprite_textures.insert("grass".to_string(), Texture2D::from_image(&sprites.grass));
    sprite_textures.insert("soil".to_string(), Texture2D::from_image(&sprites.soil));
    sprite_textures.insert(
        "watered_soil".to_string(),
        Texture2D::from_image(&sprites.watered_soil),
    );
    sprite_textures.insert("player".to_string(), Texture2D::from_image(&sprites.player));

    // Camera position
    let mut camera_x = 0.0;
    let mut camera_y = 0.0;

    let mut selected_crop = CropType::Wheat;

    loop {
        let dt = get_frame_time();

        // Get game state
        let game_state = client.game_state.lock().unwrap();
        if game_state.is_none() {
            next_frame().await;
            continue;
        }
        let state = game_state.as_ref().unwrap().clone();
        drop(game_state);

        // Find our player
        let my_player = state.players.iter().find(|p| p.name == player_name);

        if let Some(player) = my_player {
            // Handle input - movement
            let mut dx = 0.0;
            let mut dy = 0.0;

            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
                dy -= 1.0;
            }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
                dy += 1.0;
            }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
                dx -= 1.0;
            }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
                dx += 1.0;
            }

            // Normalize diagonal movement
            if dx != 0.0 && dy != 0.0 {
                dx *= 0.707;
                dy *= 0.707;
            }

            if dx != 0.0 || dy != 0.0 {
                let new_x = (player.x + dx * PLAYER_SPEED * dt)
                    .clamp(0.0, (WORLD_WIDTH as f32 - 1.0) * TILE_SIZE);
                let new_y = (player.y + dy * PLAYER_SPEED * dt)
                    .clamp(0.0, (WORLD_HEIGHT as f32 - 1.0) * TILE_SIZE);

                client
                    .send_message(&ClientMessage::Move { x: new_x, y: new_y })
                    .ok();
            }

            // Handle crop selection
            if is_key_pressed(KeyCode::Key1) {
                selected_crop = CropType::Wheat;
                client
                    .send_message(&ClientMessage::SelectCrop {
                        crop_type: CropType::Wheat,
                    })
                    .ok();
            }
            if is_key_pressed(KeyCode::Key2) {
                selected_crop = CropType::Carrot;
                client
                    .send_message(&ClientMessage::SelectCrop {
                        crop_type: CropType::Carrot,
                    })
                    .ok();
            }
            if is_key_pressed(KeyCode::Key3) {
                selected_crop = CropType::Tomato;
                client
                    .send_message(&ClientMessage::SelectCrop {
                        crop_type: CropType::Tomato,
                    })
                    .ok();
            }

            // Update camera to follow player
            camera_x = player.x;
            camera_y = player.y;

            // Handle mouse clicks for tile interactions
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mouse_x, mouse_y) = mouse_position();
                let world_x = mouse_x + camera_x - screen_width() / 2.0;
                let world_y = mouse_y + camera_y - screen_height() / 2.0;
                let tile_x = (world_x / TILE_SIZE) as usize;
                let tile_y = (world_y / TILE_SIZE) as usize;

                if tile_x < WORLD_WIDTH && tile_y < WORLD_HEIGHT {
                    if let Some(tile) = state.get_tile(tile_x, tile_y) {
                        // Harvest if crop is mature
                        if let Some(crop) = tile.crop {
                            if crop.is_harvestable() {
                                client
                                    .send_message(&ClientMessage::Harvest {
                                        x: tile_x,
                                        y: tile_y,
                                    })
                                    .ok();
                            }
                        } else if tile.tile_type == TileType::Soil
                            || tile.tile_type == TileType::WateredSoil
                        {
                            // Plant crop
                            client
                                .send_message(&ClientMessage::Plant {
                                    x: tile_x,
                                    y: tile_y,
                                    crop_type: selected_crop,
                                })
                                .ok();
                        } else if tile.tile_type == TileType::Grass {
                            // Till the soil
                            client
                                .send_message(&ClientMessage::Till {
                                    x: tile_x,
                                    y: tile_y,
                                })
                                .ok();
                        }
                    }
                }
            }

            // Handle right click for watering
            if is_mouse_button_pressed(MouseButton::Right) {
                let (mouse_x, mouse_y) = mouse_position();
                let world_x = mouse_x + camera_x - screen_width() / 2.0;
                let world_y = mouse_y + camera_y - screen_height() / 2.0;
                let tile_x = (world_x / TILE_SIZE) as usize;
                let tile_y = (world_y / TILE_SIZE) as usize;

                if tile_x < WORLD_WIDTH && tile_y < WORLD_HEIGHT {
                    client
                        .send_message(&ClientMessage::Water {
                            x: tile_x,
                            y: tile_y,
                        })
                        .ok();
                }
            }
        }

        // Rendering
        clear_background(BLACK);

        // Calculate viewport
        let offset_x = screen_width() / 2.0 - camera_x;
        let offset_y = screen_height() / 2.0 - camera_y;

        // Draw tiles
        for y in 0..WORLD_HEIGHT {
            for x in 0..WORLD_WIDTH {
                if let Some(tile) = state.get_tile(x, y) {
                    let screen_x = x as f32 * TILE_SIZE + offset_x;
                    let screen_y = y as f32 * TILE_SIZE + offset_y;

                    // Skip if off screen
                    if screen_x < -TILE_SIZE
                        || screen_x > screen_width()
                        || screen_y < -TILE_SIZE
                        || screen_y > screen_height()
                    {
                        continue;
                    }

                    // Draw tile
                    let tile_texture = match tile.tile_type {
                        TileType::Grass => sprite_textures.get("grass"),
                        TileType::Soil => sprite_textures.get("soil"),
                        TileType::WateredSoil => sprite_textures.get("watered_soil"),
                    };

                    if let Some(texture) = tile_texture {
                        draw_texture_ex(
                            texture,
                            screen_x,
                            screen_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                                ..Default::default()
                            },
                        );
                    }

                    // Draw crop
                    if let Some(crop) = tile.crop {
                        let crop_img = sprites.get_crop_sprite(&crop);
                        let crop_texture = Texture2D::from_image(crop_img);
                        draw_texture_ex(
                            &crop_texture,
                            screen_x,
                            screen_y,
                            WHITE,
                            DrawTextureParams {
                                dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
        }

        // Draw players
        for player in &state.players {
            let screen_x = player.x + offset_x;
            let screen_y = player.y + offset_y;

            if let Some(texture) = sprite_textures.get("player") {
                draw_texture_ex(
                    texture,
                    screen_x - TILE_SIZE / 2.0,
                    screen_y - TILE_SIZE / 2.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
                        ..Default::default()
                    },
                );
            }

            // Draw player name
            let name_width = measure_text(&player.name, None, 16, 1.0).width;
            draw_text(
                &player.name,
                screen_x - name_width / 2.0,
                screen_y - TILE_SIZE,
                16.0,
                WHITE,
            );
        }

        // Draw UI
        draw_text(
            &format!("Selected Crop: {:?} (1/2/3 to change)", selected_crop),
            10.0,
            30.0,
            20.0,
            WHITE,
        );
        draw_text(
            "WASD: Move | Left Click: Till/Plant/Harvest | Right Click: Water",
            10.0,
            screen_height() - 10.0,
            16.0,
            WHITE,
        );
        draw_text(
            &format!("Players: {}/{}", state.players.len(), MAX_PLAYERS),
            10.0,
            60.0,
            20.0,
            WHITE,
        );

        next_frame().await;
    }
}
