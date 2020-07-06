let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs {};
in
pkgs.mkShell {
  buildInputs = [
    rust
    pkgs.cargo-watch
    pkgs.systemfd
    pkgs.nghttp2
  ];
}
