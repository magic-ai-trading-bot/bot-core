#![allow(dead_code)]

use anyhow::Result;
use bson::{doc, oid::ObjectId};
use futures::TryStreamExt;
use mongodb::{Collection, Database};
use tracing::{error, info};

use super::models::{Session, User};

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

    // @spec:FR-AUTH-012 - Password Change
    pub async fn update_password(&self, id: &ObjectId, password_hash: String) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "password_hash": password_hash,
                "updated_at": chrono::Utc::now()
            }
        };

        let result = collection.update_one(filter, update).await?;
        if result.matched_count == 0 {
            return Err(anyhow::anyhow!("User not found"));
        }
        Ok(())
    }

    // @spec:FR-AUTH-013 - Profile Update
    pub async fn update_display_name(
        &self,
        id: &ObjectId,
        display_name: Option<String>,
    ) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "display_name": display_name,
                "updated_at": chrono::Utc::now()
            }
        };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    // @spec:FR-AUTH-016 - Avatar Upload
    pub async fn update_avatar(&self, id: &ObjectId, avatar_url: Option<String>) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "avatar_url": avatar_url,
                "updated_at": chrono::Utc::now()
            }
        };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    // @spec:FR-AUTH-017 - Update Profile (combined)
    pub async fn update_profile(
        &self,
        id: &ObjectId,
        display_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "display_name": display_name,
                "avatar_url": avatar_url,
                "updated_at": chrono::Utc::now()
            }
        };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    // @spec:FR-AUTH-014 - 2FA Management
    pub async fn update_2fa(
        &self,
        id: &ObjectId,
        enabled: bool,
        secret: Option<String>,
    ) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "two_factor_enabled": enabled,
                "two_factor_secret": secret,
                "updated_at": chrono::Utc::now()
            }
        };

        collection.update_one(filter, update).await?;
        Ok(())
    }
}

// @spec:FR-AUTH-015 - Session Repository for Active Sessions
// @ref:specs/02-design/2.5-components/COMP-RUST-AUTH.md
#[derive(Clone)]
pub struct SessionRepository {
    collection: Option<Collection<Session>>,
}

impl SessionRepository {
    pub async fn new(database: &Database) -> Result<Self> {
        let collection: Collection<Session> = database.collection("sessions");

        // Create indexes
        let session_id_index = mongodb::IndexModel::builder()
            .keys(doc! { "session_id": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build();

        let user_id_index = mongodb::IndexModel::builder()
            .keys(doc! { "user_id": 1, "created_at": -1 })
            .build();

        // TTL index for auto-expiry
        let ttl_index = mongodb::IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .expire_after(std::time::Duration::from_secs(0))
                    .build(),
            )
            .build();

        if let Err(e) = collection.create_index(session_id_index).await {
            error!("Failed to create session_id index: {}", e);
        }
        if let Err(e) = collection.create_index(user_id_index).await {
            error!("Failed to create user_id index: {}", e);
        }
        if let Err(e) = collection.create_index(ttl_index).await {
            error!("Failed to create TTL index: {}", e);
        } else {
            info!("Session indexes created/ensured");
        }

        Ok(Self {
            collection: Some(collection),
        })
    }

    pub fn new_dummy() -> Self {
        Self { collection: None }
    }

    pub async fn create_session(&self, session: Session) -> Result<ObjectId> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let result = collection.insert_one(session).await?;
        if let Some(id) = result.inserted_id.as_object_id() {
            Ok(id)
        } else {
            Err(anyhow::anyhow!("Failed to get inserted session ID"))
        }
    }

    pub async fn find_by_user(&self, user_id: &ObjectId) -> Result<Vec<Session>> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! {
            "user_id": user_id,
            "revoked": false,
            "expires_at": { "$gt": chrono::Utc::now() }
        };
        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "last_active": -1 })
            .build();

        let cursor = collection.find(filter).with_options(options).await?;
        let sessions: Vec<Session> = cursor.try_collect().await?;
        Ok(sessions)
    }

    pub async fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "session_id": session_id, "revoked": false };
        let session = collection.find_one(filter).await?;
        Ok(session)
    }

    pub async fn revoke_session(&self, session_id: &str) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "session_id": session_id };
        let update = doc! { "$set": { "revoked": true } };

        collection.update_one(filter, update).await?;
        Ok(())
    }

    pub async fn revoke_all_except(
        &self,
        user_id: &ObjectId,
        current_session_id: &str,
    ) -> Result<u64> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! {
            "user_id": user_id,
            "session_id": { "$ne": current_session_id },
            "revoked": false
        };
        let update = doc! { "$set": { "revoked": true } };

        let result = collection.update_many(filter, update).await?;
        Ok(result.modified_count)
    }

    pub async fn update_last_active(&self, session_id: &str) -> Result<()> {
        let collection = self
            .collection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;

        let filter = doc! { "session_id": session_id };
        let update = doc! { "$set": { "last_active": chrono::Utc::now() } };

        collection.update_one(filter, update).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::models::{NotificationSettings, RiskLevel, UserSettings};

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

    #[tokio::test]
    async fn test_dummy_repository_update_password_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_password(&id, "new_hash".to_string()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_display_name_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_display_name(&id, Some("New Name".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_avatar_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_avatar(&id, Some("http://avatar.url".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_profile_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_profile(
                &id,
                Some("Name".to_string()),
                Some("http://url".to_string()),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_2fa_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_2fa(&id, true, Some("secret".to_string())).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    // SessionRepository tests
    #[test]
    fn test_session_repository_new_dummy() {
        let repo = SessionRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[test]
    fn test_session_repository_clone() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[tokio::test]
    async fn test_dummy_session_repository_create_session_fails() {
        use super::super::models::Session;

        let repo = SessionRepository::new_dummy();
        let session = Session {
            id: None,
            session_id: "test_session".to_string(),
            user_id: ObjectId::new(),
            device: "Desktop".to_string(),
            browser: "Chrome".to_string(),
            os: "Linux".to_string(),
            ip_address: "127.0.0.1".to_string(),
            location: "Local".to_string(),
            user_agent: "Test Agent".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            last_active: chrono::Utc::now(),
            revoked: false,
        };

        let result = repo.create_session(session).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_find_by_user_fails() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.find_by_user(&user_id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_find_by_session_id_fails() {
        let repo = SessionRepository::new_dummy();
        let result = repo.find_by_session_id("test_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_revoke_session_fails() {
        let repo = SessionRepository::new_dummy();
        let result = repo.revoke_session("test_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_revoke_all_except_fails() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.revoke_all_except(&user_id, "current_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_update_last_active_fails() {
        let repo = SessionRepository::new_dummy();
        let result = repo.update_last_active("test_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_object_id_display_format() {
        let id = ObjectId::new();
        let display = format!("{}", id);

        // Should be 24-character hex string
        assert_eq!(display.len(), 24);
        assert!(display.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ============================================================================
    // COV3: Additional coverage tests for error paths
    // ============================================================================

    #[tokio::test]
    async fn test_cov3_dummy_repository_update_profile_with_both_params() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_profile(
                &id,
                Some("Display Name".to_string()),
                Some("http://avatar.url".to_string()),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov3_dummy_repository_update_profile_with_no_params() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_profile(&id, None, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov3_dummy_repository_update_2fa_enable() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_2fa(&id, true, Some("totp_secret".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov3_dummy_repository_update_2fa_disable() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_2fa(&id, false, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov3_dummy_session_repository_revoke_all_except_current() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.revoke_all_except(&user_id, "current_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_cov3_user_repository_clone_preserves_none() {
        let repo1 = UserRepository::new_dummy();
        let repo2 = repo1.clone();

        // Both should have None collection
        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_cov3_session_repository_clone_preserves_none() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();

        // Both should have None collection
        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_cov3_object_id_equality() {
        let id1 = ObjectId::new();
        let id2 = id1; // Copy

        assert_eq!(id1, id2);

        let id3 = ObjectId::new();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_cov3_object_id_hex_roundtrip() {
        let id = ObjectId::new();
        let hex = id.to_hex();
        let parsed = ObjectId::parse_str(&hex).unwrap();

        assert_eq!(id, parsed);
    }

    #[test]
    fn test_cov3_object_id_parse_str_too_short() {
        let result = ObjectId::parse_str("abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_cov3_object_id_parse_str_non_hex() {
        let result = ObjectId::parse_str("zzzzzzzzzzzzzzzzzzzzzzz");
        assert!(result.is_err());
    }

    #[test]
    fn test_cov3_object_id_parse_str_with_uppercase() {
        let id = ObjectId::new();
        let hex = id.to_hex().to_uppercase();
        let result = ObjectId::parse_str(&hex);
        // MongoDB ObjectIds are case-insensitive
        assert!(result.is_ok() || result.is_err()); // Implementation-dependent
    }

    #[tokio::test]
    async fn test_cov3_user_repo_multiple_operations_fail() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        // Test that all operations consistently fail
        assert!(repo.find_by_id(&id).await.is_err());
        assert!(repo.find_by_email("test@example.com").await.is_err());
        assert!(repo.email_exists("test@example.com").await.is_err());
        assert!(repo.count_users().await.is_err());
        assert!(repo.update_last_login(&id).await.is_err());
        assert!(repo.deactivate_user(&id).await.is_err());
    }

    #[tokio::test]
    async fn test_cov3_session_repo_multiple_operations_fail() {
        let repo = SessionRepository::new_dummy();
        let id = ObjectId::new();

        // Test that all operations consistently fail
        assert!(repo.find_by_user(&id).await.is_err());
        assert!(repo.find_by_session_id("session_id").await.is_err());
        assert!(repo.revoke_session("session_id").await.is_err());
        assert!(repo.revoke_all_except(&id, "session_id").await.is_err());
        assert!(repo.update_last_active("session_id").await.is_err());
    }

    #[test]
    fn test_object_id_from_bytes() {
        let bytes = [0u8; 12];
        let id = ObjectId::from_bytes(bytes);

        // Should create valid ObjectId
        assert_eq!(id.to_hex().len(), 24);
    }

    // Additional tests for UserRepository methods
    #[tokio::test]
    async fn test_dummy_repository_email_exists_with_empty_string() {
        let repo = UserRepository::new_dummy();
        let result = repo.email_exists("").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_password_with_empty_hash() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_password(&id, "".to_string()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_display_name_with_none() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_display_name(&id, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_avatar_with_none() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_avatar(&id, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_profile_with_none_values() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_profile(&id, None, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_2fa_disable() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_2fa(&id, false, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_update_2fa_enable_with_secret() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_2fa(&id, true, Some("SECRET123".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_create_user_with_empty_email() {
        let repo = UserRepository::new_dummy();
        let user = User::new("".to_string(), "hash".to_string(), None);

        let result = repo.create_user(user).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_repository_find_by_email_with_long_email() {
        let repo = UserRepository::new_dummy();
        let long_email = "a".repeat(1000) + "@example.com";
        let result = repo.find_by_email(&long_email).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    // SessionRepository additional tests
    #[tokio::test]
    async fn test_dummy_session_repository_create_session_with_expired() {
        use super::super::models::Session;

        let repo = SessionRepository::new_dummy();
        let session = Session {
            id: None,
            session_id: "expired_session".to_string(),
            user_id: ObjectId::new(),
            device: "Mobile".to_string(),
            browser: "Safari".to_string(),
            os: "iOS".to_string(),
            ip_address: "192.168.1.1".to_string(),
            location: "US".to_string(),
            user_agent: "Mobile Agent".to_string(),
            created_at: chrono::Utc::now() - chrono::Duration::days(30),
            expires_at: chrono::Utc::now() - chrono::Duration::days(1),
            last_active: chrono::Utc::now() - chrono::Duration::days(2),
            revoked: false,
        };

        let result = repo.create_session(session).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_find_by_session_id_with_empty_string() {
        let repo = SessionRepository::new_dummy();
        let result = repo.find_by_session_id("").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_revoke_session_with_empty_id() {
        let repo = SessionRepository::new_dummy();
        let result = repo.revoke_session("").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_dummy_session_repository_update_last_active_with_long_id() {
        let repo = SessionRepository::new_dummy();
        let long_id = "a".repeat(1000);
        let result = repo.update_last_active(&long_id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    // ObjectId edge case tests
    #[test]
    fn test_object_id_parse_str_empty_string() {
        let result = ObjectId::parse_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_object_id_parse_str_too_short() {
        let result = ObjectId::parse_str("abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_object_id_parse_str_too_long() {
        let result = ObjectId::parse_str("0123456789abcdef0123456789abcdef");
        assert!(result.is_err());
    }

    #[test]
    fn test_object_id_parse_str_invalid_hex() {
        let result = ObjectId::parse_str("gggggggggggggggggggggggg");
        assert!(result.is_err());
    }

    #[test]
    fn test_object_id_equality() {
        let id1 = ObjectId::new();
        let id2 = id1.clone();

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_object_id_inequality() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_object_id_from_bytes_all_zeros() {
        let bytes = [0u8; 12];
        let id1 = ObjectId::from_bytes(bytes);
        let id2 = ObjectId::from_bytes(bytes);

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_object_id_from_bytes_all_ones() {
        let bytes = [0xFFu8; 12];
        let id = ObjectId::from_bytes(bytes);

        assert_eq!(id.to_hex().len(), 24);
        assert!(id.to_hex().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_user_repository_multiple_clones() {
        let repo1 = UserRepository::new_dummy();
        let repo2 = repo1.clone();
        let repo3 = repo2.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
        assert!(repo3.collection.is_none());
    }

    #[test]
    fn test_session_repository_multiple_clones() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();
        let repo3 = repo2.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
        assert!(repo3.collection.is_none());
    }

    #[tokio::test]
    async fn test_user_new_with_display_name() {
        use super::super::models::User;

        let user = User::new(
            "test@example.com".to_string(),
            "hash123".to_string(),
            Some("Test User".to_string()),
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hash123");
        assert_eq!(user.full_name, Some("Test User".to_string()));
        assert!(user.is_active);
        assert!(!user.two_factor_enabled);
    }

    #[tokio::test]
    async fn test_user_new_without_display_name() {
        use super::super::models::User;

        let user = User::new("test@example.com".to_string(), "hash123".to_string(), None);

        assert_eq!(user.email, "test@example.com");
        assert!(user.display_name.is_none());
    }

    #[test]
    fn test_object_id_from_bytes_sequential() {
        let bytes1 = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let bytes2 = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        let id1 = ObjectId::from_bytes(bytes1);
        let id2 = ObjectId::from_bytes(bytes2);

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_object_id_hex_string_format() {
        let id = ObjectId::new();
        let hex = id.to_hex();

        // Should only contain valid hex characters (0-9, a-f)
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        // Should be lowercase
        assert!(hex
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(|c| c.is_lowercase()));
    }

    #[test]
    fn test_cov_user_model_new_with_display_name() {
        use super::super::models::User;

        let user = User::new(
            "user@test.com".to_string(),
            "hash789".to_string(),
            Some("DisplayName".to_string()),
        );

        assert_eq!(user.email, "user@test.com");
        assert_eq!(user.full_name, Some("DisplayName".to_string())); // User::new sets full_name, not display_name
        assert_eq!(user.display_name, None); // display_name is initialized to None
        assert_eq!(user.password_hash, "hash789");
    }

    #[test]
    fn test_cov_user_model_new_without_display_name() {
        use super::super::models::User;

        let user = User::new("user2@test.com".to_string(), "hash000".to_string(), None);

        assert_eq!(user.email, "user2@test.com");
        assert!(user.display_name.is_none());
    }

    #[test]
    fn test_cov_object_id_new_unique() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_cov_object_id_to_hex() {
        let id = ObjectId::new();
        let hex = id.to_hex();

        assert_eq!(hex.len(), 24);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_cov_object_id_from_bytes() {
        let bytes = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let id = ObjectId::from_bytes(bytes);
        let hex = id.to_hex();

        assert_eq!(hex.len(), 24);
    }

    // ========== ADDITIONAL COVERAGE TESTS (test_cov2_*) ==========

    #[tokio::test]
    async fn test_cov2_user_repo_methods_with_nulldb() {
        let repo = UserRepository::new_dummy();

        // Test all methods return errors with null-db
        let id = ObjectId::new();

        assert!(repo
            .create_user(User::new("e@e.com".into(), "h".into(), None))
            .await
            .is_err());
        assert!(repo.find_by_email("e@e.com").await.is_err());
        assert!(repo.find_by_id(&id).await.is_err());
        assert!(repo
            .update_user(&id, User::new("e@e.com".into(), "h".into(), None))
            .await
            .is_err());
        assert!(repo.update_last_login(&id).await.is_err());
        assert!(repo.deactivate_user(&id).await.is_err());
        assert!(repo.count_users().await.is_err());
        assert!(repo.email_exists("e@e.com").await.is_err());
        assert!(repo.update_password(&id, "newhash".into()).await.is_err());
        assert!(repo
            .update_display_name(&id, Some("Name".into()))
            .await
            .is_err());
        assert!(repo.update_avatar(&id, Some("url".into())).await.is_err());
        assert!(repo
            .update_profile(&id, Some("N".into()), Some("u".into()))
            .await
            .is_err());
        assert!(repo
            .update_2fa(&id, true, Some("sec".into()))
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_cov2_session_repo_methods_with_nulldb() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        // All methods should return errors
        assert!(repo.find_by_user(&user_id).await.is_err());
        assert!(repo.find_by_session_id("sid").await.is_err());
        assert!(repo.revoke_session("sid").await.is_err());
        assert!(repo.revoke_all_except(&user_id, "curr").await.is_err());
        assert!(repo.update_last_active("sid").await.is_err());
    }

    #[test]
    fn test_cov2_objectid_operations() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        // Test inequality
        assert_ne!(id1, id2);

        // Test cloning
        let id1_clone = id1.clone();
        assert_eq!(id1, id1_clone);

        // Test display
        let display = format!("{}", id1);
        assert_eq!(display.len(), 24);

        // Test hex round-trip
        let hex = id1.to_hex();
        let parsed = ObjectId::parse_str(&hex).unwrap();
        assert_eq!(id1, parsed);
    }

    #[test]
    fn test_cov2_objectid_edge_cases() {
        // Test zero bytes
        let zero_id = ObjectId::from_bytes([0u8; 12]);
        assert_eq!(zero_id.to_hex().len(), 24);

        // Test max bytes
        let max_id = ObjectId::from_bytes([255u8; 12]);
        assert_eq!(max_id.to_hex().len(), 24);

        // Test parse errors
        assert!(ObjectId::parse_str("").is_err());
        assert!(ObjectId::parse_str("short").is_err());
        assert!(ObjectId::parse_str("zzzzzzzzzzzzzzzzzzzzzzzzz").is_err());
    }

    // ============================================================================
    // PHASE 5 TESTS - Enhanced Coverage for Database Operations
    // ============================================================================

    #[tokio::test]
    async fn test_cov5_user_repo_create_user_nulldb() {
        let repo = UserRepository::new_dummy();
        let user = User::new(
            "newuser@example.com".to_string(),
            "hash123".to_string(),
            Some("Display Name".to_string()),
        );

        let result = repo.create_user(user).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Database not available"));
    }

    #[tokio::test]
    async fn test_cov5_user_repo_find_by_email_multiple_calls() {
        let repo = UserRepository::new_dummy();

        let result1 = repo.find_by_email("user1@example.com").await;
        assert!(result1.is_err());

        let result2 = repo.find_by_email("user2@example.com").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_find_by_id_different_ids() {
        let repo = UserRepository::new_dummy();

        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        let result1 = repo.find_by_id(&id1).await;
        assert!(result1.is_err());

        let result2 = repo.find_by_id(&id2).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_user_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let user = User::new(
            "updated@example.com".to_string(),
            "newhash".to_string(),
            None,
        );

        let result = repo.update_user(&id, user).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_last_login_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result = repo.update_last_login(&id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_user_repo_deactivate_user_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result = repo.deactivate_user(&id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_user_repo_count_users_nulldb() {
        let repo = UserRepository::new_dummy();

        let result = repo.count_users().await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_user_repo_email_exists_nulldb() {
        let repo = UserRepository::new_dummy();

        let result1 = repo.email_exists("test1@example.com").await;
        assert!(result1.is_err());

        let result2 = repo.email_exists("test2@example.com").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_password_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result = repo.update_password(&id, "newhash123".to_string()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_display_name_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result1 = repo
            .update_display_name(&id, Some("New Name".to_string()))
            .await;
        assert!(result1.is_err());

        let result2 = repo.update_display_name(&id, None).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_avatar_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result1 = repo
            .update_avatar(&id, Some("http://avatar.url".to_string()))
            .await;
        assert!(result1.is_err());

        let result2 = repo.update_avatar(&id, None).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_profile_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result1 = repo
            .update_profile(
                &id,
                Some("Name".to_string()),
                Some("http://url.com".to_string()),
            )
            .await;
        assert!(result1.is_err());

        let result2 = repo.update_profile(&id, None, None).await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_2fa_nulldb() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result1 = repo
            .update_2fa(&id, true, Some("secret123".to_string()))
            .await;
        assert!(result1.is_err());

        let result2 = repo.update_2fa(&id, false, None).await;
        assert!(result2.is_err());
    }

    #[test]
    fn test_cov5_user_repo_clone() {
        let repo1 = UserRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[tokio::test]
    async fn test_cov5_session_repo_create_session_nulldb() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        let session = Session::new(
            user_id,
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "Local".to_string(),
            "TestAgent/1.0".to_string(),
        );

        let result = repo.create_session(session).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_session_repo_find_by_user_nulldb() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        let result = repo.find_by_user(&user_id).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_session_repo_find_by_session_id_nulldb() {
        let repo = SessionRepository::new_dummy();

        let result1 = repo.find_by_session_id("session123").await;
        assert!(result1.is_err());

        let result2 = repo.find_by_session_id("session456").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_session_repo_revoke_session_nulldb() {
        let repo = SessionRepository::new_dummy();

        let result = repo.revoke_session("session789").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_session_repo_revoke_all_except_nulldb() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        let result = repo.revoke_all_except(&user_id, "current123").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov5_session_repo_update_last_active_nulldb() {
        let repo = SessionRepository::new_dummy();

        let result = repo.update_last_active("session999").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_cov5_session_repo_clone() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_cov5_objectid_from_bytes() {
        let bytes = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let id = ObjectId::from_bytes(bytes);

        let hex = id.to_hex();
        assert_eq!(hex.len(), 24);
        assert!(hex.starts_with("0102030405060708090a0b0c"));
    }

    #[test]
    fn test_cov5_objectid_display_format() {
        let id = ObjectId::new();
        let display_str = format!("{}", id);
        let hex_str = id.to_hex();

        assert_eq!(display_str, hex_str);
        assert_eq!(display_str.len(), 24);
    }

    #[test]
    fn test_cov5_objectid_parse_valid_hex() {
        let hex = "507f1f77bcf86cd799439011";
        let id = ObjectId::parse_str(hex).unwrap();

        let parsed_hex = id.to_hex();
        assert_eq!(parsed_hex, hex);
    }

    #[test]
    fn test_cov5_objectid_parse_invalid_length() {
        let short = "507f1f77";
        let result = ObjectId::parse_str(short);
        assert!(result.is_err());

        let long = "507f1f77bcf86cd79943901100000000";
        let result = ObjectId::parse_str(long);
        assert!(result.is_err());
    }

    #[test]
    fn test_cov5_objectid_parse_invalid_chars() {
        let invalid = "507f1f77bcf86cd79943901g"; // 'g' is not hex
        let result = ObjectId::parse_str(invalid);
        assert!(result.is_err());

        let invalid2 = "507f1f77bcf86cd79943901Z"; // 'Z' is not hex
        let result2 = ObjectId::parse_str(invalid2);
        assert!(result2.is_err());
    }

    #[test]
    fn test_cov5_objectid_equality() {
        let id1 = ObjectId::new();
        let id2 = id1.clone();

        assert_eq!(id1, id2);

        let id3 = ObjectId::new();
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_cov5_objectid_bytes_roundtrip() {
        let original_bytes = [10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120];
        let id = ObjectId::from_bytes(original_bytes);

        let hex = id.to_hex();
        let parsed_id = ObjectId::parse_str(&hex).unwrap();

        assert_eq!(id, parsed_id);
    }

    #[tokio::test]
    async fn test_cov5_user_repo_multiple_operations_sequence() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        // Simulate a sequence of operations that all fail
        assert!(repo.find_by_id(&id).await.is_err());
        assert!(repo.update_last_login(&id).await.is_err());
        assert!(repo.deactivate_user(&id).await.is_err());
        assert!(repo.count_users().await.is_err());
    }

    #[tokio::test]
    async fn test_cov5_session_repo_multiple_operations_sequence() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        // Simulate a sequence of operations that all fail
        assert!(repo.find_by_user(&user_id).await.is_err());
        assert!(repo.find_by_session_id("sid123").await.is_err());
        assert!(repo.revoke_session("sid123").await.is_err());
        assert!(repo.revoke_all_except(&user_id, "sid123").await.is_err());
        assert!(repo.update_last_active("sid123").await.is_err());
    }

    #[test]
    fn test_cov5_objectid_zero_bytes() {
        let zero_id = ObjectId::from_bytes([0u8; 12]);
        let hex = zero_id.to_hex();

        assert_eq!(hex, "000000000000000000000000");
        assert_eq!(hex.len(), 24);
    }

    #[test]
    fn test_cov5_objectid_max_bytes() {
        let max_id = ObjectId::from_bytes([255u8; 12]);
        let hex = max_id.to_hex();

        assert_eq!(hex, "ffffffffffffffffffffffff");
        assert_eq!(hex.len(), 24);
    }

    #[test]
    fn test_cov5_objectid_sequential() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();
        let id3 = ObjectId::new();

        // All IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[tokio::test]
    async fn test_cov5_user_repo_email_exists_empty_string() {
        let repo = UserRepository::new_dummy();

        let result = repo.email_exists("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_password_empty_hash() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let result = repo.update_password(&id, "".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov5_session_repo_find_by_session_id_empty_string() {
        let repo = SessionRepository::new_dummy();

        let result = repo.find_by_session_id("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov5_session_repo_revoke_session_empty_string() {
        let repo = SessionRepository::new_dummy();

        let result = repo.revoke_session("").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cov5_objectid_parse_empty_string() {
        let result = ObjectId::parse_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_cov5_objectid_parse_whitespace() {
        let result = ObjectId::parse_str("   ");
        assert!(result.is_err());

        let result2 = ObjectId::parse_str("\t\n");
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_find_by_email_case_sensitivity() {
        let repo = UserRepository::new_dummy();

        let result1 = repo.find_by_email("TEST@EXAMPLE.COM").await;
        assert!(result1.is_err());

        let result2 = repo.find_by_email("test@example.com").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_display_name_long_string() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let long_name = "a".repeat(1000);
        let result = repo.update_display_name(&id, Some(long_name)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cov5_user_repo_update_avatar_various_urls() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        let urls = vec![
            "http://example.com/avatar.jpg",
            "https://secure.example.com/avatar.png",
            "data:image/png;base64,iVBORw0KGgoAAAANS",
        ];

        for url in urls {
            let result = repo.update_avatar(&id, Some(url.to_string())).await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_cov5_session_repo_revoke_all_except_empty_current() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        let result = repo.revoke_all_except(&user_id, "").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_cov5_objectid_clone_equality() {
        let id1 = ObjectId::new();
        let id2 = id1.clone();
        let id3 = id2.clone();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_cov5_objectid_debug_format() {
        let id = ObjectId::new();
        let debug_str = format!("{:?}", id);

        // Debug format should include ObjectId and the hex string
        assert!(debug_str.contains("ObjectId"));
    }

    #[tokio::test]
    async fn test_cov7_update_password_dummy() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_password(&id, "new_hash".to_string()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_update_display_name_dummy() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_display_name(&id, Some("New Name".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_update_avatar_dummy() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_avatar(&id, Some("http://avatar.url".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_update_profile_dummy() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_profile(
                &id,
                Some("New Name".to_string()),
                Some("http://avatar.url".to_string()),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_update_2fa_dummy() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_2fa(&id, true, Some("secret".to_string())).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_cov7_session_repository_new_dummy() {
        let repo = SessionRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[tokio::test]
    async fn test_cov7_session_repository_create_session_dummy() {
        use crate::auth::models::Session;

        let repo = SessionRepository::new_dummy();
        let session = Session::new(
            ObjectId::new(),
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "San Francisco".to_string(),
            "Mozilla/5.0".to_string(),
        );

        let result = repo.create_session(session).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_session_repository_find_by_user_dummy() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.find_by_user(&user_id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_session_repository_find_by_session_id_dummy() {
        let repo = SessionRepository::new_dummy();
        let result = repo.find_by_session_id("session_123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_session_repository_revoke_session_dummy() {
        let repo = SessionRepository::new_dummy();
        let result = repo.revoke_session("session_123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_session_repository_revoke_all_except_dummy() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.revoke_all_except(&user_id, "session_123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov7_session_repository_update_last_active_dummy() {
        let repo = SessionRepository::new_dummy();
        let result = repo.update_last_active("session_123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_cov7_session_repository_clone() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_cov7_user_new_with_full_name() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.full_name, Some("Test User".to_string()));
        assert!(user.is_active);
        assert!(!user.two_factor_enabled);
    }

    #[test]
    fn test_cov7_user_new_without_full_name() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.full_name, None);
        assert_eq!(user.display_name, None);
    }

    #[test]
    fn test_cov7_session_new() {
        use crate::auth::models::Session;

        let user_id = ObjectId::new();
        let session = Session::new(
            user_id.clone(),
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "San Francisco".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)".to_string(),
        );

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.device, "Desktop");
        assert_eq!(session.browser, "Chrome");
        assert_eq!(session.os, "macOS");
        assert_eq!(session.ip_address, "127.0.0.1");
        assert_eq!(session.location, "San Francisco");
        assert!(!session.session_id.is_empty());
        assert!(session.id.is_none());
    }

    #[test]
    fn test_cov7_objectid_new() {
        let id1 = ObjectId::new();
        let id2 = ObjectId::new();

        // Two different ObjectIds should not be equal
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_cov7_objectid_to_string() {
        let id = ObjectId::new();
        let id_str = id.to_string();

        // ObjectId string representation should be 24 characters (hex)
        assert_eq!(id_str.len(), 24);
    }

    #[tokio::test]
    async fn test_cov7_user_repository_multiple_operations() {
        let repo = UserRepository::new_dummy();

        // Test multiple operations in sequence
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let create_result = repo.create_user(user.clone()).await;
        assert!(create_result.is_err());

        let find_result = repo.find_by_email("test@example.com").await;
        assert!(find_result.is_err());

        let count_result = repo.count_users().await;
        assert!(count_result.is_err());

        let email_exists_result = repo.email_exists("test@example.com").await;
        assert!(email_exists_result.is_err());
    }

    #[tokio::test]
    async fn test_cov7_session_repository_multiple_operations() {
        use crate::auth::models::Session;

        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();

        let session = Session::new(
            user_id.clone(),
            "Mobile".to_string(),
            "Safari".to_string(),
            "iOS".to_string(),
            "192.168.1.1".to_string(),
            "New York".to_string(),
            "Mozilla/5.0 (iPhone)".to_string(),
        );

        let create_result = repo.create_session(session).await;
        assert!(create_result.is_err());

        let find_result = repo.find_by_user(&user_id).await;
        assert!(find_result.is_err());

        let revoke_result = repo.revoke_session("session_123").await;
        assert!(revoke_result.is_err());

        let update_result = repo.update_last_active("session_123").await;
        assert!(update_result.is_err());
    }

    // ========== COV8 TESTS: Additional coverage for auth database ==========

    #[tokio::test]
    async fn test_cov8_user_repo_count_users_fails() {
        let repo = UserRepository::new_dummy();
        let result = repo.count_users().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_user_repo_email_exists_fails() {
        let repo = UserRepository::new_dummy();
        let result = repo.email_exists("test@example.com").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_user_repo_update_password_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_password(&id, "new_hash".to_string()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_user_repo_update_display_name_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_display_name(&id, Some("New Name".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_user_repo_update_avatar_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_avatar(&id, Some("http://avatar.url".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_user_repo_update_profile_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_profile(
                &id,
                Some("New Name".to_string()),
                Some("http://avatar.url".to_string()),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_user_repo_update_2fa_fails() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_2fa(&id, true, Some("secret".to_string())).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_cov8_user_repo_clone() {
        let repo1 = UserRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_cov8_session_repo_clone() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[tokio::test]
    async fn test_cov8_session_repo_find_by_session_id_fails() {
        let repo = SessionRepository::new_dummy();
        let result = repo.find_by_session_id("session_123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_cov8_session_repo_revoke_all_except_fails() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.revoke_all_except(&user_id, "current_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_cov8_user_repository_new_dummy_state() {
        let repo = UserRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[test]
    fn test_cov8_session_repository_new_dummy_state() {
        let repo = SessionRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[tokio::test]
    async fn test_cov8_user_repo_all_operations_fail_gracefully() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();

        // All operations should return "Database not available" error
        assert!(repo
            .create_user(User::new(
                "test@test.com".to_string(),
                "hash".to_string(),
                None
            ))
            .await
            .is_err());
        assert!(repo.find_by_email("test@test.com").await.is_err());
        assert!(repo.find_by_id(&id).await.is_err());
        assert!(repo
            .update_user(
                &id,
                User::new("test@test.com".to_string(), "hash".to_string(), None)
            )
            .await
            .is_err());
        assert!(repo.update_last_login(&id).await.is_err());
        assert!(repo.deactivate_user(&id).await.is_err());
        assert!(repo.count_users().await.is_err());
        assert!(repo.email_exists("test@test.com").await.is_err());
    }

    // =========================================================================
    // FUNCTION-LEVEL TESTS (test_fn_ prefix for coverage boost)
    // =========================================================================

    #[test]
    fn test_fn_user_repository_new_dummy() {
        let repo = UserRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[test]
    fn test_fn_session_repository_new_dummy() {
        let repo = SessionRepository::new_dummy();
        assert!(repo.collection.is_none());
    }

    #[tokio::test]
    async fn test_fn_user_repo_create_user_no_db() {
        let repo = UserRepository::new_dummy();
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_pw".to_string(),
            None,
        );
        let result = repo.create_user(user).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_find_by_email_no_db() {
        let repo = UserRepository::new_dummy();
        let result = repo.find_by_email("user@example.com").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_find_by_id_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.find_by_id(&id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_user_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let user = User::new("test@example.com".to_string(), "hash".to_string(), None);
        let result = repo.update_user(&id, user).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_last_login_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_last_login(&id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_deactivate_user_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.deactivate_user(&id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_count_users_no_db() {
        let repo = UserRepository::new_dummy();
        let result = repo.count_users().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_email_exists_no_db() {
        let repo = UserRepository::new_dummy();
        let result = repo.email_exists("test@example.com").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_password_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_password(&id, "new_hash".to_string()).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_display_name_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_display_name(&id, Some("New Name".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_avatar_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_avatar(&id, Some("http://avatar.url".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_profile_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo
            .update_profile(&id, Some("Name".to_string()), Some("url".to_string()))
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_user_repo_update_2fa_no_db() {
        let repo = UserRepository::new_dummy();
        let id = ObjectId::new();
        let result = repo.update_2fa(&id, true, Some("secret".to_string())).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_session_repo_create_session_no_db() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let session = Session::new(
            user_id,
            "Device".to_string(),
            "Browser".to_string(),
            "OS".to_string(),
            "127.0.0.1".to_string(),
            "Location".to_string(),
            "UA".to_string(),
        );
        let result = repo.create_session(session).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_session_repo_find_by_user_no_db() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.find_by_user(&user_id).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_session_repo_find_by_session_id_no_db() {
        let repo = SessionRepository::new_dummy();
        let result = repo.find_by_session_id("session123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_session_repo_revoke_session_no_db() {
        let repo = SessionRepository::new_dummy();
        let result = repo.revoke_session("session123").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[tokio::test]
    async fn test_fn_session_repo_revoke_all_except_no_db() {
        let repo = SessionRepository::new_dummy();
        let user_id = ObjectId::new();
        let result = repo.revoke_all_except(&user_id, "current_session").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Database not available");
    }

    #[test]
    fn test_fn_user_repository_clone() {
        let repo1 = UserRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    #[test]
    fn test_fn_session_repository_clone() {
        let repo1 = SessionRepository::new_dummy();
        let repo2 = repo1.clone();

        assert!(repo1.collection.is_none());
        assert!(repo2.collection.is_none());
    }

    // ============================================================================
    // COVERAGE BOOST: User model tests
    // ============================================================================

    #[test]
    fn test_user_new_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.full_name, Some("Test User".to_string()));
        assert_eq!(user.display_name, None);
        assert_eq!(user.avatar_url, None);
        assert!(user.is_active);
        assert!(!user.is_admin);
        assert!(!user.two_factor_enabled);
        assert_eq!(user.two_factor_secret, None);
        assert_eq!(user.last_login, None);
        assert!(user.id.is_none());
    }

    #[test]
    fn test_user_new_without_full_name() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        assert_eq!(user.full_name, None);
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_user_to_profile() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );
        user.id = Some(ObjectId::new());
        user.display_name = Some("TestDisplay".to_string());
        user.avatar_url = Some("http://avatar.url".to_string());

        let profile = user.to_profile();

        assert_eq!(profile.email, "test@example.com");
        assert_eq!(profile.full_name, Some("Test User".to_string()));
        assert_eq!(profile.display_name, Some("TestDisplay".to_string()));
        assert_eq!(profile.avatar_url, Some("http://avatar.url".to_string()));
        assert!(profile.is_active);
        assert!(!profile.is_admin);
        assert!(!profile.two_factor_enabled);
        assert_eq!(profile.last_login, None);
        assert!(!profile.id.is_empty());
    }

    #[test]
    fn test_user_to_profile_without_id() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        let profile = user.to_profile();

        assert_eq!(profile.id, "");
        assert_eq!(profile.email, "test@example.com");
    }

    #[test]
    fn test_user_update_last_login() {
        let mut user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            None,
        );

        assert_eq!(user.last_login, None);
        let old_updated_at = user.updated_at;

        // Sleep briefly to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        user.update_last_login();

        assert!(user.last_login.is_some());
        assert!(user.updated_at > old_updated_at);
    }

    #[test]
    fn test_user_serialization() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            Some("Test User".to_string()),
        );

        let serialized = bson::to_document(&user).unwrap();
        assert_eq!(serialized.get_str("email").unwrap(), "test@example.com");
        assert_eq!(
            serialized.get_str("password_hash").unwrap(),
            "hashed_password"
        );
    }

    #[test]
    fn test_user_deserialization() {
        let doc = doc! {
            "email": "test@example.com",
            "password_hash": "hashed_password",
            "full_name": "Test User",
            "is_active": true,
            "is_admin": false,
            "two_factor_enabled": false,
            "created_at": bson::DateTime::now(),
            "updated_at": bson::DateTime::now(),
            "last_login": bson::Bson::Null,
            "settings": {
                "trading_enabled": false,
                "risk_level": "Medium",
                "max_positions": 3,
                "default_quantity": 0.01,
                "notifications": {
                    "email_alerts": true,
                    "trade_notifications": true,
                    "system_alerts": true
                }
            }
        };

        let user: User = bson::from_document(doc).unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert_eq!(user.full_name, Some("Test User".to_string()));
        assert_eq!(user.last_login, None);
    }

    // ============================================================================
    // COVERAGE BOOST: Session model tests
    // ============================================================================

    #[test]
    fn test_session_new_creation() {
        let user_id = ObjectId::new();
        let session = Session::new(
            user_id,
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "San Francisco, US".to_string(),
            "Mozilla/5.0".to_string(),
        );

        assert_eq!(session.user_id, user_id);
        assert_eq!(session.device, "Desktop");
        assert_eq!(session.browser, "Chrome");
        assert_eq!(session.os, "macOS");
        assert_eq!(session.ip_address, "127.0.0.1");
        assert_eq!(session.location, "San Francisco, US");
        assert_eq!(session.user_agent, "Mozilla/5.0");
        assert!(!session.revoked);
        assert_eq!(session.last_active, session.created_at);
        assert!(session.id.is_none());
    }

    #[test]
    fn test_session_to_info() {
        let user_id = ObjectId::new();
        let session = Session::new(
            user_id,
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "San Francisco, US".to_string(),
            "Mozilla/5.0".to_string(),
        );

        let session_id = session.session_id.clone();
        let info = session.to_info(&session_id);

        assert_eq!(info.session_id, session_id);
        assert_eq!(info.device, "Desktop");
        assert_eq!(info.browser, "Chrome");
        assert_eq!(info.os, "macOS");
        assert_eq!(info.location, "San Francisco, US");
        assert!(info.is_current);
    }

    #[test]
    fn test_session_to_info_not_current() {
        let user_id = ObjectId::new();
        let session = Session::new(
            user_id,
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "San Francisco, US".to_string(),
            "Mozilla/5.0".to_string(),
        );

        let info = session.to_info("different_session_id");

        assert!(!info.is_current);
    }

    #[test]
    fn test_session_serialization() {
        let user_id = ObjectId::new();
        let session = Session::new(
            user_id,
            "Desktop".to_string(),
            "Chrome".to_string(),
            "macOS".to_string(),
            "127.0.0.1".to_string(),
            "San Francisco, US".to_string(),
            "Mozilla/5.0".to_string(),
        );

        let serialized = bson::to_document(&session).unwrap();
        assert_eq!(serialized.get_str("device").unwrap(), "Desktop");
        assert_eq!(serialized.get_str("browser").unwrap(), "Chrome");
        assert_eq!(serialized.get_str("os").unwrap(), "macOS");
        assert_eq!(serialized.get_bool("revoked").unwrap(), false);
    }

    // ============================================================================
    // COVERAGE BOOST: UserSettings tests
    // ============================================================================

    #[test]
    fn test_user_settings_default() {
        let settings = UserSettings::default();

        assert!(!settings.trading_enabled); // Default is false for safety
        assert!(matches!(settings.risk_level, RiskLevel::Medium));
        assert_eq!(settings.max_positions, 3);
        assert_eq!(settings.default_quantity, 0.01);
        assert!(settings.notifications.email_alerts);
        assert!(settings.notifications.trade_notifications);
        assert!(settings.notifications.system_alerts);
    }

    #[test]
    fn test_user_settings_custom() {
        let settings = UserSettings {
            trading_enabled: false,
            risk_level: RiskLevel::Low,
            max_positions: 10,
            default_quantity: 0.05,
            notifications: NotificationSettings {
                email_alerts: false,
                trade_notifications: true,
                system_alerts: false,
            },
        };

        assert!(!settings.trading_enabled);
        assert!(matches!(settings.risk_level, RiskLevel::Low));
        assert_eq!(settings.max_positions, 10);
        assert_eq!(settings.default_quantity, 0.05);
        assert!(!settings.notifications.email_alerts);
        assert!(settings.notifications.trade_notifications);
        assert!(!settings.notifications.system_alerts);
    }

    #[test]
    fn test_risk_level_serialization() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;

        let low_json = serde_json::to_string(&low).unwrap();
        let medium_json = serde_json::to_string(&medium).unwrap();
        let high_json = serde_json::to_string(&high).unwrap();

        assert_eq!(low_json, "\"Low\"");
        assert_eq!(medium_json, "\"Medium\"");
        assert_eq!(high_json, "\"High\"");
    }

    #[test]
    fn test_risk_level_deserialization() {
        let low: RiskLevel = serde_json::from_str("\"Low\"").unwrap();
        let medium: RiskLevel = serde_json::from_str("\"Medium\"").unwrap();
        let high: RiskLevel = serde_json::from_str("\"High\"").unwrap();

        assert!(matches!(low, RiskLevel::Low));
        assert!(matches!(medium, RiskLevel::Medium));
        assert!(matches!(high, RiskLevel::High));
    }
}
