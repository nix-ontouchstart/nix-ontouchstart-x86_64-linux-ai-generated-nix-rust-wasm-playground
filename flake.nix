{
  description = "Pure buildRustPackage WASM static generator for GitHub Pages";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "github-pages-static-assets";
        version = "0.1.0";
        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;
        auditable = false; 
        doCheck = false;

        # Add the LLVM tools and bindgen requirements explicitly
        depsBuildBuild = [ pkgs.buildPackages.stdenv.cc pkgs.lld ];
        nativeBuildInputs = [ pkgs.wasm-bindgen-cli ];
        CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "wasm-ld";

        # 1. FORCE Cargo to compile for WebAssembly by overriding the buildPhase
        buildPhase = ''
          echo "Compiling Rust package targeting WebAssembly..."
          cargo build --release --target=wasm32-unknown-unknown --offline
        '';

        # 2. Extract and run bindgen in a predictable location
        installPhase = ''
          mkdir -p $out

          # Nix isolates target folders under standard release structures when buildPhase is manual
          WASM_FILE=$(find target -name "frontend.wasm" -print -quit)

          if [ -z "$WASM_FILE" ]; then
            echo "Error: Could not locate frontend.wasm anywhere in the build tree!"
            exit 1
          fi

          echo "Found binary at $WASM_FILE. Running bindgen..."
          wasm-bindgen "$WASM_FILE" --target web --out-dir $out

          # Copy your main HTML wrapper
          cp index.html $out/
        '';
      };
    };
}

