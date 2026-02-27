// External crates
use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use axum::{
    routing::{get, post},
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Router,
    Form,
};
use axum_messages::Messages;
use sea_orm::{
    ActiveModelTrait,
    EntityTrait,
    ColumnTrait,
    QueryFilter,
    Set,
};
use tera::Context;
use tower_sessions::Session;
use validator::Validate;

// Internal modules
use crate::AppState;
use crate::handler::models::{ SignupData, LoginData };
use crate::entities::users;
use crate::entities::users::Entity as User;

pub fn user_router() -> Router<AppState> {
    Router::new()
    .route("/login", get(login))
    .route("/login", post(process_login_form))
    .route("/logout", get(logout))
    .route("/signup", get(signup))
    .route("/signup", post(process_signup_form))
}

pub async fn login(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
) -> impl IntoResponse {
    let tera = &app_state.tera;
    let mut context = Context::new();
    if session.get::<i32>("user_id").await.unwrap_or(None).is_some() {
        return Redirect::to("/").into_response();
    }
    let mut info_to_user: Vec<String> = vec![];
    for msg in messages.into_iter() {
        info_to_user.push(msg.message);
    }
    context.insert("title", "Log in");
    context.insert("messages", &info_to_user);
    Html(tera.render("partials/login.html", &context).unwrap()).into_response()
}

async fn process_login_form(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
    Form(data): Form<LoginData>,
) -> Redirect {
    let mut context = Context::new();
    let user = User::find()
        .filter(users::Column::Email.eq(data.email.clone()))
        .one(&app_state.db_connection)
        .await
        .unwrap();

    if let Some(user) = user {
        let parsed_hash = PasswordHash::new(&user.password).unwrap();
        let password_bytes = data.password.as_bytes();
        let is_valid = Argon2::default()
            .verify_password(password_bytes, &parsed_hash)
            .is_ok();

        if is_valid {
            session.insert("user_id", user.id).await.unwrap();
            let is_logged_in = true;
            context.insert("is_logged_in", &is_logged_in);
            messages.info("Login successful!");
            Redirect::to("/")
        } else {
            messages.error("Invalid password.");
            Redirect::to("/login")
        }
    } else {
        messages.error("Not found.");
        Redirect::to("/login")
    }
}

pub async fn logout(
    session: Session,
    messages: Messages,
) -> Redirect {
    session.remove::<i32>("user_id").await.unwrap();
    messages.info("You have logged out.");
    Redirect::to("/")
}

pub async fn signup(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
) -> impl IntoResponse {
    let tera = &app_state.tera;
    let mut context = Context::new();
    if session.get::<i32>("user_id").await.unwrap_or(None).is_some() {
        return Redirect::to("/").into_response();
    }
    context.insert("title", "Sign up");
    let mut info_to_user: Vec<String> = vec![];
    for msg in messages.into_iter() {
        info_to_user.push(msg.message);
    }
    context.insert("messages", &info_to_user);
    Html(tera.render("partials/signup.html", &context).unwrap()).into_response()
}

async fn process_signup_form(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
    Form(data): Form<SignupData>
) -> impl IntoResponse {
    let mut context = Context::new();
    if let Err(errors) = data.validate() {
        context.insert("title", "Sign up");
        messages.error(format!("{:?}", errors));
        return Redirect::to("/signup").into_response();
    }
    // Check if email already exists
    let existing_user = User::find()
        .filter(users::Column::Email.eq(data.email.clone()))
        .one(&app_state.db_connection)
        .await
        .unwrap();

    if existing_user.is_some() {
        context.insert("title", "Sign up");
        messages.error("Email is already registered.");
        return Redirect::to("/signup").into_response();
    }

    dotenvy::dotenv().ok();
    let password_salt = std::env::var("PASSWORD_SALT").expect("PASSWORD_SALT must be set");
    let user_password = data.password.clone();
    let user_password_bytes: &[u8] = user_password.as_bytes();
    let salt = SaltString::from_b64(&password_salt).expect("Invalid base64 salt");
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(user_password_bytes, &salt).unwrap().to_string();
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    assert!(Argon2::default().verify_password(user_password_bytes, &parsed_hash).is_ok());
    let new_user = users::ActiveModel {
        email: Set(data.email.clone()),
        username: Set(data.username.clone()),
        password: Set(password_hash),
        ..Default::default()
    };
    let res = new_user.insert(&app_state.db_connection).await.unwrap();
    session.insert("user_id", res.id).await.unwrap();
    let is_logged_in = true;
    context.insert("is_logged_in", &is_logged_in);
    messages.info("Hi!");
    Redirect::to("/").into_response()
}