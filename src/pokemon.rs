use serde::{Deserialize, Serialize};
use bincode::{Decode, Encode};

#[derive(Serialize, Deserialize, Decode, Encode, Debug, Clone, PartialEq)]
pub struct Pokemon {
    // Match the exact order of columns in the CSV file header
    pub generation: u8,
    pub name: String,
    pub form: Option<String>,
    pub type1: String,
    pub type2: Option<String>,
    pub total: u16,
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    #[serde(rename = "Sp. Atk")]  // This matches the CSV header
    pub sp_atk: u8,
    #[serde(rename = "Sp. Def")]  // This matches the CSV header
    pub sp_def: u8,
    pub speed: u8, 
    pub height: f32,
    pub weight: f32,
}


