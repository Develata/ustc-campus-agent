# USTC Simple Calendar

`ustc.simple-calendar` 是一个刻意保持最小的 Rust Calendar Plugin：

- 记录一条事项，可附 RFC 3339 时间；
- 列出当前事项；
- 按稳定 `calendar:item:N` ID 删除事项；
- 最多保留 128 条，使用 owner-local JSON 原子持久化；
- 无提醒、重复规则、共享、外部同步或自然语言日期解释。

比赛用 loopback profile 将它作为可选 Market package 的内置演示组件。正式安装、grant、跨设备同步与通知调度不在当前实现声明内。
