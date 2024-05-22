use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, PartialEq)]
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
    Ok(())
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("User", 4)?;

        state.serialize_field("name", &self.name)?;
        state.serialize_field("age", &self.age)?;
        state.serialize_field("dob", &self.dob)?;
        state.serialize_field("skills", &self.skills)?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_struct("User", &["name", "age", "dob", "skills"], UserVisitor)
    }
}

struct UserVisitor;

impl<'de> serde::de::Visitor<'de> for UserVisitor {
    type Value = User;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct User")
    }

    fn visit_seq<V>(self, mut seq: V) -> std::prelude::v1::Result<Self::Value, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let name = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let age = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let dob = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
        let skills = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?;

        Ok(User {
            name,
            age,
            dob,
            skills,
        })
    }

    fn visit_map<V>(self, mut map: V) -> std::prelude::v1::Result<Self::Value, V::Error>
    where
        V: serde::de::MapAccess<'de>,
    {
        let mut name = None;
        let mut age = None;
        let mut dob = None;
        let mut skills = None;

        while let Some(key) = map.next_key()? {
            match key {
                "name" => {
                    if name.is_some() {
                        return Err(serde::de::Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?);
                }
                "age" => {
                    if age.is_some() {
                        return Err(serde::de::Error::duplicate_field("age"));
                    }
                    age = Some(map.next_value()?);
                }
                "dob" => {
                    if dob.is_some() {
                        return Err(serde::de::Error::duplicate_field("dob"));
                    }
                    dob = Some(map.next_value()?);
                }
                "skills" => {
                    if skills.is_some() {
                        return Err(serde::de::Error::duplicate_field("skills"));
                    }
                    skills = Some(map.next_value()?);
                }
                _ => {
                    let _: serde::de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;
        let age = age.ok_or_else(|| serde::de::Error::missing_field("age"))?;
        let dob = dob.ok_or_else(|| serde::de::Error::missing_field("dob"))?;
        let skills = skills.ok_or_else(|| serde::de::Error::missing_field("skills"))?;

        Ok(User {
            name,
            age,
            dob,
            skills,
        })
    }
}
