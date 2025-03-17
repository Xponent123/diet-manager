use eframe::egui;
use egui::{FontId, FontFamily, RichText, Color32};
use egui::plot::{Line, Plot, PlotPoints};
use crate::{FoodDatabase, DailyLogs};

// Define sophisticated color palette with hex codes
struct AppColors {
    // Primary colors
    primary: Color32,          // #5383B3 (calming blue)
    primary_dark: Color32,     // #2C5F8E
    
    // Accent colors
    accent: Color32,           // #E59F3C (warm gold)
    accent_light: Color32,     // #F7C978
    
    // Background colors
    bg_dark: Color32,          // #1A1E23 (nearly black)
    bg_medium: Color32,        // #2C3037
    bg_light: Color32,         // #3B4047
    
    // Text colors
    text_primary: Color32,     // #F5F7FA
    text_secondary: Color32,   // #BFC4CC
    text_muted: Color32,       // #8C9099
}

impl AppColors {
    fn new() -> Self {
        Self {
            primary: Color32::from_rgb(83, 131, 179),
            primary_dark: Color32::from_rgb(44, 95, 142),
            accent: Color32::from_rgb(229, 159, 60),
            accent_light: Color32::from_rgb(247, 201, 120),
            bg_dark: Color32::from_rgb(26, 30, 35),
            bg_medium: Color32::from_rgb(44, 48, 55),
            bg_light: Color32::from_rgb(59, 64, 71),
            text_primary: Color32::from_rgb(245, 247, 250),
            text_secondary: Color32::from_rgb(191, 196, 204),
            text_muted: Color32::from_rgb(140, 144, 153),
        }
    }
}

pub struct MyApp {
    pub food_db: FoodDatabase,
    pub dlogs: DailyLogs,
    pub tab: usize, // 0 = Food, 1 = Daily Logs, 2 = Insights
    pub food_search: String,
 
    colors: AppColors,
    pub selected_date: Option<String>,
}

impl MyApp {
    pub fn new(food_db: FoodDatabase, dlogs: DailyLogs) -> Self {
        Self { 
            food_db, 
            dlogs, 
            tab: 0, 
            food_search: String::new(),
            
            colors: AppColors::new(),
            selected_date: None,
        }
    }

    /// Calculate calories for a food with the given servings.
    fn calculate_food_calories(&self, food_id: &str, servings: f32) -> u32 {
        // ...existing implementation...
        if let Some(basic) = self.food_db.basic_foods.iter().find(|b| b.id == food_id) {
            return (basic.calories as f32 * servings) as u32;
        }
        if let Some(composite) = self.food_db.composite_foods.iter().find(|c| c.id == food_id) {
            if let Some(cal) = self.food_db.compute_composite_calories(composite) {
                return (cal as f32 * servings) as u32;
            }
        }
        0
    }

    /// Build data for the Insights plot from daily logs.
    fn insights_plot_data(&self) -> PlotPoints {
        // ...existing implementation...
        let mut points = Vec::new();
        let mut dates: Vec<_> = self.dlogs.logs.keys().cloned().collect();
        dates.sort();
        for (i, date) in dates.iter().enumerate() {
            if let Some(log) = self.dlogs.logs.get(date) {
                let total: u32 = log.entries.iter()
                    .map(|e| self.calculate_food_calories(&e.food_id, e.servings))
                    .sum();
                points.push([i as f64, total as f64]);
            }
        }
        PlotPoints::from(points)
    }
    
    // Helper to create styled headers
    fn h1(&self, ui: &mut egui::Ui, text: &str) {
        ui.add(egui::Label::new(
            RichText::new(text)
                .font(FontId::new(28.0, FontFamily::Proportional))
                .color(self.colors.text_primary)
        ));
        ui.add_space(10.0);
    }
    
    fn h2(&self, ui: &mut egui::Ui, text: &str) {
        ui.add(egui::Label::new(
            RichText::new(text)
                .font(FontId::new(22.0, FontFamily::Proportional))
                .color(self.colors.text_primary)
        ));
        ui.add_space(8.0);
    }
    
    fn card_frame(&self, ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
        egui::Frame::none()
            .fill(self.colors.bg_medium)
            .inner_margin(egui::style::Margin::same(16.0))
            .show(ui, |ui| {
                add_contents(ui);
            });
        ui.add_space(8.0);
    }
    
    
    
    fn render_food_database(&mut self, ui: &mut egui::Ui) {
        self.h1(ui, "Food Database");
        
        ui.horizontal(|ui| {
            ui.label(RichText::new("Search:").color(self.colors.text_primary));
            let search_field = egui::TextEdit::singleline(&mut self.food_search)
                .hint_text("Enter food name or keyword...")
                .text_color(self.colors.text_primary);
            ui.add(search_field);
        });
        ui.add_space(16.0);
        
        // Basic Foods Section
        self.h2(ui, "Basic Foods");
        
        let filtered_foods: Vec<_> = self.food_db.basic_foods.iter()
            .filter(|food| {
                self.food_search.is_empty() || 
                food.id.to_lowercase().contains(&self.food_search.to_lowercase()) ||
                food.keywords.iter().any(|kw| kw.to_lowercase().contains(&self.food_search.to_lowercase()))
            })
            .collect();
            
        if filtered_foods.is_empty() {
            ui.label(RichText::new("No matching foods found").color(self.colors.text_muted).italics());
        } else {
            egui::ScrollArea::vertical().id_source("basic_foods_scroll").show(ui, |ui| {
                for food in filtered_foods {
                    self.card_frame(ui, |ui| {
                        ui.add(egui::Label::new(
                            RichText::new(&food.id).strong().color(self.colors.text_primary).font(FontId::new(18.0, FontFamily::Proportional))
                        ));
                        ui.label(format!("Calories: {} kcal", food.calories));
                        ui.horizontal(|ui| {
                            ui.label("Keywords: ");
                            for kw in &food.keywords {
                                ui.label(RichText::new(kw).color(self.colors.text_primary));
                                ui.add_space(4.0);
                            }
                        });
                    });
                }
            });
        }
        
        ui.add_space(16.0);
        
        // Composite Foods Section  
        self.h2(ui, "Composite Foods");
        
        let filtered_composite: Vec<_> = self.food_db.composite_foods.iter()
            .filter(|food| {
                self.food_search.is_empty() || 
                food.id.to_lowercase().contains(&self.food_search.to_lowercase()) ||
                food.keywords.iter().any(|kw| kw.to_lowercase().contains(&self.food_search.to_lowercase()))
            })
            .collect();
            
        if filtered_composite.is_empty() {
            ui.label(RichText::new("No matching composite foods found").color(self.colors.text_muted).italics());
        } else {
            egui::ScrollArea::vertical().id_source("composite_foods_scroll").show(ui, |ui| {
                for food in filtered_composite {
                    let total_cal = self.food_db.compute_composite_calories(food).unwrap_or(0);
                    
                    self.card_frame(ui, |ui| {
                        ui.add(egui::Label::new(
                            RichText::new(&food.id).strong().color(self.colors.text_primary).font(FontId::new(18.0, FontFamily::Proportional))
                        ));
                        ui.label(RichText::new(format!("Total Calories: {} kcal", total_cal)).color(self.colors.text_secondary));
                        
                        ui.horizontal(|ui| {
                            ui.label("Keywords: ");
                            for kw in &food.keywords {
                                ui.label(RichText::new(kw).color(self.colors.text_primary));
                                ui.add_space(4.0);
                            }
                        });
                        
                        ui.collapsing("Ingredients", |ui| {
                            for (comp_id, servings) in &food.components {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(comp_id).color(self.colors.text_primary));
                                    ui.label(RichText::new(format!("x {:.1}", servings)).color(self.colors.text_secondary));
                                });
                            }
                        });
                    });
                }
            });
        }
    }
    
    fn render_daily_logs(&mut self, ui: &mut egui::Ui) {
        self.h1(ui, "Daily Food Logs");
        
        if self.dlogs.logs.is_empty() {
            ui.label(RichText::new("No food logs recorded yet").color(self.colors.text_muted).italics());
        } else {
            egui::ScrollArea::vertical().id_source("daily_logs_scroll").show(ui, |ui| {
                let mut dates: Vec<String> = self.dlogs.logs.keys().cloned().collect();
                dates.sort();
                dates.reverse(); // Most recent first
                
                for date in dates {
                    // Extract all needed data before closure
                    let log = self.dlogs.logs.get(&date).unwrap();
                    let entries_empty = log.entries.is_empty();
                    let total_cal: u32 = log.entries.iter()
                        .map(|entry| self.calculate_food_calories(&entry.food_id, entry.servings))
                        .sum();
                    let entry_data: Vec<(String, f32, u32)> = log.entries.iter()
                        .map(|e| (
                            e.food_id.clone(),
                            e.servings,
                            self.calculate_food_calories(&e.food_id, e.servings)
                        ))
                        .collect();
                    let is_selected = self.selected_date.as_ref().map_or(false, |d| d == &date);
                    let header_text = format!("{} ({} items, {} calories)", date, log.entries.len(), total_cal);
                    let date_clone = date.clone();
                    let header = if is_selected {
                        RichText::new(header_text)
                            .color(self.colors.accent_light)
                            .strong()
                            .font(FontId::new(18.0, FontFamily::Proportional))
                    } else {
                        RichText::new(header_text)
                            .color(self.colors.text_primary)
                            .font(FontId::new(18.0, FontFamily::Proportional))
                    };

                    let mut card_clicked = false; // mutable flag to capture click

                    self.card_frame(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.add(egui::Label::new(header).sense(egui::Sense::click())).clicked() {
                                card_clicked = true;
                            }
                            let progress_percent = (total_cal as f32 / 2000.0).min(1.0);
                            ui.add(egui::ProgressBar::new(progress_percent)
                                .text(format!("{} / 2000 cal", total_cal))
                            );
                        });

                        if is_selected {
                            ui.add_space(8.0);
                            ui.separator();
                            
                            // Display daily metrics if available
                            if let Some(info) = &log.daily_info {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Daily Metrics:").strong());
                                    ui.label(RichText::new(format!(
                                        "Age: {}, Weight: {} kg, Activity: {}", 
                                        info.age, info.weight, info.activity_level
                                    )).color(self.colors.text_secondary));
                                });
                                ui.add_space(4.0);
                            }

                            if entries_empty {
                                ui.label(RichText::new("No entries for this day")
                                    .color(self.colors.text_muted)
                                    .italics());
                            } else {
                                egui::Grid::new(format!("log_entries_{}", date_clone))
                                    .striped(true)
                                    .spacing([10.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label(RichText::new("Food").strong());
                                        ui.label(RichText::new("Servings").strong());
                                        ui.label(RichText::new("Calories").strong());
                                        ui.end_row();

                                        for (food_id, servings, calories) in &entry_data {
                                            ui.label(food_id);
                                            ui.label(format!("{:.1}", *servings));
                                            ui.label(format!("{}", calories));
                                            ui.end_row();
                                        }
                                    });
                            }
                        }
                    });
                    // Now update selected_date after rendering the card
                    if card_clicked {
                        if is_selected {
                            self.selected_date = None;
                        } else {
                            self.selected_date = Some(date_clone);
                        }
                    }
                }
            });
        }
    }
    
    fn render_insights(&mut self, ui: &mut egui::Ui) {
        self.h1(ui, "Nutrition Insights");
        
        if self.dlogs.logs.is_empty() {
            ui.label(RichText::new("Not enough data to generate insights").color(self.colors.text_muted).italics());
            return;
        }
        
        // Calculate daily average calories
        let mut total_calories = 0;
        let mut days_count = 0;
        let mut highest_day = (String::new(), 0);
        
        for (date, log) in &self.dlogs.logs {
            let day_calories: u32 = log.entries.iter()
                .map(|entry| self.calculate_food_calories(&entry.food_id, entry.servings))
                .sum();
                
            if day_calories > 0 {
                total_calories += day_calories;
                days_count += 1;
                
                if day_calories > highest_day.1 {
                    highest_day = (date.clone(), day_calories);
                }
            }
        }
        
        let avg_calories = if days_count > 0 { total_calories as f32 / days_count as f32 } else { 0.0 };
        
        // Summary stats
        self.card_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Average Daily Calories").color(self.colors.text_secondary));
                    ui.label(RichText::new(format!("{:.0}", avg_calories))
                        .color(self.colors.text_primary)
                        .font(FontId::new(24.0, FontFamily::Proportional))
                        .strong()
                    );
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label(RichText::new("Days Tracked").color(self.colors.text_secondary));
                    ui.label(RichText::new(format!("{}", days_count))
                        .color(self.colors.text_primary)
                        .font(FontId::new(24.0, FontFamily::Proportional))
                        .strong()
                    );
                });
                
                ui.separator();
                
                ui.vertical(|ui| {
                    ui.label(RichText::new("Highest Intake Day").color(self.colors.text_secondary));
                    ui.label(RichText::new(format!("{} ({})", highest_day.0, highest_day.1))
                        .color(self.colors.accent)
                        .font(FontId::new(18.0, FontFamily::Proportional))
                    );
                });
            });
        });
        
        ui.add_space(16.0);
        
        // Daily calories trend chart
        self.h2(ui, "Daily Calorie Trend");
        
        self.card_frame(ui, |ui| {
            Plot::new("daily_calorie_plot")
                .height(300.0)
                .legend(egui::plot::Legend::default())
                .include_y(0.0)
                .show(ui, |plot_ui| {
                    let points = self.insights_plot_data();
                    let line = Line::new(points)
                        .color(self.colors.primary)
                        .name("Daily Calories")
                        .width(3.0);
                    plot_ui.line(line);
                });
        });
        
        // Add more insights as needed
        ui.add_space(16.0);
        self.h2(ui, "Diet Composition");
        
        self.card_frame(ui, |ui| {
            // This would be a good place to add macronutrient breakdowns or other dietary analytics
            ui.label("This section will show macronutrient breakdowns and other dietary analytics");
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set a sophisticated dark theme
        {
            let mut style = (*ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(12.0, 12.0);
            style.spacing.button_padding = egui::vec2(10.0, 6.0);
            
            // The older egui/eframe API doesn't have as many styling options
            // So we'll use what's available
            style.visuals.dark_mode = true;
            // Use available fields instead of panel_fill which doesn't exist
            style.visuals.extreme_bg_color = self.colors.bg_dark;
            style.visuals.widgets.noninteractive.bg_fill = self.colors.bg_medium;
            style.visuals.widgets.inactive.bg_fill = self.colors.bg_medium;
            style.visuals.widgets.active.bg_fill = self.colors.primary;
            style.visuals.widgets.hovered.bg_fill = self.colors.bg_light;
            
            // Text colors
            style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, self.colors.text_primary);
            style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, self.colors.text_primary);
            style.visuals.selection.stroke = egui::Stroke::new(1.0, self.colors.primary);
            style.visuals.selection.bg_fill = self.colors.primary_dark;
            
            ctx.set_style(style);
        }
        
        // App header with navigation
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(
                    RichText::new("NUTRITRACK")
                        .font(FontId::new(22.0, FontFamily::Proportional))
                        .color(self.colors.text_primary)
                        .strong()
                ));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Navigation menu
                    for (idx, label) in [("Food Database", 0), ("Daily Logs", 1), ("Insights", 2)].iter() {
                        if ui.add(egui::SelectableLabel::new(
                            self.tab == *label,
                            RichText::new(*idx).font(FontId::new(16.0, FontFamily::Proportional))
                        )).clicked() {
                            self.tab = *label;
                        }
                    }
                });
            });
            ui.separator();
        });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            // Render content based on selected tab
            match self.tab {
                0 => self.render_food_database(ui),
                1 => self.render_daily_logs(ui),
                2 => self.render_insights(ui),
                _ => {}
            }
        });
    }
}

pub fn launch_gui(food_db: FoodDatabase, dlogs: DailyLogs) {
    let app = MyApp::new(food_db, dlogs);
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1100.0, 700.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native("NUTRITRACK · Diet Management", options, Box::new(|_cc| Box::new(app)));
}
