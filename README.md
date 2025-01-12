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
tar -xzf aquatic_0.9.0.tar.gz && rm aquatic_0.9.0.tar.gz
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
./aquatic_http -c "aquatic-http-config.toml"
./aquatic_udp -c "aquatic-udp-config.toml"
./aquatic_ws -c "aquatic-ws-config.toml"
```

### 创建新配置 / Create New Configuration

如果需要使用自定义配置，可以通过以下命令生成新的配置文件：
If you need to use custom configuration, you can generate new configuration files using:

```bash
./aquatic_http -p > "aquatic-http-config.toml"
./aquatic_udp -p > "aquatic-udp-config.toml"
./aquatic_ws -p > "aquatic-ws-config.toml"
```

## 注意事项 / Notes

- 确保所有文件具有执行权限 / Ensure all files have execution permissions
- 确保配置文件路径正确 / Ensure configuration file paths are correct
- 确保端口未被占用 / Ensure ports are not in use by other applications

### 内存锁定限制设置 / Memory Lock Limits Configuration

如果您计划运行完整的 Tracker 服务（不仅仅是 UDP 服务），请确保系统的内存锁定限制充足。您可以通过在 `/etc/security/limits.conf` 文件中添加以下行来实现（添加后需要重新登录生效）：

If you plan to run the complete Tracker service (not just the UDP service), make sure the system's memory lock limits are sufficient. You can do this by adding the following lines to `/etc/security/limits.conf` and then logging out and back in:

```bash
*    hard    memlock    65536
*    soft    memlock    65536
```

如果您使用 systemd 服务文件，请在服务文件中添加以下配置：
If you're using a systemd service file, add the following configuration:

```bash
LimitMEMLOCK=65536000
```

## 更多信息 / More Information

如果您需要更详细的介绍，可以访问以下链接：
For more detailed information, please visit:

- [@lib.rs/crates/aquatic](https://lib.rs/crates/aquatic)
- [@github.com/greatest-ape/aquatic](https://github.com/greatest-ape/aquatic)
