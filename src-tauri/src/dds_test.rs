//! DDS 集成测试模块
//! 
//! 此模块包含针对 drawnix-dds 中 DDS 功能的测试用例

use std::thread;
use std::time::Duration;
use serde_json::json;
use crate::{BoardChangeData, ElementChange, DDSManager};

/// 测试 DDS 基础连接功能
pub fn test_dds_basic_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始测试 DDS 基础连接...");
    
    // 创建 DDS 管理器
    let manager = DDSManager::new()?;
    println!("✅ DDS 管理器创建成功");
    
    // 创建测试数据
    let test_data = BoardChangeData {
        children_count: 2,
        operation_count: 1,
        operations: vec![json!({
            "type": "create",
            "element": "rectangle",
            "position": {"x": 100, "y": 200},
            "size": {"width": 50, "height": 75}
        })],
        changes: vec![
            ElementChange {
                operation_type: "create".to_string(),
                element_id: Some("rect-001".to_string()),
                element_type: Some("rectangle".to_string()),
                element_data: Some(json!({
                    "x": 100,
                    "y": 200,
                    "width": 50,
                    "height": 75,
                    "color": "#ff0000"
                })),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }
        ],
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    // 测试发布
    manager.publish_board_change(&test_data)?;
    println!("✅ 白板变化数据发布成功");
    
    // 测试元素变化发布
    let added_elements = vec![json!({
        "id": "circle-001",
        "type": "circle",
        "x": 150,
        "y": 150,
        "radius": 25
    })];
    
    let removed_elements = vec![];
    let modified_elements = vec![];
    
    manager.publish_element_changes(&added_elements, &removed_elements, &modified_elements, &chrono::Utc::now().to_rfc3339())?;
    println!("✅ 元素变化数据发布成功");
    
    println!("🎉 DDS 基础连接测试完成！");
    Ok(())
}

/// 测试 DDS 高频数据发布
pub fn test_dds_high_frequency_publishing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始测试 DDS 高频数据发布...");
    
    let manager = DDSManager::new()?;
    
    // 模拟高频白板操作
    for i in 0..10 {
        let test_data = BoardChangeData {
            children_count: i + 1,
            operation_count: 1,
            operations: vec![json!({
                "type": "move",
                "element_id": format!("element-{}", i),
                "new_position": {"x": i * 10, "y": i * 10}
            })],
            changes: vec![
                ElementChange {
                    operation_type: "move".to_string(),
                    element_id: Some(format!("element-{}", i)),
                    element_type: Some("rectangle".to_string()),
                    element_data: Some(json!({
                        "x": i * 10,
                        "y": i * 10,
                        "sequence": i
                    })),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        manager.publish_board_change(&test_data)?;
        println!("📡 [{}] 发布移动操作: element-{}", i, i);
        
        // 短暂延迟模拟实际使用场景
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("🎉 DDS 高频数据发布测试完成！");
    Ok(())
}

/// 测试 DDS 复杂数据结构
pub fn test_dds_complex_data() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 开始测试 DDS 复杂数据结构...");
    
    let manager = DDSManager::new()?;
    
    // 创建复杂的白板变化数据
    let complex_data = BoardChangeData {
        children_count: 5,
        operation_count: 3,
        operations: vec![
            json!({
                "type": "create",
                "element": "group",
                "id": "group-001",
                "children": ["rect-001", "circle-001", "text-001"]
            }),
            json!({
                "type": "style_change",
                "element_id": "rect-001",
                "old_style": {"color": "#ff0000", "stroke": "#000000"},
                "new_style": {"color": "#00ff00", "stroke": "#333333"}
            }),
            json!({
                "type": "transform",
                "element_id": "group-001",
                "transform": {
                    "translate": [50, 75],
                    "rotate": 15,
                    "scale": [1.2, 1.2]
                }
            })
        ],
        changes: vec![
            ElementChange {
                operation_type: "create".to_string(),
                element_id: Some("group-001".to_string()),
                element_type: Some("group".to_string()),
                element_data: Some(json!({
                    "type": "group",
                    "children": ["rect-001", "circle-001", "text-001"],
                    "bounds": {"x": 0, "y": 0, "width": 200, "height": 150}
                })),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            ElementChange {
                operation_type: "modify".to_string(),
                element_id: Some("rect-001".to_string()),
                element_type: Some("rectangle".to_string()),
                element_data: Some(json!({
                    "style": {"color": "#00ff00", "stroke": "#333333"},
                    "previous_style": {"color": "#ff0000", "stroke": "#000000"}
                })),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            ElementChange {
                operation_type: "transform".to_string(),
                element_id: Some("group-001".to_string()),
                element_type: Some("group".to_string()),
                element_data: Some(json!({
                    "transform": {
                        "translate": [50, 75],
                        "rotate": 15,
                        "scale": [1.2, 1.2]
                    }
                })),
                timestamp: chrono::Utc::now().to_rfc3339(),
            }
        ],
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    manager.publish_board_change(&complex_data)?;
    println!("✅ 复杂数据结构发布成功");
    
    // 测试复杂元素变化
    let added = vec![
        json!({
            "id": "advanced-shape-001",
            "type": "path",
            "data": "M10,10 L50,10 L50,50 L10,50 Z",
            "style": {"fill": "#ffaa00", "stroke": "#aa0000", "stroke-width": 2}
        })
    ];
    
    let removed = vec![
        json!({"id": "old-element-001", "type": "rectangle"})
    ];
    
    let modified = vec![
        json!({
            "id": "rect-001",
            "type": "rectangle", 
            "changes": {
                "style": {"from": {"color": "#ff0000"}, "to": {"color": "#00ff00"}},
                "position": {"from": {"x": 100, "y": 100}, "to": {"x": 150, "y": 125}}
            }
        })
    ];
    
    manager.publish_element_changes(&added, &removed, &modified, &chrono::Utc::now().to_rfc3339())?;
    println!("✅ 复杂元素变化发布成功");
    
    println!("🎉 DDS 复杂数据结构测试完成！");
    Ok(())
}

/// 运行所有 DDS 测试
pub fn run_all_dds_tests() {
    println!("🚀 开始运行 DDS 集成测试套件...");
    println!("================================================");
    
    let tests = vec![
        ("基础连接测试", test_dds_basic_connection as fn() -> Result<(), Box<dyn std::error::Error>>),
        ("高频发布测试", test_dds_high_frequency_publishing),
        ("复杂数据测试", test_dds_complex_data),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (name, test_fn) in tests {
        println!("\n📋 运行测试: {}", name);
        println!("----------------------------------------");
        
        match test_fn() {
            Ok(()) => {
                println!("✅ {} - 通过", name);
                passed += 1;
            }
            Err(e) => {
                println!("❌ {} - 失败: {}", name, e);
                failed += 1;
            }
        }
    }
    
    println!("\n================================================");
    println!("📊 测试结果汇总:");
    println!("  ✅ 通过: {}", passed);
    println!("  ❌ 失败: {}", failed);
    println!("  📈 成功率: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);
    
    if failed == 0 {
        println!("🎉 所有测试通过！DDS 集成验证成功！");
    } else {
        println!("⚠️  有测试失败，请检查 DDS 配置和连接");
    }
}
