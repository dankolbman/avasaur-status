use shared::*;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SERVER_ADDR: &str = "127.0.0.1:7878";
const STATE_FILE: &str = "game_state.bin";

type ClientId = u8;

struct Server {
    game_state: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<ClientId, TcpStream>>>,
    next_player_id: Arc<Mutex<u8>>,
}

impl Server {
    fn new() -> Self {
        let game_state = Self::load_state().unwrap_or_else(|_| {
            println!("Creating new game state...");
            GameState::new()
        });

        Self {
            game_state: Arc::new(Mutex::new(game_state)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_player_id: Arc::new(Mutex::new(0)),
        }
    }

    fn load_state() -> Result<GameState, Box<dyn std::error::Error>> {
        println!("Loading game state from {}...", STATE_FILE);
        let data = fs::read(STATE_FILE)?;
        let state: GameState = bincode::deserialize(&data)?;
        println!("Game state loaded successfully!");
        Ok(state)
    }

    fn save_state(state: &GameState) -> Result<(), Box<dyn std::error::Error>> {
        let data = bincode::serialize(state)?;
        fs::write(STATE_FILE, data)?;
        Ok(())
    }

    fn broadcast_message(&self, message: &ServerMessage, except: Option<ClientId>) {
        let clients = self.clients.lock().unwrap();
        let data = bincode::serialize(message).unwrap();
        let len = (data.len() as u32).to_le_bytes();

        for (&client_id, client) in clients.iter() {
            if Some(client_id) == except {
                continue;
            }
            let mut stream = client.try_clone().unwrap();
            let _ = stream.write_all(&len);
            let _ = stream.write_all(&data);
        }
    }

    fn send_message(&self, client_id: ClientId, message: &ServerMessage) {
        let clients = self.clients.lock().unwrap();
        if let Some(client) = clients.get(&client_id) {
            let data = bincode::serialize(message).unwrap();
            let len = (data.len() as u32).to_le_bytes();
            let mut stream = client.try_clone().unwrap();
            let _ = stream.write_all(&len);
            let _ = stream.write_all(&data);
        }
    }

    fn handle_client_message(&self, client_id: ClientId, message: ClientMessage) {
        let mut state = self.game_state.lock().unwrap();

        match message {
            ClientMessage::Join { name } => {
                if state.players.len() >= MAX_PLAYERS {
                    self.send_message(
                        client_id,
                        &ServerMessage::Error {
                            message: "Server is full".to_string(),
                        },
                    );
                    return;
                }

                let player = Player::new(client_id, name);
                let player_name = player.name.clone();
                state.players.push(player.clone());

                self.send_message(client_id, &ServerMessage::Welcome { player_id: client_id });
                self.send_message(client_id, &ServerMessage::GameState(state.clone()));
                self.broadcast_message(&ServerMessage::PlayerJoined { player }, Some(client_id));

                println!("Player {} joined (ID: {})", player_name, client_id);
            }
            ClientMessage::Move { x, y } => {
                if let Some(player) = state.players.iter_mut().find(|p| p.id == client_id) {
                    player.x = x;
                    player.y = y;
                    self.broadcast_message(
                        &ServerMessage::PlayerMoved {
                            player_id: client_id,
                            x,
                            y,
                        },
                        Some(client_id),
                    );
                }
            }
            ClientMessage::Till { x, y } => {
                if let Some(tile) = state.get_tile_mut(x, y) {
                    if tile.tile_type == TileType::Grass {
                        tile.tile_type = TileType::Soil;
                        self.broadcast_message(
                            &ServerMessage::TileUpdated { x, y, tile: *tile },
                            None,
                        );
                    }
                }
            }
            ClientMessage::Water { x, y } => {
                if let Some(tile) = state.get_tile_mut(x, y) {
                    if tile.tile_type == TileType::Soil {
                        tile.tile_type = TileType::WateredSoil;
                        self.broadcast_message(
                            &ServerMessage::TileUpdated { x, y, tile: *tile },
                            None,
                        );
                    }
                }
            }
            ClientMessage::Plant { x, y, crop_type } => {
                if let Some(tile) = state.get_tile_mut(x, y) {
                    if (tile.tile_type == TileType::Soil || tile.tile_type == TileType::WateredSoil)
                        && tile.crop.is_none()
                    {
                        tile.crop = Some(Crop::new(crop_type));
                        self.broadcast_message(
                            &ServerMessage::TileUpdated { x, y, tile: *tile },
                            None,
                        );
                    }
                }
            }
            ClientMessage::Harvest { x, y } => {
                if let Some(tile) = state.get_tile_mut(x, y) {
                    if let Some(crop) = tile.crop {
                        if crop.is_harvestable() {
                            tile.crop = None;
                            self.broadcast_message(
                                &ServerMessage::TileUpdated { x, y, tile: *tile },
                                None,
                            );
                        }
                    }
                }
            }
            ClientMessage::SelectCrop { crop_type } => {
                if let Some(player) = state.players.iter_mut().find(|p| p.id == client_id) {
                    player.selected_crop = crop_type;
                }
            }
        }
    }

    fn handle_client(self: Arc<Self>, mut stream: TcpStream, client_id: ClientId) {
        stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        // Add client to the list
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert(client_id, stream.try_clone().unwrap());
        }

        println!("Client {} connected", client_id);

        loop {
            // Read message length
            let mut len_buf = [0u8; 4];
            match stream.read_exact(&mut len_buf) {
                Ok(_) => {
                    let len = u32::from_le_bytes(len_buf) as usize;
                    let mut buf = vec![0u8; len];

                    match stream.read_exact(&mut buf) {
                        Ok(_) => {
                            if let Ok(message) = bincode::deserialize::<ClientMessage>(&buf) {
                                self.handle_client_message(client_id, message);
                            }
                        }
                        Err(e) => {
                            if e.kind() != std::io::ErrorKind::WouldBlock
                                && e.kind() != std::io::ErrorKind::TimedOut
                            {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock
                        && e.kind() != std::io::ErrorKind::TimedOut
                    {
                        break;
                    }
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        // Remove client
        {
            let mut clients = self.clients.lock().unwrap();
            clients.remove(&client_id);
        }

        // Remove player from game state
        {
            let mut state = self.game_state.lock().unwrap();
            state.players.retain(|p| p.id != client_id);
        }

        self.broadcast_message(&ServerMessage::PlayerLeft { player_id: client_id }, None);
        println!("Client {} disconnected", client_id);
    }

    fn run(self: Arc<Self>) {
        // Set up graceful shutdown
        let game_state_clone = Arc::clone(&self.game_state);
        ctrlc::set_handler(move || {
            println!("\nShutting down server...");
            let state = game_state_clone.lock().unwrap();
            if let Err(e) = Self::save_state(&state) {
                eprintln!("Failed to save state: {}", e);
            } else {
                println!("Game state saved successfully!");
            }
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");

        // Start game loop in a separate thread
        let game_state = Arc::clone(&self.game_state);
        thread::spawn(move || {
            let tick_duration = Duration::from_secs_f32(1.0 / TICK_RATE);
            let mut last_tick = Instant::now();
            let mut last_save = Instant::now();

            loop {
                let now = Instant::now();
                if now.duration_since(last_tick) >= tick_duration {
                    let mut state = game_state.lock().unwrap();
                    state.tick();
                    last_tick = now;
                }

                // Auto-save every 10 seconds
                if now.duration_since(last_save) >= Duration::from_secs(10) {
                    let state = game_state.lock().unwrap();
                    if let Err(e) = Self::save_state(&state) {
                        eprintln!("Failed to auto-save state: {}", e);
                    } else {
                        println!("Game state auto-saved");
                    }
                    last_save = now;
                }

                thread::sleep(Duration::from_millis(10));
            }
        });

        // Accept client connections
        let listener = TcpListener::bind(SERVER_ADDR).expect("Failed to bind server");
        println!("Server listening on {}", SERVER_ADDR);
        println!("Press Ctrl+C to save and exit");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client_id = {
                        let mut next_id = self.next_player_id.lock().unwrap();
                        let id = *next_id;
                        *next_id = (*next_id + 1) % (MAX_PLAYERS as u8);
                        id
                    };

                    let server = Arc::clone(&self);
                    thread::spawn(move || {
                        server.handle_client(stream, client_id);
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    }
}

fn main() {
    println!("=== Farm Multiplayer Game Server ===");
    let server = Arc::new(Server::new());
    server.run();
}
