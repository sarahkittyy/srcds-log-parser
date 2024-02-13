# srcds-log-parser

Parse log lines from hl2 games `(srcds.exe)`. Works for both local .log file lines and logs received over UDP.

Note that this library does not provide methods to listen to remote udp logs, only to parse already received logs into data structures.

See `examples/` for usage. 

## Examples

Logcat simply listens to logs on port 9999 and echoes them to stdout.

```bash
cargo run --example logcat -- 9999
```