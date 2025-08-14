use anyhow::Result;
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use sysinfo::{MemoryRefreshKind, Networks, System};

#[derive(Parser, Debug)]
#[command(name = "探针客户端", author, version, about = "简单的探针", long_about = None)]
struct Cli {
    /// 节点的名称，必填参数
    #[arg(short, long)]
    node_name: String,
    /// 提交数据的服务端地址
    #[arg(short, long)]
    api_host: String,
    /// 提交数据到服务端需要用到的 token，用于认证
    #[arg(short, long)]
    token: String,
    /// 运行的秒数，默认值为1
    #[arg(short, long, default_value_t = 1)]
    seconds: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct SystemInfo {
    #[serde(rename = "nn")]
    node_name: String,
    #[serde(rename = "nr")]
    network_received: u64,
    #[serde(rename = "nt")]
    network_transmitted: u64,
    #[serde(rename = "nrs")]
    network_received_speed: u64,
    #[serde(rename = "nts")]
    network_transmitted_speed: u64,
    #[serde(rename = "ntr")]
    network_total_received: u64,
    #[serde(rename = "ntt")]
    network_total_transmitted: u64,
    #[serde(rename = "tm")]
    total_memory: u64,
    #[serde(rename = "um")]
    used_memory: u64,
    #[serde(rename = "ts")]
    total_swap: u64,
    #[serde(rename = "us")]
    used_swap: u64,
    #[serde(rename = "acu")]
    avg_cpu_usage: f32,
    #[serde(rename = "laom")]
    load_avg_one_minute: f64,
    #[serde(rename = "lafm")]
    load_avg_five_minute: f64,
    #[serde(rename = "lafifm")]
    load_avg_fifteen_minute: f64,
}

impl SystemInfo {
    pub fn new(node_name: &str) -> Self {
        SystemInfo {
            node_name: node_name.to_string(),
            ..Default::default()
        }
    }
}

// 新添加的异步函数，用于提交数据
async fn submit_info_to_server(
    client: &Client,
    api_host: &str,
    token: &str,
    info: &SystemInfo,
) -> Result<()> {
    let url = format!("{}/submit", api_host);

    println!("Submitting data to {}...", url);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("x-api-token", token)
        .json(info)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Data submitted successfully. Status: {}", response.status());
    } else {
        eprintln!(
            "Failed to submit data. Status: {}, Body: {:?}",
            response.status(),
            response.text().await?
        );
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 初始化 System 对象并填充所有数据
    let mut system = System::new_all();

    // 检查是否有网络接口
    let mut networks = Networks::new_with_refreshed_list();
    let client = Client::new();
    let delay_seconds = cli.seconds;
    loop {
        // 记录循环开始时间
        let start_time = Instant::now();

        // 存储当前网络数据
        let mut system_info = SystemInfo::new(&cli.node_name);

        // 刷新特定数据：网络、内存、CPU
        system.refresh_specifics(
            sysinfo::RefreshKind::nothing()
                .with_memory(MemoryRefreshKind::everything())
                .with_cpu(sysinfo::CpuRefreshKind::everything()),
        );
        networks.refresh(true);
        // 刷新 CPU 使用率（需要额外调用以确保数据准确）
        system.refresh_cpu_usage();
        // 确保 CPU 数据累积足够时间
        tokio::time::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

        if networks.is_empty() {
            println!("No network interfaces found!");
        } else {
            for (_, data) in &networks {
                // current_data.push((interface_name.clone(), data.received(), data.transmitted()));
                system_info.network_received += data.received();
                system_info.network_transmitted += data.transmitted();
                system_info.network_total_received += data.total_received();
                system_info.network_total_transmitted += data.total_transmitted();
            }

            system_info.network_received_speed = system_info.network_received / delay_seconds;
            system_info.network_transmitted_speed = system_info.network_transmitted / delay_seconds;
        }

        // 内存使用率统计
        system_info.total_memory = system.total_memory();
        system_info.used_memory = system.used_memory();
        system_info.total_swap = system.total_swap();
        system_info.used_swap = system.used_swap();

        // CPU 使用率统计
        let cpus = system.cpus();
        let mut total_cpu_usage = 0.0;
        let cpu_count = cpus.len() as f32;
        for (i, cpu) in cpus.iter().enumerate() {
            let usage = cpu.cpu_usage();
            total_cpu_usage += usage;
        }
        system_info.avg_cpu_usage = if cpu_count > 0.0 {
            total_cpu_usage / cpu_count
        } else {
            0.0
        };

        // 系统负载统计
        if sysinfo::IS_SUPPORTED_SYSTEM {
            let load_avg = System::load_average();
            let cpu_count = system.cpus().len() as f64;
            system_info.load_avg_one_minute = (load_avg.one / cpu_count) * 100.0;
            system_info.load_avg_five_minute = (load_avg.five / cpu_count) * 100.0;
            system_info.load_avg_fifteen_minute = (load_avg.fifteen / cpu_count) * 100.0;
        } else {
            println!("System Load Average: Not supported on this OS");
        }

        // 等待 5 秒
        tokio::time::sleep(
            Duration::from_secs(delay_seconds as u64) - sysinfo::MINIMUM_CPU_UPDATE_INTERVAL,
        )
        .await;

        println!("{:#?}", system_info);
        println!("{:?}", serde_json::to_string(&system_info));

        submit_info_to_server(&client, &cli.api_host, &cli.token, &system_info).await?;

        // 记录循环结束时间并计算耗时
        let end_time = Instant::now();
        let duration = end_time.duration_since(start_time);
        println!("Time elapsed for this iteration: {:.2?}", duration);
    }
}
