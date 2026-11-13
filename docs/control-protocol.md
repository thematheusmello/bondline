# Command contract

The delivered engine has one typed registry in `bondline-commands`. Names, Rust parameter
types, schemas and discovery metadata originate in the same declaration. The UI and external
adapters must call it; they must not implement their own authoring operations.

An engine JSON request is exactly:

```json
{"command":"page.set_size","params":{"page_id":1,"width_pt":612,"height_pt":792}}
```

Unknown fields, missing parameters, invalid IDs and non-finite/out-of-range dimensions fail
before mutation. Raw JSON is bounded to 1 MiB. Results contain an immutable authored snapshot,
monotonic revision, `changed` flag and undo/redo depths. Errors expose a stable code and message.
Inspection and exact no-ops do not advance revision or discard redo. A changed edit discards
the previous redo branch. Undo/redo restore authored state exactly but advance revision.
History retains up to 128 document snapshots. This is an explicit Bondline policy, not a
claim about another application's history limits. Revision overflow rejects the operation.

| Name | Parameters | Effect |
|---|---|---|
| `document.inspect` | `{}` | Read-only snapshot |
| `page.set_size` | `page_id`, `width_pt`, `height_pt` | One validated page edit |
| `history.undo` | `{}` | Previous authored snapshot |
| `history.redo` | `{}` | Next authored snapshot |

The delivered CLI exposes `commands` and `run --cmd '<request>'`. Repeat `--cmd` to execute
sequential operations in one in-memory engine; stdout is one final JSON snapshot. Failure
prints a structured error to stderr and exits 2 (I/O/serialization failures exit 1). It does
not save a project or make a multi-command transaction. `--help` lists actual operations.

Native Page presets/rotation, Edit Undo/Redo and history shortcuts call these same commands.
The canvas reads the authored dimensions; no native control edits the model directly.
`bondline-cli mcp` serves a process-local document over stdio using rmcp 3.9.0. It negotiates
MCP lifecycle, exposes the registry as tools and returns structured content plus text. Domain
errors are tool results with `isError`; unknown tools are protocol errors. Input lines are
bounded to 1 MiB and oversized lines are discarded before parsing. EOF closes the session.
This process has its own engine; it does not attach to the open desktop window. The stdio
process opens no network listener, HTTP transport, account or hosted service.

Start the desktop with `--control-port PORT` to opt into raw newline-delimited JSON on
127.0.0.1. Port 0 requests an available port; startup emits its address as JSON and the UI
shows it. Send the same `{command,params}` object followed by LF. Each response is exactly
one `{result:...}` or `{error:{code,message}}` object followed by LF. Requests on a connection
run in order; connections share the native document and history. Changed state requests a
native repaint. There is no MCP handshake on this separate raw control channel.

Limits are explicit Bondline policies: 1 MiB per request, 16 simultaneous connections,
30 seconds for a complete input frame or output write. Excess connections close. Missing
LF, oversize or malformed JSON/parameter shapes return an error where possible and close
the connection. An HTTP header cannot expose its body as commands. Valid domain errors
leave authored state unchanged. Dropping the server closes the listener and cancels clients;
explicit `shutdown()` also reports worker errors. No non-loopback bind option is offered.

Offscreen command capture is still missing.
