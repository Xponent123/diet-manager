use serde::{Serialize, Deserialize}; 
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use chrono::NaiveDate;
use crate::FoodDatabase; // import FoodDatabase from main.rs
use crate::calorie_calculator::{CalorieMethod, calculate_calorie_target};

/// Represents a single food consumption log entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)] 
pub struct LogEntry {
    pub food_id: String,
    pub servings: f32,
}

/// User's daily metrics that can change each day
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DailyUserInfo {
    pub age: u32,
    pub weight: f32,
    pub activity_level: String,
    #[serde(default)]  // Default to MifflinStJeor if missing
    pub calorie_method: CalorieMethod,
}

/// Represents the log for a specific day.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DailyLog {
    pub date: String, // "YYYY-MM-DD"
    #[serde(default)]   // NEW: if missing, default to empty string
    pub username: String, // owner of this log
    pub entries: Vec<LogEntry>,
    pub daily_info: Option<DailyUserInfo>,  // NEW: daily metrics
}

/// The various command actions that can be undone.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogCommand {
    AddEntry { date: String, entry: LogEntry },
    DeleteEntry { date: String, entry: LogEntry, index: usize },
    UpdateEntry { date: String, old_entry: LogEntry, new_entry: LogEntry, index: usize },
}

/// Holds all daily logs (keyed by date) and a runtime undo stack.
#[derive(Serialize, Deserialize, Debug, Clone)]  // Added Clone trait
pub struct DailyLogs {
    pub logs: BTreeMap<String, DailyLog>,
    #[serde(skip)]
    pub undo_stack: Vec<LogCommand>,
}

impl DailyLogs {
    pub fn new() -> Self {
        DailyLogs {
            logs: BTreeMap::new(),
            undo_stack: Vec::new(),
        }
    }

    /// Load daily logs from the specified JSON file.
    pub fn load_from_file(file_path: &str) -> Self {
        match fs::read_to_string(file_path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_else(|err| {
                println!("Error parsing daily logs file: {}. Starting with empty logs.", err);
                DailyLogs::new()
            }),
            Err(_) => {
                println!("Daily logs file not found. Starting with empty logs.");
                DailyLogs::new()
            }
        }
    }

    /// Save the daily logs to the specified JSON file.
    pub fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let data = serde_json::to_string_pretty(&self).unwrap();
        fs::write(file_path, data)
    }

    /// Helper: Get previous day's metrics for the given user if available.
    fn get_previous_daily_info(&self, username: &str, date: &str) -> Option<DailyUserInfo> {
        // Try to parse the current date
        let curr_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
        
        // Collect all previous logs for this user with valid dates
        let mut prior_infos: Vec<(NaiveDate, DailyUserInfo)> = Vec::new();
        
        // Fix unused variable warning
        for (_key, log) in &self.logs {
            // Only consider logs for this user
            if log.username.eq_ignore_ascii_case(username) {
                if let Ok(log_date) = NaiveDate::parse_from_str(&log.date, "%Y-%m-%d") {
                    // Only consider dates before current date
                    if log_date < curr_date {
                        // If this log has daily info, add it to our collection
                        if let Some(info) = &log.daily_info {
                            prior_infos.push((log_date, info.clone()));
                        }
                    }
                }
            }
        }
        
        // Sort by date descending, most recent first
        prior_infos.sort_by(|a, b| b.0.cmp(&a.0));
        
        // Return the most recent info if available
        prior_infos.first().map(|(_, info)| info.clone())
    }

    /// Get a mutable reference to the log for the given date and user.
    /// If no log exists for that date for this user, a new one is created.
    pub fn get_log_mut_for_user(&mut self, date: &str, username: &str) -> &mut DailyLog {
        let key = format!("{}:{}", username, date);
        
        // If the log doesn't exist, we'll need to get previous day info
        if !self.logs.contains_key(&key) {
            let prev_info = self.get_previous_daily_info(username, date);
            
            self.logs.insert(key.clone(), DailyLog {
                date: date.to_string(),
                username: username.to_string(),
                entries: Vec::new(),
                daily_info: prev_info,
            });
        }
        
        self.logs.get_mut(&key).unwrap()
    }

    /// List all entries for a specified date and user.
    pub fn list_entries_for_user(&self, date: &str, username: &str) {
        let key = format!("{}:{}", username, date);
        if let Some(log) = self.logs.get(&key) {
            println!("Daily Log for {}:", date);
            
            // Show daily metrics if available
            if let Some(info) = &log.daily_info {
                println!("  Daily Metrics - Age: {}, Weight: {} kg, Activity Level: {}", 
                         info.age, info.weight, info.activity_level);
            }
            
            for (i, entry) in log.entries.iter().enumerate() {
                println!("  {}: Food: {}, Servings: {}", i, entry.food_id, entry.servings);
            }
        } else {
            println!("No log found for {}.", date);
        }
    }

    /// Add an entry to the log for the specified date and record the action for undo.
    /// If a food with the same ID already exists in the log, it adds the servings instead of creating a new entry.
    pub fn add_entry(&mut self, key: &str, entry: LogEntry) {
        let (needs_update, existing_idx, old_entry) = if let Some(log) = self.logs.get(key) {
            if let Some(idx) = log.entries.iter().position(|e| e.food_id == entry.food_id) {
                let old = log.entries[idx].clone();
                (true, idx, old)
            } else {
                (false, 0, entry.clone())
            }
        } else {
            (false, 0, entry.clone())
        };

        if needs_update {
            let new_servings = old_entry.servings + entry.servings;
            let new_entry = LogEntry {
                food_id: old_entry.food_id.clone(),
                servings: new_servings,
            };
            let log = self.get_log_mut_generic(key);
            log.entries[existing_idx] = new_entry.clone();
            self.undo_stack.push(LogCommand::UpdateEntry {
                date: key.to_string(),
                old_entry,
                new_entry,
                index: existing_idx,
            });
            println!("Added servings to existing food entry.");
        } else {
            let log = self.get_log_mut_generic(key);
            log.entries.push(entry.clone());
            self.undo_stack.push(LogCommand::AddEntry {
                date: key.to_string(),
                entry,
            });
            println!("Entry added.");
        }
    }

    /// Delete the entry at the given index from the log for the specified date and record the action for undo.
    pub fn delete_entry(&mut self, key: &str, index: usize) {
        if let Some(log) = self.logs.get_mut(key) {
            if index < log.entries.len() {
                let entry = log.entries.remove(index);
                self.undo_stack.push(LogCommand::DeleteEntry {
                    date: key.to_string(),
                    entry,
                    index,
                });
                println!("Entry deleted.");
            } else {
                println!("Invalid index.");
            }
        } else {
            println!("No log for key {}.", key);
        }
    }

    /// Update the entry at the given index for the specified date with a new entry, and record the change for undo.
    pub fn update_entry(&mut self, key: &str, index: usize, new_entry: LogEntry) {
        if let Some(log) = self.logs.get_mut(key) {
            if index < log.entries.len() {
                let old_entry = log.entries[index].clone();
                log.entries[index] = new_entry.clone();
                self.undo_stack.push(LogCommand::UpdateEntry {
                    date: key.to_string(),
                    old_entry,
                    new_entry,
                    index,
                });
                println!("Entry updated.");
            } else {
                println!("Invalid index.");
            }
        } else {
            println!("No log for key {}.", key);
        }
    }

    /// Undo the last command that modified the daily logs.
    pub fn undo_last(&mut self) {
        if let Some(command) = self.undo_stack.pop() {
            match command {
                LogCommand::AddEntry { date, entry } => {
                    if let Some(log) = self.logs.get_mut(&date) {
                        if let Some(pos) = log.entries.iter().position(|e| e == &entry) {
                            log.entries.remove(pos);
                            println!("Undo: Removed added entry.");
                        }
                    }
                },
                LogCommand::DeleteEntry { date, entry, index } => {
                    // Changed from get_log_mut to get_log_mut_generic
                    let log = self.get_log_mut_generic(&date);
                    if index <= log.entries.len() {
                        log.entries.insert(index, entry);
                        println!("Undo: Reinserted deleted entry.");
                    }
                },
                LogCommand::UpdateEntry { date, old_entry, new_entry: _, index } => {
                    if let Some(log) = self.logs.get_mut(&date) {
                        if index < log.entries.len() {
                            log.entries[index] = old_entry;
                            println!("Undo: Reverted update on entry.");
                        }
                    }
                },
            }
        } else {
            println!("No commands to undo.");
        }
    }

    fn get_log_mut_generic(&mut self, key: &str) -> &mut DailyLog {
        self.logs.entry(key.to_string()).or_insert(DailyLog {
            date: key.split(':').nth(1).unwrap_or("").to_string(),
            username: key.split(':').nth(0).unwrap_or("").to_string(),
            entries: Vec::new(),
            daily_info: None,
        })
    }
}

// Add a helper function to validate date format.
pub fn validate_date(date: &str) -> bool {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
}

/// Helper function to allow user selection of a food.
/// Users can choose to list all foods or search by keywords (match "all" or "any").
pub fn choose_food(db: &FoodDatabase) -> Option<String> {
    println!("Select food selection method:");
    println!("1. List all foods");
    println!("2. Search by keywords");
    print!("Enter choice: ");
    io::stdout().flush().unwrap();
    let mut method = String::new();
    io::stdin().read_line(&mut method).unwrap();
    match method.trim() {
        "1" => {
            let mut all_foods: Vec<String> = Vec::new();  // Add 'mut' back since we push to it
            println!("Basic Foods:");
            for food in &db.basic_foods {
                println!("{}: {} (Calories: {})", all_foods.len(), food.id, food.calories);
                all_foods.push(food.id.clone());
            }
            println!("Composite Foods:");
            for food in &db.composite_foods {
                let total = db.compute_composite_calories(food).unwrap_or(0);
                println!("{}: {} (Total Calories: {})", all_foods.len(), food.id, total);
                all_foods.push(food.id.clone());
            }
            if all_foods.is_empty() {
                println!("No foods available.");
                return None;
            }
            print!("Enter index: ");
            io::stdout().flush().unwrap();
            let mut index_str = String::new();
            io::stdin().read_line(&mut index_str).unwrap();
            if let Ok(index) = index_str.trim().parse::<usize>() {
                if index < all_foods.len() {
                    return Some(all_foods[index].clone());
                } else {
                    println!("Invalid index.");
                    return None;
                }
            } else {
                println!("Invalid input.");
                return None;
            }
        }
        "2" => {
            print!("Enter comma-separated keywords: ");
            io::stdout().flush().unwrap();
            let mut kw_str = String::new();
            io::stdin().read_line(&mut kw_str).unwrap();
            let keywords: Vec<String> = kw_str.trim().split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
            if keywords.is_empty() {
                println!("No keywords entered.");
                return None;
            }
            print!("Match 'all' keywords or 'any'? (enter all/any): ");
            io::stdout().flush().unwrap();
            let mut mode = String::new();
            io::stdin().read_line(&mut mode).unwrap();
            let mode = mode.trim().to_lowercase();
            println!("Matching Foods:");
            let mut all_foods: Vec<String> = Vec::new(); // Keep only one declaration
            let matches_keywords = |food_keywords: &Vec<String>| {
                let lower: Vec<String> = food_keywords.iter().map(|k| k.to_lowercase()).collect();
                if mode == "all" {
                    keywords.iter().all(|kw| lower.contains(kw))
                } else {
                    keywords.iter().any(|kw| lower.contains(kw))
                }
            };
            for food in &db.basic_foods {
                if matches_keywords(&food.keywords) {
                    println!("{}: {} (Calories: {})", all_foods.len(), food.id, food.calories);
                    all_foods.push(food.id.clone());
                }
            }
            for food in &db.composite_foods {
                if matches_keywords(&food.keywords) {
                    let total = db.compute_composite_calories(food).unwrap_or(0);
                    println!("{}: {} (Total Calories: {})", all_foods.len(), food.id, total);
                    all_foods.push(food.id.clone());
                }
            }
            if all_foods.is_empty() {
                println!("No matching foods found.");
                return None;
            }
            print!("Enter index: ");
            io::stdout().flush().unwrap();
            let mut index_str = String::new();
            io::stdin().read_line(&mut index_str).unwrap();
            if let Ok(index) = index_str.trim().parse::<usize>() {
                if index < all_foods.len() {
                    return Some(all_foods[index].clone());
                } else {
                    println!("Invalid index.");
                    return None;
                }
            } else {
                println!("Invalid input.");
                return None;
            }
        }
        _ => {
            println!("Invalid selection method.");
            None
        }
    }
}

/// Presents a menu for daily logs operations for a selected date.
pub fn daily_logs_menu(dlogs: &mut DailyLogs, db: &FoodDatabase, user: &crate::login::User) {
    loop {
        println!("\nDaily Logs Menu:");
        println!("Enter a date (YYYY-MM-DD) to select its log for user '{}', or type 'back' to return:", user.username);
        let mut date = String::new();
        io::stdin().read_line(&mut date).unwrap();
        let date = date.trim();
        if date.eq_ignore_ascii_case("back") {
            break;
        }
        // Validate date format.
        if !validate_date(date) {
            println!("Invalid date format. Please enter a date in YYYY-MM-DD format.");
            continue;
        }
        // Once a date is selected, enter the submenu for that day's log.
        loop {
            // Calculate daily calories consumed vs target
            let key = format!("{}:{}", user.username, date);
            let daily_calories = if let Some(log) = dlogs.logs.get(&key) {
                log.entries.iter()
                    .map(|entry| db.calculate_food_calories(&entry.food_id, entry.servings))
                    .sum::<u32>()
            } else {
                0
            };
            
            // Calculate target calories based on user info and chosen method
            let target_calories = if let Some(log) = dlogs.logs.get(&key) {
                if let Some(info) = &log.daily_info {
                    calculate_calorie_target(
                        &info.calorie_method,
                        &user.gender,
                        info.weight,
                        user.height,
                        info.age,
                        &info.activity_level
                    )
                } else {
                    calculate_calorie_target(
                        &CalorieMethod::default(),
                        &user.gender,
                        user.weight,
                        user.height,
                        user.age,
                        &user.activity_level
                    )
                }
            } else {
                calculate_calorie_target(
                    &CalorieMethod::default(),
                    &user.gender,
                    user.weight,
                    user.height,
                    user.age,
                    &user.activity_level
                )
            };
            
            let calorie_balance = daily_calories as i32 - target_calories as i32;
            let balance_str = if calorie_balance <= 0 {
                format!("{} calories remaining", -calorie_balance)
            } else {
                format!("{} calories over target", calorie_balance)
            };
            
            println!("\nDaily Log for {}: {} of {} target calories ({})", 
                     date, daily_calories, target_calories, balance_str);
            
            println!("1. List entries");
            println!("2. Add entry");
            println!("3. Delete entry");
            println!("4. Update entry");
            println!("5. Undo last command");
            println!("6. Update Daily Metrics");
            // Remove option 7 (Change Calorie Calculation Method) as it's now in the User Menu
            println!("7. Back to date selection");
            println!("8. Return to User Menu");  // Changed from "Return to Main Menu"
            print!("Enter choice: ");
            io::stdout().flush().unwrap();
            
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            match choice.trim() {
                "1" => {
                    DailyLogs::list_entries_for_user(dlogs, date, &user.username);
                },
                "2" => {
                    if let Some(food_id) = choose_food(db) {
                        print!("Enter number of servings: ");
                        io::stdout().flush().unwrap();
                        let mut servings_str = String::new();
                        io::stdin().read_line(&mut servings_str).unwrap();
                        if let Ok(servings) = servings_str.trim().parse::<f32>() {
                            let entry = LogEntry { food_id, servings };
                            let log = dlogs.get_log_mut_for_user(date, &user.username);
                            log.entries.push(entry.clone());
                            dlogs.undo_stack.push(LogCommand::AddEntry {
                                date: format!("{}:{}", user.username, date),
                                entry,
                            });
                            println!("Entry added.");
                        } else {
                            println!("Invalid servings input.");
                        }
                    }
                },
                "3" => {
                    let mut index_str = String::new();
                    println!("Enter entry index to delete:");
                    io::stdin().read_line(&mut index_str).unwrap();
                    if let Ok(index) = index_str.trim().parse::<usize>() {
                        dlogs.delete_entry(&format!("{}:{}", user.username, date), index);
                    } else {
                        println!("Invalid index input.");
                    }
                },
                "4" => {
                    let mut index_str = String::new();
                    let mut new_servings_str = String::new();
                    println!("Enter entry index to update:");
                    io::stdin().read_line(&mut index_str).unwrap();
                    if let Ok(index) = index_str.trim().parse::<usize>() {
                        println!("Enter new number of servings:");
                        io::stdout().flush().unwrap();
                        io::stdin().read_line(&mut new_servings_str).unwrap();
                        if let Ok(new_servings) = new_servings_str.trim().parse::<f32>() {
                            if let Some(log) = dlogs.logs.get(&format!("{}:{}", user.username, date)) {
                                if index < log.entries.len() {
                                    let old_entry = log.entries[index].clone();
                                    let new_entry = LogEntry { food_id: old_entry.food_id.clone(), servings: new_servings };
                                    dlogs.update_entry(&format!("{}:{}", user.username, date), index, new_entry);
                                } else {
                                    println!("Invalid index.");
                                }
                            } else {
                                println!("No log for this date.");
                            }
                        } else {
                            println!("Invalid servings input.");
                        }
                    } else {
                        println!("Invalid index input.");
                    }
                },
                "5" => dlogs.undo_last(),
                "6" => {
                    // NEW: Update daily metrics
                    let log = dlogs.get_log_mut_for_user(date, &user.username);
                    
                    println!("Current Daily Metrics:");
                    if let Some(info) = &log.daily_info {
                        println!("Age: {}, Weight: {} kg, Activity Level: {}", 
                                 info.age, info.weight, info.activity_level);
                    } else {
                        println!("No metrics set yet. Using account defaults.");
                    }
                    
                    // Determine default values - use current metrics, previous day's metrics, or account info
                    let default_age = log.daily_info.as_ref().map_or(user.age, |info| info.age);
                    let default_weight = log.daily_info.as_ref().map_or(user.weight, |info| info.weight);
                    let default_activity = log.daily_info.as_ref().map_or(user.activity_level.clone(), |info| info.activity_level.clone());
                    
                    // Age input (optional, press Enter to keep default)
                    print!("Enter age [{}]: ", default_age);
                    io::stdout().flush().unwrap();
                    let mut age_input = String::new();
                    io::stdin().read_line(&mut age_input).unwrap();
                    let age = if age_input.trim().is_empty() {
                        default_age
                    } else {
                        age_input.trim().parse().unwrap_or(default_age)
                    };
                    
                    // Weight input (optional, press Enter to keep default)
                    print!("Enter weight in kg [{}]: ", default_weight);
                    io::stdout().flush().unwrap();
                    let mut weight_input = String::new();
                    io::stdin().read_line(&mut weight_input).unwrap();
                    let weight = if weight_input.trim().is_empty() {
                        default_weight
                    } else {
                        weight_input.trim().parse().unwrap_or(default_weight)
                    };
                    
                    // Activity level input (optional, press Enter to keep default)
                    print!("Enter activity level (low/moderate/high) [{}]: ", default_activity);
                    io::stdout().flush().unwrap();
                    let mut activity_input = String::new();
                    io::stdin().read_line(&mut activity_input).unwrap();
                    let activity_level = if activity_input.trim().is_empty() {
                        default_activity
                    } else {
                        activity_input.trim().to_string()
                    };
                    
                    // Update the daily metrics
                    log.daily_info = Some(DailyUserInfo {
                        age,
                        weight,
                        activity_level,
                        calorie_method: log.daily_info.as_ref().map_or(CalorieMethod::default(), |info| info.calorie_method.clone()),
                    });
                    
                    println!("Daily metrics updated successfully.");
                },
                "7" => break,
                "8" => return, // Return to User Menu
                _ => println!("Invalid choice."),
            }
        }
    }
}
