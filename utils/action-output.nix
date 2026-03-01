{ artifactName, actionRun, zipDerivationHash, pkgs }:
let
  repo = "natri0/dots";
  runInfoUrl = "https://api.github.com/repos/${repo}/actions/runs/${actionRun}/artifacts";

  matchArtifact = name: artifact: artifact.name == name;
  artifactJson = builtins.head builtins.filter (matchArtifact artifactName) (builtins.fromJSON builtins.readFile builtins.fetchurl runInfoUrl).artifacts;

  artifact = pkgs.fetchzip {
    url = builtins.fetchurl artifactJson.archive_download_url;
    hash = zipDerivationHash;
  };
in pkgs.stdenv.mkDerivation {
  name = artifactName;
  src = artifact;
}
