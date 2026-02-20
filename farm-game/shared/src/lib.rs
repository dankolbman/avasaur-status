use serde::{Deserialize, Serialize};

/// Maximum number of simultaneous players
pub const MAX_PLAYERS: usize = 8;

/// World dimensions
pub const WORLD_WIDTH: usize = 32;
pub const WORLD_HEIGHT: usize = 32;

/// Tile size in pixels
pub const TILE_SIZE: f32 = 32.0;

/// Game tick rate (ticks per second)
pub const TICK_RATE: f32 = 20.0;

/// Crop growth time in ticks
pub const CROP_GROWTH_TIME: u32 = 200; // ~10 seconds at 20 ticks/sec

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    Soil,
    WateredSoil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    Wheat,
    Carrot,
    Tomato,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrowthStage {
    Seed,
    Sprout,
    Growing,
    Mature,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Crop {
    pub crop_type: CropType,
    pub growth_stage: GrowthStage,
    pub ticks_in_stage: u32,
}

impl Crop {
    pub fn new(crop_type: CropType) -> Self {
        Self {
            crop_type,
            growth_stage: GrowthStage::Seed,
            ticks_in_stage: 0,
        }
    }

    pub fn tick(&mut self, is_watered: bool) {
        if !is_watered {
            return;
        }

        self.ticks_in_stage += 1;
        let stage_duration = CROP_GROWTH_TIME / 4; // 4 growth stages

        if self.ticks_in_stage >= stage_duration {
            self.ticks_in_stage = 0;
            self.growth_stage = match self.growth_stage {
                GrowthStage::Seed => GrowthStage::Sprout,
                GrowthStage::Sprout => GrowthStage::Growing,
                GrowthStage::Growing => GrowthStage::Mature,
                GrowthStage::Mature => GrowthStage::Mature, // Stay mature
            };
        }
    }

    pub fn is_harvestable(&self) -> bool {
        self.growth_stage == GrowthStage::Mature
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub crop: Option<Crop>,
}

impl Tile {
    pub fn new(tile_type: TileType) -> Self {
        Self {
            tile_type,
            crop: None,
        }
    }

    pub fn tick(&mut self) {
        // Dry out watered soil
        if self.tile_type == TileType::WateredSoil && self.crop.is_none() {
            self.tile_type = TileType::Soil;
        }

        // Tick crop growth
        if let Some(ref mut crop) = self.crop {
            crop.tick(self.tile_type == TileType::WateredSoil);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u8,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub selected_crop: CropType,
}

impl Player {
    pub fn new(id: u8, name: String) -> Self {
        Self {
            id,
            name,
            x: (WORLD_WIDTH / 2) as f32 * TILE_SIZE,
            y: (WORLD_HEIGHT / 2) as f32 * TILE_SIZE,
            selected_crop: CropType::Wheat,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tiles: Vec<Vec<Tile>>,
    pub players: Vec<Player>,
}

impl GameState {
    pub fn new() -> Self {
        let mut tiles = Vec::with_capacity(WORLD_HEIGHT);
        for _ in 0..WORLD_HEIGHT {
            let mut row = Vec::with_capacity(WORLD_WIDTH);
            for _ in 0..WORLD_WIDTH {
                row.push(Tile::new(TileType::Grass));
            }
            tiles.push(row);
        }

        Self {
            tiles,
            players: Vec::new(),
        }
    }

    pub fn tick(&mut self) {
        for row in &mut self.tiles {
            for tile in row {
                tile.tick();
            }
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }

    pub fn get_tile_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(y).and_then(|row| row.get_mut(x))
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

/// Network messages between client and server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join { name: String },
    Move { x: f32, y: f32 },
    Till { x: usize, y: usize },
    Water { x: usize, y: usize },
    Plant { x: usize, y: usize, crop_type: CropType },
    Harvest { x: usize, y: usize },
    SelectCrop { crop_type: CropType },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome { player_id: u8 },
    GameState(GameState),
    PlayerJoined { player: Player },
    PlayerLeft { player_id: u8 },
    PlayerMoved { player_id: u8, x: f32, y: f32 },
    TileUpdated { x: usize, y: usize, tile: Tile },
    Error { message: String },
}
