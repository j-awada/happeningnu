// External crates
use axum::{
    routing::get,
    routing::post,
    extract::{State, Path},
    Router,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use axum_messages::{Message, Messages};
use serde_json;
use tera::Context;
use tower_sessions::Session;
use validator::Validate;

use sea_orm::{
    ActiveModelTrait,
    EntityTrait,
    QueryOrder,
    ColumnTrait,
    QueryFilter,
    PaginatorTrait,
    Set,
};

// Internal modules
use crate::AppState;
use crate::handler::models::{ EVENT_LOCATIONS, EVENT_CATEGORIES, NewEventData };
use crate::entities::users::Entity as User;
use crate::entities::events;
use crate::entities::events::Entity as Event;
use crate::entities::user_events;
use crate::entities::user_events::Entity as UserEvent;
use crate::helper::get_username_from_session;

pub fn event_router() -> Router<AppState> {
    Router::new()
    .route("/", get(all_events))
    .route("/user_events", get(user_events))
    .route("/new_event", get(new_event_form))
    .route("/new_event", post(process_new_event_form))
    .route("/event/{id}/delete", post(delete_event))
    .route("/api/event/{id}/going", post(mark_event_going))
}

pub async fn all_events(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
) -> Html<String> {
    let tera = &app_state.tera;
    let mut context = Context::new();

    let mut info_to_user: Vec<String> = vec![];
    for msg in messages.into_iter() {
        info_to_user.push(msg.message);
    }

    let is_logged_in = session.get::<i32>("user_id").await.unwrap_or(None).is_some();
    context.insert("is_logged_in", &is_logged_in);
    let logged_in_username = get_username_from_session(&session, &app_state.db_connection).await;
    context.insert("logged_in_username", &logged_in_username);

    let events = Event::find()
        .order_by_asc(events::Column::Date)
        .all(&app_state.db_connection)
        .await
        .unwrap();
    let mut events_with_count = Vec::new();
    for event in &events {
        // Query the username of the event creator
        let username = if let Some(user) = User::find_by_id(event.user_id)
            .one(&app_state.db_connection)
            .await
            .unwrap()
        {
            user.username.clone()
        } else {
            String::from("unknown")
        };
        let count = UserEvent::find()
            .filter(user_events::Column::EventId.eq(event.id))
            .count(&app_state.db_connection)
            .await
            .unwrap();
        events_with_count.push(serde_json::json!({
            "id": event.id,
            "title": event.title,
            "url": event.url,
            "location": event.location,
            "date": event.date,
            "category": event.category,
            "attendee_count": count,
            "username": username
        }));
    }
    context.insert("all_events", &events_with_count);
    context.insert("messages", &info_to_user);
    context.insert("title", "Happening nu");
    Html(tera.render("partials/home.html", &context).unwrap())
}

pub async fn user_events(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
) -> Html<String> {
    let tera = &app_state.tera;
    let mut context = Context::new();

    let mut info_to_user: Vec<String> = vec![];
    for msg in messages.into_iter() {
        info_to_user.push(msg.message);
    }

    let is_logged_in = session.get::<i32>("user_id").await.unwrap_or(None).is_some();
    context.insert("is_logged_in", &is_logged_in);
    let logged_in_username = get_username_from_session(&session, &app_state.db_connection).await;
    context.insert("logged_in_username", &logged_in_username);
    context.insert("not_home", &true);

    let user_id = session.get::<i32>("user_id").await.unwrap_or(None);

    let mut events_with_count = Vec::new();

    if let Some(uid) = user_id {
        // Find all events created by this user
        let user_events = Event::find()
            .filter(events::Column::UserId.eq(uid))
            .order_by_asc(events::Column::Date)
            .all(&app_state.db_connection)
            .await
            .unwrap();

        for event in &user_events {
            let count = UserEvent::find()
                .filter(user_events::Column::EventId.eq(event.id))
                .count(&app_state.db_connection)
                .await
                .unwrap();
            events_with_count.push(serde_json::json!({
                "id": event.id,
                "title": event.title,
                "url": event.url,
                "location": event.location,
                "date": event.date,
                "category": event.category,
                "attendee_count": count
            }));
        }
    }
    context.insert("user_events", &events_with_count);
    context.insert("messages", &info_to_user);
    context.insert("title", "Happening nu");
    Html(tera.render("partials/user_events.html", &context).unwrap())
}

pub async fn new_event_form(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
) -> impl IntoResponse {
    if session.get::<i32>("user_id").await.unwrap_or(None).is_none() {
        // Not logged in, redirect to login page
        return Redirect::to("/login").into_response();
    }

    let tera = &app_state.tera;
    let mut context = Context::new();
    let mut info_to_user: Vec<Message> = vec![];
    for msg in messages.into_iter() {
        info_to_user.push(msg);
    }
    let is_logged_in = session.get::<i32>("user_id").await.unwrap_or(None).is_some();
    context.insert("is_logged_in", &is_logged_in);
    let logged_in_username = get_username_from_session(&session, &app_state.db_connection).await;
    context.insert("logged_in_username", &logged_in_username);
    context.insert("messages", &info_to_user);
    context.insert("title", "New event");
    context.insert("event_categories", &EVENT_CATEGORIES);
    context.insert("event_locations", &EVENT_LOCATIONS);
    Html(tera.render("partials/new_event.html", &context).unwrap()).into_response()
}

pub async fn process_new_event_form(
    State(app_state): State<AppState>,
    messages: Messages,
    session: Session,
    Form(data): Form<NewEventData>,
) -> impl IntoResponse {
    let mut context = Context::new();
    if let Err(errors) = data.validate() {
        println!("{:?}", errors);
        context.insert("title", "New event");
        messages.error(format!("{:?}", errors));
        return Redirect::to("/new_event").into_response();
    } else {
        let user_id = session.get::<i32>("user_id").await.unwrap_or(None);
        if let Some(uid) = user_id {
            let new_event = events::ActiveModel {
                title: Set(data.title.clone()),
                url: Set(data.url.clone()),
                location: Set(data.location.clone()),
                date: Set(data.date.clone()),
                category: Set(data.category.clone()),
                user_id: Set(uid), // <-- Add this line
                ..Default::default()
            };
            let _ = new_event.insert(&app_state.db_connection).await.unwrap();
        } else {
            // Not logged in, redirect
            return Redirect::to("/login").into_response();
        }
    };
    Redirect::to("/").into_response()
}

pub async fn delete_event(
    State(app_state): State<AppState>,
    Path(event_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    let user_id = session.get::<i32>("user_id").await.unwrap_or(None);

    if let Some(uid) = user_id {
        // Find the event
        if let Some(event) = Event::find_by_id(event_id)
            .one(&app_state.db_connection)
            .await
            .unwrap()
        {
            // Check ownership (replace `user_id` with your actual owner column)
            if event.user_id == uid {
                // 1. Delete all user_events for this event
                //let _ = UserEvent::delete_many()
                //    .filter(user_events::Column::EventId.eq(event_id))
                //    .exec(&app_state.db_connection)
                //    .await;
                // 2. Now delete the event
                let active_model: events::ActiveModel = event.into();
                let _ = active_model.delete(&app_state.db_connection).await;
                // Return an empty string or a message to remove the row in htmx
                Redirect::to("/user_events").into_response()
            } else {
                Redirect::to("/user_events").into_response()
            }
        } else {
            Redirect::to("/user_events").into_response()
        }
    } else {
        Redirect::to("/login").into_response()
    }
} 

pub async fn mark_event_going(
    State(app_state): State<AppState>,
    Path(event_id): Path<i32>,
    session: Session,
) -> impl IntoResponse {
    let user_id = session.get::<i32>("user_id").await.unwrap_or(None);
    if let Some(uid) = user_id {
        let user_already_going = UserEvent::find()
            .filter(user_events::Column::UserId.eq(uid))
            .filter(user_events::Column::EventId.eq(event_id))
            .one(&app_state.db_connection)
            .await
            .unwrap();

        if let Some(rec) = user_already_going {
            let active_model: user_events::ActiveModel = rec.into();
            let _ = active_model.delete(&app_state.db_connection).await;
        } else {
            let new_user_event = user_events::ActiveModel {
                user_id: Set(uid),
                event_id: Set(event_id),
                ..Default::default()
            };
            let _ = new_user_event.insert(&app_state.db_connection).await;
        }
    }
    let count = UserEvent::find()
        .filter(user_events::Column::EventId.eq(event_id))
        .count(&app_state.db_connection)
        .await
        .unwrap();
    Html(format!(
        r#"<span style="color: #828282; font-size: 0.75em;" id="attendee-count-{}">Going: {}</span>"#,
        event_id, count
    )).into_response()
}