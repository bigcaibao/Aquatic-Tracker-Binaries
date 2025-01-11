# Aquatic Tracker 二进制文件 / Aquatic Tracker Binaries

## 简介 / Introduction

这是一个高性能的 BitTorrent Tracker 服务预编译二进制文件包。为了方便使用，做成了开箱即用的二进制文件以及附带了相关配置文件。

This is a pre-compiled binary package of a high-performance BitTorrent Tracker server. For convenience, we provide ready-to-use binary files and configuration files.

## 下载和安装 / Download and Installation

从以下地址获取最新的二进制文件：
Get the latest binary files from:
```bash
https://github.com/bigcaibao/Aquatic-Tracker-Binaries/releases/download/0.9.0/aquatic_0.9.0.tar.gz
```

解压文件并赋予可执行权限：
Extract files and grant executable permissions:

```bash
chmod +x aquatic/*
```

## 运行说明 / Usage Instructions

### 默认端口 / Default Ports

为了便于管理，各服务的默认监听端口如下：
For easier management, the default listening ports are:

- HTTP Tracker: 3000
- UDP Tracker: 3001
- WebSocket Tracker: 3002

### 启动服务 / Start Services

使用预配置的配置文件启动服务：
Start services with pre-configured files:

```bash
./aquatic/aquatic_http -c "aquatic-http-config.toml"
./aquatic/aquatic_udp -c "aquatic-udp-config.toml"
./aquatic/aquatic_ws -c "aquatic-ws-config.toml"
```

### 创建新配置 / Create New Configuration

如果需要使用自定义配置，可以通过以下命令生成新的配置文件：
If you need to use custom configuration, you can generate new configuration files using:

```bash
./aquatic/aquatic_http -p > "aquatic-http-config.toml"
./aquatic/aquatic_udp -p > "aquatic-udp-config.toml"
./aquatic/aquatic_ws -p > "aquatic-ws-config.toml"
```

## 注意事项 / Notes

- 确保所有文件具有执行权限 / Ensure all files have execution permissions
- 确保配置文件路径正确 / Ensure configuration file paths are correct
- 确保端口未被占用 / Ensure ports are not in use by other applications
