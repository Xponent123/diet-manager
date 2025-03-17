use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use std::fs;
use std::collections::BTreeMap;
// mod daily_logs;
// use daily_logs::{DailyLogs, daily_logs_menu};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BasicFood {
    pub id: String,
    pub keywords: Vec<String>,
    pub calories: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompositeFood {
    pub id: String,
    pub keywords: Vec<String>,
    // Each component is defined as (food identifier, servings)
    pub components: Vec<(String, f32)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FoodDatabase {
    pub basic_foods: Vec<BasicFood>,
    pub composite_foods: Vec<CompositeFood>,
}

impl FoodDatabase {
    pub fn new() -> Self {
        FoodDatabase {
            basic_foods: Vec::new(),
            composite_foods: Vec::new(),
        }
    }

    /// Load the food database from a JSON file. If the file does not exist or cannot be parsed,
    /// an empty database is returned.
    pub fn load_from_file(file_path: &str) -> Self {
        match fs::read_to_string(file_path) {
            Ok(data) => {
                serde_json::from_str(&data).unwrap_or_else(|err| {
                    println!("Error parsing database file: {}. Starting with empty database.", err);
                    FoodDatabase::new()
                })
            },
            Err(_) => {
                println!("Database file not found. Starting with empty database.");
                FoodDatabase::new()
            }
        }
    }

    /// Save the database to the given file path in pretty JSON format.
    pub fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let data = serde_json::to_string_pretty(&self).unwrap();
        fs::write(file_path, data)
    }

    /// Checks if a food with the given id exists in basic or composite foods (case-insensitive).
    pub fn food_exists(&self, id: &str) -> bool {
        self.basic_foods.iter().any(|b| b.id.eq_ignore_ascii_case(id)) ||
        self.composite_foods.iter().any(|c| c.id.eq_ignore_ascii_case(id))
    }

    /// Adds a new basic food only if a food with the same id (case-insensitive) does not exist.
    pub fn add_basic_food(&mut self, food: BasicFood) {
        if self.food_exists(&food.id) {
            println!("Basic food with id '{}' already exists.", food.id);
            return;
        }
        self.basic_foods.push(food);
    }

    /// Adds a new composite food only if a food with the same id (case-insensitive) does not exist.
    pub fn add_composite_food(&mut self, food: CompositeFood) {
        if self.food_exists(&food.id) {
            println!("Composite food with id '{}' already exists.", food.id);
            return;
        }
        self.composite_foods.push(food);
    }

    /// List all basic and composite foods.
    pub fn list_foods(&self) {
        println!("Basic Foods:");
        for food in &self.basic_foods {
            println!("  {} - Keywords: {:?}, Calories: {}", food.id, food.keywords, food.calories);
        }
        println!("Composite Foods:");
        for food in &self.composite_foods {
            let total_calories = self.compute_composite_calories(food).unwrap_or(0);
            println!("  {} - Keywords: {:?}, Components: {:?}, Total Calories: {}",
                     food.id, food.keywords, food.components, total_calories);
        }
    }

    /// Given a composite food, compute its total calories per serving by summing the calories of its components.
    pub fn compute_composite_calories(&self, composite: &CompositeFood) -> Option<u32> {
        let mut total = 0;
        for (component_id, servings) in &composite.components {
            // Check basic foods first.
            if let Some(basic) = self.basic_foods.iter().find(|b| b.id.eq_ignore_ascii_case(component_id)) {
                total += (basic.calories as f32 * servings) as u32;
            } else if let Some(comp) = self.composite_foods.iter().find(|c| c.id.eq_ignore_ascii_case(component_id)) {
                if let Some(cal) = self.compute_composite_calories(comp) {
                    total += (cal as f32 * servings) as u32;
                } else {
                    return None;
                }
            } else {
                // Component not found in either list.
                return None;
            }
        }
        Some(total)
    }
}

/// When adding a component to a composite food, check if a basic food with the given id exists.
/// If not, prompt the user to add it as a basic food.
/// (Note: This function checks only by id.)
fn ensure_basic_food_exists(db: &mut FoodDatabase, id: &str) -> String {
    if let Some(b) = db.basic_foods.iter().find(|b| b.id.eq_ignore_ascii_case(id)) {
        return b.id.clone();
    }
    // If the food exists in composite foods, we return its id (though ideally composite foods
    // should not be used as components in this prototype).
    if let Some(c) = db.composite_foods.iter().find(|c| c.id.eq_ignore_ascii_case(id)) {
        return c.id.clone();
    }
    println!("Basic food with id '{}' not found.", id);
    println!("Please add '{}' as a basic food.", id);

    let mut keywords_str = String::new();
    let mut calories_str = String::new();

    println!("Enter keywords for '{}' (comma separated):", id);
    io::stdin().read_line(&mut keywords_str).unwrap();
    let keywords: Vec<String> = keywords_str
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    println!("Enter calories per serving for '{}':", id);
    io::stdin().read_line(&mut calories_str).unwrap();
    let calories: u32 = calories_str.trim().parse().unwrap_or(0);

    let food = BasicFood {
        id: id.to_string(),
        keywords,
        calories,
    };
    db.add_basic_food(food);
    println!("Basic food '{}' added.", id);
    id.to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub food_id: String,
    pub servings: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DailyLog {
    pub date: String, // Format "YYYY-MM-DD"
    pub entries: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogCommand {
    AddEntry { date: String, entry: LogEntry },
    DeleteEntry { date: String, entry: LogEntry, index: usize },
    UpdateEntry { date: String, old_entry: LogEntry, new_entry: LogEntry, index: usize },
}

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let data = serde_json::to_string_pretty(&self).unwrap();
        fs::write(file_path, data)
    }

    pub fn get_log_mut(&mut self, date: &str) -> &mut DailyLog {
        self.logs.entry(date.to_string()).or_insert(DailyLog {
            date: date.to_string(),
            entries: Vec::new(),
        })
    }

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

    pub fn add_entry(&mut self, date: &str, entry: LogEntry) {
        let log = self.get_log_mut(date);
        log.entries.push(entry.clone());
        self.undo_stack.push(LogCommand::AddEntry { date: date.to_string(), entry });
        println!("Entry added.");
    }

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

pub fn daily_logs_menu(dlogs: &mut DailyLogs) {
    loop {
        println!("\nDaily Logs Menu:");
        println!("Enter a date (YYYY-MM-DD) to select its log, or type 'back' to return:");
        let mut date = String::new();
        io::stdin().read_line(&mut date).unwrap();
        let date = date.trim();
        if date.eq_ignore_ascii_case("back") {
            break;
        }
        loop {
            println!("\nDaily Log for {}:", date);
            println!("1. List entries");
            println!("2. Add entry");
            println!("3. Delete entry");
            println!("4. Update entry");
            println!("5. Undo last command");
            println!("6. Back to date selection");
            print!("Enter choice: ");
            io::stdout().flush().unwrap();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            match choice.trim() {
                "1" => dlogs.list_entries(date),
                "2" => {
                    let mut food_id = String::new();
                    let mut servings_str = String::new();
                    println!("Enter food identifier:");
                    io::stdin().read_line(&mut food_id).unwrap();
                    let food_id = food_id.trim().to_string();
                    println!("Enter number of servings:");
                    io::stdin().read_line(&mut servings_str).unwrap();
                    let servings: f32 = servings_str.trim().parse().unwrap_or(1.0);
                    let entry = LogEntry { food_id, servings };
                    dlogs.add_entry(date, entry);
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
                _ => println!("Invalid choice."),
            }
        }
    }
}

fn main() {
    // Load Food Database from file.
    let db_file = "food_database.json";
    let mut db = FoodDatabase::load_from_file(db_file);
    
    // Load Daily Logs from file.
    let logs_file = "daily_logs.json";
    let mut daily_logs = DailyLogs::load_from_file(logs_file);

    loop {
        println!("\nMain Menu:");
        println!("1. Food Database Menu");
        println!("2. Daily Logs Menu");
        println!("3. Save Food Database");
        println!("4. Save Daily Logs");
        println!("5. Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        match choice.trim() {
            "1" => {
                // Food Database sub-menu (reuse existing loop)
                loop {
                    println!("\nFood Database Menu:");
                    println!("1. List foods");
                    println!("2. Add basic food");
                    println!("3. Add composite food");
                    println!("4. Back to Main Menu");
                    print!("Enter choice: ");
                    io::stdout().flush().unwrap();
                    let mut sub_choice = String::new();
                    io::stdin().read_line(&mut sub_choice).unwrap();
                    match sub_choice.trim() {
                        "1" => db.list_foods(),
                        "2" => add_basic_food_cli(&mut db),
                        "3" => add_composite_food_cli(&mut db),
                        "4" => break,
                        _ => println!("Invalid choice."),
                    }
                }
            },
            "2" => daily_logs_menu(&mut daily_logs),
            "3" => {
                match db.save_to_file(db_file) {
                    Ok(_) => println!("Food database saved successfully."),
                    Err(e) => println!("Error saving food database: {}", e),
                }
            },
            "4" => {
                match daily_logs.save_to_file(logs_file) {
                    Ok(_) => println!("Daily logs saved successfully."),
                    Err(e) => println!("Error saving daily logs: {}", e),
                }
            },
            "5" => {
                // Save both databases before exit.
                let _ = db.save_to_file(db_file);
                let _ = daily_logs.save_to_file(logs_file);
                println!("Databases saved. Exiting.");
                break;
            },
            _ => println!("Invalid choice."),
        }
    }
}

fn add_basic_food_cli(db: &mut FoodDatabase) {
    let mut id = String::new();
    let mut keywords_str = String::new();
    let mut calories_str = String::new();

    println!("Enter basic food identifier:");
    io::stdin().read_line(&mut id).unwrap();
    let id = id.trim().to_string();

    // Check immediately if the food exists in either basic or composite foods.
    if db.food_exists(&id) {
        println!("Food with id '{}' already exists.", id);
        return;
    }

    println!("Enter keywords (comma separated):");
    io::stdin().read_line(&mut keywords_str).unwrap();
    let keywords: Vec<String> = keywords_str
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    println!("Enter calories per serving:");
    io::stdin().read_line(&mut calories_str).unwrap();
    let calories: u32 = calories_str.trim().parse().unwrap_or(0);

    let food = BasicFood {
        id: id.clone(),
        keywords,
        calories,
    };

    db.add_basic_food(food);
    println!("Basic food '{}' added.", id);
}

fn add_composite_food_cli(db: &mut FoodDatabase) {
    let mut id = String::new();
    let mut keywords_str = String::new();
    let mut components = Vec::new();

    println!("Enter composite food identifier:");
    io::stdin().read_line(&mut id).unwrap();
    let id = id.trim().to_string();

    // Check immediately if the food exists in either basic or composite foods.
    if db.food_exists(&id) {
        println!("Food with id '{}' already exists.", id);
        return;
    }

    println!("Enter keywords (comma separated):");
    io::stdin().read_line(&mut keywords_str).unwrap();
    let keywords: Vec<String> = keywords_str
        .trim()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    loop {
        println!("Add a component? (yes/no):");
        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();
        let response = response.trim().to_lowercase();
        if response == "no" {
            break;
        } else if response == "yes" {
            let mut comp_input = String::new();
            let mut servings_str = String::new();

            println!("Enter component basic food identifier:");
            io::stdin().read_line(&mut comp_input).unwrap();
            let comp_input = comp_input.trim().to_string();

            // Check if the basic food exists; if not, prompt the user to add it.
            let comp_id = ensure_basic_food_exists(db, &comp_input);

            println!("Enter number of servings:");
            io::stdin().read_line(&mut servings_str).unwrap();
            let servings: f32 = servings_str.trim().parse().unwrap_or(1.0);

            components.push((comp_id, servings));
        } else {
            println!("Invalid response. Please enter 'yes' or 'no'.");
        }
    }

    let composite = CompositeFood {
        id: id.clone(),
        keywords,
        components,
    };

    // Optionally, compute and display total calories for the composite food.
    if let Some(cal) = db.compute_composite_calories(&composite) {
        println!("Composite food total calories per serving: {}", cal);
    } else {
        println!("Warning: Some components were not found; calorie calculation may be incomplete.");
    }

    db.add_composite_food(composite);
    println!("Composite food '{}' added.", id);
}
