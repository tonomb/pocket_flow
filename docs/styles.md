# Styling Guide for Pocket Flow (egui)

## Overview

**Important:** This project uses **egui** (not gpui). Egui is an immediate mode GUI framework for Rust.

If you're coming from React/CSS/JavaScript, think of egui as a different paradigm where:
- **No virtual DOM**: UI is redrawn every frame based on application state
- **No separate CSS files**: Styling is done inline using Rust code
- **Immediate mode**: UI code runs every frame (like a game loop)

## Key Concepts for React Developers

### React vs Egui Comparison

| React Concept | Egui Equivalent | Description |
|--------------|-----------------|-------------|
| JSX | `ui.vertical()` / `ui.horizontal()` | Layout containers |
| CSS classes | `RichText` / `ui.style_mut()` | Inline styling |
| useState | Rust struct fields | State management |
| useEffect | Code in `update()` method | Side effects |
| onClick | `if ui.button("Click").clicked()` | Event handlers |
| className | Method chaining | Styling approach |

### Component Structure

In React:
```javascript
function Component() {
  const [state, setState] = useState(0);
  
  return (
    <div className="container">
      <h1 style={{fontSize: 24}}>Title</h1>
      <button onClick={() => setState(state + 1)}>Click</button>
    </div>
  );
}
```

In egui:
```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Title"); // h1 equivalent
                if ui.button("Click").clicked() {
                    self.state += 1;
                }
            });
        });
    }
}
```

## Styling in Egui

### 1. Text Styling with RichText

`RichText` is like inline styles in React:

```rust
// Basic text with size (like fontSize in CSS)
ui.label(
    egui::RichText::new("Hello World")
        .size(24.0)  // CSS: font-size: 24px
);

// Bold text
ui.label(
    egui::RichText::new("Bold Text")
        .strong()  // CSS: font-weight: bold
);

// Colored text
ui.label(
    egui::RichText::new("Colored")
        .color(egui::Color32::RED)  // CSS: color: red
);

// Monospace font (like CSS font-family: monospace)
ui.label(
    egui::RichText::new("12:34")
        .size(64.0)
        .monospace()
);

// Combine multiple styles (method chaining)
ui.label(
    egui::RichText::new("Styled Text")
        .size(32.0)
        .color(egui::Color32::from_rgb(100, 200, 150))
        .strong()
);
```

### 2. Layout (Flexbox-like)

Egui uses containers similar to CSS flexbox:

```rust
// Vertical layout (like flex-direction: column)
ui.vertical(|ui| {
    ui.label("First");
    ui.label("Second");
});

// Horizontal layout (like flex-direction: row)
ui.horizontal(|ui| {
    ui.label("Left");
    ui.label("Right");
});

// Centered layout (like align-items: center, justify-content: center)
ui.vertical_centered(|ui| {
    ui.label("Centered");
});

ui.horizontal_centered(|ui| {
    ui.label("Centered horizontally");
});
```

### 3. Spacing (Margin/Padding)

```rust
// Add space (like margin-top in CSS)
ui.add_space(20.0);  // CSS: margin-top: 20px

// Get available space (like CSS height: 100%)
let available_height = ui.available_height();
let spacing = available_height * 0.3; // 30% of available space
ui.add_space(spacing);
```

### 4. Colors

```rust
// Predefined colors
egui::Color32::RED
egui::Color32::GREEN
egui::Color32::BLUE
egui::Color32::WHITE
egui::Color32::BLACK

// Custom RGB (like CSS rgb(255, 128, 0))
egui::Color32::from_rgb(255, 128, 0)

// With alpha/transparency (like CSS rgba(255, 128, 0, 0.5))
egui::Color32::from_rgba_unmultiplied(255, 128, 0, 128)
```

### 5. Button Styling

```rust
// Basic button
if ui.button("Click Me").clicked() {
    // Handle click
}

// Styled button with custom size
let button = egui::Button::new(
    egui::RichText::new("Big Button")
        .size(24.0)
        .color(egui::Color32::WHITE)
);
if ui.add(button).clicked() {
    // Handle click
}
```

### 6. Window/Viewport Configuration

Set in `main.rs`:

```rust
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])  // CSS: width: 400px; height: 300px
            .with_resizable(true)  // Allow window resizing
            .with_min_inner_size([300.0, 200.0])  // Minimum size
            .with_max_inner_size([800.0, 600.0])  // Maximum size
            .with_transparent(true)  // Transparent window
            .with_decorations(false),  // Remove title bar
        ..Default::default()
    };
    
    eframe::run_native(/* ... */)
}
```

### 7. Global Theme Customization

```rust
// Modify global style (like CSS :root variables)
ctx.style_mut(|style| {
    // Change default text color
    style.visuals.override_text_color = Some(egui::Color32::WHITE);
    
    // Change button colors
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(70, 70, 70);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(90, 90, 90);
    
    // Change spacing (like CSS gap)
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
});
```

## Common Styling Patterns in Pocket Flow

### Large Timer Display
```rust
ui.label(
    egui::RichText::new(self.format_time())
        .size(64.0)      // Large font
        .monospace()     // Fixed-width numbers
);
```

### Session Dots Display
```rust
if self.today_session_count > 0 {
    let dots = "• ".repeat(self.today_session_count);
    ui.label(
        egui::RichText::new(dots.trim_end())
            .size(20.0)
    );
    ui.add_space(10.0);
}
```

### Responsive Spacing
```rust
let available_height = ui.available_height();
let spacing = if self.remaining_seconds > 0 {
    // Fullscreen mode - more space
    available_height * 0.3
} else {
    // Normal window - less space
    40.0
};
ui.add_space(spacing);
```

### Conditional UI Elements
```rust
// Only show certain elements based on state
if self.remaining_seconds > 0 {
    ui.label(
        egui::RichText::new("Press ESC to minimize")
            .size(16.0)
    );
}
```

## Advanced Styling Techniques

### Custom Frames (Like CSS Borders/Backgrounds)

```rust
egui::Frame::none()
    .fill(egui::Color32::from_rgb(40, 40, 40))  // Background color
    .stroke(egui::Stroke::new(2.0, egui::Color32::WHITE))  // Border
    .inner_margin(10.0)  // Padding
    .show(ui, |ui| {
        ui.label("Framed content");
    });
```

### Custom Widgets with Sizing

```rust
// Fixed size widget
ui.add_sized([200.0, 50.0], egui::Button::new("Fixed Size"));

// Full width button
ui.allocate_ui_with_layout(
    egui::vec2(ui.available_width(), 40.0),
    egui::Layout::top_down(egui::Align::Center),
    |ui| {
        ui.button("Full Width");
    }
);
```

### Dark Mode / Light Mode

```rust
// Set dark mode (like CSS prefers-color-scheme: dark)
ctx.set_visuals(egui::Visuals::dark());

// Set light mode
ctx.set_visuals(egui::Visuals::light());
```

## Practical Examples

### Example 1: Styled Card Component

```rust
egui::Frame::none()
    .fill(egui::Color32::from_rgb(30, 30, 35))
    .rounding(10.0)  // Rounded corners
    .inner_margin(20.0)
    .show(ui, |ui| {
        ui.vertical(|ui| {
            ui.heading("Card Title");
            ui.add_space(10.0);
            ui.label("Card content goes here");
        });
    });
```

### Example 2: Custom Button with Icon

```rust
let button_text = format!("▶ Start Timer");
if ui.button(
    egui::RichText::new(button_text)
        .size(20.0)
        .color(egui::Color32::from_rgb(100, 200, 100))
).clicked() {
    self.start(ctx);
}
```

### Example 3: Progress Bar Style

```rust
let progress = 1.0 - (self.remaining_seconds as f32 / WORK_DURATION as f32);
ui.add(
    egui::ProgressBar::new(progress)
        .text(format!("{}%", (progress * 100.0) as i32))
);
```

## Resources

- **egui Documentation**: https://docs.rs/egui/
- **egui Demo**: Run `cargo run --example demo` in the egui repository
- **Visual Style Editor**: egui has a built-in style editor you can enable:
  ```rust
  ctx.style_ui(ui);  // Shows style editor in your app
  ```

## Tips for React Developers

1. **Think in terms of immediate mode**: Your UI code runs every frame, so don't worry about "updating" the UI - just describe what it should look like right now
2. **State lives in your struct**: Instead of `useState`, add fields to your app struct
3. **No CSS classes**: All styling is done inline with method chaining
4. **Layout is code**: Instead of CSS flexbox/grid, use `vertical()`, `horizontal()`, and spacing
5. **Colors are values**: Instead of CSS color strings, use `Color32` types
6. **Inspect with F12**: Many egui apps support pressing F12 to show debug UI

## Common Questions

**Q: How do I create reusable styled components?**
A: Create functions or methods that take `&mut egui::Ui`:

```rust
impl PomodoroApp {
    fn styled_timer(&self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new(self.format_time())
                .size(64.0)
                .monospace()
        );
    }
}

// Use it:
self.styled_timer(ui);
```

**Q: How do I animate things?**
A: Use `ctx.animate_bool()` or manually interpolate values and call `ctx.request_repaint()`:

```rust
let animation_progress = ctx.animate_value_with_time(
    egui::Id::new("my_animation"),
    1.0,  // target value
    0.5   // animation time in seconds
);
```

**Q: How do I make responsive layouts?**
A: Use `ui.available_width()` and `ui.available_height()` to calculate sizes:

```rust
let width = ui.available_width();
let size = if width > 600.0 { 64.0 } else { 32.0 };
```
