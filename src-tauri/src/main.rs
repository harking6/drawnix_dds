#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shared_types;
mod dds_manager;

use shared_types::*;
use dds_manager::DDSManager;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager}; // 🔥 添加 Manager trait
use uuid;

#[tauri::command]
async fn broadcast_board_change(
    board_change: BoardChangeData,
    state: tauri::State<'_, Arc<Mutex<Option<DDSManager>>>>
) -> Result<(), String> {
    println!("📤 准备广播用户操作: {:?}", board_change.operations.len());
    
    if let Ok(manager_guard) = state.lock() {
        if let Some(ref manager) = *manager_guard {
            if let Err(e) = manager.publish_board_change(&board_change) {
                eprintln!("❌ DDS广播失败: {}", e);
                return Err(format!("DDS广播失败: {}", e));
            }
            println!("✅ 用户操作已广播到DDS");
        } else {
            println!("⚠️ DDS管理器未初始化，跳过广播");
        }
    }
    
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle: AppHandle = app.handle().clone();
            
            // 创建 DDS 管理器
            let dds_manager = match DDSManager::new() {
                Ok(manager) => Some(Arc::new(Mutex::new(manager))),
                Err(e) => {
                    eprintln!("⚠️ DDS 初始化失败: {}，将仅使用本地模式", e);
                    None
                }
            };
            
            let source_id = uuid::Uuid::new_v4().to_string();
            
            // 启动 DDS 订阅线程（如果 DDS 可用）
            if let Some(dds_manager_subscribe) = dds_manager.clone() {
                let handle_subscribe = handle.clone();
                let source_id_subscribe = source_id.clone();
                
                thread::spawn(move || {
                    loop {
                        if let Ok(mut manager_lock) = dds_manager_subscribe.lock() {
                            match manager_lock.try_receive_board_change() {
                                Ok(Some(board_data)) => {
                                    // 避免回环：不处理自己发送的消息
                                    if board_data.source_id != source_id_subscribe {
                                        println!("📨 收到远程白板变化: {:?} 个操作", board_data.operations.len());
                                        // 转发到前端
                                        if let Err(e) = handle_subscribe.emit("board-change", &board_data) {
                                            eprintln!("转发到前端失败: {}", e);
                                        }
                                    }
                                }
                                Ok(None) => {
                                    // 没有消息，短暂休眠
                                    thread::sleep(Duration::from_millis(100));
                                }
                                Err(e) => {
                                    eprintln!("DDS接收失败: {}", e);
                                    thread::sleep(Duration::from_millis(1000));
                                }
                            }
                        } else {
                            thread::sleep(Duration::from_millis(100));
                        }
                    }
                });
            }
            
            // 将DDS管理器作为状态管理
            app.manage(Arc::new(Mutex::new(dds_manager)));
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![broadcast_board_change])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
