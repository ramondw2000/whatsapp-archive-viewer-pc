#!/usr/bin/env python3
"""
Generate a fun WhatsApp archive viewer app icon
"""

from PIL import Image, ImageDraw, ImageFont
import os

def create_icon(size=512):
    """Create a fun WhatsApp-themed app icon"""
    
    # Create image with WhatsApp green background
    img = Image.new('RGB', (size, size), '#25D366')
    draw = ImageDraw.Draw(img)
    
    # Draw phone outline
    margin = size // 8
    phone_rect = [
        margin, margin, 
        size - margin, size - margin
    ]
    draw.rectangle(phone_rect, outline='white', width=size//64)
    
    # Draw chat bubbles
    bubble_size = size // 8
    chat_bubble1 = [
        margin * 2, margin * 2,
        margin * 2 + bubble_size * 2, margin * 2 + bubble_size
    ]
    chat_bubble2 = [
        margin * 2, margin * 3 + bubble_size // 2,
        margin * 2 + bubble_size * 1.5, margin * 3 + bubble_size * 1.5
    ]
    draw.rectangle(chat_bubble1, fill='white')
    draw.rectangle(chat_bubble2, fill='white')
    
    # Draw archive symbol in center
    center = size // 2
    archive_size = size // 6
    archive_rect = [
        center - archive_size // 2, center - archive_size // 3,
        center + archive_size // 2, center + archive_size // 2
    ]
    draw.rectangle(archive_rect, fill='#25D366', outline='white', width=size//128)
    
    # Draw archive handle
    handle_rect = [
        center - archive_size // 4, center - archive_size // 4,
        center + archive_size // 4, center - archive_size // 6
    ]
    draw.rectangle(handle_rect, fill='white')
    
    # Add fun emoji dots in corners
    emoji_positions = [
        (size // 6, size // 6),
        (size * 5 // 6, size // 6),
        (size // 6, size * 5 // 6),
        (size * 5 // 6, size * 5 // 6)
    ]
    
    colors = ['#FFD93D', '#6BCF7F', '#FF6B6B', '#4ECDC4']
    
    for i, (x, y) in enumerate(emoji_positions):
        # Draw colored circle
        draw.ellipse([x-size//32, y-size//32, x+size//32, y+size//32], fill=colors[i])
        
        # Draw simple smiley face
        eye_size = size // 256
        # Eyes
        draw.ellipse([x-size//48, y-size//48, x-size//48+eye_size, y-size//48+eye_size], fill='black')
        draw.ellipse([x+size//48-eye_size, y-size//48, x+size//48, y-size//48+eye_size], fill='black')
        # Smile
        draw.arc([x-size//48, y-size//96, x+size//48, y+size//48], 0, 180, fill='black', width=size//256)
    
    return img

def generate_all_sizes():
    """Generate all required icon sizes"""
    
    sizes = [
        (32, '32x32.png'),
        (128, '128x128.png'),
        (256, '128x128@2x.png'),
        (512, 'icon.png')
    ]
    
    # Create icons directory if it doesn't exist
    os.makedirs('icons', exist_ok=True)
    
    for size, filename in sizes:
        icon = create_icon(size)
        icon.save(f'icons/{filename}')
        print(f"Generated {filename}")
    
    # Generate ICO and ICNS (requires additional tools, but we can try)
    try:
        icon_512 = create_icon(512)
        icon_512.save('icons/icon.ico', format='ICO', sizes=[(32,32), (64,64), (128,128), (256,256)])
        print("Generated icon.ico")
    except Exception as e:
        print(f"Could not generate ICO: {e}")
    
    try:
        icon_512 = create_icon(512)
        icon_512.save('icons/icon.icns', format='ICNS')
        print("Generated icon.icns")
    except Exception as e:
        print(f"Could not generate ICNS: {e}")

if __name__ == '__main__':
    generate_all_sizes()
    print("Icon generation complete!")
