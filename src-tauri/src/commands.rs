use crate::dds_manager::DDSManager;
use std::sync::{Arc, Mutex};
use tauri::State;

// 共享状态：可选的 DDS 管理器句柄
pub struct SharedState(pub Option<Arc<Mutex<DDSManager>>>);

#[tauri::command]
pub async fn test_connection() -> String {
    println!("[TAURI][cmd] test_connection invoked");
    "ok".to_string()
}

#[tauri::command]
pub async fn relay_board_change(state: State<'_, SharedState>, data: String) -> Result<(), String> {
    println!("[TAURI][cmd] relay_board_change 被调用, payload 大小: {} bytes", data.len());
    // 尝试解析关键信息以便调试
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
        let kind = val.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
        let src = val.get("source_id").and_then(|v| v.as_str()).unwrap_or("?");
        let ops = val.get("operations").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        let children = val.get("children").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
        println!("[TAURI][cmd] kind={}, source_id={}, ops={}, children={}", kind, src, ops, children);
    } else {
        println!("[TAURI][cmd] payload 不是有效 JSON");
    }

    if let Some(ref arc) = state.0 {
        match arc.lock() {
            Ok(manager) => {
                let res = manager.publish_raw(&data);
                match res {
                    Ok(_) => {
                        println!("[TAURI][cmd] DDS 发布成功");
                        Ok(())
                    },
                    Err(e) => {
                        println!("[TAURI][cmd] DDS 发布失败: {:?}", e);
                        Err(format!("DDS发布失败: {:?}", e))
                    }
                }
            }
            Err(_) => {
                println!("[TAURI][cmd] 无法获取DDS状态锁");
                Err("无法获取DDS状态锁".into())
            },
        }
    } else {
        println!("[TAURI][cmd] DDS 未初始化");
        Err("DDS 未初始化".into())
    }
}
