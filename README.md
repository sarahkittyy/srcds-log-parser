# srcds-log-parser

Parse log lines from hl2 games `(srcds.exe)`. Works for both local .log file lines and logs received over UDP.

Note that this library does not provide methods to listen to remote udp logs, only to parse already received logs into data structures.

I have only tested this on TF2. I don't own any other source games to create logs with. If this doesn't work for your srcds logs, send them to me in an issue and I will implement them.

See `examples/` for usage. 

## Examples

Logcat simply listens to logs on port 9999 and echoes them to stdout.

```bash
cargo run --example logcat -- 9999
```