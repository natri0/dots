{ fetchzip, stdenv }:
stdenv.mkDerivation {
  name = "tinycd";

  src = fetchzip {
    url = "https://nightly.link/natri0/dots/workflows/build-utils/main/tinycd.zip";
    hash = "sha256-U3P95fK55lcak1CKmKo3EHC3JrAG9Gz3oEdlwetd9Jw=";
  };

  dontBuild = true;
  installPhase = ''
    mkdir -p $out/bin
    cp $src/tinycd $out/bin/tinycd
    chmod +x $out/bin/tinycd
  ''
}
