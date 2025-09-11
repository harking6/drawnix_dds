use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Point(pub f64, pub f64);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlaitElement {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String, // ⚠️ 序列化时会变成 "type"
    pub shape: String,        // "rectangle" | "ellipse" | ...
    pub points: Vec<Point>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<PlaitElement>>,
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
    pub node: PlaitElement // 新属性
}



#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]  // 👈 关键：避免多一层 "type":"Insert"
pub enum Operation {
    Insert(InsertNodeOperation),
    Remove(RemoveNodeOperation),
    Set(SetNodeOperation),
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoardChangeData {
    pub operations: Vec<Operation>,
    pub timestamp: String,
    pub source_id: String,
}
