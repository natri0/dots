{ pkgs }:
let
  actionOutput = import ../utils/action-output.nix;
in
actionOutput {
  artifactName = "tinycd";
  actionRun = "22548255794";
  zipDerivationHash = "";
  inherit pkgs;
}