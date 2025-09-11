use zrdds_safe::prelude::*;
use crate::shared_types::{BoardChangeData, DDSBoardChangeData};
use std::sync::Arc;
use std::collections::HashMap;
use chrono;

pub struct DDSManager {
    domain: Domain,
    board_publisher: Publisher,
    board_subscriber: Subscriber,
    // 冲突检测相关
    last_operation_timestamp: Option<String>,
    pending_operations: HashMap<String, DDSBoardChangeData>, // source_id -> 操作
}

impl DDSManager {
    pub fn new() -> zrdds_safe::Result<Self> {
        println!("🔌 初始化 DDS 连接...");
        
        let domain = Domain::builder()
            .domain_id(150)
            .build()?;
            
        let board_publisher = domain.create_publisher("DrawnixBoardChanges")?;
        let board_subscriber = domain.create_subscriber("DrawnixBoardChanges")?;
        
        println!("✅ DDS 连接已建立");
        
        Ok(DDSManager {
            domain,
            board_publisher,
            board_subscriber,
            last_operation_timestamp: None,
            pending_operations: HashMap::new(),
        })
    }
    
    // 发布白板变化（轻量级DDS传输）
    pub fn publish_board_change(&self, data: &BoardChangeData, source_id: &str) -> zrdds_safe::Result<()> {
        // 转换为轻量级DDS传输格式
        let dds_data = DDSBoardChangeData {
            operations: data.operations.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            source_id: source_id.to_string(),
            children: None, // 通常不传输完整状态，只传输操作
            viewport: None,
        };
        
        let json_data = serde_json::to_string(&dds_data)
            .map_err(|e| zrdds_safe::Error::Other { message: format!("JSON序列化失败: {}", e) })?;
            
        self.board_publisher.publish(json_data.as_bytes())?;
        
        println!("📤 已发布DDS消息: {} 个操作", dds_data.operations.len());
        Ok(())
    }
    
    // 接收白板变化并进行冲突检测
    pub fn try_receive_board_change(&mut self) -> zrdds_safe::Result<Option<DDSBoardChangeData>> {
        match self.board_subscriber.try_recv()? {
            Some(data) => {
                let json_str = String::from_utf8(data)
                    .map_err(|e| zrdds_safe::Error::Other { message: format!("UTF8转换失败: {}", e) })?;
                    
                let dds_data: DDSBoardChangeData = serde_json::from_str(&json_str)
                    .map_err(|e| zrdds_safe::Error::Other { message: format!("JSON反序列化失败: {}", e) })?;
                
                // 冲突检测逻辑
                if let Some(resolved_data) = self.resolve_conflicts(dds_data)? {
                    Ok(Some(resolved_data))
                } else {
                    Ok(None) // 冲突未解决，暂不处理
                }
            }
            None => Ok(None),
        }
    }
    
    // 冲突检测和解决
    fn resolve_conflicts(&mut self, incoming_data: DDSBoardChangeData) -> zrdds_safe::Result<Option<DDSBoardChangeData>> {
        let incoming_timestamp = &incoming_data.timestamp;
        let source_id = &incoming_data.source_id;
        
        // 检查时间戳冲突
        if let Some(ref last_timestamp) = self.last_operation_timestamp {
            if incoming_timestamp <= last_timestamp {
                println!("⚠️ 检测到时间戳冲突: {} <= {}", incoming_timestamp, last_timestamp);
                
                // 简单的冲突解决策略：按source_id排序，字典序小的优先
                if let Some(pending) = self.pending_operations.get(source_id) {
                    if pending.timestamp > *incoming_timestamp {
                        println!("🔄 应用较早的操作: {}", incoming_timestamp);
                        self.pending_operations.insert(source_id.clone(), incoming_data.clone());
                        return Ok(Some(incoming_data));
                    } else {
                        println!("⏭️ 跳过较晚的操作: {}", incoming_timestamp);
                        return Ok(None);
                    }
                }
            }
        }
        
        // 检查操作冲突（同一路径的并发修改）
        if self.has_path_conflicts(&incoming_data) {
            println!("⚠️ 检测到路径冲突，应用合并策略");
            return Ok(Some(self.merge_conflicting_operations(incoming_data)?));
        }
        
        // 更新状态
        self.last_operation_timestamp = Some(incoming_timestamp.clone());
        self.pending_operations.insert(source_id.clone(), incoming_data.clone());
        
        Ok(Some(incoming_data))
    }
    
    // 检查路径冲突
    fn has_path_conflicts(&self, incoming_data: &DDSBoardChangeData) -> bool {
        for (_, pending_data) in &self.pending_operations {
            for incoming_op in &incoming_data.operations {
                for pending_op in &pending_data.operations {
                    if self.operations_conflict(incoming_op, pending_op) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    // 检查两个操作是否冲突
    fn operations_conflict(&self, op1: &crate::shared_types::Operation, op2: &crate::shared_types::Operation) -> bool {
        use crate::shared_types::Operation;
        
        match (op1, op2) {
            (Operation::Set(set1), Operation::Set(set2)) => {
                // 同一路径的set操作冲突
                set1.path == set2.path
            }
            (Operation::Remove(rem1), Operation::Set(set2)) => {
                // 删除和修改同一路径冲突
                rem1.path == set2.path
            }
            (Operation::Set(set1), Operation::Remove(rem2)) => {
                // 修改和删除同一路径冲突
                set1.path == rem2.path
            }
            _ => false, // 其他情况暂不认为冲突
        }
    }
    
    // 合并冲突操作
    // 在 merge_conflicting_operations 函数中，第154行
    fn merge_conflicting_operations(&self, incoming_data: DDSBoardChangeData) -> zrdds_safe::Result<DDSBoardChangeData> {
    // 简单的合并策略：保留最新的操作，丢弃冲突的旧操作
    let merged_data = incoming_data.clone(); // 👈 移除 mut 关键字
    
    // 这里可以实现更复杂的合并逻辑，比如：
    // 1. 属性级别的合并
    // 2. 操作转换（Operational Transformation）
    // 3. 基于优先级的冲突解决
    
    println!("🔀 应用简单合并策略：保留最新操作");
    
    Ok(merged_data)
    }
    
    // 清理过期的待处理操作
    pub fn cleanup_pending_operations(&mut self, max_age_seconds: i64) {
        let now = chrono::Utc::now();
        let mut to_remove = Vec::new();
        
        for (source_id, data) in &self.pending_operations {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&data.timestamp) {
                let age = now.signed_duration_since(timestamp.with_timezone(&chrono::Utc));
                if age.num_seconds() > max_age_seconds {
                    to_remove.push(source_id.clone());
                }
            }
        }
        
        for source_id in to_remove {
            self.pending_operations.remove(&source_id);
            println!("🧹 清理过期操作: {}", source_id);
        }
    }
}