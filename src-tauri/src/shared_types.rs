use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Point(pub f64, pub f64);

// 完整的Plait元素结构，支持动态属性
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaitElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,
    
    // 可选的基础属性
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(rename = "textHeight", skip_serializing_if = "Option::is_none")]
    pub text_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<Point>>,
    #[serde(rename = "strokeWidth", skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
    
    // 使用 flatten 来支持任意额外属性
    #[serde(flatten)]
    pub extra_props: HashMap<String, serde_json::Value>,
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
pub struct RemoveNodeOperation {
    #[serde(rename = "type")]
    pub op_type: String,
    pub node: PlaitElement,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Operation {
    Insert(InsertNodeOperation),
    Set(SetNodeOperation),
    Remove(RemoveNodeOperation),
}

// 用于DDS传输的扩展结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardChangeData {
    pub operations: Vec<Operation>,
    pub timestamp: String,
    pub source_id: String, // 用于避免回环
}