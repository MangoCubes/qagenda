{
  config,
  inputs,
  pkgs,
  lib,
  ...
}:
{
  options = {
    programs.qagenda = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable qagenda, a quick tasks/events overlay";
      };
      settings = lib.mkOption {
        type = lib.types.attrs;
        default = { };
        description = "Any settings that are not yet implemented in the flake goes here. They will automatically be converted into JSON.";
      };
    };
  };
  config =
    let
      cfg = config.programs.qagenda;
    in
    lib.mkIf cfg.enable {
      # Install package
      home.packages = [
        inputs.qagenda.packages."${pkgs.stdenv.hostPlatform.system}".default
      ];
      xdg = {
        # Create config.json
        configFile."qagenda/config.json".text = (builtins.toJSON (cfg.settings));
      };
    };
}
