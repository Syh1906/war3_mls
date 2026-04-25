# README 索引规范

> Parent: [README.md](README.md)

## 适用范围

本规范适用于 `docs/README.md` 和 `docs/**/README.md`。

## 固定结构

导航型 README 使用以下结构：

```markdown
# 标题

一句话说明目录职责。

## 给 LLM 的快速索引

| 任务 | 入口 |
| --- | --- |

## 给人类的导航

| 文件或目录 | 作用 |
| --- | --- |
```

## 维护要求

- README 只做导航，不承载大段正文。
- 新增、删除、迁移子文档时，同步更新对应 README。
- 链接使用相对路径。
- 子目录 README 需要链接父级 README。
