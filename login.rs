// diet_profile.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActivityLevel {
    Sedentary,
    Light,
    Moderate,
    Active,
    VeryActive,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CalculationMethod {
    MifflinStJeor,
    HarrisBenedict,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DietProfile {
    pub gender: Gender,
    pub height_cm: f32,
    pub age: u32,
    pub weight_kg: f32,
    pub activity_level: ActivityLevel,
    pub calculation_method: CalculationMethod,
}

impl Default for DietProfile {
    fn default() -> Self {
        DietProfile {
            gender: Gender::Male,
            height_cm: 175.0,
            age: 30,
            weight_kg: 70.0,
            activity_level: ActivityLevel::Moderate,
            calculation_method: CalculationMethod::MifflinStJeor,
        }
    }
}

impl DietProfile {
    /// Compute the target calorie intake using the selected method.
    pub fn compute_target_calories(&self) -> f32 {
        let activity_factor = match self.activity_level {
            ActivityLevel::Sedentary => 1.2,
            ActivityLevel::Light    => 1.375,
            ActivityLevel::Moderate => 1.55,
            ActivityLevel::Active   => 1.725,
            ActivityLevel::VeryActive => 1.9,
        };

        match self.calculation_method {
            CalculationMethod::MifflinStJeor => {
                // Mifflin-St Jeor Equation:
                let bmr = match self.gender {
                    Gender::Male => (10.0 * self.weight_kg) + (6.25 * self.height_cm) - (5.0 * self.age as f32) + 5.0,
                    Gender::Female => (10.0 * self.weight_kg) + (6.25 * self.height_cm) - (5.0 * self.age as f32) - 161.0,
                    Gender::Other => (10.0 * self.weight_kg) + (6.25 * self.height_cm) - (5.0 * self.age as f32),
                };
                bmr * activity_factor
            },
            CalculationMethod::HarrisBenedict => {
                // Harris-Benedict Equation:
                let bmr = match self.gender {
                    Gender::Male => 66.47 + (13.75 * self.weight_kg) + (5.003 * self.height_cm) - (6.755 * self.age as f32),
                    Gender::Female => 655.1 + (9.563 * self.weight_kg) + (1.850 * self.height_cm) - (4.676 * self.age as f32),
                    Gender::Other => {
                        // Use an average of both formulas for "Other"
                        0.5 * ((66.47 + (13.75 * self.weight_kg) + (5.003 * self.height_cm) - (6.755 * self.age as f32)) +
                               (655.1 + (9.563 * self.weight_kg) + (1.850 * self.height_cm) - (4.676 * self.age as f32)))
                    },
                };
                bmr * activity_factor
            },
        }
    }
}
