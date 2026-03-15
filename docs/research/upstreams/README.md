# Upstream Intake Workflow

这里保存 OSM 的竞品吸收与上游治理资料。

核心原则：

- `third_party/upstreams/catalog.json` 是唯一权威清单
- `docs/research/upstreams/*.md` 是从 catalog 生成的研究产物
- `docs/release/open-source-attribution.md` 是发布页致谢与许可证姿态的权威来源
- `third_party/upstreams/mirrors/` 只用于本地镜像，不直接进入仓库版本历史

## 如何刷新

```bash
node scripts/intake-upstreams.mjs
```

只想看将会写入什么文件，而不真正落盘：

```bash
node scripts/intake-upstreams.mjs --dry-run
```

## 吸收姿态

- `candidate-absorb`
  - 许可证和工程边界允许继续做代码级评估、干净继承、协议兼容或局部重写
- `reference-only`
  - 只能吸收设计信号、数据模型思路或清单覆盖范围，不能直接复制实现

## 本地镜像规范

脚本会为每个上游仓库计算镜像目录：

```text
third_party/upstreams/mirrors/<owner>-<repo>/
```

镜像目录用于后续手工 `git clone` 或 `git fetch`，不参与发布产物。
