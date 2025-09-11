use zrdds_safe::prelude::*;
use crate::shared_types::BoardChangeData;
use std::sync::Arc;

pub struct DDSManager {
    domain: Domain,
    board_publisher: Publisher,
    board_subscriber: Subscriber,
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
        })
    }
    
    pub fn publish_board_change(&self, data: &BoardChangeData) -> zrdds_safe::Result<()> {
        let json_data = serde_json::to_string(data)
            .map_err(|e| zrdds_safe::Error::Other { message: format!("JSON序列化失败: {}", e) })?;
        self.board_publisher.publish(json_data.as_bytes())?;
        Ok(())
    }

    // 直接发布原始 JSON，便于与前端协议演进对齐
    pub fn publish_raw(&self, json: &str) -> zrdds_safe::Result<()> {
        println!("[DDS][publish_raw] 发布 JSON 大小: {} bytes", json.len());
        self.board_publisher.publish(json.as_bytes())?;
        println!("[DDS][publish_raw] 发布完成");
        Ok(())
    }
    
    pub fn try_receive_board_change(&mut self) -> zrdds_safe::Result<Option<BoardChangeData>> {
        match self.board_subscriber.try_recv()? {
            Some(data) => {
                let json_str = String::from_utf8(data)
                    .map_err(|e| zrdds_safe::Error::Other { message: format!("UTF8转换失败: {}", e) })?;
                let board_data: BoardChangeData = serde_json::from_str(&json_str)
                    .map_err(|e| zrdds_safe::Error::Other { message: format!("JSON反序列化失败: {}", e) })?;
                Ok(Some(board_data))
            }
            None => Ok(None),
        }
    }

    pub fn try_receive_raw(&mut self) -> zrdds_safe::Result<Option<String>> {
        match self.board_subscriber.try_recv()? {
            Some(data) => {
                println!("[DDS][try_receive_raw] 收到数据, 大小: {} bytes", data.len());
                let json_str = String::from_utf8(data)
                    .map_err(|e| zrdds_safe::Error::Other { message: format!("UTF8转换失败: {}", e) })?;
                println!("[DDS][try_receive_raw] UTF8 转换完成");
                Ok(Some(json_str))
            }
            None => Ok(None),
        }
    }
}
