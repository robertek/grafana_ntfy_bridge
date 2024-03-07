# Grafana_ntfy_bridge

## Bridge setup

The bridge can be set up using two options, Toml file or command line options.
The command line has a precedence, so if you set something in the Toml you may override the value with command line option.

A mandatory option is the `topic`, others has defaults or are optional.

### TOML

Here is an example config with all the possible options:

```toml
### NTFY server url
# default: https://ntfy.sh
#url="https://ntfy.sh"

### NTFY topic
topic="grafana_to_ntfy-test"

### port to listen on
# default: 8080
#port=8080

### authorisation key set in Grafana
#key="0123456789ABCDEF"
```

To start the bridge you will run it using `--config-file` option.

```sh
grafana_ntfy_bridge --config-file="/path/to/config.toml"
```

If no config is specified the bridge tries to open `grafana_ntfy_bridge.toml` in CWD.

### Command line options

The `--help` is quite self explanatory.

```
Usage: grafana_ntfy_bridge [OPTIONS]

Options:
  -c, --config-file <CONFIG_FILE>  Config file
  -u, --url <URL>                  NTFY url
  -t, --topic <TOPIC>              NTFY topic
  -p, --port <PORT>                port to listen on
  -k, --key <KEY>                  grafana connector key
  -h, --help                       Print help
  -V, --version                    Print version
```

Without config it is possible to run the bridge with only --topic specified:

```
grafana_ntfy_bridge --topic="my_test_topic"
```

It will listen on `0.0.0.0:8080` to the Grafana connection and use default `https://ntfy.sh` server.


## Grafana setup

## Setup consideration

Since the Grafana <-> Grafana_ntfy_bridge connection is not encrypted, so it is advised routing it only through a protected network.
Ideally it will run in the same container with Grafana or on the same local network.

