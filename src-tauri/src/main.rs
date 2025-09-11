#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod shared_types;
mod dds_manager;
mod commands;

use shared_types::*;
use dds_manager::DDSManager;
use commands::{SharedState, relay_board_change, test_connection};
use serde::Serialize;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager};

use chrono;
use uuid;

fn main() {
    tauri::Builder::default()
        // 注册前端可调用的命令
        .invoke_handler(tauri::generate_handler![test_connection, relay_board_change])
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

            // 注入共享状态，供前端命令与后台线程共享同一 DDS 实例
            app.manage(SharedState(dds_manager.clone()));
            
            let source_id = uuid::Uuid::new_v4().to_string();
            
            // 移除演示线程：避免每5秒自动发送 node-1 变化干扰联调
            
            // 启动 DDS 订阅线程（如果 DDS 可用）
            if let Some(dds_manager_subscribe) = dds_manager {
                let handle_subscribe = handle.clone();
                let source_id_subscribe = source_id.clone();
                
                thread::spawn(move || {
                    loop {
                        if let Ok(mut manager_lock) = dds_manager_subscribe.lock() {
                            match manager_lock.try_receive_raw() {
                                Ok(Some(json_str)) => {
                                    println!("[DDS][subscribe] 收到 JSON: {} bytes", json_str.len());
                                    // 直接将原始 JSON 转发到前端，避免协议不匹配
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                                        // 如果包含 source_id，与自身一致则忽略
                                        if val.get("source_id").and_then(|v| v.as_str()) == Some(&source_id_subscribe) {
                                            println!("[DDS][subscribe] 丢弃自身 source_id 的消息");
                                            continue;
                                        }
                                        if let Err(e) = handle_subscribe.emit("board-change", &val) {
                                            eprintln!("转发到前端失败: {}", e);
                                        } else {
                                            println!("[DDS][subscribe] 已转发到前端");
                                        }
                                    } else {
                                        eprintln!("DDS 消息 JSON 解析失败，已忽略");
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
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
