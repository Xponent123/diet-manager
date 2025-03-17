use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use chrono::NaiveDate;
use crate::{FoodDatabase}; // import FoodDatabase from main.rs

/// Represents a single food consumption log entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub food_id: String,
    pub servings: f32,
}

/// Represents the log for a specific day.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DailyLog {
    pub date: String, // Format "YYYY-MM-DD"
    pub entries: Vec<LogEntry>,
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

    /// Get a mutable reference to the log for the given date.
    /// If no log exists for that date, a new one is created.
    pub fn get_log_mut(&mut self, date: &str) -> &mut DailyLog {
        self.logs.entry(date.to_string()).or_insert(DailyLog {
            date: date.to_string(),
            entries: Vec::new(),
        })
    }

    /// List all entries for a specified date.
    pub fn list_entries(&self, date: &str) {
        if let Some(log) = self.logs.get(date) {
            println!("Daily Log for {}:", date);
            for (i, entry) in log.entries.iter().enumerate() {
                println!("  {}: Food: {}, Servings: {}", i, entry.food_id, entry.servings);
            }
        } else {
            println!("No log found for {}.", date);
        }
    }

    /// Add an entry to the log for the specified date and record the action for undo.
    /// If a food with the same ID already exists in the log, it adds the servings instead of creating a new entry.
    pub fn add_entry(&mut self, date: &str, entry: LogEntry) {
        // First, check if the food already exists and prepare data
        let (needs_update, existing_idx, old_entry) = if let Some(log) = self.logs.get(date) {
            // Check if the food already exists in the log
            if let Some(idx) = log.entries.iter().position(|e| e.food_id == entry.food_id) {
                // Food already exists
                let old = log.entries[idx].clone();
                (true, idx, old)
            } else {
                (false, 0, entry.clone()) // No existing entry
            }
        } else {
            (false, 0, entry.clone()) // No log for this date
        };

        if needs_update {
            // Update existing entry
            let new_servings = old_entry.servings + entry.servings;
            let new_entry = LogEntry {
                food_id: old_entry.food_id.clone(),
                servings: new_servings,
            };
            
            // Get the log mut and update the entry
            let log = self.get_log_mut(date);
            log.entries[existing_idx] = new_entry.clone();
            
            // Record the update in the undo stack
            self.undo_stack.push(LogCommand::UpdateEntry {
                date: date.to_string(),
                old_entry,
                new_entry,
                index: existing_idx,
            });
            println!("Added servings to existing food entry.");
        } else {
            // Add new food entry
            let log = self.get_log_mut(date);
            log.entries.push(entry.clone());
            
            // Record the add in the undo stack
            self.undo_stack.push(LogCommand::AddEntry {
                date: date.to_string(),
                entry,
            });
            println!("Entry added.");
        }
    }

    /// Delete the entry at the given index from the log for the specified date and record the action for undo.
    pub fn delete_entry(&mut self, date: &str, index: usize) {
        if let Some(log) = self.logs.get_mut(date) {
            if index < log.entries.len() {
                let entry = log.entries.remove(index);
                self.undo_stack.push(LogCommand::DeleteEntry { date: date.to_string(), entry, index });
                println!("Entry deleted.");
            } else {
                println!("Invalid index.");
            }
        } else {
            println!("No log for date {}.", date);
        }
    }

    /// Update the entry at the given index for the specified date with a new entry, and record the change for undo.
    pub fn update_entry(&mut self, date: &str, index: usize, new_entry: LogEntry) {
        if let Some(log) = self.logs.get_mut(date) {
            if index < log.entries.len() {
                let old_entry = log.entries[index].clone();
                log.entries[index] = new_entry.clone();
                self.undo_stack.push(LogCommand::UpdateEntry { date: date.to_string(), old_entry, new_entry, index });
                println!("Entry updated.");
            } else {
                println!("Invalid index.");
            }
        } else {
            println!("No log for date {}.", date);
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
                    let log = self.get_log_mut(&date);
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
}

// Add a helper function to validate date format.
fn validate_date(date: &str) -> bool {
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
pub fn daily_logs_menu(dlogs: &mut DailyLogs, db: &FoodDatabase) {
    loop {
        println!("\nDaily Logs Menu:");
        println!("Enter a date (YYYY-MM-DD) to select its log, or type 'back' to return:");
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
            println!("\nDaily Log for {}:", date);
            println!("1. List entries");
            println!("2. Add entry");
            println!("3. Delete entry");
            println!("4. Update entry");
            println!("5. Undo last command");
            println!("6. Back to date selection");
            println!("7. Return to Main Menu");  // New option.
            print!("Enter choice: ");
            io::stdout().flush().unwrap();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            match choice.trim() {
                "1" => dlogs.list_entries(date),
                "2" => {
                    // New add entry: select food from FoodDatabase.
                    if let Some(food_id) = choose_food(db) {
                        print!("Enter number of servings: ");
                        io::stdout().flush().unwrap();
                        let mut servings_str = String::new();
                        io::stdin().read_line(&mut servings_str).unwrap();
                        if let Ok(servings) = servings_str.trim().parse::<f32>() {
                            let entry = LogEntry { food_id, servings };
                            dlogs.add_entry(date, entry);
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
                        dlogs.delete_entry(date, index);
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
                            if let Some(log) = dlogs.logs.get(date) {
                                if index < log.entries.len() {
                                    let old_entry = log.entries[index].clone();
                                    let new_entry = LogEntry { food_id: old_entry.food_id.clone(), servings: new_servings };
                                    dlogs.update_entry(date, index, new_entry);
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
                "6" => break,
                "7" => return, // Direct return to main menu.
                _ => println!("Invalid choice."),
            }
        }
    }
}
