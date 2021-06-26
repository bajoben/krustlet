let
  unstable = import (fetchTarball https://releases.nixos.org/nixpkgs/nixpkgs-21.11pre292293.84aa23742f6/nixexprs.tar.xz) { };
in
{ pkgs ? import <nixpkgs> {} }:
with pkgs; mkShell {
  nativeBuildInputs = with pkgs; [
  unstable.cargo
  gcc
  openssl
  pkg-config
  unstable.rustc
  #rustup #https://stackoverflow.com/questions/44303915/no-default-toolchain-configured-after-installing-rustup                                                                                                                    
];

  buildInputs = [
    pkgs.protobuf
];
  
    
  PROTOC = "protoc";
  RUST_BACKTRACE = "1";
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
}
