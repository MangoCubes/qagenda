{
  config,
  inputs,
  pkgs,
  lib,
  ...
}:
{
  options = {
    programs.qcal = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable qcal, a quick calendar overlay";
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
      cfg = config.programs.qcal;
    in
    lib.mkIf cfg.enable {
      # Install package
      home.packages = [
        inputs.qcal.packages."${pkgs.stdenv.hostPlatform.system}".default
      ];
      xdg = {
        # Create config.json
        configFile."qcal/config.json".text = (builtins.toJSON (cfg.settings));
      };
    };
}
