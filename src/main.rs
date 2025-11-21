use eframe::egui;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};

mod db;
mod models;

use db::Database;
use models::WorkSession;

const WORK_DURATION: u64 = 25 * 60; // 25 minutes in seconds
const BREAK_DURATION: u64= 5 * 60; // 5 minutes in seconds

// Test Values
// const WORK_DURATION: u64 = 5; 
// const BREAK_DURATION: u64 = 5; 

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
        Box::new(|_cc| Ok(Box::new(PomodoroApp::default()))),
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
}

impl Default for PomodoroApp {
    fn default() -> Self {
        let db = Database::new().expect("Failed to initialize database");
        let today_session_count = db.get_sessions_count_for_today()
            .unwrap_or(0);
        
        Self {
            mode: PomodoroMode::Work,
            state: TimerState::Stopped,
            remaining_seconds: WORK_DURATION,
            last_tick: None,
            work_session_start: None,
            today_session_count,
            db,
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
    }

    fn pause(&mut self) {
        self.state = TimerState::Paused;
        self.last_tick = None;
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
    }

    fn start_break(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Break;
        self.remaining_seconds = BREAK_DURATION;
        self.state = TimerState::Running;
        self.last_tick = Some(Instant::now());
        
        // Reset work session tracking
        self.work_session_start = None;
        
        // Request fullscreen
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
    }

    fn start_work(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Work;
        self.remaining_seconds = WORK_DURATION;
        self.state = TimerState::Stopped;
        self.last_tick = None;
        
        // Exit fullscreen
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    }

    fn skip_break(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Work;
        self.remaining_seconds = WORK_DURATION;
        self.state = TimerState::Stopped;
        self.last_tick = None;
        
        // Exit fullscreen and minimize window
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
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
                                // Exit fullscreen when break ends
                                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
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
}

impl eframe::App for PomodoroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_timer(ctx);

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
                        );
                        ui.add_space(10.0);
                    }
                    
                    ui.heading("Pomodoro Timer");
                    ui.add_space(20.0);
                    
                    // Display timer
                    ui.label(
                        egui::RichText::new(self.format_time())
                            .size(64.0)
                            .monospace()
                    );
                    
                    ui.add_space(30.0);
                    
                    // Control buttons
                    ui.horizontal(|ui| {
                        match self.state {
                            TimerState::Stopped => {
                                if ui.button("Start").clicked() {
                                    self.start(ctx);
                                }
                            }
                            TimerState::Running => {
                                if ui.button("Pause").clicked() {
                                    self.pause();
                                }
                            }
                            TimerState::Paused => {
                                if ui.button("Resume").clicked() {
                                    self.start(ctx);
                                }
                            }
                        }
                        
                        if self.state != TimerState::Stopped {
                            if ui.button("Restart").clicked() {
                                self.restart();
                            }
                        }
                    });
                });
            });
        } else {
            // Break period UI
            egui::CentralPanel::default().show(ctx, |ui| {
                // Check for Enter key press during break
                if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && self.remaining_seconds > 0 {
                    self.skip_break(ctx);
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
                    
                    ui.heading("Break Time!");
                    ui.add_space(20.0);
                    
                    // Display break timer
                    let timer_size = if self.remaining_seconds > 0 { 96.0 } else { 64.0 };
                    ui.label(
                        egui::RichText::new(self.format_time())
                            .size(timer_size)
                            .monospace()
                    );
                    
                    ui.add_space(30.0);
                    
                    // Show Enter key hint during active break
                    if self.remaining_seconds > 0 {
                        ui.label(
                            egui::RichText::new("Press Enter to stay in the pocket and keep your flow")
                                .size(16.0)
                        );
                        ui.add_space(20.0);
                    }
                    
                    // Break control buttons
                    ui.horizontal(|ui| {
                        if self.remaining_seconds == 0 {
                            if ui.button("Start New Timer").clicked() {
                                self.start_work(ctx);
                            }
                        } else {
                            if ui.button("Skip Break").clicked() {
                                self.skip_break(ctx);
                            }
                        }
                    });
                });
            });
        }
    }
}
