use serde::{Serialize, Deserialize};
use std::io::{self, Write};
use std::fs;

// Add this to import modules
mod daily_logs;
mod gui;
mod login;
mod calorie_calculator;  // NEW: calorie calculator module

use daily_logs::DailyLogs;
use gui::launch_gui;
use login::login_page;

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

#[derive(Serialize, Deserialize, Debug, Clone)]  // Added Clone trait
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

    /// Calculate calories for a food with the given servings.
    pub fn calculate_food_calories(&self, food_id: &str, servings: f32) -> u32 {
        if let Some(basic) = self.basic_foods.iter().find(|b| b.id == food_id) {
            return (basic.calories as f32 * servings) as u32;
        }
        if let Some(composite) = self.composite_foods.iter().find(|c| c.id == food_id) {
            if let Some(cal) = self.compute_composite_calories(composite) {
                return (cal as f32 * servings) as u32;
            }
        }
        0
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
    id.to_string()  // Add explicit return statement
}

// ...existing code...

fn main() {
    // First, ask user to login or signup.
    let user = login_page();

    // Load food database.
    let db_file = "food_database.json";
    let mut db = FoodDatabase::load_from_file(db_file);

    // Load daily logs using the daily_logs module.
    let logs_file = "daily_logs.json";
    let mut dlogs = DailyLogs::load_from_file(logs_file);

    loop {
        println!("\nMain Menu:");
        println!("1. Food Database Menu");
        println!("2. User Menu");  // Changed from "Daily Logs Menu" to "User Menu"
        println!("3. Save Food Database");
        println!("4. Save Daily Logs");
        println!("5. Launch GUI");
        println!("6. Exit");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
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
            "2" => user_menu(&mut dlogs, &db, &user), // New function instead of daily_logs_menu
            "3" => {
                match db.save_to_file(db_file) {
                    Ok(_) => println!("Food database saved successfully."),
                    Err(e) => println!("Error saving food database: {}", e),
                }
            },
            "4" => {
                match dlogs.save_to_file(logs_file) {
                    Ok(_) => println!("Daily logs saved successfully."),
                    Err(e) => println!("Error saving daily logs: {}", e),
                }
            },
            "5" => {
                // Launch GUI with the current databases and user.
                launch_gui(db.clone(), dlogs.clone(), user.clone());
            },
            "6" => {
                let _ = db.save_to_file(db_file);
                let _ = dlogs.save_to_file(logs_file);
                println!("Databases saved. Exiting.");
                break;
            },
            _ => println!("Invalid choice."),
        }
    }
}

// New function for the User Menu
fn user_menu(dlogs: &mut DailyLogs, db: &FoodDatabase, user: &login::User) {
    loop {
        println!("\nUser Menu for {}:", user.username);
        println!("1. Daily Logs Menu");
        println!("2. List All Log Entries");
        println!("3. Change Calorie Calculation Method");
        println!("4. Back to Main Menu");
        print!("Enter choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => daily_logs::daily_logs_menu(dlogs, db, user),
            "2" => list_user_log_dates(dlogs, user),
            "3" => change_calorie_method(dlogs, user),
            "4" => break,
            _ => println!("Invalid choice."),
        }
    }
}

// Function to list all log dates for the current user
fn list_user_log_dates(dlogs: &DailyLogs, user: &login::User) {
    let mut user_dates: Vec<String> = dlogs.logs.keys()
        .filter(|k| k.starts_with(&format!("{}:", user.username)))
        .map(|k| k.split(':').nth(1).unwrap_or("").to_string())
        .collect();
    
    if user_dates.is_empty() {
        println!("No log entries found for user: {}", user.username);
        return;
    }

    user_dates.sort();
    user_dates.reverse(); // Most recent first
    
    println!("\nLog dates for user {}:", user.username);
    for (i, date) in user_dates.iter().enumerate() {
        let key = format!("{}:{}", user.username, date);
        if let Some(log) = dlogs.logs.get(&key) {
            let entry_count = log.entries.len();
            let has_metrics = log.daily_info.is_some();
            
            print!("{}. {}: {} entries", i+1, date, entry_count);
            if has_metrics {
                print!(" (has metrics)");
            }
            println!();
        }
    }
}

// Function to change calorie calculation method for the user
fn change_calorie_method(dlogs: &mut DailyLogs, user: &login::User) {
    use calorie_calculator::{CalorieMethod, calculate_calorie_target};
    
    println!("Available Calculation Methods:");
    for (i, (_method, description)) in CalorieMethod::all_methods().iter().enumerate() {
        println!("{}. {}", i+1, description);
    }
    
    print!("Select method (1-{}): ", CalorieMethod::all_methods().len());
    io::stdout().flush().unwrap();
    let mut method_choice = String::new();
    io::stdin().read_line(&mut method_choice).unwrap();
    
    if let Ok(idx) = method_choice.trim().parse::<usize>() {
        if idx > 0 && idx <= CalorieMethod::all_methods().len() {
            let selected_method = CalorieMethod::all_methods()[idx-1].0.clone();
            
            println!("Apply this method to:");
            println!("1. All future log entries");
            println!("2. A specific date");
            println!("3. Cancel");
            
            let mut scope_choice = String::new();
            print!("Enter choice: ");
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut scope_choice).unwrap();
            
            match scope_choice.trim() {
                "1" => {
                    // Saved for all future entries
                    println!("Calorie method set to {} for all future entries.", selected_method.name());
                    // In a real app, you'd save this preference to the user profile
                },
                "2" => {
                    // Apply to a specific date
                    print!("Enter date (YYYY-MM-DD): ");
                    io::stdout().flush().unwrap();
                    let mut date = String::new();
                    io::stdin().read_line(&mut date).unwrap();
                    let date = date.trim();
                    
                    if daily_logs::validate_date(date) {
                        let log = dlogs.get_log_mut_for_user(date, &user.username);
                        
                        // Create or update daily info with the selected method
                        if let Some(info) = &mut log.daily_info {
                            info.calorie_method = selected_method.clone();
                        } else {
                            log.daily_info = Some(daily_logs::DailyUserInfo {
                                age: user.age,
                                weight: user.weight,
                                activity_level: user.activity_level.clone(),
                                calorie_method: selected_method.clone(),
                            });
                        }
                        
                        // Calculate and display the new target
                        let target = calculate_calorie_target(
                            &selected_method,
                            &user.gender,
                            log.daily_info.as_ref().unwrap().weight,
                            user.height,
                            log.daily_info.as_ref().unwrap().age,
                            &log.daily_info.as_ref().unwrap().activity_level
                        );
                        
                        println!("Calorie method for {} changed to: {}", date, selected_method.name());
                        println!("New calorie target: {} calories", target);
                    } else {
                        println!("Invalid date format. Please use YYYY-MM-DD.");
                    }
                },
                _ => println!("Operation canceled."),
            }
        } else {
            println!("Invalid selection.");
        }
    } else {
        println!("Invalid input.");
    }
}

// ...existing code...

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

            println!("Enter component basic/composite food identifier:");
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
