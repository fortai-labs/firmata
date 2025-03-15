use anyhow::Result;
use redis::{Client, Connection, Commands};
use std::time::Duration;

/// A client for interacting with Redis for general operations
pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    /// Create a new Redis client
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        
        // Test the connection
        let mut conn = client.get_connection()?;
        redis::cmd("PING").query::<String>(&mut conn)?;
        
        Ok(Self { client })
    }
    
    /// Get a Redis connection
    pub fn get_connection(&self) -> Result<Connection> {
        Ok(self.client.get_connection()?)
    }
    
    /// Set a key with a value and optional expiration
    pub fn set_key(&self, key: &str, value: &str, expiry_seconds: Option<u64>) -> Result<()> {
        let mut conn = self.get_connection()?;
        
        match expiry_seconds {
            Some(seconds) => {
                conn.set_ex(key, value, seconds)?;
            },
            None => {
                conn.set(key, value)?;
            }
        }
        
        Ok(())
    }
    
    /// Get a value by key
    pub fn get_key(&self, key: &str) -> Result<Option<String>> {
        let mut conn = self.get_connection()?;
        let value: Option<String> = conn.get(key)?;
        Ok(value)
    }
    
    /// Delete a key
    pub fn delete_key(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let deleted: i32 = conn.del(key)?;
        Ok(deleted > 0)
    }
    
    /// Check if a key exists
    pub fn key_exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let exists: bool = conn.exists(key)?;
        Ok(exists)
    }
    
    /// Set a key with expiration
    pub fn set_key_with_expiry(&self, key: &str, value: &str, expiry: Duration) -> Result<()> {
        let mut conn = self.get_connection()?;
        conn.set_ex(key, value, expiry.as_secs())?;
        Ok(())
    }
    
    /// Increment a counter
    pub fn increment(&self, key: &str) -> Result<i64> {
        let mut conn = self.get_connection()?;
        let value: i64 = conn.incr(key, 1)?;
        Ok(value)
    }
    
    /// Add a value to a set
    pub fn add_to_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let added: i32 = conn.sadd(set_name, value)?;
        Ok(added > 0)
    }
    
    /// Check if a value is in a set
    pub fn is_member_of_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let is_member: bool = conn.sismember(set_name, value)?;
        Ok(is_member)
    }
    
    /// Get all members of a set
    pub fn get_set_members(&self, set_name: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection()?;
        let members: Vec<String> = conn.smembers(set_name)?;
        Ok(members)
    }
    
    /// Remove a value from a set
    pub fn remove_from_set(&self, set_name: &str, value: &str) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let removed: i32 = conn.srem(set_name, value)?;
        Ok(removed > 0)
    }
    
    /// Add a value to a sorted set with score
    pub fn add_to_sorted_set(&self, set_name: &str, value: &str, score: f64) -> Result<bool> {
        let mut conn = self.get_connection()?;
        let added: i32 = conn.zadd(set_name, value, score)?;
        Ok(added > 0)
    }
    
    /// Get values from a sorted set by score range
    pub fn get_sorted_set_by_score(&self, set_name: &str, min: f64, max: f64) -> Result<Vec<String>> {
        let mut conn = self.get_connection()?;
        let members: Vec<String> = conn.zrangebyscore(set_name, min, max)?;
        Ok(members)
    }
    
    /// Publish a message to a channel
    pub fn publish(&self, channel: &str, message: &str) -> Result<i32> {
        let mut conn = self.get_connection()?;
        let receivers: i32 = conn.publish(channel, message)?;
        Ok(receivers)
    }
} 