{ }:

let
  mozillaOverlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  pkgs = import <nixpkgs> {
    overlays = [ mozillaOverlay ];
  };
  rustChannel = pkgs.rustChannelOf { date = "2020-09-03"; channel = "nightly"; };

in pkgs.mkShell {
  buildInputs = with pkgs; [
    rustChannel.rust
    rustChannel.rust-src
    rustChannel.cargo
    pkg-config
    openssl
    nodejs
  ];

  RUST_BACKTRACE = 1;
  RUST_SRC = "${rustChannel.rust-src}/lib/rustlib/src/rust";
}
