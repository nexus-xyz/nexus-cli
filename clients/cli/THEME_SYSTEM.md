# 🎨 Nexus CLI Theme System

A comprehensive theme system for customizing the Nexus CLI appearance with built-in themes and easy rotation.

## ✨ Features

- **5 Built-in Themes**: Default, Cyberpunk Neon, Professional Blue, Retro Terminal, and Minimalist
- **Live Theme Rotation**: Press 'T' in TUI mode to cycle through themes
- **Dynamic Colors**: All UI elements adapt to the current theme
- **Theme Display**: Current theme name shown in the header
- **Easy Customization**: JSON-based theme configuration

## 🎯 Built-in Themes

### 1. **Default** 
- **Colors**: Deep sky blue primary, coral red secondary
- **Style**: Original Nexus CLI appearance
- **Best for**: Standard use, familiar interface

### 2. **Cyberpunk Neon** ⭐ **Demo Theme**
- **Colors**: Matrix green (#00ff41), hot pink (#ff0080)
- **Background**: Very dark (#0a0a0a)
- **Style**: Futuristic neon aesthetics
- **Best for**: Gaming, futuristic vibes, Matrix fans

### 3. **Professional Blue**
- **Colors**: Professional blue (#2563eb), slate gray
- **Background**: Light gray (#f8fafc)
- **Style**: Clean, business-like
- **Best for**: Professional environments, presentations

### 4. **Retro Terminal**
- **Colors**: Amber (#ffa500), green (#00ff00)
- **Background**: Black (#000000)
- **Style**: 80s computer terminal vibes
- **Best for**: Nostalgia, retro computing enthusiasts

### 5. **Minimalist**
- **Colors**: Gray tones, subtle colors
- **Background**: White (#ffffff)
- **Style**: Clean lines, minimal distraction
- **Best for**: Focus, minimal interfaces

## 🚀 Usage

### **Theme Rotation in TUI Mode**
```bash
# Start CLI in TUI mode
nexus-network start --orchestrator-url 'https://staging.orchestrator.nexus.xyz' --node-id 4170008

# Press 'T' to rotate through themes
# Watch the header change colors dynamically!
```

### **Demo Script**
```bash
# Run the theme demo
./demo-themes.sh
```

## 🎨 Theme Structure

Each theme is defined by a JSON structure:

```json
{
  "name": "Cyberpunk Neon",
  "author": "Nexus Team",
  "version": "1.0.0",
  "description": "Futuristic neon colors inspired by cyberpunk aesthetics",
  "colors": {
    "primary": "#00ff41",     // Matrix green
    "secondary": "#ff0080",   // Hot pink
    "background": "#0a0a0a",  // Very dark
    "text": "#ffffff",        // White
    "success": "#00ff00",     // Bright green
    "error": "#ff0000",       // Red
    "warning": "#ffff00",     // Yellow
    "info": "#0080ff"         // Blue
  },
  "ui": {
    "border_style": "double",
    "progress_bar_style": "block",
    "logo_style": "ascii_art"
  },
  "sharing": {
    "public": true,
    "shareable": true,
    "tags": ["cyberpunk", "neon", "matrix"]
  }
}
```

## 🔧 Technical Implementation

### **Theme Manager**
- **ThemeManager**: Handles theme rotation and management
- **Theme**: Individual theme configuration
- **ColorScheme**: Color definitions with hex-to-rgb conversion
- **UIStyles**: UI styling options

### **Integration Points**
- **Dashboard State**: Theme manager integrated into main state
- **Header Component**: Shows current theme and uses theme colors
- **Key Handling**: 'T' key rotates themes in TUI mode
- **Renderer**: Background colors adapt to theme

### **Color System**
- **Hex Support**: Full hex color support (#RRGGBB)
- **Fallback**: White fallback for invalid colors
- **Dynamic**: Colors applied in real-time during rotation

## 🎯 UI Elements Themed

- **Header**: Title, version, theme name, progress gauge
- **Background**: Main dashboard background
- **Borders**: All border colors
- **Progress Bars**: Gauge colors for different states
- **Text**: Primary text colors
- **Status Colors**: Success, error, warning, info colors

## 🚀 Future Enhancements

### **Planned Features**
- **Custom Theme Creation**: Interactive theme builder
- **Theme Gallery**: Web-based theme sharing
- **Theme Import/Export**: Save and load custom themes
- **Theme Validation**: Ensure themes work correctly
- **Advanced Styling**: Fonts, animations, layouts

### **Community Features**
- **Theme Contests**: Community theme creation contests
- **Theme Sharing**: Share themes with other users
- **Theme Ratings**: Rate and review themes
- **Creator Recognition**: Highlight theme creators

## 🎨 Theme Customization

### **Creating Custom Themes**
1. **Define Colors**: Choose hex colors for each element
2. **Set UI Styles**: Configure border and progress bar styles
3. **Add Metadata**: Name, author, description, tags
4. **Test Theme**: Ensure all colors work well together

### **Color Guidelines**
- **Contrast**: Ensure good contrast for readability
- **Accessibility**: Consider colorblind users
- **Consistency**: Use consistent color relationships
- **Purpose**: Colors should match the theme's purpose

## 🎯 Demo Instructions

1. **Run Demo**: `./demo-themes.sh`
2. **Start TUI**: CLI starts in TUI mode
3. **Press 'T'**: Rotate through themes
4. **Observe Changes**: Watch colors change in real-time
5. **Try All Themes**: Cycle through all 5 themes
6. **Exit**: Press 'Q' or 'Esc' to exit

## 🎨 Theme Showcase

### **Cyberpunk Neon** (Demo Theme)
- **Primary**: Matrix green (#00ff41)
- **Secondary**: Hot pink (#ff0080)
- **Background**: Very dark (#0a0a0a)
- **Perfect for**: Gaming, futuristic aesthetics, Matrix fans

### **Professional Blue**
- **Primary**: Professional blue (#2563eb)
- **Background**: Light gray (#f8fafc)
- **Perfect for**: Business environments, presentations

### **Retro Terminal**
- **Primary**: Amber (#ffa500)
- **Background**: Black (#000000)
- **Perfect for**: Nostalgia, retro computing

## 🚀 Getting Started

1. **Build CLI**: `cargo build --release --features build_proto`
2. **Run Demo**: `./demo-themes.sh`
3. **Press 'T'**: Rotate through themes
4. **Enjoy**: Experience the dynamic theming system!

---

*The theme system makes the Nexus CLI more personalized and visually appealing while maintaining full functionality across all themes.*

