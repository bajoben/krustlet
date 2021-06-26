{ pkgs ? import <nixpkgs> {}, 
stdenv, lib
, fetchurl
, unzip
, openssl
, zlib
, autoPatchelfHook
}:

pkgs.stdenv.mkDerivation rec {
  name = "krustlet-wasi";
  version = "0.7.0";
  src = fetchurl {
    url = "https://krustlet.blob.core.windows.net/releases/krustlet-v${version}-linux-amd64.tar.gz";
    sha256 = "01f1kjcvxsyy1g1cvlarfqjffpgzgn6zxm4i2ahqir7iz41dcqw8";
  };

  nativeBuildInputs = [
    autoPatchelfHook
  ];

  buildInputs = [
    openssl
    zlib
  ];

  unpackPhase = ''
    tar xzf $src
  '';

  installPhase = ''
    install -m755 -D ${name} $out/bin/krustlet-wasi
  '';

  shellHook = ''
    export NIXPKGS_ALLOW_INSECURE=1
  '';
}
