use serde::{ Serialize, Deserialize };
use validator::{Validate, ValidationError};
use chrono::NaiveDate;

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct SignupData {
    #[validate(email(message="Email not valid."))]
    pub email: String,
    #[validate(length(
        min=4,
        max=20,
        message="username should be between 4 to 20 characters."
    ))]
    pub username: String,
    #[validate(length(
        min=4,
        max=15,
        message="password should be between 8 to 15 characters."
    ))]
    pub password: String,
    #[validate(must_match(other=password, message="Passwords not identical."))]
    pub confirm_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginData {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct NewEventData {
    #[validate(length(
        min=4,
        max=30,
        message="event title should be between 4 to 30 characters."
    ))]
    pub title: String,
    #[validate(url(message = "URL not valid."))]
    pub url: String,
    #[validate(custom(function = "validate_event_location"))]
    pub location: String,
    #[validate(custom(function = "validate_event_date"))]
    pub date: String,
    #[validate(custom(function = "validate_event_category"))]
    pub category: String,
}

fn validate_event_location(location: &str) -> Result<(), ValidationError> {
    if EVENT_LOCATIONS.contains(&location) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_location"))
    }
}

fn validate_event_date(date: &str) -> Result<(), ValidationError> {
    match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("invalid_date")),
    }
}

fn validate_event_category(category: &str) -> Result<(), ValidationError> {
    if EVENT_CATEGORIES.contains(&category) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_category"))
    }
}

pub const EVENT_LOCATIONS: [&str; 20] = [
        "Stockholm",
        "Göteborg",
        "Malmö",
        "Uppsala",
        "Västerås",
        "Örebro",
        "Linköping",
        "Helsingborg",
        "Jönköping",
        "Norrköping",
        "Lund",
        "Umeå",
        "Gävle",
        "Borås",
        "Eskilstuna",
        "Södertälje",
        "Karlstad",
        "Täby",
        "Växjö",
        "Halmstad",
    ];

pub const EVENT_CATEGORIES: [&str; 6] = [
        "Languages",
        "Sports",
        "Social",
        "Arts and theatre",
        "Xmas",
        "Other",
    ];