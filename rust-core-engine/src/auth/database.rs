#![allow(dead_code)]

use anyhow::Result;
use bson::{doc, oid::ObjectId};
use mongodb::{Collection, Database};
use tracing::{error, info};

use super::models::User;

#[derive(Clone)]
pub struct UserRepository {
    collection: Option<Collection<User>>,
}

impl UserRepository {
    pub async fn new(database: &Database) -> Result<Self> {
        let collection: Collection<User> = database.collection("users");

        // Create unique index on email
        let index_options = mongodb::options::IndexOptions::builder()
            .unique(true)
            .build();

        let index_model = mongodb::IndexModel::builder()
            .keys(doc! { "email": 1 })
            .options(index_options)
            .build();

        if let Err(e) = collection.create_index(index_model).await {
            error!("Failed to create email index: {}", e);
        } else {
            info!("Email unique index created/ensured");
        }

        Ok(Self {
            collection: Some(collection),
        })
    }

    pub fn new_dummy() -> Self {
        // Create a dummy repository that will fail for all operations
        // This is used when no database is available
        Self { collection: None }
    }

    pub async fn create_user(&self, user: User) -> Result<ObjectId> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let result = collection.insert_one(user).await?;

        if let Some(id) = result.inserted_id.as_object_id() {
            Ok(id)
        } else {
            Err(anyhow::anyhow!("Failed to get inserted user ID"))
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "email": email };
        let user = collection.find_one(filter).await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<User>> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let user = collection.find_one(filter).await?;
        Ok(user)
    }

    pub async fn update_user(&self, id: &ObjectId, user: User) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": bson::to_document(&user)?
        };

        let result = collection.update_one(filter, update).await?;

        if result.matched_count == 0 {
            return Err(anyhow::anyhow!("User not found"));
        }

        Ok(())
    }

    pub async fn update_last_login(&self, id: &ObjectId) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "last_login": chrono::Utc::now(),
                "updated_at": chrono::Utc::now()
            }
        };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn deactivate_user(&self, id: &ObjectId) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "is_active": false,
                "updated_at": chrono::Utc::now()
            }
        };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn count_users(&self) -> Result<u64> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let count = collection.count_documents(doc! {}).await?;
        Ok(count)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "email": email };
        let count = collection.count_documents(filter).await?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_repository_new_dummy() {
        let repo = UserRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[tokio::test]
    async fn test_dummy_repository_create_user_fails() {
        let repo = UserRepository::new_dummy();
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        let result = repo.create_user(user).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_find_by_email_fails() {
        let repo = UserRepository::new_dummy();
        let result = repo.find_by_email("test@example.com").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_find_by_id_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.find_by_id(&id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_user_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        let result = repo.update_user(&id, user).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_last_login_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_last_login(&id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_deactivate_user_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.deactivate_user(&id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_count_users_fails() {
        let repo = UserRepository::new_dummy();
        let result = repo.count_users().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_email_exists_fails() {
        let repo = UserRepository::new_dummy();
        let result = repo.email_exists("test@example.com").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_user_repository_clone() {
        let repo1 = UserRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_object_id_generation() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        // ObjectIds should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_object_id_to_hex() {
        let id = ObjectId::new();
        let hex_string = id.to_hex();

        // MongoDB ObjectId hex representation is 24 characters
        assert_eq!(hex_string.len(), 24);
    }

    #[test]
    fn test_object_id_parse_str_valid() {
        let id = ObjectId::new();
        let hex_string = id.to_hex();

        let parsed_id = ObjectId::parse_str(&hex_string).unwrap();
        assert_eq!(id, parsed_id);
    }

    #[test]
    fn test_object_id_parse_str_invalid() {
        let result = ObjectId::parse_str("invalid_id");
        assert!(result.is_err());
    }
}
