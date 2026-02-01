```rust
use util::delay_test::{DelayTester, TcpPingConfig, quick_batch_ping};
use std::time::Duration;
use clap::{Parser, Subcommand};

/// TCP Ping测试工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 测试单个目标
    Single {
        /// 目标地址（格式：host:port 或 host）
        target: String,
  
        /// 超时时间（秒）
        #[arg(short, long, default_value_t = 5)]
        timeout: u64,
  
        /// 重试次数
        #[arg(short, long, default_value_t = 2)]
        retry: usize,
    },
  
    /// 批量测试多个目标
    Batch {
        /// 目标地址列表文件（每行一个目标）
        #[arg(short, long)]
        file: Option<String>,
  
        /// 目标地址（可多个）
        #[arg()]
        targets: Vec<String>,
  
        /// 并发数
        #[arg(short, long, default_value_t = 50)]
        concurrency: usize,
  
        /// 超时时间（秒）
        #[arg(short, long, default_value_t = 5)]
        timeout: u64,
  
        /// 输出格式
        #[arg(short, long, default_value = "text")]
        format: String,
  
        /// 保存结果到文件
        #[arg(short, long)]
        output: Option<String>,
    },
  
    /// 生成测试报告
    Report {
        /// 输入文件（JSON格式）
        #[arg(short, long)]
        input: String,
  
        /// 输出格式
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
  
    match args.command {
        Commands::Single { target, timeout, retry } => {
            let config = TcpPingConfig {
                timeout: Duration::from_secs(timeout),
                retry_count: retry,
                ..Default::default()
            };
    
            let tester = DelayTester::new(config);
            let result = tester.ping_single(&target).await;
    
            println!("TCP Ping 测试结果:");
            println!("==================");
            println!("目标: {}", result.target);
            println!("主机: {}", result.host);
            println!("端口: {}", result.port);
            println!("状态: {}", if result.success { "✅ 成功" } else { "❌ 失败" });
    
            if let Some(latency) = result.latency_ms {
                println!("延迟: {:.2} ms", latency);
            }
    
            if let Some(error) = result.error {
                println!("错误: {}", error);
            }
    
            println!("时间: {}", result.timestamp.format("%Y-%m-%d %H:%M:%S"));
        }
  
        Commands::Batch { file, targets, concurrency, timeout, format, output } => {
            let config = TcpPingConfig {
                timeout: Duration::from_secs(timeout),
                concurrency_limit: concurrency,
                ..Default::default()
            };
    
            let tester = DelayTester::new(config);
            let mut all_targets = targets;
    
            // 如果指定了文件，从文件读取目标
            if let Some(file_path) = file {
                let file_targets = std::fs::read_to_string(&file_path)?
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .map(|line| line.trim().to_string())
                    .collect::<Vec<_>>();
        
                all_targets.extend(file_targets);
            }
    
            if all_targets.is_empty() {
                eprintln!("错误: 未指定测试目标");
                std::process::exit(1);
            }
    
            println!("开始批量TCP Ping测试...");
            println!("目标数量: {}", all_targets.len());
            println!("并发限制: {}", concurrency);
            println!("超时时间: {}秒", timeout);
            println!();
    
            let stats = tester.ping_batch(all_targets).await;
    
            // 输出报告
            match format.as_str() {
                "text" => {
                    println!("{}", tester.generate_text_report(&stats));
                }
                "markdown" => {
                    println!("{}", tester.generate_markdown_report(&stats));
                }
                "json" => {
                    let json = serde_json::to_string_pretty(&stats)?;
                    println!("{}", json);
                }
                _ => {
                    println!("{}", tester.generate_text_report(&stats));
                }
            }
    
            // 保存结果
            if let Some(output_path) = output {
                if output_path.ends_with(".json") {
                    tester.save_as_json(&stats, &output_path)?;
                    println!("结果已保存到: {}", output_path);
                } else if output_path.ends_with(".csv") {
                    tester.save_as_csv(&stats, &output_path)?;
                    println!("结果已保存到: {}", output_path);
                } else {
                    std::fs::write(&output_path, tester.generate_text_report(&stats))?;
                    println!("结果已保存到: {}", output_path);
                }
            }
        }
  
        Commands::Report { input, format } => {
            let content = std::fs::read_to_string(&input)?;
            let stats: util::delay_test::PingStats = serde_json::from_str(&content)?;
    
            let tester = DelayTester::with_default_config();
    
            match format.as_str() {
                "text" => {
                    println!("{}", tester.generate_text_report(&stats));
                }
                "markdown" => {
                    println!("{}", tester.generate_markdown_report(&stats));
                }
                _ => {
                    println!("{}", tester.generate_text_report(&stats));
                }
            }
        }
    }
  
    Ok(())
}
```
