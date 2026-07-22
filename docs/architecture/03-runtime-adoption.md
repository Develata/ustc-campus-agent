# Runtime and framework adoption

## Principle

Own campus semantics and authority. Reuse stable protocols and low-differentiation plumbing. Do not merge multiple Agent frameworks into one runtime.

| Reference | Borrow | Do not borrow |
|---|---|---|
| Rig | Rust provider/tool types, structured output, MCP/provider plumbing | canonical Agent run authority |
| goose | MCP-first extension UX, extension diagnostics, permission UX | local-first arbitrary command authority in central plane |
| Pi | package/resource/session organization | in-process TS hot-load or packages with broad system access |
| LangGraph | durable checkpoint/interruption benchmark | platform authority or source of grants/audit truth |

## Hard boundary

Framework checkpoint state is adapter state keyed by `platform_run_id`. If it conflicts with Rust grant/approval/receipt/audit, the platform fails closed.
