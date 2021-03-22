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
  cargoSha256 = "0yqrkbysn3lydv9gwqc9ifkwgqh0vayjnv22igb3192cgn2d4bw4";
  meta = with pkgs.lib; {
    description = "Simple timer that executes commands on update";
    homepage = "https://github.com/xFA25E/rimer";
    license = licenses.gpl3;
    maintainers = [ "Valeriy Litkovskyy <vlr.ltkvsk@protonmail.com>" ];
  };
}
