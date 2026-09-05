# USTC Campus Agent：SSO 接口预留样例

## 对外说明

> 正式接入学校统一身份认证，需要校方接入授权、应用登记及回调地址等配置。当前版本未配置 USTC SSO，使用本机演示用户会话，不收集学校账号密码。项目提供了独立的认证接口样例；后续取得相关授权并完成协议适配、安全验证和会话集成后，可通过接口接入学校统一身份认证。

以上不表示学校已批准或拒绝申请，也不宣称已经完成普通账号的注册、密码登录或真实 SSO。本目录作为源码附带的独立样例保存；不接入 `ustc-agentd` 的运行时路由，也不替换其身份或会话 authority。

## 本次实现的范围

这是可独立启动的 Python 标准库 HTTP 原型，用于核对接口和未配置行为，不是生产身份认证服务。使用 Python 是为了让接口样例无需构建完整 Rust 工作区即可运行；最终集成沿用项目的 Rust M00 身份/会话 authority 与 M10 ingress。

- `GET /api/v1/auth/sso/ustc/status`：返回 `enabled: false`、`state: not_configured`、`protocol: null`。
- `POST /api/v1/auth/sso/ustc/start`：返回 HTTP 503、`sso_not_configured`，不发起跳转。
- `GET /api/v1/auth/sso/ustc/callback`：返回相同拒绝；附带任何 `code`、`ticket`、`state` 或用户标识也不会创建会话。
- 已知路径的方法错误返回 405 和 `Allow`；未知路径或 provider 返回 404。
- 不接受请求体；不设置 Cookie、不返回 Location、不访问校园服务器、不读取环境凭据、不保存账号或回调值。
- 仅监听 IPv4 `127.0.0.1`，校验 Host，无 CORS 放行，响应禁止缓存。请求路径和查询参数不写入访问日志。

它只实现**未配置状态的真实 HTTP 行为**，没有模拟登录成功。OpenAPI 描述见 `openapi.json`。

## 运行与验证

要求 Python 3.10+，无额外依赖。从解压后的本目录运行：

```bash
python3 sso_interface.py --port 8891
```

另一个终端：

```bash
curl -i http://127.0.0.1:8891/api/v1/auth/sso/ustc/status
curl -i -X POST http://127.0.0.1:8891/api/v1/auth/sso/ustc/start
curl -i 'http://127.0.0.1:8891/api/v1/auth/sso/ustc/callback?code=synthetic-test&state=synthetic-test'
python3 -B -m unittest -v test_sso_interface.py
```

测试自动使用系统分配的空闲端口，并关闭自己创建的服务器。手动服务器用 Ctrl+C 停止。本样例不是原应用的反向代理，也不会接触原应用数据目录。

## 授权后的接入位置

```text
用户明确点击 SSO 登录
  → M10 登录入口
  → 服务端认证适配器发起认证
  → 校方身份提供方
  → 服务端回调验证
  → M00 映射外部身份并签发本应用会话
  → 客户端只收到必要的会话结果
```

真实适配器必须根据获批接入文档选用实际协议；这里不假定 USTC 当前开放 OIDC、CAS 或 SAML 中的哪一种，也不填造校园端点或 client_id。

接口扩展职责：

- `begin_login`：使用服务端预登记的固定回调地址；生成有时效、与发起浏览器绑定的单次认证事务。不得接受任意 `return_url` 或浏览器指定的 issuer。
- `verify_callback`：服务端向获批身份提供方验证凭证，检查事务绑定、有效期和重放；按所选协议执行必要的 state/nonce/PKCE、票据或签名及受众校验。浏览器传入的学号、邮箱或 `user_id` 均不是已验证身份。
- 身份映射：以获验证的身份提供方标识与稳定 subject 为键，经 M00 的受控流程关联应用用户；不按姓名或未核验邮箱自动绑定，不因 SSO 登录自动赋予管理员或校园数据访问权限。
- 会话：认证适配器不直接修改画像、grant 或 session 文件；M00 签发和撤销应用会话，并完成会话轮换、CSRF 防护、HTTPS、Cookie 属性、退出与脱敏审计等实际集成。
- 配置与密钥：只在服务端保存；未授权、未配置、校验失败或身份映射不明确时保持拒绝，不能回退为任意演示用户。

启用真实成功路径前，须新增相应版本化成功/错误 schema、官方测试环境联调、过期/重放/错 issuer/错受众/错误事务/跨用户绑定测试和独立安全审查。**不是填入一个 URL 或开关就完成 SSO。**

## 验收边界

本目录的测试只证明接口样例的 HTTP 合约、拒绝行为和无凭证泄露；不证明真实学校认证、生产账号登录或原应用集成。原应用、main、既有安装包及校方系统均不因运行本样例而改变。
