#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Serialize, Clone)]
struct Point(f64, f64);

#[derive(Debug, Serialize, Clone)]
struct PlaitElement {
    id: String,
    #[serde(rename = "type")]
    element_type: String, // "geometry"
    shape: String,        // "rectangle" | "ellipse"
    points: Vec<Point>,
}

#[derive(Debug, Serialize, Clone)]
struct InsertNodeOperation {
    #[serde(rename = "type")]
    op_type: String,   // "insert_node"
    node: PlaitElement,
}

#[derive(Debug, Serialize, Clone)]
struct SetNodeOperation {
    #[serde(rename = "type")]
    op_type: String,   // "set_node"
    node: PlaitElement,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum Operation {
    Insert(InsertNodeOperation),
    Set(SetNodeOperation),
}

#[derive(Debug, Serialize, Clone)]
struct BoardChangeData {
    operations: Vec<Operation>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle: AppHandle = app.handle().clone();

            thread::spawn(move || {
                let mut x = 0.0;
                let mut first = true;

                loop {
                    thread::sleep(Duration::from_secs(5));
                    x += 20.0;

                    // 矩形节点
                    let rect = PlaitElement {
                        id: "node-1".into(),
                        element_type: "geometry".into(),
                        shape: "rectangle".into(),
                        points: vec![Point(x, 0.0), Point(x + 100.0, 100.0)],
                    };

                    let change = if first {
                        first = false;
                        BoardChangeData {
                            operations: vec![Operation::Insert(InsertNodeOperation {
                                op_type: "insert_node".into(),
                                node: rect,
                            })],
                        }
                    } else {
                        BoardChangeData {
                            operations: vec![Operation::Set(SetNodeOperation {
                                op_type: "set_node".into(),
                                node: rect,
                            })],
                        }
                    };

                    handle.emit("board-change", change).unwrap();
                    println!("✅ 已发送操作，x = {}", x);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
