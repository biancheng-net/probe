## 服务器探针客户端


## 用法

根据 https://github.com/biancheng-net/probe-server  配置好服务端，然后在节点运行下面命令即可


### linux上以服务运行（仅测试过debian12）

```
wget -O install-probe.sh https://raw.githubusercontent.com/biancheng-net/probe/refs/heads/main/install.sh && chmod +x install-probe.sh && ./install-probe.sh --api-host=https://服务端地址.com --token=和服务端token一样 --node-name='当前服务器名称，比如 新加坡01'
```

以 linux 为例子，其他系统请下载其他版本，还支持 mac 和 win
```
wget -O probe https://github.com/biancheng-net/probe/releases/download/v0.8/probe-linux-musl && chmod +x probe
nohup ./probe --token=和服务端配置的token一样 --api-host='http://z.ipconfig.la这里改成你的服务器ip和端口' --log-level=warn --node-name='🇺🇸美国-01' > /dev/null 2>&1 &
```


