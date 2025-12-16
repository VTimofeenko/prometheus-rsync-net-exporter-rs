{ withSystem, ... }:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.prometheus.exporters.rsync-net;
  inherit (lib) types mkOption;
in
{
  options.services.prometheus.exporters.rsync-net = {
    enable = lib.mkEnableOption "rsync.net prometheus exporter";

    package = mkOption {
      type = types.package;
      description = "The rsync-net-exporter package to use.";
      default = withSystem pkgs.stdenv.hostPlatform.system ({ config, ... }: config.packages.default);
    };
    port = mkOption {
      type = types.port;
      default = 9000;
      description = "Port to listen on.";
    };

    listenAddress = mkOption {
      type = types.str;
      default = "0.0.0.0";
      description = "Address to listen on.";
    };

    username = mkOption {
      type = types.nullOr types.str;
      description = "Rsync.net username.";
      default = null;
    };

    host = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "Rsync.net host (optional, defaults to username.rsync.net).";
    };

    sshKeyPath = mkOption {
      type = types.path;
      description = "Path to the SSH private key.";
    };

    usernameFile = mkOption {
      type = types.nullOr types.path;
      description = "Path to the SSH private key.";
      default = null;
    };

    interval = mkOption {
      type = types.int;
      default = 3600;
      description = "Fetch interval in seconds.";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open port in firewall.";
    };

  };

  config = lib.mkIf cfg.enable {

    assertions = [
      {
        assertion = lib.xor (builtins.isNull cfg.usernameFile) (builtins.isNull cfg.username);
        message = "Exactly one of usernameFile and username must be set.";
      }
    ];

    networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [ cfg.port ];

    systemd.services.prometheus-rsync-net-exporter = {
      description = "Prometheus rsync.net exporter";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      environment = {
        RSYNC_EXPORTER_PORT = toString cfg.port;
        RSYNC_LISTEN_ADDRESS = cfg.listenAddress;
        RSYNC_FETCH_INTERVAL_SECONDS = toString cfg.interval;
        RSYNC_SSH_KEY_PATH = "%d/ssh_key";
      }
      // (lib.optionalAttrs (!builtins.isNull cfg.host) {
        RSYNC_HOST = cfg.host;
      })
      // (lib.optionalAttrs (!builtins.isNull cfg.username) {
        RSYNC_USERNAME = cfg.username;
      })
      // (lib.optionalAttrs (!builtins.isNull cfg.usernameFile) {
        RSYNC_USERNAME_FILE = "%d/username";
      });

      serviceConfig = {
        LoadCredential = [
          "ssh_key:${cfg.sshKeyPath}"
        ]
        ++ (lib.optional (!builtins.isNull cfg.usernameFile) "username:${cfg.usernameFile}");
        ExecStart = "${lib.getExe cfg.package}";
        DynamicUser = true;
        Restart = "always";

        # Hardening
        CapabilityBoundingSet = "";
        DevicePolicy = "closed";
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        ProtectHome = "yes";
        ProtectSystem = "strict";
        ProtectControlGroups = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectClock = true;
        ProtectKernelLogs = true;
        ProtectHostname = true;
        PrivateUsers = true;
        ProtectProc = "noaccess";
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
          "AF_UNIX"
        ];
        RestrictNamespaces = true;
        RestrictRealtime = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = "~@clock @cpu-emulation @debug @module @mount @obsolete @privileged @raw-io @reboot @resources @swap";
        UMask = "0077";
      };
    };
  };
}
