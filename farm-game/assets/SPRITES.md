# Custom Sprites Guide

The game supports custom PNG sprites with automatic fallback to procedural generation.

## How It Works

The client tries to load sprites from the `assets/` directory. If a sprite file is not found, it automatically generates a procedural sprite with appropriate colors.

## Sprite Specifications

- **Format:** PNG
- **Size:** 32x32 pixels recommended
- **Transparency:** Supported (alpha channel)

## Required Sprite Files

Place your custom sprites in the `farm-game/assets/` directory:

### Tiles
- `grass.png` - Green grass tile
- `soil.png` - Brown tilled soil
- `watered_soil.png` - Darker brown watered soil

### Wheat Crop (Yellow/Golden)
- `wheat_seed.png` - Just planted
- `wheat_sprout.png` - Small sprout
- `wheat_growing.png` - Growing plant
- `wheat_mature.png` - Ready to harvest

### Carrot Crop (Orange)
- `carrot_seed.png` - Just planted
- `carrot_sprout.png` - Small sprout
- `carrot_growing.png` - Growing plant
- `carrot_mature.png` - Ready to harvest (visible orange)

### Tomato Crop (Red)
- `tomato_seed.png` - Just planted
- `tomato_sprout.png` - Small sprout
- `tomato_growing.png` - Growing plant with green tomatoes
- `tomato_mature.png` - Ready to harvest (red tomatoes)

### Player
- `player.png` - Player character sprite

## Creating Sprites

You can create sprites using any image editor:
- **GIMP** (Free, open-source)
- **Aseprite** (Pixel art editor, paid)
- **Piskel** (Free, web-based)
- **Photoshop** or **Krita**

## Quick Start with Procedural Sprites

Don't have custom sprites? No problem! The game works out-of-the-box with procedurally generated sprites that have appropriate colors:

- Grass: Green (#33cc33)
- Soil: Brown (#66502e)
- Watered Soil: Dark brown (#4d3319)
- Crops: Various colors based on type and growth stage
- Player: Blue (#4d4dcc)

## Example: Adding a Custom Grass Sprite

1. Create a 32x32 PNG image of grass
2. Save it as `grass.png`
3. Place it in `farm-game/assets/`
4. Run the client - it will automatically load your custom sprite!

## Tips for Good Sprites

- Use a consistent color palette across all sprites
- Keep the style simple and readable at 32x32
- Use transparency for irregular shapes
- Test different growth stages to ensure visual progression
- Consider adding subtle details (texture, shading)

## Pixel Art Resources

- [Lospec Palette List](https://lospec.com/palette-list) - Color palettes for pixel art
- [OpenGameArt](https://opengameart.org/) - Free game art assets
- [Kenney Assets](https://kenney.nl/assets) - Free game assets

## Example Sprite Generator Script

See `generate_example_sprites.py` for a simple Python script that generates basic example sprites to get you started.
