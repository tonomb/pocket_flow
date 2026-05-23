use chrono::{DateTime, Utc};
use eframe::egui;
use std::time::{Duration, Instant};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

mod db;
mod models;

use db::Database;
use models::WorkSession;

const WORK_DURATION: u64 = 45 * 60; // 45 minutes in seconds
const BREAK_DURATION: u64 = 15 * 60; // 15 minutes in seconds
const OVERFLOW_DURATION: u64 = 5 * 60; // 5 minutes in seconds (UX label: "Pocket")

// Test Values
// const WORK_DURATION: u64 = 5;
// const BREAK_DURATION: u64 = 5;
// const OVERFLOW_DURATION: u64 = 3;

// Color Palette
const COLOR_MAIN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x12, 0x40); // #001240
const COLOR_BACKGROUND: egui::Color32 = egui::Color32::from_rgb(0xFA, 0xFA, 0xFA); // #FAFAFA
const COLOR_ACCENT: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x73, 0x1C); // #FF731C
#[allow(dead_code)]
const COLOR_ALT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xF7, 0xEA); // #FFF7EA
const COLOR_SECONDARY: egui::Color32 = egui::Color32::from_rgb(0x60, 0x9E, 0xF6); // #609EF6
const COLOR_SECONDARY_DARK: egui::Color32 = egui::Color32::from_rgb(0x16, 0x46, 0xA1); // #1646A1

/// Compute remaining whole seconds until a deadline.
/// Returns (remaining_seconds, deadline_reached).
fn remaining_until(deadline: DateTime<Utc>, now: DateTime<Utc>) -> (u64, bool) {
    if now >= deadline {
        (0, true)
    } else {
        ((deadline - now).num_seconds() as u64, false)
    }
}

/// Compute the absolute deadline for a timer with the given duration in seconds.
fn compute_deadline(now: DateTime<Utc>, duration_secs: u64) -> DateTime<Utc> {
    now + chrono::TimeDelta::seconds(duration_secs as i64)
}

fn break_finished_state() -> (PomodoroMode, u64, TimerState) {
    (PomodoroMode::Work, WORK_DURATION, TimerState::Stopped)
}

/// State produced when the overflow ("Pocket") timer completes.
/// Auto-transitions into a running Break timer. The actual transition is
/// performed by `start_break(ctx)` in `update_timer`; this helper exists so
/// the contract can be asserted by tests.
#[cfg_attr(not(test), allow(dead_code))]
fn overflow_finished_state() -> (PomodoroMode, u64, TimerState) {
    (PomodoroMode::Break, BREAK_DURATION, TimerState::Running)
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_resizable(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            .with_fullsize_content_view(true),
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
                )),
            );

            // Load SF Pro Display Bold
            fonts.font_data.insert(
                "SF Pro Display Bold".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/sf-pro-display/SFPRODISPLAYBOLD.OTF"
                )),
            );

            // Load SF Pro Display Medium
            fonts.font_data.insert(
                "SF Pro Display Medium".to_owned(),
                egui::FontData::from_static(include_bytes!(
                    "../assets/fonts/sf-pro-display/SFPRODISPLAYMEDIUM.OTF"
                )),
            );

            // Set SF Pro Display as the default proportional font
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "SF Pro Display".to_owned());

            // Also use it for monospace (timer display)
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "SF Pro Display Medium".to_owned());

            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(PomodoroApp::default()))
        }),
    )
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum TimerState {
    Stopped,
    Running,
    Paused,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum PomodoroMode {
    Work,
    Break,
    /// Configurable short timer (default 5 min) shown to the user as "Pocket".
    /// Lets the user wrap up their train of thought during a break before the
    /// break resumes. Not counted as a completed work session.
    Overflow,
}

struct PomodoroApp {
    mode: PomodoroMode,
    state: TimerState,
    remaining_seconds: u64,
    /// Absolute wall-clock time when the running timer reaches zero.
    /// `Some` when running, `None` when stopped or paused.
    /// Using an absolute deadline instead of tick deltas makes the timer
    /// immune to macOS screen-lock / sleep suspending the process.
    end_time: Option<DateTime<Utc>>,
    work_session_start: Option<DateTime<Utc>>,
    today_session_count: usize,
    db: Database,
    break_window_minimized: bool,
    tray_icon: Option<TrayIcon>,
    minimize_after: Option<Instant>,
}

impl Default for PomodoroApp {
    fn default() -> Self {
        let db = Database::new().expect("Failed to initialize database");
        let today_session_count = db.get_sessions_count_for_today().unwrap_or(0);

        // Create tray icon for menu bar timer display
        let initial_time = format!("{:02}:00", WORK_DURATION / 60);
        let tray_icon = TrayIconBuilder::new()
            .with_title(&initial_time)
            .with_tooltip("Pocket Flow - Pomodoro Timer")
            .build()
            .ok();

        Self {
            mode: PomodoroMode::Work,
            state: TimerState::Stopped,
            remaining_seconds: WORK_DURATION,
            end_time: None,
            work_session_start: None,
            today_session_count,
            db,
            break_window_minimized: false,
            tray_icon,
            minimize_after: None,
        }
    }
}

impl PomodoroApp {
    /// Set the timer to a specific duration and optionally start it running.
    fn set_timer(&mut self, duration: u64, start_running: bool) {
        self.remaining_seconds = duration;
        if start_running {
            self.state = TimerState::Running;
            self.end_time = Some(compute_deadline(Utc::now(), duration));
        } else {
            self.state = TimerState::Stopped;
            self.end_time = None;
        }
        self.update_menu_bar();
    }

    /// Resume the timer from its current position (does not reset duration).
    fn resume(&mut self, ctx: &egui::Context) {
        self.state = TimerState::Running;
        self.end_time = Some(compute_deadline(Utc::now(), self.remaining_seconds));

        // Track work session start time
        if self.mode == PomodoroMode::Work && self.work_session_start.is_none() {
            self.work_session_start = Some(Utc::now());
            // Minimize window when starting work session
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        self.update_menu_bar();
    }

    fn pause(&mut self) {
        // Snapshot remaining time from the deadline before clearing it,
        // so we preserve the exact seconds left at the moment of pause.
        if let Some(end_time) = self.end_time {
            let (remaining, _) = remaining_until(end_time, Utc::now());
            self.remaining_seconds = remaining;
        }
        self.state = TimerState::Paused;
        self.end_time = None;
        self.update_menu_bar();
    }

    fn restart(&mut self) {
        let duration = match self.mode {
            PomodoroMode::Work => WORK_DURATION,
            PomodoroMode::Break => BREAK_DURATION,
            PomodoroMode::Overflow => OVERFLOW_DURATION,
        };
        self.set_timer(duration, false);
        // Reset work session tracking (uncompleted sessions are not saved)
        self.work_session_start = None;
    }

    fn start_break(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Break;
        self.set_timer(BREAK_DURATION, true);
        // Reset work session tracking
        self.work_session_start = None;
        // Reset minimized state and request fullscreen
        self.break_window_minimized = false;
        self.minimize_after = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
    }

    fn finish_break(&mut self, ctx: &egui::Context) {
        let (mode, duration, _) = break_finished_state();
        self.mode = mode;
        self.set_timer(duration, false);
        self.break_window_minimized = false;
        self.minimize_after = None;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(400.0, 300.0)));
    }

    fn skip_break(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Work;
        self.set_timer(WORK_DURATION, true);
        // Track new work session start time
        self.work_session_start = Some(Utc::now());
        // Exit fullscreen and resize to small window first;
        // defer minimize so macOS can finish the fullscreen exit animation
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(400.0, 300.0)));
        self.minimize_after = Some(Instant::now() + Duration::from_millis(600));
    }

    fn minimize_break_window(&mut self, ctx: &egui::Context) {
        // Exit fullscreen and mark as minimized
        self.break_window_minimized = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    }

    /// Enter overflow ("Pocket") mode: a short configurable timer to wrap up
    /// work before the break begins. Reached via the Pocket button or `P` key
    /// during a break. When this timer completes, the break starts
    /// automatically. Not counted as a completed work session.
    fn start_overflow(&mut self, ctx: &egui::Context) {
        self.mode = PomodoroMode::Overflow;
        self.set_timer(OVERFLOW_DURATION, true);
        // Pocket time is overflow from the just-completed work session — do
        // not start a new work_session_start, so it isn't double-counted.
        self.work_session_start = None;
        // Exit fullscreen and resize to small window first;
        // defer minimize so macOS can finish the fullscreen exit animation.
        self.break_window_minimized = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(400.0, 300.0)));
        self.minimize_after = Some(Instant::now() + Duration::from_millis(600));
    }

    fn update_timer(&mut self, ctx: &egui::Context) {
        if self.state == TimerState::Running {
            if let Some(end_time) = self.end_time {
                let (new_remaining, completed) = remaining_until(end_time, Utc::now());

                // Only update display when the whole-second value changes
                if new_remaining != self.remaining_seconds {
                    self.remaining_seconds = new_remaining;
                    self.update_menu_bar();
                }

                if completed {
                    self.end_time = None;

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
                            self.finish_break(ctx);
                        }
                        PomodoroMode::Overflow => {
                            // Pocket time done — auto-start the break.
                            // Not saved as a session (overflow is wrap-up
                            // time from the just-completed work session).
                            self.start_break(ctx);
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
                TimerState::Stopped => match self.mode {
                    PomodoroMode::Work => "Ready".to_string(),
                    PomodoroMode::Break => "Break Done".to_string(),
                    PomodoroMode::Overflow => "Pocket".to_string(),
                },
                TimerState::Paused => match self.mode {
                    PomodoroMode::Overflow => format!("P {} (Paused)", self.format_time()),
                    _ => format!("{} (Paused)", self.format_time()),
                },
                TimerState::Running => match self.mode {
                    PomodoroMode::Overflow => format!("P {}", self.format_time()),
                    _ => self.format_time(),
                },
            };
            tray.set_title(Some(&title));
        }
    }
}

impl eframe::App for PomodoroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle deferred minimize after exiting fullscreen
        if let Some(target) = self.minimize_after {
            if Instant::now() >= target {
                self.minimize_after = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        }

        self.update_timer(ctx);

        // Handle tray icon click - show window centered at small size
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click { .. } = event {
                self.break_window_minimized = false;
                self.minimize_after = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(400.0, 300.0)));
                if let Some(cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                    ctx.send_viewport_cmd(cmd);
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }

        // Keep polling for tray events even when minimized
        ctx.request_repaint_after(Duration::from_millis(100));

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

        if self.mode != PomodoroMode::Break {
            // Normal window for work period (and Overflow / "Pocket" mode,
            // which reuses the same layout with a different title).
            let is_overflow = self.mode == PomodoroMode::Overflow;
            egui::CentralPanel::default().show(ctx, |ui| {
                // Disable default item spacing for precise control
                ui.spacing_mut().item_spacing.y = 0.0;

                let available_height = ui.available_height();
                let available_width = ui.available_width();

                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        ui.min_rect().min,
                        egui::vec2(available_width, available_height),
                    )),
                    |ui| {
                        ui.vertical_centered(|ui| {
                            // Measure content height first
                            let dots_height = if self.today_session_count > 0 {
                                24.0
                            } else {
                                0.0
                            };
                            let dots_spacing = if self.today_session_count > 0 {
                                4.0
                            } else {
                                0.0
                            };
                            let title_height = 28.0;
                            let title_spacing = 8.0;
                            let timer_height = 70.0;
                            let timer_spacing = 8.0;
                            let button_height = 36.0;

                            let content_height = dots_height
                                + dots_spacing
                                + title_height
                                + title_spacing
                                + timer_height
                                + timer_spacing
                                + button_height;
                            let top_padding = ((available_height - content_height) / 2.0).max(12.0);

                            ui.add_space(top_padding);

                            // Display session dots
                            if self.today_session_count > 0 {
                                let dots = "• ".repeat(self.today_session_count);
                                ui.label(
                                    egui::RichText::new(dots.trim_end())
                                        .size(20.0)
                                        .color(COLOR_ACCENT),
                                );
                                ui.add_space(4.0);
                            }

                            let (title_text, title_color) = if is_overflow {
                                ("Wrapping Up", COLOR_ACCENT)
                            } else {
                                ("Pomodoro Timer", COLOR_BACKGROUND)
                            };
                            ui.label(
                                egui::RichText::new(title_text)
                                    .size(24.0)
                                    .color(title_color)
                                    .strong(),
                            );
                            ui.add_space(8.0);

                            // Display timer
                            ui.label(
                                egui::RichText::new(self.format_time())
                                    .size(64.0)
                                    .monospace()
                                    .color(COLOR_BACKGROUND),
                            );

                            ui.add_space(8.0);

                            // Control buttons (centered)
                            ui.horizontal(|ui| {
                                let button_width = if is_overflow { 140.0 } else { 100.0 };
                                let num_buttons = if is_overflow {
                                    2.0
                                } else if self.state != TimerState::Stopped {
                                    2.0
                                } else {
                                    1.0
                                };
                                let spacing = ui.spacing().item_spacing.x;
                                let total_width =
                                    button_width * num_buttons + spacing * (num_buttons - 1.0);
                                let available_width = ui.available_width();
                                ui.add_space((available_width - total_width) / 2.0);

                                if is_overflow {
                                    // Pocket / overflow mode has exactly two
                                    // actions: abandon the wrap-up and start a
                                    // fresh work session, or end the wrap-up
                                    // early and begin the break.
                                    if ui
                                        .add_sized(
                                            [button_width, 36.0],
                                            egui::Button::new(
                                                egui::RichText::new("Start New Session")
                                                    .size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.skip_break(ctx);
                                    }
                                    if ui
                                        .add_sized(
                                            [button_width, 36.0],
                                            egui::Button::new(
                                                egui::RichText::new("Start Break Early")
                                                    .size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.start_break(ctx);
                                    }
                                } else {
                                    match self.state {
                                        TimerState::Stopped => {
                                            if ui
                                                .add_sized(
                                                    [button_width, 36.0],
                                                    egui::Button::new(
                                                        egui::RichText::new("Start").size(18.0),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                self.resume(ctx);
                                            }
                                        }
                                        TimerState::Running => {
                                            if ui
                                                .add_sized(
                                                    [button_width, 36.0],
                                                    egui::Button::new(
                                                        egui::RichText::new("Pause").size(18.0),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                self.pause();
                                            }
                                        }
                                        TimerState::Paused => {
                                            if ui
                                                .add_sized(
                                                    [button_width, 36.0],
                                                    egui::Button::new(
                                                        egui::RichText::new("Resume").size(18.0),
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                self.resume(ctx);
                                            }
                                        }
                                    }

                                    if self.state != TimerState::Stopped
                                        && ui
                                            .add_sized(
                                                [button_width, 36.0],
                                                egui::Button::new(
                                                    egui::RichText::new("Restart").size(18.0),
                                                ),
                                            )
                                            .clicked()
                                    {
                                        self.restart();
                                    }
                                }
                            });
                        });
                    },
                );
            });
        } else {
            // Break period UI
            egui::CentralPanel::default().show(ctx, |ui| {
                // Check for keyboard shortcuts during break
                if self.remaining_seconds > 0 {
                    // Enter key to skip break (start a new full work timer)
                    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.skip_break(ctx);
                    }
                    // P key to enter Pocket (overflow) mode for short wrap-up
                    if ctx.input(|i| i.key_pressed(egui::Key::P)) {
                        self.start_overflow(ctx);
                    }
                    // ESC key to minimize fullscreen break window
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape))
                        && !self.break_window_minimized
                    {
                        self.minimize_break_window(ctx);
                    }
                }

                // Disable default item spacing for precise control
                ui.spacing_mut().item_spacing.y = 0.0;

                let available_height = ui.available_height();
                let available_width = ui.available_width();

                ui.allocate_new_ui(
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        ui.min_rect().min,
                        egui::vec2(available_width, available_height),
                    )),
                    |ui| {
                        ui.vertical_centered(|ui| {
                            // Measure content height first
                            let title_height = 28.0;
                            let title_spacing = 8.0;
                            let timer_height = 70.0;
                            let timer_spacing = 8.0;
                            let hint_height = if self.remaining_seconds > 0 {
                                32.0
                            } else {
                                0.0
                            };
                            let hint_spacing = if self.remaining_seconds > 0 {
                                12.0
                            } else {
                                0.0
                            };
                            let button_height = 36.0;

                            let content_height = title_height
                                + title_spacing
                                + timer_height
                                + timer_spacing
                                + hint_height
                                + hint_spacing
                                + button_height;
                            let top_padding = ((available_height - content_height) / 2.0).max(12.0);

                            ui.add_space(top_padding);

                            ui.label(
                                egui::RichText::new("Break Time!")
                                    .size(24.0)
                                    .color(COLOR_BACKGROUND)
                                    .strong(),
                            );
                            ui.add_space(8.0);

                            // Display break timer
                            ui.label(
                                egui::RichText::new(self.format_time())
                                    .size(64.0)
                                    .monospace()
                                    .color(COLOR_BACKGROUND),
                            );

                            ui.add_space(8.0);

                            // Show hint text only during active break
                            if self.remaining_seconds > 0 {
                                ui.label(
                                    egui::RichText::new(
                                        "Press Enter to stay in the pocket and keep your flow",
                                    )
                                    .size(14.0)
                                    .color(COLOR_BACKGROUND),
                                );
                                ui.add_space(12.0);
                            }

                            // Break control buttons (centered)
                            ui.horizontal(|ui| {
                                let button_width = 120.0;
                                let num_buttons = if self.remaining_seconds == 0 {
                                    1.0
                                } else {
                                    2.0
                                };
                                let spacing = ui.spacing().item_spacing.x;
                                let total_width =
                                    button_width * num_buttons + spacing * (num_buttons - 1.0);
                                let available_width = ui.available_width();
                                ui.add_space((available_width - total_width) / 2.0);

                                if self.remaining_seconds == 0 {
                                    if ui
                                        .add_sized(
                                            [button_width, 36.0],
                                            egui::Button::new(
                                                egui::RichText::new("Start New Timer").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.finish_break(ctx);
                                    }
                                } else {
                                    if ui
                                        .add_sized(
                                            [button_width, 36.0],
                                            egui::Button::new(
                                                egui::RichText::new("Pocket (5 min)").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.start_overflow(ctx);
                                    }
                                    if ui
                                        .add_sized(
                                            [button_width, 36.0],
                                            egui::Button::new(
                                                egui::RichText::new("Skip Break").size(18.0),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        self.skip_break(ctx);
                                    }
                                }
                            });
                        });
                    },
                );
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    // ── remaining_until tests ──────────────────────────────────────────

    #[test]
    fn test_remaining_until_future_deadline() {
        let now = Utc::now();
        let deadline = now + TimeDelta::seconds(120);
        let (remaining, completed) = remaining_until(deadline, now);
        assert_eq!(remaining, 120);
        assert!(!completed);
    }

    #[test]
    fn test_remaining_until_exact_deadline() {
        let now = Utc::now();
        let (remaining, completed) = remaining_until(now, now);
        assert_eq!(remaining, 0);
        assert!(completed);
    }

    #[test]
    fn test_remaining_until_past_deadline_short() {
        let now = Utc::now();
        let deadline = now - TimeDelta::seconds(30);
        let (remaining, completed) = remaining_until(deadline, now);
        assert_eq!(remaining, 0);
        assert!(completed);
    }

    #[test]
    fn test_remaining_until_past_deadline_large_overshoot() {
        // Computer slept for 2 hours past a 15-minute break
        let now = Utc::now();
        let deadline = now - TimeDelta::seconds(7200);
        let (remaining, completed) = remaining_until(deadline, now);
        assert_eq!(remaining, 0);
        assert!(completed);
    }

    #[test]
    fn test_remaining_until_sub_second_truncates() {
        // 10.7 seconds remaining → reports 10 whole seconds
        let now = Utc::now();
        let deadline = now + TimeDelta::milliseconds(10_700);
        let (remaining, completed) = remaining_until(deadline, now);
        assert_eq!(remaining, 10);
        assert!(!completed);
    }

    // ── compute_deadline tests ───────────────────────────────────────

    #[test]
    fn test_compute_deadline_from_duration() {
        let now = Utc::now();
        let deadline = compute_deadline(now, 300);
        // Deadline should be 300 seconds (5 min) in the future
        let diff = (deadline - now).num_seconds();
        assert_eq!(diff, 300);
    }

    #[test]
    fn test_compute_deadline_zero_duration() {
        let now = Utc::now();
        let deadline = compute_deadline(now, 0);
        assert_eq!(deadline, now);
    }

    #[test]
    fn test_compute_deadline_round_trips_with_remaining_until() {
        // compute_deadline then remaining_until should return the original duration
        let now = Utc::now();
        let deadline = compute_deadline(now, WORK_DURATION);
        let (remaining, completed) = remaining_until(deadline, now);
        assert_eq!(remaining, WORK_DURATION);
        assert!(!completed);
    }

    // ── break mode tests ─────────────────────────────────────────────

    #[test]
    fn test_break_finished_state_returns_work_mode() {
        let (mode, remaining, state) = break_finished_state();
        assert_eq!(mode, PomodoroMode::Work);
        assert_eq!(remaining, WORK_DURATION);
        assert_eq!(state, TimerState::Stopped);
    }

    #[test]
    fn test_break_timer_completion_triggers_transition() {
        // Given: break timer deadline reached
        let now = Utc::now();
        let deadline = compute_deadline(now, 1);
        let later = now + TimeDelta::seconds(1);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, 0);
        assert!(completed);

        // When: break completes, we get the finished state
        let (mode, new_remaining, state) = break_finished_state();

        // Then: transitions to work mode, stopped, full duration
        assert_eq!(mode, PomodoroMode::Work);
        assert_eq!(new_remaining, WORK_DURATION);
        assert_eq!(state, TimerState::Stopped);
    }

    #[test]
    fn test_break_timer_not_yet_complete() {
        let now = Utc::now();
        let deadline = compute_deadline(now, BREAK_DURATION);
        let later = now + TimeDelta::seconds(60);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, BREAK_DURATION - 60);
        assert!(!completed);
    }

    #[test]
    fn test_break_timer_completes_with_overshoot() {
        let now = Utc::now();
        let deadline = compute_deadline(now, BREAK_DURATION);
        let later = now + TimeDelta::seconds((BREAK_DURATION + 100) as i64);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, 0);
        assert!(completed);
    }

    // ── overflow (pocket) mode tests ───────────────────────────────────

    #[test]
    fn test_overflow_duration_is_five_minutes() {
        assert_eq!(OVERFLOW_DURATION, 5 * 60);
    }

    #[test]
    fn test_overflow_finished_state_returns_break_mode() {
        let (mode, remaining, state) = overflow_finished_state();
        assert_eq!(mode, PomodoroMode::Break);
        assert_eq!(remaining, BREAK_DURATION);
        assert_eq!(state, TimerState::Running);
    }

    #[test]
    fn test_overflow_timer_not_yet_complete() {
        let now = Utc::now();
        let deadline = compute_deadline(now, OVERFLOW_DURATION);
        let later = now + TimeDelta::seconds(60);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, OVERFLOW_DURATION - 60);
        assert!(!completed);
    }

    #[test]
    fn test_overflow_timer_completes_exactly() {
        let now = Utc::now();
        let deadline = compute_deadline(now, OVERFLOW_DURATION);
        let later = now + TimeDelta::seconds(OVERFLOW_DURATION as i64);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, 0);
        assert!(completed);
    }

    #[test]
    fn test_overflow_timer_completes_with_overshoot() {
        // Long sleep / screen lock during pocket: still detects completion
        let now = Utc::now();
        let deadline = compute_deadline(now, OVERFLOW_DURATION);
        let later = now + TimeDelta::seconds((OVERFLOW_DURATION as i64) + 7200);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, 0);
        assert!(completed);
    }

    #[test]
    fn test_overflow_timer_completion_triggers_break_transition() {
        // Given: overflow timer deadline reached
        let now = Utc::now();
        let deadline = compute_deadline(now, 1);
        let later = now + TimeDelta::seconds(1);
        let (remaining, completed) = remaining_until(deadline, later);
        assert_eq!(remaining, 0);
        assert!(completed);

        // When: overflow completes, we get the finished state
        let (mode, new_remaining, state) = overflow_finished_state();

        // Then: transitions to break mode, running, full break duration
        assert_eq!(mode, PomodoroMode::Break);
        assert_eq!(new_remaining, BREAK_DURATION);
        assert_eq!(state, TimerState::Running);
    }
}
