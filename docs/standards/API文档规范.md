# API 文档规范

> Parent: [README.md](README.md)

## 适用范围

用于 HTTP 接口、Lua API、Bridge 协议和接口变更记录。

## 写作规则

- 说明稳定语义、调用边界和阅读顺序。
- 不在每个端点下重复通用规则。
- 请求和响应示例必须能对应实现。
- 接口行为变化时，同步接口文档、客户端文档和变更记录。

## 真相源

HTTP 行为以 `mls-sim-rs/src/api/` 为准。Lua API 行为以房间运行时注入实现和 `参考/mls-master/` 为准。
