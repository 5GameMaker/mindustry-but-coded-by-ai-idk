# Rust Mindustry / MDT

计划使用rust将mdt完整迁移，实在是想看看rust mdt性能会不会好一点

## 当前状态

- 迁移基线：`D:\MDT\mindustry-upstream-v157.4`
- Rust 工作区：`D:\MDT\rust-mindustry`。
- 当前工程仍处于迁移中，暂未达到完整可玩状态。

## 迁移进度

- 当前总体完成度：约 **14.9%**。

## 作者的话

计划上是做到rust版和原版游戏联机上的互通，所以理论上最后会做客户和服务端双端，不过JAR模组由于语言差异大概是不会受支持了，非jar理论上是能用的，大概吧，暂时还没法玩，能玩了这段话你们也看不到了
因为完全重写了所以会考虑留一些方便做辅助性功能的接口出来，不过应该不会留可能造成不平衡的，虽然一键装弹什么的已经人手一个了
做这个大概是因为一点执念？所以后继更新我有时间就做，没时间就摆了，还有很多话想说，不过还是留到以后吧

## 验证

```bash
cargo fmt
cargo check --workspace --manifest-path "D:/MDT/rust-mindustry/Cargo.toml"
cargo test --workspace --manifest-path "D:/MDT/rust-mindustry/Cargo.toml"
```
