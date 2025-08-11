{ pkgs }: let
  dev-help = pkgs.writeShellScriptBin "dev-help" /* sh */ ''
    printf '%s\n' \
    'phph'
  '';
  rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable."1.88.0".default.override {
    targets = [ "wasm32-unknown-unknown" ];
  };
in
pkgs.mkShell {
  buildInputs = [ dev-help ];
  nativeBuildInputs = [ rustToolchain ] ++ (with pkgs; [ 
    pkg-config
    openssl
    cargo-leptos
    cargo-generate
    sass
  ]);

  shellHook = ''
    set -a
    source .env
    set +a
  '';

  # environment
  PAGER="${pkgs.hectic.nvim-pager}/bin/pager";
}
