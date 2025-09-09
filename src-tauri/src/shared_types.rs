use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Point(pub f64, pub f64);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaitElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    pub shape: String,
    pub points: Vec<Point>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertNodeOperation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub node: PlaitElement,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetNodeOperation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub node: PlaitElement,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Operation {
    Insert(InsertNodeOperation),
    Set(SetNodeOperation),
}

// 用于DDS传输的扩展结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardChangeData {
    pub operations: Vec<Operation>,
    pub timestamp: String,
    pub source_id: String, // 用于避免回环
}