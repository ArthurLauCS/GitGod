# 性能复测（2026-10-04）

## 现有优化确认

`GitGod-bench` 中包含完整 Linux 仓库、额外测试分支、界面验证截图和开发脚本；对应的运行时代码已经在本项目中，并非另一份待合并的引擎。

已实现的机制包括：gix 进程内读取、首批 2000 条与后台完整加载、只保存 id 和父子关系的提交图骨架、每 1024 行的泳道检查点、按窗口获取作者/标题、Canvas 连线与虚拟滚动，以及异步获取各分支同步状态。

本次没有重写这些机制，而是修复两个仍存在的成本：不必要的全图重建，以及超大差异在判断上限前完整进入应用内存。

## 环境与大仓库基线

- Windows 11 10.0.26200，Intel Core i5-12400F，约 32 GiB 内存。
- Git 2.55.0.windows.5，Rust release 构建。
- `GitGod-bench/linux`，HEAD `6812ce4e4379`，非浅克隆。
- `git rev-list --all --count`：1,483,973 条提交；本地及远程分支引用共 1003 个。
- 1000 个 `bench/*` 分支没有 upstream；同步状态测量不代表 1000 个已跟踪且高度分叉的分支。
- 测量基线为 `2560d3c` 的引擎；基准增加了 `tracking` 项。使用已有本地对象和系统文件缓存，不代表冷磁盘或网络克隆耗时。
- Criterion：10 个样本，预热 1 秒，目标测量 2 秒；慢项目自动延长收集时间。

| 项目 | 耗时估计 | 95% 区间 |
| --- | ---: | ---: |
| 首批 2000 条提交图 | 71.26 ms | 69.46–73.16 ms |
| 30 万条提交图 | 504.75 ms | 495.77–515.60 ms |
| 完整提交图 | 2.337 s | 2.313–2.361 s |
| 检查点附近 60 行 | 0.652 ms | 0.645–0.661 ms |
| 引用列表 | 157.50 ms | 154.37–162.18 ms |
| 分支同步状态 | 133.16 ms | 128.05–139.51 ms |
| 工作区状态 | 132.93 ms | 127.92–138.21 ms |
| 提交详情 | 38.25 ms | 37.44–39.04 ms |

复现：

```powershell
$env:GITGOD_BENCH_REPO = 'C:\path\to\GitGod-bench\linux'
cargo bench -p engine --bench read -- --sample-size 10 --warm-up-time 1 --measurement-time 2
```

该基准的固定窗口位于第 15 万条之后，需要至少 30 万条提交的仓库。

## 本次优化

### 复用已有提交图

之前按“引用名称 + id + HEAD”判断是否重建；重命名引用、切换已有分支也会失效。现在比较去重、排序后的提交起点集合；只有该集合改变才重建。HEAD 改变时仍更新选中提交。

这避免了上述操作触发本机约 2.34 秒的完整图构建；这不是新的图构建速度，也不是整个分支切换操作的端到端时间。回归检查覆盖切换分支、引用改名/重排/别名，以及必须失效的新提交、删除唯一引用和新 detached HEAD。

### 限制差异读取

未跟踪文件最多读取 4 MiB + 1 字节。工作区、暂存区、历史提交的 Git 补丁输出使用相同上限；超限时终止并回收 Git 进程，同时排空 stderr，避免管道互相等待。按行操作也拒绝超限补丁，防止文件变化后继续解析旧操作对应的超大补丁。

输入为恰好 128 MiB 的 ASCII 文本（1,048,576 行，每行 128 字节）。同一文件先测未跟踪读取，再暂存并测新增文件差异。两版各预热一次，再交替运行 5 次；以下为中位数。

| 路径 | 优化前耗时 | 优化后耗时 | 优化前峰值内存 | 优化后峰值内存 |
| --- | ---: | ---: | ---: | ---: |
| 未跟踪文件 | 52.60 ms | 3.93 ms | 133.81 MiB | 14.81 MiB |
| 暂存区差异 | 638.50 ms | 410.87 ms | 262.80 MiB | 14.88 MiB |

内存是独立引擎探针进程的 `PeakWorkingSet64`，**不包含 WebView 或 Git 子进程**；耗时包含相应的 Git 调用。两版均返回 `too_large=true`，没有改变超限文件的展示语义。4 MiB + 1 是保留的原始字节数上限，不是整个进程的内存上限。

复现方法：在基线 `2560d3c` 的引擎及当前引擎中放入同一份 `crates/engine/examples/diff_probe.rs`，分别构建并保存为 `before.exe` / `after.exe`，然后运行：

```powershell
cargo build --release -p engine --example diff_probe
# 在两份代码中各构建一次，并分别保留生成的 diff_probe.exe。
./scripts/measure-diff.ps1 -Before C:\bench\before.exe -After C:\bench\after.exe -Repo C:\bench\repo -File large.txt -Mode untracked
git -C C:\bench\repo add large.txt
./scripts/measure-diff.ps1 -Before C:\bench\before.exe -After C:\bench\after.exe -Repo C:\bench\repo -File large.txt -Mode staged
```

## 保留的边界

- 完整历史仍在后台一次性构建，引用起点确实改变时仍需重建。
- Git 在输出补丁前仍可能读取或计算整个文件；本次不承诺 Git 子进程内存有固定上限。
- 提交图显示轨道仍受可用宽度和 24 条上限约束。
- 多工作树的状态收集会启动多个 Git 进程；当前数据不支持宣称已经解决大量脏工作树或大量分叉 upstream 的所有性能问题。
