use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point(pub f64, pub f64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaitElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String, // ⚠️ 序列化时会变成 "type"
    pub shape: Option<String>,        // "rectangle" | "ellipse" | ...
    pub points: Option<Vec<Point>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<PlaitElement>>,
    // 添加更多可能的字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Viewport {
    pub zoom: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Selection {
    // 简化的选择结构，根据实际需要调整
    pub anchor: Option<serde_json::Value>,
    pub focus: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaitTheme {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_mode: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertNodeOperation {
    #[serde(rename = "type")]   // 👈 关键
    pub op_type: String,        // "insert_node"
    pub path: Vec<usize>,
    pub node: PlaitElement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveNodeOperation {
    #[serde(rename = "type")]
    pub op_type: String,        // "remove_node"
    pub path: Vec<usize>,
    pub node: PlaitElement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetNodeOperation {
    #[serde(rename = "type")]
    pub op_type: String,   // "set_node"
    pub path: Vec<usize>,  // 要更新的节点路径
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<PlaitElement>, // 新节点（全量更新）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<std::collections::HashMap<String, serde_json::Value>>, // 增量更新
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_properties: Option<std::collections::HashMap<String, serde_json::Value>>, // 新属性
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]  // 👈 关键：避免多一层 "type":"Insert"
pub enum Operation {
    Insert(InsertNodeOperation),
    Remove(RemoveNodeOperation),
    Set(SetNodeOperation),
    // 支持其他操作类型的通用结构
    Generic(serde_json::Value),
}

// 匹配前端的BoardChangeData结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoardChangeData {
    pub children: Vec<PlaitElement>,
    pub operations: Vec<Operation>,
    pub viewport: Viewport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<Selection>,
    pub theme: PlaitTheme,
    // DDS传输需要的额外字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

// 为DDS传输优化的轻量级结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DDSBoardChangeData {
    pub operations: Vec<Operation>,
    pub timestamp: String,
    pub source_id: String,
    // 可选的完整状态信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<PlaitElement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
}
