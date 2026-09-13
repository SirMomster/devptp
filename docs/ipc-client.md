# devptp IPC client integration

This document describes the local IPC interface for integrations written in
Python, Node.js, shell scripts, or other applications.

## Transport

The daemon exposes a local, newline-delimited JSON stream:

- Linux: `$XDG_RUNTIME_DIR/devptp.sock` when `XDG_RUNTIME_DIR` is set,
  otherwise `/tmp/devptp.sock`.
- Windows: the platform temporary directory, as `devptp.sock`.

The socket is owned by the daemon user and has mode `0600` on Unix. Clients
must run as the same OS user as the daemon. Only one daemon may own the socket.

Each request and response is one complete JSON object followed by `\n`.
Multiple requests may be sent on one connection. Responses are returned in the
same order as requests on that connection.

## Request format

```json
{"id":1,"method":"status","params":{}}
```

Fields:

| Field | Type | Required | Description |
|---|---|---:|---|
| `id` | JSON value | no | Client-generated correlation ID, echoed by the response |
| `method` | string | yes | Command name |
| `params` | object | no | Method parameters; send `{}` for parameterless methods |

Use unique IDs when several requests are in flight. Notifications without an
`id` are accepted, but the response also has a `null` ID.

## Response format

Success:

```json
{"id":1,"ok":true,"result":{"running":true}}
```

Failure:

```json
{"id":1,"ok":false,"error":{"code":"command_failed","message":"No peer is connected"}}
```

A successful response has `ok: true` and `result`; an unsuccessful response has
`ok: false` and `error`. Do not rely on the ordering of JSON object fields.

Current error codes are `invalid_request` for malformed JSON and
`command_failed` for invalid parameters, unknown methods, or command failures.

## Methods

### `ping`

Checks the connected peer.

```json
{"id":1,"method":"ping","params":{}}
```

Result:

```json
{"pong":true}
```

Fails if no peer is connected.

### `start_serving`

Starts the serving/router side and returns its ticket. This operation is only
supported on Linux because port discovery reads Linux `/proc/net` tables.

```json
{"id":2,"method":"start_serving","params":{}}
```

Result:

```json
{"ticket":"..."}
```

Calling it again after serving has started returns an error.

### `connect`

Connects to a serving peer using an endpoint ticket.

```json
{"id":3,"method":"connect","params":{"ticket":"..."}}
```

Result:

```json
{"connected":true}
```

### `expose_port`

Requests that a local service port be exposed to connected peers.

```json
{"id":4,"method":"expose_port","params":{"local_port":8083}}
```

Result:

```json
{"local_port":8083}
```

`local_port` must be an integer from `0` through `65535`. Port discovery and
configuration filtering are performed by the Linux serving side.

### `status`

Returns the current daemon state.

```json
{"id":5,"method":"status","params":{}}
```

Result:

```json
{
  "running": true,
  "role": "connecting",
  "serving": false,
  "connected": true,
  "ticket": null,
  "forwarded_ports": [8083],
  "available_ports": []
}
```

`role` is `idle`, `serving`, or `connecting`. A serving daemon owns the remote
connection and reports `available_ports`; a connecting daemon owns the local
forwarding listeners and reports `forwarded_ports`.

`forwarded_ports` contains ports currently being forwarded by this client.
`available_ports` contains ports detected by the daemon and allowed by
`shared_ports` in `.devptp.toml`. Before the port monitor has completed its
first scan it may be empty.

### `list_forwarded_ports`

Returns the ports currently being forwarded by this client.

```json
{"id":6,"method":"list_forwarded_ports","params":{}}
```

Result:

```json
{"ports":[8083]}
```

### `disconnect`

With no port, closes the active peer connection, aborts forwarding listeners,
and clears connection state. With a `port`, only that forwarding listener is
stopped.

Disconnect one forwarded port:

```json
{"id":7,"method":"disconnect","params":{"port":8083}}
```

Result:

```json
{"port":8083,"forwarded":false}
```

Disconnect the peer entirely:

```json
{"id":8,"method":"disconnect","params":{}}
```

Result:

```json
{"connected":false}
```

### `shutdown`

Gracefully stops the daemon, including the peer, router, port monitor, and IPC
listener. The response is sent before the daemon exits.

```json
{"id":8,"method":"shutdown","params":{}}
```

Result:

```json
{"shutdown":true}
```

## Minimal client flow

1. Start the daemon process: `devptp --daemon`.
2. Connect to the socket.
3. Send a `start_serving` request on Linux, or a `connect` request on Windows
   when connecting to a Linux serving machine.
4. Read one newline-terminated response per request.
5. Match responses using `id` and check `ok` before reading `result`.
6. Send `disconnect` or `shutdown` when finished.

Integrations should treat unknown response fields as forward-compatible and
should not parse human-readable daemon stdout/stderr as API output.
