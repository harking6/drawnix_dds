use zrdds_safe::prelude::*;
use serde_json::json;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动简单发布者测试...");
    
    let domain = Domain::builder()
        .domain_id(150)
        .build()?;
    
    let board_publisher = domain.create_publisher("DrawnixBoardChanges")?;
    let element_publisher = domain.create_publisher("DrawnixElementChanges")?;
    
    println!("📡 发布者创建成功，开始发送测试消息...");
    
    // 发送白板变化测试消息
    for i in 1..=3 {
        let test_data = json!({
            "children_count": i,
            "operation_count": 1,
            "operations": [{
                "type": "test",
                "message": format!("测试消息 #{}", i),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }],
            "changes": [{
                "operation_type": "create",
                "element_id": format!("test-element-{}", i),
                "element_type": "test",
                "element_data": {
                    "x": i * 10,
                    "y": i * 10,
                    "test": true
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            }],
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        board_publisher.publish(serde_json::to_string(&test_data)?.as_bytes())?;
        println!("📨 发送白板变化消息 #{}", i);
        
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    
    // 发送元素变化测试消息
    for i in 1..=2 {
        let test_data = json!({
            "added": [{
                "id": format!("new-element-{}", i),
                "type": "rectangle",
                "x": i * 50,
                "y": i * 50
            }],
            "removed": [],
            "modified": [],
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        element_publisher.publish(serde_json::to_string(&test_data)?.as_bytes())?;
        println!("📨 发送元素变化消息 #{}", i);
        
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    
    println!("✅ 测试消息发送完成");
    
    // 等待一下确保消息都发送出去
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    Ok(())
}
