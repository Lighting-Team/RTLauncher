pub mod delay_test {
    use futures::{stream, StreamExt};
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::net::{SocketAddr, ToSocketAddrs};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::net::TcpStream;
    use tokio::sync::Semaphore;

    /// TCP Ping测试结果
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PingResult {
        /// 原始目标字符串
        pub target: String,
        /// 解析后的主机名或IP地址
        pub host: String,
        /// 端口号
        pub port: u16,
        /// 测试是否成功
        pub success: bool,
        /// 延迟（毫秒），成功时有值
        pub latency_ms: Option<f64>,
        /// 错误信息，失败时有值
        pub error: Option<String>,
        /// 测试时间戳
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }

    /// TCP Ping统计信息
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PingStats {
        /// 总测试次数
        pub total: usize,
        /// 成功次数
        pub successful: usize,
        /// 失败次数
        pub failed: usize,
        /// 平均延迟（毫秒）
        pub avg_latency_ms: Option<f64>,
        /// 最小延迟（毫秒）
        pub min_latency_ms: Option<f64>,
        /// 最大延迟（毫秒）
        pub max_latency_ms: Option<f64>,
        /// 中位数延迟（毫秒）
        pub median_latency_ms: Option<f64>,
        /// 成功率（百分比）
        pub success_rate: f64,
        /// 所有测试结果的详细列表
        pub results: Vec<PingResult>,
    }

    /// TCP Ping配置
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TcpPingConfig {
        /// 连接超时时间
        pub timeout: Duration,
        /// 最大并发连接数
        pub concurrency_limit: usize,
        /// 失败重试次数
        pub retry_count: usize,
        /// 重试间隔
        pub retry_delay: Duration,
        /// 默认端口号（当目标中未指定端口时使用）
        pub default_port: u16,
    }

    impl Default for TcpPingConfig {
        fn default() -> Self {
            Self {
                timeout: Duration::from_secs(5),
                concurrency_limit: 100,
                retry_count: 2,
                retry_delay: Duration::from_millis(100),
                default_port: 80,
            }
        }
    }

    /// TCP Ping测试器
    #[derive(Debug, Clone)]
    pub struct DelayTester {
        config: TcpPingConfig,
    }

    impl DelayTester {
        /// 使用自定义配置创建测试器
        pub fn new(config: TcpPingConfig) -> Self {
            Self { config }
        }

        /// 使用默认配置创建测试器
        pub fn with_default_config() -> Self {
            Self {
                config: TcpPingConfig::default(),
            }
        }

        /// 获取当前配置
        pub fn config(&self) -> &TcpPingConfig {
            &self.config
        }

        /// 更新配置
        pub fn update_config(&mut self, config: TcpPingConfig) {
            self.config = config;
        }

        /// 解析目标地址字符串
        ///
        /// # 支持的格式
        /// - `example.com:443` - 带端口的域名
        /// - `192.168.1.1:80` - 带端口的IP地址
        /// - `example.com` - 仅域名（使用默认端口）
        /// - `192.168.1.1` - 仅IP地址（使用默认端口）
        ///
        /// # 参数
        /// - `target`: 目标地址字符串
        /// - `default_port`: 默认端口号
        ///
        /// # 返回
        /// (主机名或IP地址, 端口号)
        pub fn parse_target(target: &str, default_port: u16) -> (String, u16) {
            let target = target.trim();

            // 处理IPv6地址
            if target.contains('[') && target.contains(']') {
                if let Some(port_start) = target.rfind(':') {
                    if port_start > target.rfind(']').unwrap() {
                        let host = target[..port_start].to_string();
                        let port = target[port_start + 1..].parse().unwrap_or(default_port);
                        return (host, port);
                    }
                }
                return (target.to_string(), default_port);
            }

            let parts: Vec<&str> = target.rsplitn(2, ':').collect();
            match parts.len() {
                1 => (parts[0].to_string(), default_port),
                2 => {
                    let host = parts[1].to_string();
                    match parts[0].parse::<u16>() {
                        Ok(port) => (host, port),
                        Err(_) => (target.to_string(), default_port),
                    }
                }
                _ => (target.to_string(), default_port),
            }
        }

        /// 执行单个TCP Ping测试
        ///
        /// # 参数
        /// - `target`: 目标地址字符串
        ///
        /// # 返回
        /// 测试结果
        pub async fn ping_single(&self, target: &str) -> PingResult {
            let (host, port) = Self::parse_target(target, self.config.default_port);
            let target_str = format!("{}:{}", host, port);

            let mut last_error = None;

            // 重试机制
            for attempt in 0..=self.config.retry_count {
                if attempt > 0 {
                    tokio::time::sleep(self.config.retry_delay).await;
                }

                match self.do_ping(&host, port).await {
                    Ok(latency) => {
                        return PingResult {
                            target: target.to_string(),
                            host: host.clone(),
                            port,
                            success: true,
                            latency_ms: Some(latency),
                            error: None,
                            timestamp: chrono::Utc::now(),
                        };
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                        if attempt < self.config.retry_count {
                            continue;
                        }
                    }
                }
            }

            PingResult {
                target: target.to_string(),
                host: host.clone(),
                port,
                success: false,
                latency_ms: None,
                error: last_error,
                timestamp: chrono::Utc::now(),
            }
        }

        /// 执行实际的TCP连接测试
        async fn do_ping(
            &self,
            host: &str,
            port: u16,
        ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
            let addr_string = format!("{}:{}", host, port);

            // 解析DNS
            let socket_addr = tokio::net::lookup_host(&addr_string)
                .await?
                .next()
                .ok_or_else(|| format!("无法解析地址: {}", addr_string))?;

            let start = Instant::now();

            // 使用tokio::time::timeout设置超时
            let stream = tokio::time::timeout(self.config.timeout, TcpStream::connect(socket_addr))
                .await??;

            let latency = start.elapsed();

            // 立即关闭连接
            drop(stream);

            Ok(latency.as_secs_f64() * 1000.0) // 转换为毫秒
        }

        /// 批量执行TCP Ping测试
        ///
        /// # 参数
        /// - `targets`: 目标地址字符串列表
        ///
        /// # 返回
        /// 统计信息和所有结果
        pub async fn ping_batch(&self, targets: Vec<String>) -> PingStats {
            let semaphore = Arc::new(Semaphore::new(self.config.concurrency_limit));
            let config = self.config.clone();

            // 创建异步任务流
            let tasks = stream::iter(targets)
                .map(move |target| {
                    let semaphore = Arc::clone(&semaphore);
                    let config = config.clone();

                    async move {
                        let _permit = semaphore.acquire().await.unwrap();

                        let (host, port) = DelayTester::parse_target(&target, config.default_port);
                        let mut last_error = None;

                        for attempt in 0..=config.retry_count {
                            if attempt > 0 {
                                tokio::time::sleep(config.retry_delay).await;
                            }

                            match Self::static_do_ping(&host, port, config.timeout).await {
                                Ok(latency) => {
                                    return PingResult {
                                        target: target.clone(),
                                        host: host.clone(),
                                        port,
                                        success: true,
                                        latency_ms: Some(latency),
                                        error: None,
                                        timestamp: chrono::Utc::now(),
                                    };
                                }
                                Err(e) => {
                                    last_error = Some(e.to_string());
                                    if attempt < config.retry_count {
                                        continue;
                                    }
                                }
                            }
                        }

                        PingResult {
                            target: target.clone(),
                            host: host.clone(),
                            port,
                            success: false,
                            latency_ms: None,
                            error: last_error,
                            timestamp: chrono::Utc::now(),
                        }
                    }
                })
                .buffer_unordered(self.config.concurrency_limit);

            let results: Vec<PingResult> = tasks.collect().await;

            self.calculate_stats(results)
        }

        /// 从文件读取目标列表并执行测试
        ///
        /// # 参数
        /// - `file_path`: 文件路径，每行一个目标地址
        ///
        /// # 返回
        /// 统计信息和所有结果
        pub async fn ping_from_file(&self, file_path: &str) -> std::io::Result<PingStats> {
            let content = std::fs::read_to_string(file_path)?;
            let targets: Vec<String> = content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .map(|line| line.trim().to_string())
                .collect();

            Ok(self.ping_batch(targets).await)
        }

        /// 静态方法用于执行ping
        async fn static_do_ping(
            host: &str,
            port: u16,
            timeout: Duration,
        ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
            let addr_string = format!("{}:{}", host, port);

            let socket_addr = tokio::net::lookup_host(&addr_string)
                .await?
                .next()
                .ok_or_else(|| format!("无法解析地址: {}", addr_string))?;

            let start = Instant::now();

            let stream = tokio::time::timeout(timeout, TcpStream::connect(socket_addr)).await??;

            let latency = start.elapsed();
            drop(stream);

            Ok(latency.as_secs_f64() * 1000.0)
        }

        /// 计算统计数据
        fn calculate_stats(&self, results: Vec<PingResult>) -> PingStats {
            let total = results.len();
            let successful_results: Vec<&PingResult> =
                results.iter().filter(|r| r.success).collect();
            let successful = successful_results.len();
            let failed = total - successful;
            let success_rate = if total > 0 {
                successful as f64 / total as f64 * 100.0
            } else {
                0.0
            };

            let latencies: Vec<f64> = successful_results
                .iter()
                .filter_map(|r| r.latency_ms)
                .collect();

            let (avg_latency, min_latency, max_latency, median_latency) = if !latencies.is_empty() {
                let mut sorted = latencies.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

                let sum: f64 = latencies.iter().sum();
                let avg = sum / latencies.len() as f64;
                let min = *sorted.first().unwrap();
                let max = *sorted.last().unwrap();
                let median = if latencies.len() % 2 == 0 {
                    let mid = latencies.len() / 2;
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted[latencies.len() / 2]
                };

                (Some(avg), Some(min), Some(max), Some(median))
            } else {
                (None, None, None, None)
            };

            PingStats {
                total,
                successful,
                failed,
                avg_latency_ms: avg_latency,
                min_latency_ms: min_latency,
                max_latency_ms: max_latency,
                median_latency_ms: median_latency,
                success_rate,
                results,
            }
        }

        /// 按主机分组统计结果
        pub fn group_by_host(&self, stats: &PingStats) -> HashMap<String, Vec<&PingResult>> {
            let mut groups = HashMap::new();

            for result in &stats.results {
                groups
                    .entry(result.host.clone())
                    .or_insert_with(Vec::new)
                    .push(result);
            }

            groups
        }

        /// 按成功率排序结果
        pub fn sort_by_latency(&self, stats: &mut PingStats, ascending: bool) {
            stats.results.sort_by(|a, b| {
                let a_lat = a.latency_ms.unwrap_or(f64::INFINITY);
                let b_lat = b.latency_ms.unwrap_or(f64::INFINITY);

                if ascending {
                    a_lat.partial_cmp(&b_lat).unwrap()
                } else {
                    b_lat.partial_cmp(&a_lat).unwrap()
                }
            });
        }

        /// 过滤结果
        pub fn filter_results(&self, stats: &PingStats, success_only: bool) -> Vec<&PingResult> {
            if success_only {
                stats.results.iter().filter(|r| r.success).collect()
            } else {
                stats.results.iter().collect()
            }
        }

        /// 生成文本格式报告
        pub fn generate_text_report(&self, stats: &PingStats) -> String {
            let mut report = String::new();

            report.push_str("TCP Ping 测试报告\n");
            report.push_str("================\n\n");
            report.push_str(&format!(
                "测试时间: {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            ));
            report.push_str(&format!(
                "配置: 超时={:?}, 并发数={}, 重试次数={}\n",
                self.config.timeout, self.config.concurrency_limit, self.config.retry_count
            ));
            report.push_str(&format!("测试总数: {}\n", stats.total));
            report.push_str(&format!(
                "成功: {} ({:.1}%)\n",
                stats.successful, stats.success_rate
            ));
            report.push_str(&format!("失败: {}\n\n", stats.failed));

            if let Some(avg) = stats.avg_latency_ms {
                report.push_str(&format!("平均延迟: {:.2} ms\n", avg));
            }
            if let Some(min) = stats.min_latency_ms {
                report.push_str(&format!("最小延迟: {:.2} ms\n", min));
            }
            if let Some(max) = stats.max_latency_ms {
                report.push_str(&format!("最大延迟: {:.2} ms\n", max));
            }
            if let Some(median) = stats.median_latency_ms {
                report.push_str(&format!("中位数延迟: {:.2} ms\n\n", median));
            }

            report.push_str("详细结果:\n");
            report.push_str("--------\n");

            for (i, result) in stats.results.iter().enumerate() {
                report.push_str(&format!("{}. {}:{} - ", i + 1, result.host, result.port));
                if result.success {
                    report.push_str(&format!("✅ {:.2} ms", result.latency_ms.unwrap_or(0.0)));
                } else {
                    report.push_str(&format!(
                        "❌ 失败: {}",
                        result.error.as_deref().unwrap_or("未知错误")
                    ));
                }
                report.push('\n');
            }

            report
        }

        /// 生成Markdown格式报告
        pub fn generate_markdown_report(&self, stats: &PingStats) -> String {
            let mut report = String::new();

            report.push_str("# TCP Ping 测试报告\n\n");
            report.push_str(&format!(
                "**测试时间**: {}\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            ));
            report.push_str(&format!(
                "**配置**: 超时={:?}, 并发数={}, 重试次数={}\n\n",
                self.config.timeout, self.config.concurrency_limit, self.config.retry_count
            ));

            report.push_str("## 统计概览\n\n");
            report.push_str("| 指标 | 值 |\n");
            report.push_str("|------|----|\n");
            report.push_str(&format!("| 测试总数 | {} |\n", stats.total));
            report.push_str(&format!("| 成功 | {} |\n", stats.successful));
            report.push_str(&format!("| 失败 | {} |\n", stats.failed));
            report.push_str(&format!("| 成功率 | {:.1}% |\n", stats.success_rate));

            if let Some(avg) = stats.avg_latency_ms {
                report.push_str(&format!("| 平均延迟 | {:.2} ms |\n", avg));
            }
            if let Some(min) = stats.min_latency_ms {
                report.push_str(&format!("| 最小延迟 | {:.2} ms |\n", min));
            }
            if let Some(max) = stats.max_latency_ms {
                report.push_str(&format!("| 最大延迟 | {:.2} ms |\n", max));
            }
            if let Some(median) = stats.median_latency_ms {
                report.push_str(&format!("| 中位数延迟 | {:.2} ms |\n\n", median));
            }

            report.push_str("## 详细结果\n\n");
            report.push_str("| # | 目标 | 主机:端口 | 状态 | 延迟(ms) | 错误信息 |\n");
            report.push_str("|---|------|-----------|------|----------|----------|\n");

            for (i, result) in stats.results.iter().enumerate() {
                let status = if result.success {
                    "✅ 成功"
                } else {
                    "❌ 失败"
                };
                let latency = result
                    .latency_ms
                    .map_or("N/A".to_string(), |l| format!("{:.2}", l));
                let error = result.error.as_deref().unwrap_or("");

                report.push_str(&format!(
                    "| {} | {} | {}:{} | {} | {} | {} |\n",
                    i + 1,
                    result.target,
                    result.host,
                    result.port,
                    status,
                    latency,
                    error
                ));
            }

            report
        }

        /// 保存结果为JSON文件
        pub fn save_as_json(&self, stats: &PingStats, file_path: &str) -> std::io::Result<()> {
            let json = serde_json::to_string_pretty(stats)?;
            std::fs::write(file_path, json)
        }

        /// 保存结果为CSV文件
        pub fn save_as_csv(&self, stats: &PingStats, file_path: &str) -> std::io::Result<()> {
            let mut wtr = csv::Writer::from_path(file_path)?;

            wtr.write_record(&[
                "目标",
                "主机",
                "端口",
                "成功",
                "延迟(ms)",
                "错误信息",
                "时间戳",
            ])?;

            for result in &stats.results {
                let success = if result.success { "是" } else { "否" };
                let latency = result
                    .latency_ms
                    .map_or("".to_string(), |l| format!("{:.2}", l));
                let error = result.error.as_deref().unwrap_or("");
                let timestamp = result.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

                wtr.write_record(&[
                    &result.target,
                    &result.host,
                    &result.port.to_string(),
                    success,
                    &latency,
                    error,
                    &timestamp,
                ])?;
            }

            wtr.flush()?;
            Ok(())
        }
    }

    /// 便捷函数：快速测试单个目标
    pub async fn quick_ping(
        target: &str,
        timeout_secs: u64,
    ) -> Result<PingResult, Box<dyn std::error::Error>> {
        let config = TcpPingConfig {
            timeout: Duration::from_secs(timeout_secs),
            ..Default::default()
        };

        let tester = DelayTester::new(config);
        Ok(tester.ping_single(target).await)
    }

    /// 便捷函数：快速批量测试
    pub async fn quick_batch_ping(targets: Vec<String>, concurrency: usize) -> PingStats {
        let config = TcpPingConfig {
            concurrency_limit: concurrency,
            ..Default::default()
        };

        let tester = DelayTester::new(config);
        tester.ping_batch(targets).await
    }

    // 测试模块
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_target() {
            // 测试带端口的地址
            assert_eq!(
                DelayTester::parse_target("example.com:443", 80),
                ("example.com".to_string(), 443)
            );

            // 测试不带端口的地址
            assert_eq!(
                DelayTester::parse_target("example.com", 80),
                ("example.com".to_string(), 80)
            );

            // 测试IP地址
            assert_eq!(
                DelayTester::parse_target("192.168.1.1:8080", 80),
                ("192.168.1.1".to_string(), 8080)
            );

            // 测试IPv6地址
            assert_eq!(
                DelayTester::parse_target("[2001:db8::1]:443", 80),
                ("[2001:db8::1]".to_string(), 443)
            );

            // 测试空格处理
            assert_eq!(
                DelayTester::parse_target("  example.com:443  ", 80),
                ("example.com".to_string(), 443)
            );
        }

        #[tokio::test]
        async fn test_ping_single() {
            let tester = DelayTester::with_default_config();

            // 测试已知可访问的地址（Google DNS）
            let result = tester.ping_single("8.8.8.8:53").await;

            // 由于网络环境不同，可能成功也可能失败
            // 我们主要测试函数是否正常工作
            assert_eq!(result.port, 53);
            assert_eq!(result.host, "8.8.8.8");

            // 如果是成功的，应该有延迟值
            if result.success {
                assert!(result.latency_ms.is_some());
                assert!(result.error.is_none());
            } else {
                assert!(result.latency_ms.is_none());
                assert!(result.error.is_some());
            }
        }

        #[tokio::test]
        async fn test_ping_batch() {
            let config = TcpPingConfig {
                timeout: Duration::from_secs(3),
                concurrency_limit: 5,
                retry_count: 1,
                retry_delay: Duration::from_millis(50),
                default_port: 80,
            };

            let tester = DelayTester::new(config);

            let targets = vec![
                "google.com:80".to_string(),
                "github.com:443".to_string(),
                "cloudflare.com:443".to_string(),
                "1.1.1.1:53".to_string(),
            ];

            let stats = tester.ping_batch(targets).await;

            // 验证统计数据
            assert_eq!(stats.total, 4);
            assert!(stats.successful + stats.failed == 4);

            // 生成报告
            let text_report = tester.generate_text_report(&stats);
            let markdown_report = tester.generate_markdown_report(&stats);

            assert!(text_report.contains("TCP Ping 测试报告"));
            assert!(markdown_report.contains("# TCP Ping 测试报告"));

            // 测试分组功能
            let groups = tester.group_by_host(&stats);
            assert!(!groups.is_empty());

            // 测试排序
            let mut stats_clone = stats.clone();
            tester.sort_by_latency(&mut stats_clone, true);

            // 测试过滤
            let success_results = tester.filter_results(&stats, true);
            assert_eq!(success_results.len(), stats.successful);
        }

        #[test]
        fn test_stats_calculation() {
            let tester = DelayTester::with_default_config();

            let results = vec![
                PingResult {
                    target: "test1.com".to_string(),
                    host: "test1.com".to_string(),
                    port: 80,
                    success: true,
                    latency_ms: Some(10.0),
                    error: None,
                    timestamp: chrono::Utc::now(),
                },
                PingResult {
                    target: "test2.com".to_string(),
                    host: "test2.com".to_string(),
                    port: 80,
                    success: true,
                    latency_ms: Some(20.0),
                    error: None,
                    timestamp: chrono::Utc::now(),
                },
                PingResult {
                    target: "test3.com".to_string(),
                    host: "test3.com".to_string(),
                    port: 80,
                    success: false,
                    latency_ms: None,
                    error: Some("Connection failed".to_string()),
                    timestamp: chrono::Utc::now(),
                },
            ];

            let stats = tester.calculate_stats(results);

            assert_eq!(stats.total, 3);
            assert_eq!(stats.successful, 2);
            assert_eq!(stats.failed, 1);
            assert_eq!(stats.success_rate, 66.66666666666667);
            assert_eq!(stats.avg_latency_ms, Some(15.0));
            assert_eq!(stats.min_latency_ms, Some(10.0));
            assert_eq!(stats.max_latency_ms, Some(20.0));
            assert_eq!(stats.median_latency_ms, Some(15.0));
        }

        #[tokio::test]
        async fn test_quick_functions() {
            // 测试快速单个ping
            let result = quick_ping("8.8.8.8:53", 3).await;
            assert!(result.is_ok());

            // 测试快速批量ping
            let targets = vec!["google.com:80".to_string(), "github.com:443".to_string()];

            let stats = quick_batch_ping(targets, 10).await;
            assert_eq!(stats.total, 2);
        }
    }
}
