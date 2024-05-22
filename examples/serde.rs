use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, PartialEq, Serialize, Deserialize)]
struct User {
    name: String,
    age: u8,
    dob: DateTime<Utc>,
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("Alice".to_string())
        .age(20)
        .dob(Utc::now())
        .skills(vec!["C++".to_string(), "Rust".to_string()])
        .build()?;

    // Serialize to JSON
    let user_json = serde_json::to_string(&user)?;
    println!("User: {user_json}");

    // Deserialize from JSON
    let user1: User = serde_json::from_str(&user_json)?;
    println!("user1: {:?}", user1);

    assert_eq!(user, user1);

    // Serialize to TOML
    let user_toml = toml::to_string(&user)?;
    println!("User: {user_toml}");

    // Deserialize from TOML
    let user2: User = toml::from_str(&user_toml)?;
    println!("user2: {:?}", user2);

    assert_eq!(user, user2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_json() {
        let user = UserBuilder::default()
            .name("Alice".to_string())
            .age(20)
            .dob(Utc::now())
            .skills(vec!["C++".to_string(), "Rust".to_string()])
            .build()
            .unwrap();
        let user_json = serde_json::to_string(&user).unwrap();
        let user1: User = serde_json::from_str(&user_json).unwrap();
        assert_eq!(user, user1);
    }

    #[test]
    fn test_serde_toml() {
        let user = UserBuilder::default()
            .name("Alice".to_string())
            .age(20)
            .dob(Utc::now())
            .skills(vec!["C++".to_string(), "Rust".to_string()])
            .build()
            .unwrap();
        let user_toml = toml::to_string(&user).unwrap();
        let user2: User = toml::from_str(&user_toml).unwrap();
        assert_eq!(user, user2);
    }
}
