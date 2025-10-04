use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub nim: String,
}

// You will add other structs like Course, Topic, etc., here later.
