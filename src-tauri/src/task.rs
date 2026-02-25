use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;
use dashmap::DashMap;
use std::collections::HashMap;

/// 任务状态
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,      // 等待中
    Running,      // 运行中
    Paused,       // 已暂停
    Completed,    // 已完成
    Failed(String), // 失败
}

/// 任务信息
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub progress: TaskProgress,
    pub created_at: Instant,
    pub started_at: Option<Instant>,
    pub finished_at: Option<Instant>,
}

impl TaskInfo {
    pub fn new(id: String, name: String, task_type: TaskType) -> Self {
        Self {
            id,
            name,
            task_type,
            status: TaskStatus::Pending,
            progress: TaskProgress::new(),
            created_at: Instant::now(),
            started_at: None,
            finished_at: None,
        }
    }
}

/// 任务进度
#[derive(Debug, Clone, Default)]
pub struct TaskProgress {
    pub total: u64,           // 总数
    pub completed: u64,        // 已完成
    pub current_speed: f64,   // 当前速度 (MB/s)
    pub total_bytes: u64,     // 总字节数
    pub downloaded_bytes: u64, // 已下载字节数
}

impl TaskProgress {
    pub fn new() -> Self {
        Self {
            total: 0,
            completed: 0,
            current_speed: 0.0,
            total_bytes: 0,
            downloaded_bytes: 0,
        }
    }
    
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }
}

/// 任务类型
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    DownloadClient,     // 下载客户端
    DownloadAssets,    // 下载资源文件
    DownloadLibraries, // 下载依赖库
    CheckAssets,       // 检查资源文件
    InstallForge,      // 安装 Forge
    InstallOptiFine,   // 安装 OptiFine
    InstallFabric,    // 安装 Fabric
    InstallNeoForge,  // 安装 NeoForge
    InstallLiteLoader,// 安装 LiteLoader
    Custom(String),   // 自定义任务
}

impl TaskType {
    pub fn display_name(&self) -> &str {
        match self {
            TaskType::DownloadClient => "下载客户端",
            TaskType::DownloadAssets => "下载资源文件",
            TaskType::DownloadLibraries => "下载依赖库",
            TaskType::CheckAssets => "检查资源文件",
            TaskType::InstallForge => "安装 Forge",
            TaskType::InstallOptiFine => "安装 OptiFine",
            TaskType::InstallFabric => "安装 Fabric",
            TaskType::InstallNeoForge => "安装 NeoForge",
            TaskType::InstallLiteLoader => "安装 LiteLoader",
            TaskType::Custom(name) => name,
        }
    }
}

/// 任务 Trait - 所有任务必须实现此接口
#[async_trait::async_trait]
pub trait Task: Send + Sync {
    /// 获取任务类型
    fn task_type(&self) -> TaskType;
    
    /// 获取任务名称
    fn name(&self) -> &str;
    
    /// 执行任务
    /// 返回: (成功, 错误信息)
    async fn execute(
        &self, 
        task_id: &str,
        progress_tx: mpsc::Sender<TaskProgressUpdate>,
    ) -> Result<(), String>;
    
    /// 获取任务描述（可选）
    fn description(&self) -> Option<&str> {
        None
    }
}

/// 任务进度更新
#[derive(Debug, Clone)]
pub struct TaskProgressUpdate {
    pub task_id: String,
    pub progress: TaskProgress,
    pub status: TaskStatus,
}

/// 任务包装器 - 用于存储动态任务
pub struct TaskWrapper {
    pub task: Arc<dyn Task>,
}

impl Clone for TaskWrapper {
    fn clone(&self) -> Self {
        Self {
            task: Arc::clone(&self.task),
        }
    }
}

/// 任务管理器
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<DashMap<String, TaskInfo>>,
    task_wrappers: Arc<RwLock<HashMap<String, TaskWrapper>>>,
    progress_tx: mpsc::Sender<TaskProgressUpdate>,
    speed_tx: mpsc::Sender<(String, u64)>,
}

impl TaskManager {
    /// 创建任务管理器
    pub fn new() -> Self {
        let (progress_tx, mut progress_rx) = mpsc::channel::<TaskProgressUpdate>(100);
        let (speed_tx, mut speed_rx) = mpsc::channel::<(String, u64)>(1000);
        
        let tasks: Arc<DashMap<String, TaskInfo>> = Arc::new(DashMap::new());
        let task_wrappers: Arc<RwLock<HashMap<String, TaskWrapper>>> = Arc::new(RwLock::new(HashMap::new()));
        
        // 启动速度统计任务
        let tasks_clone = tasks.clone();
        tokio::spawn(async move {
            use tokio::time::{interval, Duration};
            
            let mut last_update = Instant::now();
            let mut bytes_accumulated: HashMap<String, u64> = HashMap::new();
            let mut ticker = interval(Duration::from_millis(500));
            
            loop {
                ticker.tick().await;
                
                // 收集所有下载字节数
                while let Ok((task_id, bytes)) = speed_rx.try_recv() {
                    *bytes_accumulated.entry(task_id).or_insert(0) += bytes;
                }
                
                // 每秒钟计算一次速度
                let now = Instant::now();
                let elapsed = now.duration_since(last_update).as_secs_f64();
                
                if elapsed >= 1.0 {
                    for (task_id, bytes) in bytes_accumulated.drain() {
                        if let Some(mut task) = tasks_clone.get_mut(&task_id) {
                            task.progress.current_speed = (bytes as f64) / (1024.0 * 1024.0) / elapsed;
                            task.progress.downloaded_bytes += bytes;
                        }
                    }
                    last_update = now;
                }
            }
        });
        
        // 启动进度更新处理任务
        let tasks_clone2 = tasks.clone();
        tokio::spawn(async move {
            while let Some(update) = progress_rx.recv().await {
                if let Some(mut task) = tasks_clone2.get_mut(&update.task_id) {
                    task.progress = update.progress;
                    task.status = update.status.clone();
                    
                    if update.status == TaskStatus::Completed || matches!(&update.status, TaskStatus::Failed(_)) {
                        task.finished_at = Some(Instant::now());
                    }
                }
            }
        });
        
        Self {
            tasks,
            task_wrappers,
            progress_tx,
            speed_tx,
        }
    }
    
    /// 添加新任务（不立即开始）
    pub async fn append_task<T: Task + 'static>(&self, task: T) -> String {
        let id = format!("{}-{}", task.task_type().display_name(), &uuid::Uuid::new_v4().to_string()[..8]);
        
        let task_info = TaskInfo::new(
            id.clone(),
            task.name().to_string(),
            task.task_type(),
        );
        
        self.tasks.insert(id.clone(), task_info);
        
        // 存储任务包装器
        let wrapper = TaskWrapper {
            task: Arc::new(task),
        };
        self.task_wrappers.write().await.insert(id.clone(), wrapper);
        
        id
    }
    
    /// 开始任务
    pub async fn start_task(&self, task_id: &str) -> Result<(), String> {
        // 获取任务信息
        let task_info = self.tasks.get(task_id).ok_or("任务不存在")?.clone();
        
        // 检查任务状态
        if task_info.status != TaskStatus::Pending {
            return Err("任务不是等待状态".to_string());
        }
        
        // 更新状态为运行中
        {
            let mut task = self.tasks.get_mut(task_id).unwrap();
            task.status = TaskStatus::Running;
            task.started_at = Some(Instant::now());
        }
        
        // 获取任务并执行
        let wrapper = {
            let wrappers = self.task_wrappers.read().await;
            wrappers.get(task_id).cloned().ok_or("任务不存在")?
        };
        
        let progress_tx = self.progress_tx.clone();
        let task_id = task_id.to_string();
        let tasks = self.tasks.clone();
        
        // 在后台执行任务
        tokio::spawn(async move {
            let result = wrapper.task.execute(&task_id, progress_tx.clone()).await;
            
            // 更新最终状态
            if let Some(mut task) = tasks.get_mut(&task_id) {
                match result {
                    Ok(()) => {
                        task.status = TaskStatus::Completed;
                        task.progress.completed = task.progress.total;
                    }
                    Err(e) => {
                        task.status = TaskStatus::Failed(e);
                    }
                }
                task.finished_at = Some(Instant::now());
            }
        });
        
        Ok(())
    }
    
    /// 获取任务信息
    pub fn get_task_info(&self, task_id: &str) -> Option<TaskInfo> {
        self.tasks.get(task_id).map(|t| t.clone())
    }
    
    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<TaskInfo> {
        self.tasks.iter().map(|t| t.clone()).collect()
    }
    
    /// 获取速度统计发送器
    pub fn get_speed_sender(&self) -> mpsc::Sender<(String, u64)> {
        self.speed_tx.clone()
    }
    
    /// 更新任务进度
    pub fn update_progress(&self, task_id: &str, progress: TaskProgress, status: TaskStatus) {
        if let Some(mut task) = self.tasks.get_mut(task_id) {
            task.progress = progress;
            task.status = status.clone();
            
            if status == TaskStatus::Completed || matches!(&status, TaskStatus::Failed(_)) {
                task.finished_at = Some(Instant::now());
            }
        }
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

