use eframe::egui;
use egui::{FontId, FontFamily, RichText, Color32};
use egui::plot::PlotPoints;
use crate::{FoodDatabase, DailyLogs};
use crate::calorie_calculator::calculate_calorie_target;
use crate::login::User;
use eframe::egui::{CentralPanel, SidePanel};

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

#[derive(PartialEq)]
pub enum Tab {
    FoodDatabase,
    DailyLogs,
    Insights,
}

pub struct MyApp {
    pub food_db: FoodDatabase,
    pub dlogs: DailyLogs,
    pub food_search: String,
    pub current_user: User,  // NEW: Add current user to the app
    pub selected_tab: Tab,  // NEW: store which tab is active
    colors: AppColors,
    pub selected_date: Option<String>,
}

impl MyApp {
    pub fn new(food_db: FoodDatabase, dlogs: DailyLogs, user: User) -> Self {  // Updated to include user
        Self { 
            food_db, 
            dlogs, 
            food_search: String::new(),
            current_user: user,  // Store the user
            selected_tab: Tab::FoodDatabase,
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
        self.h1(ui, &format!("Daily Food Logs for {}", self.current_user.username));
        
        if self.dlogs.logs.is_empty() {
            ui.label(RichText::new("No food logs recorded yet").color(self.colors.text_muted).italics());
        } else {
            egui::ScrollArea::vertical().id_source("daily_logs_scroll").show(ui, |ui| {
                // Filter logs by current user
                let mut dates: Vec<String> = self.dlogs.logs.keys()
                    .filter(|k| k.starts_with(&format!("{}:", self.current_user.username)))
                    .cloned()
                    .collect();
                
                // If no logs found for current user
                if dates.is_empty() {
                    ui.label(RichText::new(format!("No logs found for user {}", self.current_user.username))
                        .color(self.colors.text_muted).italics());
                    return;
                }
                
                dates.sort();
                dates.reverse(); // Most recent first
                
                for date in dates {
                    // Extract all needed data before closure
                    let log = self.dlogs.logs.get(&date).unwrap();
                    let entries_empty = log.entries.is_empty();
                    
                    // Calculate consumed calories
                    let total_cal: u32 = log.entries.iter()
                        .map(|entry| self.calculate_food_calories(&entry.food_id, entry.servings))
                        .sum();
                    
                    // Extract just the date part for display
                    let display_date = date.split(':').nth(1).unwrap_or(&date);
                    
                    // Calculate target calories using user's preferred method
                    let target_cal = if let Some(info) = &log.daily_info {
                        calculate_calorie_target(
                            &info.calorie_method,
                            &self.current_user.gender,
                            info.weight,
                            self.current_user.height,
                            info.age,
                            &info.activity_level
                        )
                    } else {
                        calculate_calorie_target(
                            &crate::calorie_calculator::CalorieMethod::default(),
                            &self.current_user.gender,
                            self.current_user.weight,
                            self.current_user.height,
                            self.current_user.age,
                            &self.current_user.activity_level
                        )
                    };
                    
                    let calorie_balance = total_cal as i32 - target_cal as i32;
                    let balance_color = if calorie_balance <= 0 {
                        self.colors.primary // under target - good
                    } else {
                        self.colors.accent // over target - warning
                    };
                    
                    let entry_data: Vec<(String, f32, u32)> = log.entries.iter()
                        .map(|e| (
                            e.food_id.clone(),
                            e.servings,
                            self.calculate_food_calories(&e.food_id, e.servings)
                        ))
                        .collect();
                    let is_selected = self.selected_date.as_ref().map_or(false, |d| d == &date);
                    let header_text = format!("{} ({} items)", display_date, log.entries.len());
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
                            
                            // Show calories with target
                            let balance_text = if calorie_balance <= 0 {
                                format!("{} / {} cal ({} remaining)", total_cal, target_cal, -calorie_balance)
                            } else {
                                format!("{} / {} cal ({} over)", total_cal, target_cal, calorie_balance)
                            };
                            
                            // Progress bar - removed the .fill() method which is not supported in this version
                            let progress_percent = (total_cal as f32 / target_cal as f32).min(1.0);
                            ui.add(egui::ProgressBar::new(progress_percent)
                                .text(balance_text)
                            );
                            
                            // Add a colored indicator next to the progress bar instead
                            if calorie_balance <= 0 {
                                ui.colored_label(self.colors.primary, "✓");
                            } else {
                                ui.colored_label(self.colors.accent, "!");
                            }
                        });

                        if is_selected {
                            ui.add_space(8.0);
                            ui.separator();
                            
                            // Display daily metrics and calorie balance
                            if let Some(info) = &log.daily_info {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Daily Metrics:").strong());
                                    ui.label(RichText::new(format!(
                                        "Age: {}, Weight: {} kg, Activity: {}", 
                                        info.age, info.weight, info.activity_level
                                    )).color(self.colors.text_secondary));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Calorie Method:").strong());
                                    ui.label(RichText::new(info.calorie_method.name())
                                        .color(self.colors.text_secondary));
                                });
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Calorie Status:").strong());
                                    let status_text = if calorie_balance <= 0 {
                                        format!("{} calories remaining", -calorie_balance)
                                    } else {
                                        format!("{} calories over target", calorie_balance)
                                    };
                                    ui.label(RichText::new(status_text).color(balance_color));
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
        self.h1(ui, &format!("Nutrition Insights for {}", self.current_user.username));
        
        // Filter logs by current user
        let user_logs: Vec<(&String, &crate::daily_logs::DailyLog)> = self.dlogs.logs.iter()
            .filter(|(k, _)| k.starts_with(&format!("{}:", self.current_user.username)))
            .collect();
        
        if user_logs.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(RichText::new("Not enough data to generate insights")
                    .color(self.colors.text_muted)
                    .size(20.0)
                    .italics());
                ui.add_space(10.0);
                ui.label(RichText::new("Try adding some food entries to your daily logs")
                    .color(self.colors.text_secondary));
            });
            return;
        }
        
        // Calculate key metrics and statistics
        let mut total_calories = 0;
        let mut days_count = 0;
        let mut highest_day = (String::new(), 0);
        let mut lowest_day = (String::new(), u32::MAX);
        let mut calorie_data: Vec<(String, u32)> = Vec::new();
        let mut avg_target = 0;
        let mut days_over_target = 0;
        
        for (date, log) in &user_logs {
            let display_date = date.split(':').nth(1).unwrap_or(date);
            
            // Calculate calories for this day
            let day_calories: u32 = log.entries.iter()
                .map(|entry| self.calculate_food_calories(&entry.food_id, entry.servings))
                .sum();
                
            if day_calories > 0 {
                // Track individual day data
                calorie_data.push((display_date.to_string(), day_calories));
                
                // Update aggregate stats
                total_calories += day_calories;
                days_count += 1;
                
                // Calculate target calories for this log
                let target_cal = if let Some(info) = &log.daily_info {
                    let target = crate::calorie_calculator::calculate_calorie_target(
                        &info.calorie_method,
                        &self.current_user.gender,
                        info.weight,
                        self.current_user.height,
                        info.age,
                        &info.activity_level
                    );
                    avg_target += target;
                    if day_calories > target {
                        days_over_target += 1;
                    }
                    target
                } else {
                    2000 // Default if no info
                };
                
                // Update highest and lowest days
                if day_calories > highest_day.1 {
                    highest_day = (display_date.to_string(), day_calories);
                }
                
                if day_calories < lowest_day.1 {
                    lowest_day = (display_date.to_string(), day_calories);
                }
            }
        }
        
        let avg_calories = if days_count > 0 { total_calories as f32 / days_count as f32 } else { 0.0 };
        let avg_target = if days_count > 0 { avg_target as f32 / days_count as f32 } else { 2000.0 };
        let target_achievement = if avg_target > 0.0 { avg_calories / avg_target } else { 0.0 };
        
        // Sort calorie data by date for proper display
        calorie_data.sort_by(|a, b| a.0.cmp(&b.0));
        
        // Calculate smoothed average trend for visualization
        let mut trend_data = Vec::new();
        let window_size = 3.min(calorie_data.len());
        if window_size > 0 {
            for i in 0..calorie_data.len() {
                let start = i.saturating_sub(window_size / 2);
                let end = (i + window_size / 2 + 1).min(calorie_data.len());
                let window_sum: u32 = calorie_data[start..end].iter().map(|(_, cal)| cal).sum();
                let window_avg = window_sum as f32 / (end - start) as f32;
                trend_data.push((i as f64, window_avg as f64));
            }
        }
        
        // Create summary stats cards in a row
        ui.horizontal(|ui| {
            // Left Stats Group
            ui.vertical(|ui| {
                egui::Frame::none()
                    .fill(self.colors.bg_light)
                    .rounding(8.0)
                    .inner_margin(egui::style::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new("CALORIC INTAKE")
                            .color(self.colors.text_secondary)
                            .size(14.0));
                        let display_avg = format!("{:.0}", avg_calories);
                        ui.add(egui::Label::new(
                            RichText::new(display_avg)
                                .color(self.colors.text_primary)
                                .font(FontId::new(32.0, FontFamily::Proportional))
                                .strong()
                        ));
                        ui.label(RichText::new("daily average calories")
                            .color(self.colors.text_muted)
                            .size(14.0));
                        
                        // Show target indicator
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let target_text = format!("{:.0}% of target", target_achievement * 100.0);
                            let target_color = if target_achievement <= 1.05 {
                                self.colors.primary
                            } else {
                                self.colors.accent
                            };
                            let icon = if target_achievement <= 1.0 { "✓" } else { "!" };
                            ui.label(RichText::new(icon).color(target_color).size(16.0));
                            ui.label(RichText::new(target_text).color(target_color).strong());
                        });
                    });
            });
            
            ui.add_space(16.0);
            
            // Middle Stats Group
            ui.vertical(|ui| {
                egui::Frame::none()
                    .fill(self.colors.bg_light)
                    .rounding(8.0)
                    .inner_margin(egui::style::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new("TRACKING STATS")
                            .color(self.colors.text_secondary)
                            .size(14.0));
                        ui.add(egui::Label::new(
                            RichText::new(format!("{}", days_count))
                                .color(self.colors.text_primary)
                                .font(FontId::new(32.0, FontFamily::Proportional))
                                .strong()
                        ));
                        ui.label(RichText::new("days tracked")
                            .color(self.colors.text_muted)
                            .size(14.0));
                            
                        // Tracking consistency stats  
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            let balance = days_count - days_over_target;
                            let percent_on_target = if days_count > 0 {
                                100.0 * balance as f32 / days_count as f32 
                            } else { 
                                0.0 
                            };
                            ui.label(RichText::new(format!("{:.0}%", percent_on_target))
                                .color(self.colors.primary)
                                .strong());
                            ui.label(RichText::new("days within target").color(self.colors.text_secondary));
                        });
                    });
            });
            
            ui.add_space(16.0);
            
            // Right Stats Group
            ui.vertical(|ui| {
                egui::Frame::none()
                    .fill(self.colors.bg_light)
                    .rounding(8.0)
                    .inner_margin(egui::style::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new("PEAK VALUES")
                            .color(self.colors.text_secondary)
                            .size(14.0));
                        ui.columns(2, |cols| {
                            // Highest day column
                            cols[0].vertical(|ui| {
                                ui.add(egui::Label::new(
                                    RichText::new(format!("{}", highest_day.1))
                                        .color(self.colors.accent)
                                        .font(FontId::new(24.0, FontFamily::Proportional))
                                        .strong()
                                ));
                                ui.label(RichText::new("highest")
                                    .color(self.colors.text_muted)
                                    .size(14.0));
                                ui.label(RichText::new(&highest_day.0)
                                    .color(self.colors.text_secondary)
                                    .size(14.0));
                            });
                            
                            // Lowest day column
                            cols[1].vertical(|ui| {
                                let lowest_cal = if lowest_day.1 == u32::MAX { 0 } else { lowest_day.1 };
                                ui.add(egui::Label::new(
                                    RichText::new(format!("{}", lowest_cal))
                                        .color(self.colors.primary)
                                        .font(FontId::new(24.0, FontFamily::Proportional))
                                        .strong()
                                ));
                                ui.label(RichText::new("lowest")
                                    .color(self.colors.text_muted)
                                    .size(14.0));
                                ui.label(RichText::new(&lowest_day.0)
                                    .color(self.colors.text_secondary)
                                    .size(14.0));
                            });
                        });
                    });
            });
        });
        
        ui.add_space(16.0);
        
        // Add nutrition insights section using cards in a grid
        ui.horizontal(|ui| {
            // Left panel - Estimated Macros
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width() / 2.0 - 10.0);
                self.h2(ui, "Nutrition Distribution");
                self.card_frame(ui, |ui| {
                    // Since we don't track actual macros, we'll show a simple estimate
                    // based on common macro ratios in typical diets
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Estimated Macronutrient Distribution")
                            .color(self.colors.text_primary)
                            .size(16.0));
                        ui.add_space(8.0);
                    });
                    
                    // Macro pie chart visualization (simulated with progress bars)
                    ui.vertical(|ui| {
                        // Protein: ~25%
                        ui.label(RichText::new("Protein").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::ProgressBar::new(0.25)
                                .text(format!("25% (est. {:.0}g)", avg_calories * 0.25 / 4.0)));
                        });
                        ui.add_space(8.0);
                        
                        // Carbs: ~50%
                        ui.label(RichText::new("Carbohydrates").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::ProgressBar::new(0.50)
                                .text(format!("50% (est. {:.0}g)", avg_calories * 0.50 / 4.0)));
                        });
                        ui.add_space(8.0);
                        
                        // Fat: ~25%
                        ui.label(RichText::new("Fat").strong());
                        ui.horizontal(|ui| {
                            ui.add(egui::ProgressBar::new(0.25)
                                .text(format!("25% (est. {:.0}g)", avg_calories * 0.25 / 9.0)));
                        });
                        
                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(RichText::new("* Based on estimated distribution for a typical balanced diet")
                            .color(self.colors.text_muted)
                            .italics()
                            .size(12.0));
                    });
                });
            });
            
            ui.add_space(16.0);
            
            // Right panel - AI Recommendations
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                self.h2(ui, "Smart Insights");
                self.card_frame(ui, |ui| {
                    // Generate recommendations based on user's data
                    ui.vertical_centered(|ui| {
                        ui.add_space(8.0);
                        ui.label(RichText::new("Personalized Recommendations")
                            .color(self.colors.text_primary)
                            .size(16.0));
                        ui.add_space(16.0);
                    });
                    
                    // Simulated AI recommendations based on user's data
                    let recommendations = if avg_calories > avg_target * 1.1 {
                        vec![
                            "Your calorie intake is consistently above your target. Consider adding more protein-rich foods to increase satiety.",
                            "Try adding a 30-minute walk to your daily routine to help balance your caloric intake.",
                            "Your highest calorie day was significantly above average. Identify what foods contributed most on that day."
                        ]
                    } else if avg_calories < avg_target * 0.9 {
                        vec![
                            "Your calorie intake is below your target. Consider adding healthy energy-dense foods like nuts or avocados.",
                            "Make sure you're getting enough essential nutrients despite the lower calorie intake.",
                            "Consider consulting with a nutrition professional if you're intentionally restricting calories."
                        ]
                    } else {
                        vec![
                            "Your calorie intake is well aligned with your targets. Keep up the good work!",
                            "Consider diversifying your food choices to ensure balanced nutrient intake.",
                            "Your consistent tracking is providing valuable insights into your nutrition patterns."
                        ]
                    };
                    
                    for (i, recommendation) in recommendations.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}.", i+1))
                                .color(self.colors.accent)
                                .strong());
                            ui.label(RichText::new(*recommendation)
                                .color(self.colors.text_primary));
                        });
                        ui.add_space(8.0);
                    }
                    
                    ui.add_space(8.0);
                    
                    // Optional goal setting prompt at the bottom
                    egui::Frame::none()
                        .fill(self.colors.bg_dark)
                        .inner_margin(egui::style::Margin::same(8.0))
                        .rounding(4.0)
                        .show(ui, |ui| {
                            ui.label(RichText::new("💡 Pro Tip: Setting specific nutrition goals can increase success rate by up to 70%")
                                .color(self.colors.accent_light)
                                .size(14.0));
                        });
                });
            });
        });
    }
    
    /// Build data for the Insights plot from daily logs for current user only
    fn user_insights_plot_data(&self) -> PlotPoints {
        let mut points = Vec::new();
        
        // Filter dates for current user
        let mut user_dates: Vec<&String> = self.dlogs.logs.keys()
            .filter(|k| k.starts_with(&format!("{}:", self.current_user.username)))
            .collect();
            
        user_dates.sort();
        
        for (i, date_key) in user_dates.iter().enumerate() {
            if let Some(log) = self.dlogs.logs.get(*date_key) {
                let total: u32 = log.entries.iter()
                    .map(|e| self.calculate_food_calories(&e.food_id, e.servings))
                    .sum();
                points.push([i as f64, total as f64]);
            }
        }
        
        PlotPoints::from(points)
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
        
        // Navigation panel on the left
        SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Menu");
            if ui.selectable_label(self.selected_tab == Tab::FoodDatabase, "Food Database").clicked() {
                self.selected_tab = Tab::FoodDatabase;
            }
            if ui.selectable_label(self.selected_tab == Tab::DailyLogs, "Daily Logs").clicked() {
                self.selected_tab = Tab::DailyLogs;
            }
            if ui.selectable_label(self.selected_tab == Tab::Insights, "Insights").clicked() {
                self.selected_tab = Tab::Insights;
            }
        });
        
        // Main content area
        CentralPanel::default().show(ctx, |ui| {
            match self.selected_tab {
                Tab::FoodDatabase => self.render_food_database(ui),
                Tab::DailyLogs => self.render_daily_logs(ui),
                Tab::Insights => self.render_insights(ui),
            }
        });
    }
}

pub fn launch_gui(food_db: FoodDatabase, dlogs: DailyLogs, user: User) {  // Updated to include user
    let app = MyApp::new(food_db, dlogs, user);
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1100.0, 700.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native("NUTRITRACK · Diet Management", options, Box::new(|_cc| Box::new(app)));
}
