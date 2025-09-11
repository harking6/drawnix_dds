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
use tauri::{AppHandle, Emitter};
use chrono;
use uuid;

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

            // 为发布线程克隆
            let dds_manager_publish = dds_manager.clone();
            let source_id_publish = source_id.clone();
            let handle_publish = handle.clone();

            thread::spawn(move || {
                let mut x = 0.0;
                let mut first = true;

                loop {
                    thread::sleep(Duration::from_secs(1));
                    x += 20.0;

                    let rect = PlaitElement {
                        id: "node-1".into(),
                        element_type: "geometry".into(),
                        shape: "rectangle".into(),
                        points: vec![Point(x, 0.0), Point(x + 100.0, 100.0)],
                        children: None,
                    };

                    let change = if first {
                        first = false;
                        // 第一次：插入节点
                        BoardChangeData {
                            operations: vec![Operation::Insert(InsertNodeOperation {
                                op_type: "insert_node".into(),
                                path: vec![0], // 插到根节点 children[0]
                                node: rect,
                            })],
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            source_id: source_id_publish.clone(),
                        }
                    } else {
                        // 后续：只更新节点（移动矩形）
                        BoardChangeData {
                            operations: vec![Operation::Set(SetNodeOperation {
                                op_type: "set_node".into(),
                                path: vec![0], // 更新第一个节点
                                node: rect,
                            })],
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            source_id: source_id_publish.clone(),
                        }
                    };

                    // 发送到前端
                    if let Err(e) = handle_publish.emit("board-change", &change) {
                        eprintln!("前端发送失败: {}", e);
                    }

                    // 通过DDS广播（如果可用）
                    if let Some(ref manager) = dds_manager_publish {
                        if let Ok(manager_lock) = manager.lock() {
                            if let Err(e) = manager_lock.publish_board_change(&change) {
                                eprintln!("DDS发布失败: {}", e);
                            }
                        }
                    }

                    println!("✅ 已发送操作，x = {}", x);
                }
            });

            // 启动 DDS 订阅线程（如果 DDS 可用）
            if let Some(dds_manager_subscribe) = dds_manager {
                let handle_subscribe = handle.clone();
                let source_id_subscribe = source_id.clone();

                thread::spawn(move || loop {
                    if let Ok(mut manager_lock) = dds_manager_subscribe.lock() {
                        match manager_lock.try_receive_board_change() {
                            Ok(Some(board_data)) => {
                                // 避免回环：不处理自己发送的消息
                                if board_data.source_id != source_id_subscribe {
                                    println!("📨 收到远程白板变化: {:?}", board_data.operations.len());
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
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
