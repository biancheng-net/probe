#!/bin/bash
# 文件名: install-probe.sh
# 用途: 下载探针程序，安装并启动 systemd 服务，支持参数传递

# 默认参数（可通过命令行覆盖）
API_HOST=""
TOKEN=""
NODE_NAME="未命名服务器"
LOG_LEVEL="warn"

# 解析命令行参数
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --api-host=*) API_HOST="${1#*=}" ;;
        --token=*) TOKEN="${1#*=}" ;;
        --node-name=*) NODE_NAME="${1#*=}" ;;
        *) echo "未知参数: $1"; exit 1 ;;
    esac
    shift
done

# 验证必要参数
if [ -z "$API_HOST" ] || [ -z "$TOKEN" ] || [ -z "$NODE_NAME" ]; then
    echo "错误: 必须提供 --api-host、--token 和 --node-name 参数"
    exit 1
fi

# 定义路径
PROBE_DIR="/probe"
PROBE_BIN="$PROBE_DIR/probe"
SERVICE_FILE="/etc/systemd/system/probe.service"

# 创建目录
echo "创建目录 $PROBE_DIR ..."
sudo mkdir -p "$PROBE_DIR"

# 下载探针程序
echo "下载探针程序到 $PROBE_BIN ..."
sudo wget -O "$PROBE_BIN" https://github.com/biancheng-net/probe/releases/download/v0.8/probe-linux-musl
if [ $? -ne 0 ]; then
    echo "错误: 下载探针程序失败"
    exit 1
fi

# 赋予探针执行权限
sudo chmod +x "$PROBE_BIN"

# 创建 systemd 服务文件
echo "创建 systemd 服务文件 $SERVICE_FILE ..."
cat << EOF | sudo tee "$SERVICE_FILE" > /dev/null
[Unit]
Description=Probe Service
After=network.target

[Service]
ExecStart=$PROBE_BIN --token=$TOKEN --api-host=$API_HOST --log-level=$LOG_LEVEL --node-name=$NODE_NAME
Restart=always
RestartSec=10
User=nobody
Group=nogroup
WorkingDirectory=$PROBE_DIR
Environment="PATH=/usr/local/bin:/usr/bin:/bin"

[Install]
WantedBy=multi-user.target
EOF

# 重新加载 systemd 配置
echo "重新加载 systemd 配置..."
sudo systemctl daemon-reload

# 启用服务
echo "启用 probe 服务..."
sudo systemctl enable probe.service

# 启动服务
echo "启动 probe 服务..."
sudo systemctl start probe.service

# 检查服务状态
echo "检查 probe 服务状态..."
sudo systemctl status probe.service --no-pager

echo "安装完成！可以通过以下命令查看服务日志："
echo "sudo journalctl -u probe.service -f"
