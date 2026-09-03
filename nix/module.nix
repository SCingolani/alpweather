{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.alpine-weather-route;
  package = cfg.package;
in {
  options.services.alpine-weather-route = {
    enable = mkEnableOption "the Alpine weather route service";

    package = mkOption {
      type = types.package;
      defaultText = literalExpression "self.packages.\${system}.default";
      description = "Package providing the alpine-weather-route executable.";
    };

    listenAddress = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Address on which the HTTP service listens.";
    };

    port = mkOption {
      type = types.port;
      default = 8080;
      description = "HTTP port on which the service listens.";
    };

    dataDir = mkOption {
      type = types.path;
      default = "/var/lib/alpine-weather-route";
      description = "Directory used for cached weather data and uploads.";
    };

    environmentFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Optional systemd EnvironmentFile containing API credentials and overrides.";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open the configured TCP port in the firewall.";
    };
  };

  config = mkIf cfg.enable {
    users.users.alpine-weather-route = {
      isSystemUser = true;
      group = "alpine-weather-route";
      home = cfg.dataDir;
      createHome = true;
    };
    users.groups.alpine-weather-route = { };

    systemd.services.alpine-weather-route = {
      description = "Weather forecasts along GPX cycling routes";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      environment = {
        WEATHER_LISTEN_ADDRESS = cfg.listenAddress;
        WEATHER_PORT = toString cfg.port;
        WEATHER_DATA_DIR = cfg.dataDir;
        WEATHER_STATIC_DIR = "${package}/share/alpine-weather-route/static";
      };
      serviceConfig = {
        ExecStart = "${package}/bin/alpine-weather-route";
        User = "alpine-weather-route";
        Group = "alpine-weather-route";
        WorkingDirectory = cfg.dataDir;
        StateDirectory = "alpine-weather-route";
        EnvironmentFile = optional (cfg.environmentFile != null) cfg.environmentFile;
        Restart = "on-failure";
        RestartSec = 5;
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateDevices = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        ReadWritePaths = [ cfg.dataDir ];
        UMask = "0077";
      };
    };

    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [ cfg.port ];
  };
}
