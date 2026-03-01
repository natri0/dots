{ artifactName, actionRun, zipDerivationHash, pkgs }:
let
  repo = "natri0/dots";
  runInfoUrl = "https://api.github.com/repos/${repo}/actions/runs/${actionRun}/artifacts";

  matchArtifact = artifact: artifact.name == artifactName;
  artifactsList = (builtins.fromJSON builtins.readFile builtins.fetchurl runInfoUrl).artifacts;
  artifactJson = builtins.head (builtins.filter matchArtifact artifactsList);

  artifact = pkgs.fetchzip {
    url = builtins.fetchurl artifactJson.archive_download_url;
    hash = zipDerivationHash;
  };
in pkgs.stdenv.mkDerivation {
  name = artifactName;
  src = artifact;
}
