# Rust 后端迁移说明

当前服务入口、HTTP 路由、Cookie/请求体处理、静态资源、上游网络请求、代理和进程生命周期均由 Rust 实现，运行时不再需要 Node.js。

为避免一次性重写 169 个接口时改变登录签名、歌词解码、云盘上传等既有行为，尚未迁移的接口处理器会在构建时打包进 Rust 可执行文件，由嵌入式 QuickJS 兼容层执行。`rust-native.json` 中的接口已经由原生 Rust 实现，不会进入兼容包；该清单最终覆盖全部接口后会删除 QuickJS。

## 常用命令

```bash
# 重新生成嵌入式接口包并运行开发版
pnpm dev

# 运行 Rust 测试
cargo test

# 同时启动 Node/Rust 并对照已迁移接口
pnpm compare:native

# 生成 release 可执行文件
pnpm build

# 直接启动已有 release 构建
pnpm start
```

仅修改 Rust 代码时可直接使用 `cargo run`。修改 `module/` 或 `util/` 后必须先运行 `pnpm build:compat`；生成的 `rust-assets/compat.js` 会被 `include_str!` 编译进二进制。
