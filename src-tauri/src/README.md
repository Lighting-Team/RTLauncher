# 使用说明
首先，我们需要创建一个`TaskManager`实例，用于管理各种下载任务。
```rust
let task_manager = TaskManager::new();
```
`TaskManager`目前只有下载原版客户端的功能，以后可以扩展安装`Forge`、安装`OptiFine`，检查游戏文件完整性等功能。

然后，我们要配置`DownloadConfig`，用于设置下载的线程池大小、大文件阈值、大文件分块数、下载策略、最大重试次数、连接超时时间、读取超时时间等参数。
```rust

let config = DownloadConfig {
    thread_pool_size: 64,                   // 线程池大小，默认64
    large_file_threshold: 10 * 1024 * 1024, // 大文件阈值，默认10MB
    large_file_chunks: 8,                   // 大文件分块数，默认8
    strategy: DownloadStrategy::Hybrid,     // 下载策略，默认混合策略(即优先使用官方源，失败多次后再使用镜像源)
    max_retries: 3,                         // 最大重试次数，默认3次    
    connect_timeout: 30,                    // 连接超时时间，默认30秒
    read_timeout: 60,                       // 读取超时时间，默认60秒
};
```
随后，我们便可以创建`DownloadClientTask`实例，用于下载原版客户端。
```rust
let task = DownloadClientTask::new(
    "1.20.1",
    "1.20.1 绝赞原版客户端23333",
    MINECRAFT_DIR,
    config,
);
```
最后，我们只需要将这个任务添加到`TaskManager`中，即可开始下载。
```rust
let task_id = task_manager.append_task(task).await;

if let Err(e) = task_manager.start_task(&task_id).await {
    eprintln!("启动任务失败: {}", e);
    return;
}
```
以下是完整的代码示例：

```rust
use mc_download::{
    download::{DownloadConfig, DownloadStrategy},
    task::TaskManager,
    tasks::DownloadClientTask,
};
use console::Term;
use tokio::time::Instant;

const MINECRAFT_DIR: &str = ".minecraft";

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let term = Term::stdout();

    term.write_line("=======================================").unwrap();
    term.write_line("Minecraft 启动器后端").unwrap();
    term.write_line("=======================================").unwrap();
    term.write_line("").unwrap();

    let task_manager = TaskManager::new();

    let config = DownloadConfig {
        thread_pool_size: 64,
        large_file_threshold: 10 * 1024 * 1024,
        large_file_chunks: 8,
        strategy: DownloadStrategy::Hybrid,
        max_retries: 3,
        connect_timeout: 30,
        read_timeout: 60,
    };

    let task = DownloadClientTask::new(
        "1.20.1",
        "1.20.1 绝赞原版客户端23333",
        MINECRAFT_DIR,
        config,
    );
    let task_id = task_manager.append_task(task).await;

    if let Err(e) = task_manager.start_task(&task_id).await {
        eprintln!("启动任务失败: {}", e);
        return;
    }

    // 等待任务完成并显示进度
    loop {
        let info = task_manager.get_task_info(&task_id);
        match &info {
            Some(task_info) => {
                let percentage = if task_info.progress.total > 0 {
                    (task_info.progress.completed as f64 / task_info.progress.total as f64) * 100.0
                } else {
                    0.0
                };
                println!("进度: {:.2}%", percentage);

                // 检查任务是否完成
                match &task_info.status {
                    mc_download::task::TaskStatus::Completed => {
                        println!("任务完成！");
                        break;
                    }
                    mc_download::task::TaskStatus::Failed(msg) => {
                        println!("任务失败: {}", msg);
                        break;
                    }
                    _ => {}
                }
            }
            None => {
                println!("未找到任务");
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    let duration = start.elapsed();

    term.write_line("").unwrap();
    term.write_line("=======================================").unwrap();
    term.write_line("所有下载任务完成！").unwrap();
    term.write_line("=======================================").unwrap();
    term.write_line(&format!("总耗时: {:?}", duration)).unwrap();
    term.write_line("").unwrap();
}

```