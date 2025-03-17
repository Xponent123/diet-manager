use std::io::{self, Write};
use std::fs;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct User {
    pub username: String,
    pub gender: String,
    pub height: f32,
    pub age: u32,
    pub weight: f32,
    pub activity_level: String,
}

pub fn login_page() -> User {
    println!("Welcome to NutriTrack!");
    println!("1. Login");
    println!("2. Signup");
    print!("Enter choice: ");
    io::stdout().flush().unwrap();
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    match choice.trim() {
        "1" => login(),
        "2" => signup(),
        _ => {
            println!("Invalid choice, try again.");
            login_page()
        }
    }
}

fn login() -> User {
    let users_file = "users.json";
    let users: Vec<User> = if let Ok(data) = fs::read_to_string(users_file) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };
    print!("Enter username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim().to_string();
    if let Some(user) = users.into_iter().find(|u| u.username == username) {
        println!("Logged in as {}.", username);
        user
    } else {
        println!("Username does not exist. Please signup.");
        signup()
    }
}

fn signup() -> User {
    let users_file = "users.json";
    // Load existing users.
    let mut users: Vec<User> = if let Ok(data) = fs::read_to_string(users_file) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Loop until a unique username is provided or login is chosen.
    let username = loop {
        print!("Enter username: ");
        io::stdout().flush().unwrap();
        let mut username_input = String::new();
        io::stdin().read_line(&mut username_input).unwrap();
        let username_input = username_input.trim().to_string();

        // Check if the username already exists (case-insensitive).
        if let Some(existing_user) = users.iter().find(|u| u.username.eq_ignore_ascii_case(&username_input)) {
            println!("Username '{}' already exists.", username_input);
            print!("Do you want to log in as '{}' (y/n)? ", username_input);
            io::stdout().flush().unwrap();
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            let choice = choice.trim().to_lowercase();
            if choice == "y" || choice == "yes" {
                println!("Logged in as {}.", username_input);
                return existing_user.clone();
            } else {
                println!("Please choose a different username.");
                continue;
            }
        } else {
            // Username is unique. Proceed with signup.
            break username_input;
        }
    };

    // Continue with signup for a new user.
    print!("Enter gender: ");
    io::stdout().flush().unwrap();
    let mut gender = String::new();
    io::stdin().read_line(&mut gender).unwrap();

    print!("Enter height (cm): ");
    io::stdout().flush().unwrap();
    let mut height_str = String::new();
    io::stdin().read_line(&mut height_str).unwrap();
    let height = height_str.trim().parse().unwrap_or(170.0);

    print!("Enter age: ");
    io::stdout().flush().unwrap();
    let mut age_str = String::new();
    io::stdin().read_line(&mut age_str).unwrap();
    let age = age_str.trim().parse().unwrap_or(30);

    print!("Enter weight (kg): ");
    io::stdout().flush().unwrap();
    let mut weight_str = String::new();
    io::stdin().read_line(&mut weight_str).unwrap();
    let weight = weight_str.trim().parse().unwrap_or(70.0);

    print!("Enter activity level (low/moderate/high): ");
    io::stdout().flush().unwrap();
    let mut activity = String::new();
    io::stdin().read_line(&mut activity).unwrap();

    let user = User {
        username: username.clone(),
        gender: gender.trim().to_string(),
        height,
        age,
        weight,
        activity_level: activity.trim().to_string(),
    };

    // Save the new user to "users.json"
    users.push(user.clone());
    if let Ok(json) = serde_json::to_string_pretty(&users) {
        if let Err(e) = fs::write(users_file, json) {
            println!("Warning: Failed to save users file: {}", e);
        }
    }
    println!("Signup successful. You are now logged in as {}.", username);
    user
}
