{
  description = "Zed XML Language Server Extension Development Environment";

  inputs = {
    # Use the system's nixpkgs for glibc compatibility
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Individual Rust toolchain components
        rustc = pkgs.rustc;
        cargo = pkgs.cargo;
        rustfmt = pkgs.rustfmt;
        clippy = pkgs.clippy;

        jdk = pkgs.jdk21;

        graalvm = pkgs.graalvmPackages.graalvm-ce;

        dev-tools = with pkgs; [
          rust-analyzer
          git
          curl
          jq
          maven
          lld
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rustc cargo rustfmt clippy jdk graalvm ] ++ dev-tools;

          JAVA_HOME = "${jdk}";
          GRAALVM_HOME = "${graalvm}";

          shellHook = ''
            echo "🦀 Zed XML Language Server Development Environment"
            echo "  Rust: $(rustc --version)"
            echo "  Cargo: $(cargo --version)"
            echo "  Java: $(java --version 2>&1 | head -1)"
            echo "  GraalVM: $GRAALVM_HOME"
            echo ""
            echo "Available commands:"
            echo "  make dev       - Run Zed with dev extension (install via Zed UI)"
            echo "  make test      - Run cargo test"
            echo "  make build     - Build release WASM"
            echo "  make build-native - Build native binary (x86_64 Linux/macOS)"
            echo "  make native    - Build LemMinX native binary (requires GraalVM)"
            echo "  make check     - fmt + clippy + test"
            echo "  make clean     - Clean build artifacts"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "zed-xml-language-server";
          version = "0.1.0";
          src = self;
          cargoSha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        };
      });
}