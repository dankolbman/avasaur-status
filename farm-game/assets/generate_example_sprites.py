#!/usr/bin/env python3
"""
Simple sprite generator for the Farm Multiplayer Game.
Generates basic 32x32 PNG sprites with solid colors and simple patterns.

Requirements: pip install pillow
Usage: python3 generate_example_sprites.py
"""

from PIL import Image, ImageDraw
import os

SPRITE_SIZE = 32

def create_sprite(color, pattern="solid"):
    """Create a simple sprite with the given color and pattern."""
    img = Image.new('RGBA', (SPRITE_SIZE, SPRITE_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    if pattern == "solid":
        draw.rectangle([0, 0, SPRITE_SIZE-1, SPRITE_SIZE-1], fill=color)

    elif pattern == "grass":
        # Green background with darker grass blades
        draw.rectangle([0, 0, SPRITE_SIZE-1, SPRITE_SIZE-1], fill=color)
        dark_green = tuple(max(0, c - 40) if i < 3 else c for i, c in enumerate(color))
        # Add some grass blade patterns
        for i in range(0, SPRITE_SIZE, 6):
            for j in range(0, SPRITE_SIZE, 6):
                draw.line([(i+2, j+4), (i+2, j)], fill=dark_green, width=1)
                draw.line([(i+4, j+5), (i+4, j+1)], fill=dark_green, width=1)

    elif pattern == "soil":
        # Brown with some texture
        draw.rectangle([0, 0, SPRITE_SIZE-1, SPRITE_SIZE-1], fill=color)
        dark_brown = tuple(max(0, c - 20) if i < 3 else c for i, c in enumerate(color))
        # Add soil texture dots
        for i in range(0, SPRITE_SIZE, 4):
            for j in range(0, SPRITE_SIZE, 4):
                if (i + j) % 8 == 0:
                    draw.point((i+1, j+1), fill=dark_brown)

    elif pattern == "seed":
        # Small brown dot in center
        draw.ellipse([14, 14, 18, 18], fill=(139, 115, 85, 255))

    elif pattern == "sprout":
        # Small green stem
        draw.rectangle([15, 20, 17, 28], fill=(100, 200, 100, 255))
        draw.ellipse([14, 18, 18, 22], fill=(120, 220, 120, 255))

    elif pattern == "growing":
        # Larger plant
        draw.rectangle([14, 16, 16, 28], fill=(80, 180, 80, 255))
        draw.rectangle([17, 18, 19, 28], fill=(80, 180, 80, 255))
        draw.ellipse([12, 14, 20, 20], fill=(100, 200, 100, 255))

    elif pattern == "wheat_mature":
        # Golden wheat
        draw.rectangle([14, 20, 16, 28], fill=(180, 140, 60, 255))
        draw.ellipse([10, 12, 22, 20], fill=(230, 200, 80, 255))
        draw.ellipse([11, 14, 21, 22], fill=(240, 210, 90, 255))

    elif pattern == "carrot_mature":
        # Green top, orange carrot
        draw.ellipse([14, 12, 18, 16], fill=(100, 200, 100, 255))
        draw.polygon([(16, 16), (13, 24), (19, 24)], fill=(255, 140, 60, 255))

    elif pattern == "tomato_mature":
        # Tomato with green stem
        draw.ellipse([10, 16, 22, 26], fill=(220, 60, 60, 255))
        draw.rectangle([15, 14, 17, 18], fill=(100, 180, 100, 255))
        draw.ellipse([14, 14, 18, 16], fill=(120, 200, 120, 255))

    elif pattern == "player":
        # Simple player character
        # Head
        draw.ellipse([10, 8, 22, 20], fill=color)
        # Body
        draw.rectangle([12, 20, 20, 26], fill=color)
        # Arms
        draw.rectangle([8, 20, 12, 24], fill=color)
        draw.rectangle([20, 20, 24, 24], fill=color)
        # Legs
        draw.rectangle([12, 26, 15, 30], fill=color)
        draw.rectangle([17, 26, 20, 30], fill=color)

    return img

def generate_all_sprites():
    """Generate all sprites for the farm game."""
    sprites = {
        # Tiles
        "grass.png": ((51, 153, 51, 255), "grass"),
        "soil.png": ((102, 76, 51, 255), "soil"),
        "watered_soil.png": ((77, 51, 25, 255), "soil"),

        # Wheat
        "wheat_seed.png": ((204, 178, 128, 255), "seed"),
        "wheat_sprout.png": ((128, 178, 76, 255), "sprout"),
        "wheat_growing.png": ((178, 204, 102, 255), "growing"),
        "wheat_mature.png": ((230, 204, 76, 255), "wheat_mature"),

        # Carrot
        "carrot_seed.png": ((204, 178, 128, 255), "seed"),
        "carrot_sprout.png": ((76, 153, 76, 255), "sprout"),
        "carrot_growing.png": ((76, 178, 76, 255), "growing"),
        "carrot_mature.png": ((230, 128, 51, 255), "carrot_mature"),

        # Tomato
        "tomato_seed.png": ((204, 178, 128, 255), "seed"),
        "tomato_sprout.png": ((76, 128, 76, 255), "sprout"),
        "tomato_growing.png": ((102, 153, 76, 255), "growing"),
        "tomato_mature.png": ((204, 51, 51, 255), "tomato_mature"),

        # Player
        "player.png": ((76, 76, 204, 255), "player"),
    }

    print("Generating example sprites...")
    for filename, (color, pattern) in sprites.items():
        sprite = create_sprite(color, pattern)
        sprite.save(filename)
        print(f"  Created: {filename}")

    print(f"\n✓ Generated {len(sprites)} sprites!")
    print("Place these in farm-game/assets/ to use them in the game.")

if __name__ == "__main__":
    # Check if PIL is available
    try:
        from PIL import Image
    except ImportError:
        print("Error: PIL (Pillow) is not installed.")
        print("Install it with: pip install pillow")
        exit(1)

    generate_all_sprites()
