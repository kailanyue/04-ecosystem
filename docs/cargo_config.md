## RsProxy.cn

[A high speed crates.io mirror](https://rsproxy.cn/)

## 提升 cargo 编译速度
使用 `rustflags` 传递自定义标志给 Cargo 的 rustc 编译器

## sccache
[Shared Compilation Cache](https://github.com/mozilla/sccache)

SCCache 是一个类似 ccache 的编译器缓存工具。它用作编译器包装器，并尽可能避免编译，将缓存的结果存储在本地磁盘或多个云存储后端之一中。
```sh
cargo install sccache
```

### 修改 Cache location

```sh
# linux
export SCCACHE_DIR=/my/new/cache/location

# windows 通过用户环境变量设置 SCCACHE_DIR
set SCCACHE_DIR=C:\my\new\cache\location

# 重启服务
sccache --stop-server
sccache --start-server

# 查看当前状态，确认配置已经生效
sccache --show-stats
sccache -s
```

### 修改 Cache Size
```sh
# linux
export SCCACHE_CACHE_SIZE="20G"

# windows 通过用户环境变量设置 SCCACHE_CACHE_SIZE
set SCCACHE_CACHE_SIZE="20G"

# 重启服务
```

## rustflags

rustflags 是一个环境变量，用于传递自定义标志给 Cargo 的 rustc 编译器。通过设置 rustflags，你可以影响编译过程中的一些行为。
```toml
#以下两种配置方式效果相同
rustflags = ["-C", "target-cpu=native"]
rustflags = ["-Ctarget-cpu=native"]
```


## cargo config
```toml
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
git-fetch-with-cli = true

[build]
target-dir = "D:\\Document\\cargo\\target"
rustc-wrapper = ".cargo\\bin\\sccache.exe"
rustflags = ["-Ctarget-cpu=native"]
jobs = 24

[target.x86_64-pc-windows-msvc]
rustflags = ["-Ctarget-feature=+crt-static"]
```
