#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shared_types;
mod dds_manager;

use shared_types::*;
use dds_manager::DDSManager;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, State, Manager}; // 👈 添加 Manager 导入
use chrono;
use uuid;

// 全局DDS管理器状态
struct AppState {
    dds_manager: Option<Arc<Mutex<DDSManager>>>,
    source_id: String,
}

// Tauri命令：发布白板变化到DDS
#[tauri::command]
async fn publish_board_change(
    state: State<'_, AppState>,
    board_data: BoardChangeData,
) -> Result<(), String> {
    if let Some(ref dds_manager) = state.dds_manager {
        if let Ok(manager_lock) = dds_manager.lock() {
            // 使用新的DDS管理器接口
            manager_lock.publish_board_change(&board_data, &state.source_id)
                .map_err(|e| format!("DDS发布失败: {}", e))?;
            
            println!("✅ 已通过DDS发布白板变化，操作数: {}", board_data.operations.len());
            Ok(())
        } else {
            Err("无法获取DDS管理器锁".to_string())
        }
    } else {
        Err("DDS未初始化".to_string())
    }
}

// Tauri命令：获取DDS连接状态
#[tauri::command]
async fn get_dds_status(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.dds_manager.is_some())
}

// Tauri命令：手动触发冲突检测清理
#[tauri::command]
async fn cleanup_dds_conflicts(state: State<'_, AppState>) -> Result<(), String> {
    if let Some(ref dds_manager) = state.dds_manager {
        if let Ok(mut manager_lock) = dds_manager.lock() {
            manager_lock.cleanup_pending_operations(30); // 清理30秒前的操作
            println!("🧹 已清理DDS冲突检测缓存");
            Ok(())
        } else {
            Err("无法获取DDS管理器锁".to_string())
        }
    } else {
        Err("DDS未初始化".to_string())
    }
}

fn main() {
    let source_id = uuid::Uuid::new_v4().to_string();
    println!("🆔 当前实例ID: {}", source_id);

    tauri::Builder::default()
        .setup(move |app| { // 👈 添加 move 关键字
            let handle: AppHandle = app.handle().clone();

            // 创建 DDS 管理器
            let dds_manager = match DDSManager::new() {
                Ok(manager) => {
                    println!("✅ DDS 连接已建立，支持冲突检测");
                    Some(Arc::new(Mutex::new(manager)))
                }
                Err(e) => {
                    eprintln!("⚠️ DDS 初始化失败: {}，将仅使用本地模式", e);
                    None
                }
            };

            // 设置应用状态
            let app_state = AppState {
                dds_manager: dds_manager.clone(),
                source_id: source_id.clone(), // 现在可以安全使用
            };
            app.manage(app_state);

            // 启动 DDS 订阅线程（如果 DDS 可用）
            if let Some(dds_manager_subscribe) = dds_manager {
                let handle_subscribe = handle.clone();
                let source_id_subscribe = source_id.clone(); // 现在可以安全使用

                thread::spawn(move || {
                    println!("🔄 启动DDS订阅线程，支持冲突检测...");
                    
                    let mut cleanup_counter = 0;
                    
                    loop {
                        if let Ok(mut manager_lock) = dds_manager_subscribe.lock() {
                            match manager_lock.try_receive_board_change() {
                                Ok(Some(dds_data)) => {
                                    // 避免回环：不处理自己发送的消息
                                    if dds_data.source_id != source_id_subscribe {
                                        println!("📨 收到远程白板变化: {} 个操作，来源: {}，时间: {}", 
                                            dds_data.operations.len(), 
                                            &dds_data.source_id[..8],
                                            &dds_data.timestamp[11..19] // 只显示时分秒
                                        );
                                        
                                        // 转换为前端需要的BoardChangeData格式
                                        let board_change = BoardChangeData {
                                            children: dds_data.children.unwrap_or_default(),
                                            operations: dds_data.operations,
                                            viewport: dds_data.viewport.unwrap_or(Viewport { zoom: 1.0, x: 0.0, y: 0.0 }),
                                            selection: None,
                                            theme: PlaitTheme { 
                                                color_mode: Some("light".to_string()),
                                                extra: std::collections::HashMap::new()
                                            },
                                            timestamp: Some(dds_data.timestamp),
                                            source_id: Some(dds_data.source_id),
                                        };
                                        
                                        // 转发到前端
                                        if let Err(e) = handle_subscribe.emit("remote-board-change", &board_change) {
                                            eprintln!("转发到前端失败: {}", e);
                                        }
                                    }
                                }
                                Ok(None) => {
                                    // 没有消息，短暂休眠
                                    thread::sleep(Duration::from_millis(50));
                                }
                                Err(e) => {
                                    eprintln!("DDS接收失败: {}", e);
                                    thread::sleep(Duration::from_millis(1000));
                                }
                            }
                            
                            // 定期清理冲突检测缓存
                            cleanup_counter += 1;
                            if cleanup_counter >= 1200 { // 约1分钟（50ms * 1200）
                                manager_lock.cleanup_pending_operations(60);
                                cleanup_counter = 0;
                            }
                        } else {
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            publish_board_change,
            get_dds_status,
            cleanup_dds_conflicts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
