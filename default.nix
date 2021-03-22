{ pkgs ? (import <nixpkgs> {}) }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "rimer";
  version = "0.1.1";
  src = pkgs.fetchFromGitHub {
    owner = "xFA25E";
    repo = pname;
    rev = version;
    sha256 = "0vqxkq2sg8ac44aibarvrgjm09966m35fl58s865vqkzd92ilhvl";
  };
  cargoSha256 = "0qxmy2gynk5r4327mfsw0ab7y0g7vx1qarcl0i7d5i5brvykijxv";
  meta = with pkgs.lib; {
    description = "Simple timer that executes commands on update";
    homepage = "https://github.com/xFA25E/rimer";
    license = licenses.gpl3;
    maintainers = [ "Valeriy Litkovskyy <vlr.ltkvsk@protonmail.com>" ];
  };
}
