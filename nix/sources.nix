{
  nixpkgs = <nixpkgs>;
  nixpkgs-mozilla = (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
}
