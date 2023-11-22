use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Food {
    Ration,
    Slime,
    // TODO
    Custom,
}

impl fmt::Display for Food {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Food::Ration => write!(f, "food"),
            Food::Slime => write!(f, "slime ration"),
            Food::Custom => write!(f, "strange food"),
        }
    }
}
