{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.openssl
    pkgs.pkg-config
    pkgs.zlib
    pkgs.curl
    pkgs.cmake
    pkgs.libffi
  ];

  shellHook = ''
    export CARGO_NET_GIT_FETCH_WITH_CLI=true
  '';
}

