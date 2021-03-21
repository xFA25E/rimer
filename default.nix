{ pkgs ? (import <nixpkgs> {}) }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "rimer";
  version = "0.1.0";
  src = pkgs.fetchFromGitHub {
    owner = "xFA25E";
    repo = pname;
    rev = version;
    sha256 = "1111111111111111111111111111111111111111111111111111";
  };
  cargoSha256 = "1r03334c8y5kj102cz2f9x57h1v3z3dw7nxhjm7gpin16lwvd5ca";
  meta = with pkgs.lib; {
    description = "Simple timer that executes commands on update";
    homepage = "https://github.com/xFA25E/rimer";
    license = licenses.unlicense;
    maintainers = [ "Valeriy Litkovskyy <vlr.ltkvsk@protonmail.com>" ];
  };
}
