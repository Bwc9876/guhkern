{
  description = "Flake for guhkern's shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    rustupToolchain = "nightly";

    rustBuildTargetTriple = "riscv64gc-unknown-none-elf";
    rustBuildHostTriple = "x86_64-unknown-linux-gnu";

    forAllSystems = nixpkgs.lib.genAttrs [
      "aarch64-linux"
      "x86_64-linux"
    ];

    pkgsFor = system: import nixpkgs {inherit system;};
  in {
    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell rec {
        name = "guhkern-dev-shell";
        buildInputs = with pkgs; [
          rustup
          just
          qemu
        ];

        RUSTUP_TOOLCHAIN = rustupToolchain;
        RUSTFLAGS = "-C link-arg=-Tsrc/linker.ld";
        CARGO_BUILD_TARGET = rustBuildTargetTriple;

        # export PATH=$PATH:${CARGO_HOME}/bin
        # export PATH=$PATH:${RUSTUP_HOME}/toolchains/${rustupToolchain}-${rustBuildHostTriple}/bin/

        shellHook = ''
          # Ensures our riscv target is added via rustup.
          rustup target add "${rustBuildTargetTriple}"
        '';
      };
    });
    formatter = forAllSystems (system: (pkgsFor system).alejandra);
  };
}
