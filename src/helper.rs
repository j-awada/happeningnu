use sea_orm::EntityTrait;
use tower_sessions::Session;
use crate::entities::users::Entity as User;

pub async fn get_username_from_session(session: &Session, db: &sea_orm::DatabaseConnection) -> Option<String> {
    if let Some(user_id) = session.get::<i32>("user_id").await.unwrap_or(None) {
        if let Some(user) = User::find_by_id(user_id)
            .one(db)
            .await
            .unwrap()
        {
            Some(user.username.clone())
        } else {
            None
        }
    } else {
        None
    }
}