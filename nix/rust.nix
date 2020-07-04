{ sources ? import ./sources.nix }:

let
  pkgs = import sources.nixpkgs { overlays = [ (import sources.nixpkgs-mozilla) ]; };
  channel = "nightly";
  date = "2020-07-04";
  targets = [];
  #rust = pkgs.latest.rustChannels.nightly.rust;
  rust = (pkgs.rustChannelOf { date = date; channel = channel; }).rust;
in rust
