use eframe::egui;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use tray_icon::{TrayIcon, TrayIconBuilder};

mod db;
mod models;

use db::Database;
use models::WorkSession;

const WORK_DURATION: u64 = 25 * 60; // 25 minutes in seconds
const BREAK_DURATION: u64= 5 * 60; // 5 minutes in seconds

// Test Values
// const WORK_DURATION: u64 = 5; 
// const BREAK_DURATION: u64 = 5;

// Color Palette
const COLOR_MAIN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x12, 0x40); // #001240
const COLOR_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xFA, 0xFA); // #FAFAFA
const COLOR_ACCENT: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x73, 0x1C); // #FF731C
const COLOR_ALT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xF7, 0xEA); // #FFF7EA
const COLOR_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x60, 0x9E, 0xF6); // #609EF6
const COLOR_SECONDARY_DARK: egui::Color32 = egui::Color32::from_rgb(0x16, 0x46, 0xA1); // #1646A1 

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Pocket Flow - Pomodoro Timer",
        options,
        Box::new(|cc| {
            // Load custom fonts
            let mut fonts = egui::FontDefinitions::default();
            
            // Load SF Pro Display Regular
            fonts.font_data.insert(
                "SF Pro Display".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/sf-pro-display/SFPRODISPLAYREGULAR.OTF"
                )).into(),
            );
            
            // Load SF Pro Display Bold
            fonts.font_data.insert(
                "SF Pro Display Bold".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/sf-pro-display/SFPRODISPLAYBOLD.OTF"
                )).into(),
            );
            
            // Load SF Pro Display Medium
            fonts.font_data.insert(
                "SF Pro Display Medium".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/sf-pro-display/SFPRODISPLAYMEDIUM.OTF"
                )).into(),
            );
            
            // Set SF Pro Display as the default proportional font
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "SF Pro Display".to_owned());
            
            // Also use it for monospace (timer display)
            fonts.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "SF Pro Display Medium".to_owned());
            
            cc.egui_ctx.set_fonts(fonts);
            
            Ok(Box::new(PomodoroApp::default()))
        }),
    )
}

#[derive(PartialEq, Clone, Copy)]
enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(PartialEq, Clone, Copy)]
enum PomodoroMode {
    Work,
    Break,
}

struct PomodoroApp {
    mode: PomodoroMode,
    state: TimerState,
    remaining_seconds: u64,
    last_tick: Option<Instant>,
    work_session_start: Option<DateTime<Utc>>,
    today_session_count: usize,
    db: Database,
    break_window_minimized: bool,
    tray_icon: Option<TrayIcon>,
}

impl Default for PomodoroApp {
    fn default() -> Self {
        let db = Database::new().expect("Failed to initialize database");
        let today_session_count = db.get_sessions_count_for_today()
            .unwrap_or(0);
        
        // Create tray icon for menu bar timer display
        let tray_icon = TrayIconBuilder::new()
            .with_title("25:00")
            .with_tooltip("Pocket Flow - Pomodoro Timer")
            .build()
            .ok();
        
        Self {
            mode: PomodoroMode::Work,
            state: TimerState::Stopped,
            remaining_seconds: WORK_DURATION,
            last_tick: None,
            work_session_start: None,
            today_session_count,
            db,
            break_window_minimized: false,
            tray_icon,
        }
    }
}

impl PomodoroApp {
    fn start(&mut self, ctx: &egui::Context) {
        self.state = TimerState::Running;
        self.last_tick = Some(Instant::now());
        
        // Track work session start time
        if self.mode == PomodoroMode::Work && self.work_session_start.is_none() {
            self.work_session_start = Some(Utc::now());
            // Minimize window when starting work session
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        
        self.update_menu_bar();
    }

    fn pause(&mut self) {
        self.state = TimerState::Paused;
        self.last_tick = None;
        self.update_menu_bar();
    }

    fn restart(&mut self) {
        self.state = TimerState::Stopped;
        self.remaining_seconds = match self.mode {
            PomodoroMode::Work => WORK_DURATION,
            PomodoroMode::Break => BREAK_DURATION,
        };
        self.last_tick = None;
        
        // Reset work session tracking (uncompleted sessions are not saved)
        self.work_session_start = None;
        self.update_menu_bar();
    }

    fn start_break(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Break;
        self.remaining_seconds = BREAK_DURATION;
        self.state = TimerState::Running;
        self.last_tick = Some(Instant::now());
        
        // Reset work session tracking
        self.work_session_start = None;
        
        // Reset minimized state and request fullscreen
        self.break_window_minimized = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
        self.update_menu_bar();
    }

    fn start_work(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Work;
        self.remaining_seconds = WORK_DURATION;
        self.state = TimerState::Stopped;
        self.last_tick = None;
        
        // Exit fullscreen
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        self.update_menu_bar();
    }

    fn skip_break(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Work;
        self.remaining_seconds = WORK_DURATION;
        self.state = TimerState::Running;
        self.last_tick = Some(Instant::now());
        
        // Track new work session start time
        self.work_session_start = Some(Utc::now());
        
        // Exit fullscreen and minimize window
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        self.update_menu_bar();
    }

    fn minimize_break_window(&mut self, ctx: &egui::Context) {
        // Exit fullscreen and mark as minimized
        self.break_window_minimized = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    }

    fn update_timer(&mut self, ctx: &egui::Context) {
        if self.state == TimerState::Running {
            if let Some(last_tick) = self.last_tick {
                let elapsed = last_tick.elapsed();
                
                if elapsed >= Duration::from_secs(1) {
                    self.last_tick = Some(Instant::now());
                    
                    if self.remaining_seconds > 0 {
                        self.remaining_seconds -= 1;
                    }
                    
                    // Update menu bar timer display
                    self.update_menu_bar();
                    
                    // Check if timer completed
                    if self.remaining_seconds == 0 {
                        match self.mode {
                            PomodoroMode::Work => {
                                // Save completed work session
                                if let Some(start_time) = self.work_session_start {
                                    let completed_at = Utc::now();
                                    let session = WorkSession::new(start_time, completed_at);
                                    
                                    if let Err(e) = self.db.save_work_session(&session) {
                                        eprintln!("Failed to save work session: {}", e);
                                    } else {
                                        // Increment session count on successful save
                                        self.today_session_count += 1;
                                    }
                                }
                                
                                // Work period done, start break
                                self.start_break(ctx);
                            }
                            PomodoroMode::Break => {
                                // Break done, stop and wait for user
                                self.state = TimerState::Stopped;
                                self.last_tick = None;
                                // Update menu bar to show break is done
                                self.update_menu_bar();
                                // Exit fullscreen when break ends (only if not already minimized)
                                if !self.break_window_minimized {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                                }
                            }
                        }
                    }
                }
            }
            
            // Request repaint for smooth countdown
            ctx.request_repaint();
        }
    }

    fn format_time(&self) -> String {
        let minutes = self.remaining_seconds / 60;
        let seconds = self.remaining_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
    
    fn update_menu_bar(&self) {
        if let Some(tray) = &self.tray_icon {
            let title = match self.state {
                TimerState::Stopped => {
                    match self.mode {
                        PomodoroMode::Work => "Ready".to_string(),
                        PomodoroMode::Break => "Break Done".to_string(),
                    }
                }
                TimerState::Paused => format!("{} (Paused)", self.format_time()),
                TimerState::Running => self.format_time(),
            };
            let _ = tray.set_title(Some(&title));
        }
    }
}

impl eframe::App for PomodoroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_timer(ctx);
        
        // Apply custom theme
        ctx.style_mut(|style| {
            // Set overall background color to main dark blue
            style.visuals.panel_fill = COLOR_MAIN;
            
            // Set text colors to white/light
            style.visuals.override_text_color = Some(COLOR_BACKGROUND);
            
            // Button styling - inverted (dark inactive, light hover)
            style.visuals.widgets.inactive.weak_bg_fill = COLOR_SECONDARY_DARK;
            style.visuals.widgets.inactive.bg_fill = COLOR_SECONDARY_DARK;
            style.visuals.widgets.inactive.fg_stroke.color = COLOR_BACKGROUND;
            
            style.visuals.widgets.hovered.weak_bg_fill = COLOR_SECONDARY;
            style.visuals.widgets.hovered.bg_fill = COLOR_SECONDARY;
            style.visuals.widgets.hovered.fg_stroke.color = COLOR_MAIN;
            
            style.visuals.widgets.active.weak_bg_fill = COLOR_SECONDARY;
            style.visuals.widgets.active.bg_fill = COLOR_SECONDARY;
            style.visuals.widgets.active.fg_stroke.color = COLOR_MAIN;
            
            // Rounding for buttons
            style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
            style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
            style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
            
            // Button padding
            style.spacing.button_padding = egui::vec2(16.0, 8.0);
        });

        if self.mode == PomodoroMode::Work {
            // Normal window for work period
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    
                    // Display session dots
                    if self.today_session_count > 0 {
                        let dots = "• ".repeat(self.today_session_count);
                        ui.label(
                            egui::RichText::new(dots.trim_end())
                                .size(20.0)
                                .color(COLOR_ACCENT)
                        );
                        ui.add_space(10.0);
                    }
                    
                    ui.label(
                        egui::RichText::new("Pomodoro Timer")
                            .size(24.0)
                            .color(COLOR_BACKGROUND)
                            .strong()
                    );
                    ui.add_space(20.0);
                    
                    // Display timer
                    ui.label(
                        egui::RichText::new(self.format_time())
                            .size(64.0)
                            .monospace()
                            .color(COLOR_BACKGROUND)
                    );
                    
                    ui.add_space(30.0);
                    
                    // Control buttons (centered)
                    ui.horizontal(|ui| {
                        let button_width = 100.0;
                        let num_buttons = if self.state != TimerState::Stopped { 2.0 } else { 1.0 };
                        let spacing = ui.spacing().item_spacing.x;
                        let total_width = button_width * num_buttons + spacing * (num_buttons - 1.0);
                        let available_width = ui.available_width();
                        ui.add_space((available_width - total_width) / 2.0);
                        
                        match self.state {
                            TimerState::Stopped => {
                                if ui.add_sized([button_width, 36.0], egui::Button::new(
                                    egui::RichText::new("Start").size(18.0)
                                )).clicked() {
                                    self.start(ctx);
                                }
                            }
                            TimerState::Running => {
                                if ui.add_sized([button_width, 36.0], egui::Button::new(
                                    egui::RichText::new("Pause").size(18.0)
                                )).clicked() {
                                    self.pause();
                                }
                            }
                            TimerState::Paused => {
                                if ui.add_sized([button_width, 36.0], egui::Button::new(
                                    egui::RichText::new("Resume").size(18.0)
                                )).clicked() {
                                    self.start(ctx);
                                }
                            }
                        }
                        
                        if self.state != TimerState::Stopped {
                            if ui.add_sized([button_width, 36.0], egui::Button::new(
                                egui::RichText::new("Restart").size(18.0)
                            )).clicked() {
                                self.restart();
                            }
                        }
                    });
                });
            });
        } else {
            // Break period UI
            egui::CentralPanel::default().show(ctx, |ui| {
                // Check for keyboard shortcuts during break
                if self.remaining_seconds > 0 {
                    // Enter key to skip break
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.skip_break(ctx);
                    }
                    // ESC key to minimize fullscreen break window
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !self.break_window_minimized {
                        self.minimize_break_window(ctx);
                    }
                }
                
                ui.vertical_centered(|ui| {
                    // Use flexible spacing based on available space
                    let available_height = ui.available_height();
                    let spacing = if self.remaining_seconds > 0 {
                        // Still in break - use more spacing for fullscreen
                        available_height * 0.3
                    } else {
                        // Break ended - use less spacing for normal window
                        40.0
                    };
                    ui.add_space(spacing);
                    
                    ui.label(
                        egui::RichText::new("Break Time!")
                            .size(32.0)
                            .color(COLOR_BACKGROUND)
                            .strong()
                    );
                    ui.add_space(20.0);
                    
                    // Display break timer
                    let timer_size = if self.remaining_seconds > 0 { 96.0 } else { 64.0 };
                    ui.label(
                        egui::RichText::new(self.format_time())
                            .size(timer_size)
                            .monospace()
                            .color(COLOR_BACKGROUND)
                    );
                    
                    ui.add_space(30.0);
                    
                    // Show keyboard hints during active break
                    if self.remaining_seconds > 0 {
                        ui.label(
                            egui::RichText::new("Press Enter to stay in the pocket and keep your flow")
                                .size(16.0)
                                .color(COLOR_BACKGROUND)
                        );
                        ui.add_space(10.0);
                        if !self.break_window_minimized {
                            ui.label(
                                egui::RichText::new("Press ESC to minimize and multitask during break")
                                    .size(16.0)
                                    .color(COLOR_BACKGROUND)
                            );
                        }
                        ui.add_space(20.0);
                    }
                    
                    // Break control buttons (centered)
                    ui.horizontal(|ui| {
                        let button_width = 120.0;
                        let num_buttons = if self.remaining_seconds == 0 { 1.0 } else if self.break_window_minimized { 1.0 } else { 2.0 };
                        let spacing = ui.spacing().item_spacing.x;
                        let total_width = button_width * num_buttons + spacing * (num_buttons - 1.0);
                        let available_width = ui.available_width();
                        ui.add_space((available_width - total_width) / 2.0);
                        
                        if self.remaining_seconds == 0 {
                            if ui.add_sized([button_width, 36.0], egui::Button::new(
                                egui::RichText::new("Start New Timer").size(18.0)
                            )).clicked() {
                                self.start_work(ctx);
                            }
                        } else {
                            if ui.add_sized([button_width, 36.0], egui::Button::new(
                                egui::RichText::new("Skip Break").size(18.0)
                            )).clicked() {
                                self.skip_break(ctx);
                            }
                            
                            // Only show Minimize button if not already minimized
                            if !self.break_window_minimized {
                                if ui.add_sized([button_width, 36.0], egui::Button::new(
                                    egui::RichText::new("Minimize").size(18.0)
                                )).clicked() {
                                    self.minimize_break_window(ctx);
                                }
                            }
                        }
                    });
                });
            });
        }
    }
}
