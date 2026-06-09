from PIL import Image, ImageDraw, ImageFont
import os

# WhatsApp color scheme from App.css
TEAL = (18, 140, 126)      # #128c7e
TEAL_DARK = (7, 94, 84)    # #075e54
LIGHT = (220, 248, 198)    # #dcf8c6
WHITE = (255, 255, 255)

def create_header_image():
    """Create 150x57px header image with gradient"""
    width, height = 150, 57
    img = Image.new('RGB', (width, height), TEAL_DARK)
    draw = ImageDraw.Draw(img)
    
    # Add gradient effect
    for y in range(height):
        ratio = y / height
        r = int(TEAL_DARK[0] + (TEAL[0] - TEAL_DARK[0]) * ratio)
        g = int(TEAL_DARK[1] + (TEAL[1] - TEAL_DARK[1]) * ratio)
        b = int(TEAL_DARK[2] + (TEAL[2] - TEAL_DARK[2]) * ratio)
        draw.rectangle([(0, y), (width, y+1)], fill=(r, g, b))
    
    # Add subtle accent line at bottom
    draw.rectangle([(0, height-3), (width, height)], fill=TEAL)
    
    return img

def create_sidebar_image():
    """Create 164x314px sidebar image with WhatsApp branding"""
    width, height = 164, 314
    img = Image.new('RGB', (width, height), TEAL_DARK)
    draw = ImageDraw.Draw(img)
    
    # Gradient from top to bottom
    for y in range(height):
        ratio = y / height
        r = int(TEAL[0] + (TEAL_DARK[0] - TEAL[0]) * ratio)
        g = int(TEAL[1] + (TEAL_DARK[1] - TEAL[1]) * ratio)
        b = int(TEAL[2] + (TEAL_DARK[2] - TEAL[2]) * ratio)
        draw.rectangle([(0, y), (width, y+1)], fill=(r, g, b))
    
    # Add decorative elements
    # Top accent
    draw.rectangle([(0, 0), (width, 8)], fill=TEAL)
    
    # Bottom accent
    draw.rectangle([(0, height-8), (width, height)], fill=TEAL)
    
    # Add subtle chat bubble pattern
    for i in range(3):
        y_pos = 60 + i * 80
        # Message bubble
        draw.rounded_rectangle([(20, y_pos), (width-20, y_pos+30)], 
                               radius=8, fill=(30, 120, 110))
        # Smaller reply bubble
        draw.rounded_rectangle([(40, y_pos+40), (width-30, y_pos+60)], 
                               radius=6, fill=(40, 130, 120))
    
    return img

def main():
    output_dir = os.path.join(os.path.dirname(__file__), 'icons')
    os.makedirs(output_dir, exist_ok=True)
    
    # Create header image
    header = create_header_image()
    header_path = os.path.join(output_dir, 'installer-header.bmp')
    header.save(header_path, 'BMP')
    print(f'Created: {header_path}')
    
    # Create sidebar image
    sidebar = create_sidebar_image()
    sidebar_path = os.path.join(output_dir, 'installer-sidebar.bmp')
    sidebar.save(sidebar_path, 'BMP')
    print(f'Created: {sidebar_path}')
    
    # Create uninstaller header (same as installer header)
    uninstaller_header = create_header_image()
    uninstaller_path = os.path.join(output_dir, 'uninstaller-header.bmp')
    uninstaller_header.save(uninstaller_path, 'BMP')
    print(f'Created: {uninstaller_path}')
    
    print('\nInstaller images created successfully!')

if __name__ == '__main__':
    main()
