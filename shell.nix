let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };
  ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
    extensions = [
      "rust-src" "rust-std" "rustfmt-preview" "rust-analysis"
      # "rust-analyzer-preview"
      "rls-preview" "clippy-preview"
    ];}
  );
in
with nixpkgs;
stdenv.mkDerivation {
  name = "rust";
  buildInputs = [ rustup ruststable ];
}
