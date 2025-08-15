## 服务器探针客户端


## 用法

根据 https://github.com/biancheng-net/probe-server  配置好服务端，然后在节点运行下面命令即可

以 linux 为例子，其他系统请下载其他版本，还支持 mac 和 win
```
wget -O probe https://github.com/biancheng-net/probe/releases/download/v0.8/probe-linux-musl && chmod +x probe
nohup ./probe --token=和服务端配置的token一样 --api-host='http://z.ipconfig.la这里改成你的服务器ip和端口' --log-level=warn --node-name='🇺🇸美国-01' > /dev/null 2>&1 &
```

