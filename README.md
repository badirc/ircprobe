# ircprobe

A small line-based IRC client.

## For IRC developers

Download and install this tool:

```bash
cargo install ircprobe
# more installation methods to be added
```

Connect to a server:

```bash
ircprobe localhost:6667
>
```

Now you can enter commands (try `CAP LS`), and you should be able
to see any responses from the server.

## For end users

This is a client mainly designed for quick manual debugging of IRC servers.
It therefore doesn't really lend itself to being "useable", sorry!
