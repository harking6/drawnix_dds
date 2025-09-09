use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use zrdds_safe::prelude::*;

/// Drawnix DDS 订阅者 - 监听白板数据变化
#[derive(Parser)]
#[command(name = "drawnix-dds-subscriber")]
#[command(about = "监听 Drawnix 白板的 DDS 数据变化")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 监听白板变化数据
    Listen {
        /// DDS 域 ID
        #[arg(short, long, default_value = "150")]
        domain_id: u32,
        
        /// 要监听的主题
        #[arg(short, long)]
        topic: Option<String>,
        
        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,
        
        /// 监听时长（秒），0 表示无限期监听
        #[arg(short = 'T', long, default_value = "0")]
        timeout: u64,
    },
    /// 运行测试模式
    Test {
        /// DDS 域 ID
        #[arg(short, long, default_value = "150")]
        domain_id: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct ElementChange {
    operation_type: String,
    element_id: Option<String>,
    element_type: Option<String>,
    element_data: Option<serde_json::Value>,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BoardChangeData {
    children_count: usize,
    operation_count: usize,
    operations: Vec<serde_json::Value>,
    changes: Vec<ElementChange>,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ElementChangesData {
    added: Vec<serde_json::Value>,
    removed: Vec<serde_json::Value>,
    modified: Vec<serde_json::Value>,
    timestamp: String,
}

struct DDSSubscriber {
    domain: Domain,
    board_subscriber: Subscriber,
    element_subscriber: Subscriber,
}

impl DDSSubscriber {
    fn new(domain_id: u32) -> zrdds_safe::Result<Self> {
        println!("{}", "🔌 初始化 DDS 订阅者连接...".cyan());
        
        let domain = Domain::builder()
            .domain_id(domain_id)
            .build()?;
            
        let board_subscriber = domain.create_subscriber("DrawnixBoardChanges")?;
        let element_subscriber = domain.create_subscriber("DrawnixElementChanges")?;
        
        println!("{}", "✅ DDS 订阅者连接已建立".green());
        println!("   📡 监听主题: DrawnixBoardChanges, DrawnixElementChanges");
        println!("   🌐 域 ID: {}", domain_id);
        
        Ok(DDSSubscriber {
            domain,
            board_subscriber,
            element_subscriber,
        })
    }
    
    fn listen(&mut self, verbose: bool, timeout: u64) -> zrdds_safe::Result<()> {
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        
        // 设置 Ctrl+C 处理
        ctrlc::set_handler(move || {
            println!("\n{}", "🛑 接收到中断信号，正在优雅退出...".yellow());
            r.store(false, Ordering::SeqCst);
        }).expect("设置 Ctrl+C 处理器失败");
        
        let start_time = std::time::Instant::now();
        let mut board_count = 0;
        let mut element_count = 0;
        
        println!("{}", "👂 开始监听 DDS 消息...".bright_green());
        println!("{}", "   按 Ctrl+C 停止监听".dimmed());
        if timeout > 0 {
            println!("   ⏱️  监听时长: {} 秒", timeout);
        }
        println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed());
        
        while running.load(Ordering::SeqCst) {
            // 检查超时
            if timeout > 0 && start_time.elapsed().as_secs() >= timeout {
                println!("\n{}", "⏰ 监听时间到达，退出...".yellow());
                break;
            }
            
            // 检查白板变化消息
            match self.board_subscriber.try_recv() {
                Ok(Some(data)) => {
                    board_count += 1;
                    self.handle_board_message(data, board_count, verbose)?;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("{} {}", "❌ 白板消息接收错误:".red(), e);
                }
            }
            
            // 检查元素变化消息
            match self.element_subscriber.try_recv() {
                Ok(Some(data)) => {
                    element_count += 1;
                    self.handle_element_message(data, element_count, verbose)?;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("{} {}", "❌ 元素消息接收错误:".red(), e);
                }
            }
            
            // 短暂休眠避免过度消耗 CPU
            thread::sleep(Duration::from_millis(10));
        }
        
        println!("\n{}", "📊 监听统计:".bright_blue());
        println!("   📋 白板变化消息: {}", board_count.to_string().bright_green());
        println!("   🔧 元素变化消息: {}", element_count.to_string().bright_green());
        println!("   ⏱️  总监听时长: {:.2} 秒", start_time.elapsed().as_secs_f64());
        println!("{}", "🎉 监听结束".bright_green());
        
        Ok(())
    }
    
    fn handle_board_message(&self, data: Vec<u8>, count: u32, verbose: bool) -> zrdds_safe::Result<()> {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        
        match String::from_utf8(data.clone()) {
            Ok(json_str) => {
                match serde_json::from_str::<BoardChangeData>(&json_str) {
                    Ok(board_data) => {
                        println!("{} {} #{} 📋 白板变化", 
                            format!("[{}]", timestamp).dimmed(),
                            "📨".bright_green(),
                            count.to_string().bright_yellow()
                        );
                        println!("   📊 元素数量: {}", board_data.children_count.to_string().cyan());
                        println!("   🔧 操作数量: {}", board_data.operation_count.to_string().cyan());
                        
                        if verbose {
                            println!("   📋 操作详情:");
                            for (i, op) in board_data.operations.iter().enumerate() {
                                println!("     [{}] {}", i, serde_json::to_string_pretty(op).unwrap_or_default());
                            }
                            
                            println!("   🔍 变化详情:");
                            for (i, change) in board_data.changes.iter().enumerate() {
                                println!("     [{}] 类型: {}, ID: {:?}, 元素类型: {:?}", 
                                    i, 
                                    change.operation_type.bright_magenta(), 
                                    change.element_id, 
                                    change.element_type
                                );
                                if let Some(data) = &change.element_data {
                                    println!("         数据: {}", serde_json::to_string_pretty(data).unwrap_or_default());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} {} #{} ❌ JSON 解析失败: {}", 
                            format!("[{}]", timestamp).dimmed(),
                            "📨".bright_green(),
                            count.to_string().bright_yellow(),
                            e.to_string().red()
                        );
                        if verbose {
                            println!("   原始数据: {}", json_str);
                        }
                    }
                }
            }
            Err(e) => {
                println!("{} {} #{} ❌ UTF-8 解码失败: {}", 
                    format!("[{}]", timestamp).dimmed(),
                    "📨".bright_green(),
                    count.to_string().bright_yellow(),
                    e.to_string().red()
                );
                if verbose {
                    println!("   原始字节: {:?}", data);
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_element_message(&self, data: Vec<u8>, count: u32, verbose: bool) -> zrdds_safe::Result<()> {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        
        match String::from_utf8(data.clone()) {
            Ok(json_str) => {
                match serde_json::from_str::<ElementChangesData>(&json_str) {
                    Ok(element_data) => {
                        println!("{} {} #{} 🔧 元素变化", 
                            format!("[{}]", timestamp).dimmed(),
                            "📨".bright_green(),
                            count.to_string().bright_yellow()
                        );
                        println!("   ➕ 新增: {}", element_data.added.len().to_string().green());
                        println!("   ➖ 删除: {}", element_data.removed.len().to_string().red());
                        println!("   🔄 修改: {}", element_data.modified.len().to_string().blue());
                        
                        if verbose {
                            if !element_data.added.is_empty() {
                                println!("   ➕ 新增元素:");
                                for (i, elem) in element_data.added.iter().enumerate() {
                                    println!("     [{}] {}", i, serde_json::to_string_pretty(elem).unwrap_or_default());
                                }
                            }
                            
                            if !element_data.removed.is_empty() {
                                println!("   ➖ 删除元素:");
                                for (i, elem) in element_data.removed.iter().enumerate() {
                                    println!("     [{}] {}", i, serde_json::to_string_pretty(elem).unwrap_or_default());
                                }
                            }
                            
                            if !element_data.modified.is_empty() {
                                println!("   🔄 修改元素:");
                                for (i, elem) in element_data.modified.iter().enumerate() {
                                    println!("     [{}] {}", i, serde_json::to_string_pretty(elem).unwrap_or_default());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("{} {} #{} ❌ JSON 解析失败: {}", 
                            format!("[{}]", timestamp).dimmed(),
                            "📨".bright_green(),
                            count.to_string().bright_yellow(),
                            e.to_string().red()
                        );
                        if verbose {
                            println!("   原始数据: {}", json_str);
                        }
                    }
                }
            }
            Err(e) => {
                println!("{} {} #{} ❌ UTF-8 解码失败: {}", 
                    format!("[{}]", timestamp).dimmed(),
                    "📨".bright_green(),
                    count.to_string().bright_yellow(),
                    e.to_string().red()
                );
                if verbose {
                    println!("   原始字节: {:?}", data);
                }
            }
        }
        
        Ok(())
    }
    
    fn run_test(&mut self) -> zrdds_safe::Result<()> {
        println!("{}", "🧪 启动测试模式".bright_blue());
        println!("   监听 10 秒钟，然后显示统计信息");
        
        let start_time = std::time::Instant::now();
        let mut messages = Vec::new();
        
        while start_time.elapsed().as_secs() < 10 {
            // 检查白板变化消息
            if let Ok(Some(data)) = self.board_subscriber.try_recv() {
                messages.push(("board", data, chrono::Local::now()));
            }
            
            // 检查元素变化消息
            if let Ok(Some(data)) = self.element_subscriber.try_recv() {
                messages.push(("element", data, chrono::Local::now()));
            }
            
            thread::sleep(Duration::from_millis(100));
        }
        
        println!("\n{}", "📊 测试结果:".bright_blue());
        println!("   📨 接收到消息总数: {}", messages.len().to_string().bright_green());
        
        let board_count = messages.iter().filter(|(t, _, _)| *t == "board").count();
        let element_count = messages.iter().filter(|(t, _, _)| *t == "element").count();
        
        println!("   📋 白板变化消息: {}", board_count.to_string().cyan());
        println!("   🔧 元素变化消息: {}", element_count.to_string().cyan());
        
        if messages.is_empty() {
            println!("\n{}", "⚠️  未接收到任何消息".yellow());
            println!("   可能的原因:");
            println!("   1. 发布者尚未启动");
            println!("   2. DDS 域配置不匹配");
            println!("   3. 网络连接问题");
        } else {
            println!("\n{}", "✅ 测试成功！DDS 通信正常".bright_green());
        }
        
        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Listen { domain_id, topic, verbose, timeout } => {
            match DDSSubscriber::new(domain_id) {
                Ok(mut subscriber) => {
                    if let Some(topic_name) = topic {
                        println!("⚠️  注意: 当前版本不支持自定义主题过滤，将监听所有主题");
                        println!("   请求的主题: {}", topic_name);
                    }
                    
                    if let Err(e) = subscriber.listen(verbose, timeout) {
                        eprintln!("{} {}", "❌ 监听失败:".red(), e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "❌ 初始化失败:".red(), e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Test { domain_id } => {
            match DDSSubscriber::new(domain_id) {
                Ok(mut subscriber) => {
                    if let Err(e) = subscriber.run_test() {
                        eprintln!("{} {}", "❌ 测试失败:".red(), e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "❌ 初始化失败:".red(), e);
                    std::process::exit(1);
                }
            }
        }
    }
}
