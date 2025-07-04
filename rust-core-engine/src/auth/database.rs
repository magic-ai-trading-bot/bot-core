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

        if let Err(e) = collection.create_index(index_model, None).await {
            error!("Failed to create email index: {}", e);
        } else {
            info!("Email unique index created/ensured");
        }

        Ok(Self { collection: Some(collection) })
    }

    pub fn new_dummy() -> Self {
        // Create a dummy repository that will fail for all operations
        // This is used when no database is available
        Self { collection: None }
    }

    pub async fn create_user(&self, user: User) -> Result<ObjectId> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let result = collection.insert_one(user, None).await?;
        
        if let Some(id) = result.inserted_id.as_object_id() {
            Ok(id)
        } else {
            Err(anyhow::anyhow!("Failed to get inserted user ID"))
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let filter = doc! { "email": email };
        let user = collection.find_one(filter, None).await?;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<User>> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let filter = doc! { "_id": id };
        let user = collection.find_one(filter, None).await?;
        Ok(user)
    }

    pub async fn update_user(&self, id: &ObjectId, user: User) -> Result<()> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": bson::to_document(&user)?
        };
        
        let result = collection.update_one(filter, update, None).await?;
        
        if result.matched_count == 0 {
            return Err(anyhow::anyhow!("User not found"));
        }
        
        Ok(())
    }

    pub async fn update_last_login(&self, id: &ObjectId) -> Result<()> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "last_login": chrono::Utc::now(),
                "updated_at": chrono::Utc::now()
            }
        };
        
        collection.update_one(filter, update, None).await?;
        Ok(())
    }

    pub async fn deactivate_user(&self, id: &ObjectId) -> Result<()> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "is_active": false,
                "updated_at": chrono::Utc::now()
            }
        };
        
        collection.update_one(filter, update, None).await?;
        Ok(())
    }

    pub async fn count_users(&self) -> Result<u64> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let count = collection.count_documents(None, None).await?;
        Ok(count)
    }

    pub async fn email_exists(&self, email: &str) -> Result<bool> {
        let collection = self.collection.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not available"))?;
            
        let filter = doc! { "email": email };
        let count = collection.count_documents(filter, None).await?;
        Ok(count > 0)
    }
} 