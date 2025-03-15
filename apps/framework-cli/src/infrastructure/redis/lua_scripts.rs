//! Redis Lua Scripts
//!
//! This module contains Lua scripts for optimized Redis operations.
//! Using Lua scripts allows us to execute complex operations atomically
//! and reduce the number of round-trips to Redis.

use lazy_static::lazy_static;
use redis::Script;

lazy_static! {
    /// Script for atomic check and renew lock operation
    ///
    /// This script checks if a lock exists and is owned by the current instance,
    /// and then renews it if it does. It returns a tuple of (now_has_lock, is_new_lock).
    ///
    /// # Arguments
    ///
    /// * `KEYS[1]` - Lock key
    /// * `ARGV[1]` - Instance ID
    /// * `ARGV[2]` - TTL in milliseconds
    ///
    /// # Returns
    ///
    /// A tuple of (now_has_lock, is_new_lock)
    pub static ref CHECK_AND_RENEW_LOCK: Script = Script::new(r#"
        local lock_key = KEYS[1]
        local instance_id = ARGV[1]
        local ttl = tonumber(ARGV[2])
        
        local current_owner = redis.call('GET', lock_key)
        local had_lock = (current_owner == instance_id)
        
        -- Try to acquire the lock if we don't have it
        local acquired = false
        if not had_lock then
            acquired = redis.call('SET', lock_key, instance_id, 'PX', ttl, 'NX') and true or false
        end
        
        -- Renew the lock if we had it or just acquired it
        if had_lock or acquired then
            redis.call('PEXPIRE', lock_key, ttl)
            return {1, acquired and 1 or 0}  -- {now_has_lock, is_new_lock}
        end
        
        return {0, 0}  -- Don't have lock, didn't acquire it
    "#);

    /// Script for atomic attempt lock operation
    ///
    /// This script attempts to acquire a lock if it doesn't exist or has expired.
    ///
    /// # Arguments
    ///
    /// * `KEYS[1]` - Lock key
    /// * `ARGV[1]` - Instance ID
    /// * `ARGV[2]` - TTL in milliseconds
    ///
    /// # Returns
    ///
    /// 1 if the lock was acquired, 0 otherwise
    pub static ref ATTEMPT_LOCK: Script = Script::new(r#"
        local lock_key = KEYS[1]
        local instance_id = ARGV[1]
        local ttl = tonumber(ARGV[2])
        
        local result = redis.call('SET', lock_key, instance_id, 'PX', ttl, 'NX')
        return result and 1 or 0
    "#);

    /// Script for atomic renew lock operation
    ///
    /// This script renews a lock if it exists and is owned by the current instance.
    ///
    /// # Arguments
    ///
    /// * `KEYS[1]` - Lock key
    /// * `ARGV[1]` - Instance ID
    /// * `ARGV[2]` - TTL in milliseconds
    ///
    /// # Returns
    ///
    /// 1 if the lock was renewed, 0 otherwise
    pub static ref RENEW_LOCK: Script = Script::new(r#"
        local lock_key = KEYS[1]
        local instance_id = ARGV[1]
        local ttl = tonumber(ARGV[2])
        
        local current_owner = redis.call('GET', lock_key)
        if current_owner == instance_id then
            redis.call('PEXPIRE', lock_key, ttl)
            return 1
        end
        
        return 0
    "#);

    /// Script for atomic release lock operation
    ///
    /// This script releases a lock if it exists and is owned by the current instance.
    ///
    /// # Arguments
    ///
    /// * `KEYS[1]` - Lock key
    /// * `ARGV[1]` - Instance ID
    ///
    /// # Returns
    ///
    /// 1 if the lock was released, 0 otherwise
    pub static ref RELEASE_LOCK: Script = Script::new(r#"
        local lock_key = KEYS[1]
        local instance_id = ARGV[1]
        
        local current_owner = redis.call('GET', lock_key)
        if current_owner == instance_id then
            redis.call('DEL', lock_key)
            return 1
        end
        
        return 0
    "#);
}
