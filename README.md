A `prometheus` [rsync.net](https://rsync.net) scraper that reports usage by
running `quota` command in the background.

# Example

```shell
❯ curl http://10.200.0.9:9770/metrics
# HELP rsync_net_usage Amount of space occupied by current backups (GB)
# TYPE rsync_net_usage gauge
rsync_net_usage 55.621

# HELP rsync_net_files Count of files
# TYPE rsync_net_files gauge
rsync_net_files 22093

# HELP rsync_net_up 1 if fetch was OK, 0 if not OK
# TYPE rsync_net_up gauge
rsync_net_up 1

# HELP rsync_net_hard_quota Hard quota (GB)
# TYPE rsync_net_hard_quota gauge
rsync_net_hard_quota 123.2

# HELP rsync_net_billed_usage Disk space that is billed (GB)
# TYPE rsync_net_billed_usage gauge
rsync_net_billed_usage 86.059

# HELP rsync_net_custom_snaps Amount of space occupied by custom snapshots (GB)
# TYPE rsync_net_custom_snaps gauge
rsync_net_custom_snaps 30.438

# HELP rsync_net_soft_quota Soft quota (GB)
# TYPE rsync_net_soft_quota gauge
rsync_net_soft_quota 112

# HELP rsync_net_free_snaps Amount of space occupied by free snapshots (GB)
# TYPE rsync_net_free_snaps gauge
rsync_net_free_snaps 0
```

# Usage

## Setup

This exporter uses SSH to access the rsync.net account and run the `quota`
command. Using `.ssh/authorized_keys` it's possible to force the connection to
run a specific command, `quota` in our case.

1. Generate a new SSH key, e.g. `~/.ssh/id_rsync_quota`
2. Add the ssh key to the authorized_keys and limit it to `quota only`:

> [!CAUTION]
> Double check the command output so as not to break or overwrite authorized_keys

```shell
cat <<EOF | ssh rsync.net 'dd of=.ssh/authorized_keys oflag=append conv=notrunc' restrict,command="quota" $(cat ~/.ssh/id_rsync_quota.pub)
EOF
```

## Nix

Systemd code was written targeting usage in NixOS with flakes, but the general
idea can be adapted to other distros.

1. Add this project as a flake input
2. Import this flake's `nixosModules.default`
3. Configure the service, for example, passing secrets using `age`:

```nix
  services.prometheus.exporters.rsync-net = {
    enable = true;
    port = 9000;
    usernameFile = config.age.secrets."rsync-net-user".path;
    sshKeyPath = config.age.secrets."rsync-net-key".path;
  };
```


## Running the binary directly

The binary accepts config from the environment. A minimal example:
```shell
export RSYNC_SSH_KEY_PATH="<pathToSSHKey"
export RSYNC_USERNAME="id123"
./prometheus-rsync-net-exporter
```

# Configuration

The application is configured via environment variables:

| Variable | Description | Default |
|---|---|---|
| `RSYNC_USERNAME` | rsync.net username. Mutually exclusive with `RSYNC_USERNAME_FILE`. | |
| `RSYNC_USERNAME_FILE` | Path to file containing the rsync.net username. | |
| `RSYNC_SSH_KEY_PATH` | Path to the SSH private key for authentication. | |
| `RSYNC_HOST` | Hostname of the rsync.net server. | `<RSYNC_USERNAME>.rsync.net` |
| `RSYNC_EXPORTER_PORT` | Port to listen on for metrics. | `9000` |
| `RSYNC_LISTEN_ADDRESS` | IP address to bind to. | `0.0.0.0` |
| `RSYNC_FETCH_INTERVAL_SECONDS` | Interval between quota fetches in seconds. | `3600` |

# See also

- [Python implementation][1] that lets `prometheus` scrape the usage from the RSS
  feed

[1]: https://github.com/yrro/rsync.net-exporter
