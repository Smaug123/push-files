{
  description = "Ratatui app to push changes to a GitHub repo";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    crate2nix = {
      url = "github:kolloch/crate2nix";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = {
    nixpkgs,
    utils,
    crate2nix,
    ...
  }: let
    name = "push-files";
  in
    utils.lib.eachDefaultSystem
    (
      system: let
        # Imports
        pkgs = import nixpkgs {
          inherit system;
        };
        inherit
          (import "${crate2nix}/tools.nix" {inherit pkgs;})
          generatedCargoNix
          ;

        # Create the cargo2nix project
        project =
          pkgs.callPackage
          (generatedCargoNix {
            inherit name;
            src = ./.;
          })
          {
            # Individual crate overrides go here
            # Example: https://github.com/balsoft/simple-osd-daemons/blob/6f85144934c0c1382c7a4d3a2bbb80106776e270/flake.nix#L28-L50
            defaultCrateOverrides =
              pkgs.defaultCrateOverrides
              // {
                # The app crate itself is overriden here. Typically we
                # configure non-Rust dependencies (see below) here.
                ${name} = oldAttrs:
                  {
                    inherit buildInputs nativeBuildInputs;
                  }
                  // buildEnvVars;
              };
          };

        # Configuration for the non-Rust dependencies
        buildInputs = with pkgs; [openssl.dev];
        nativeBuildInputs = with pkgs; [rustc cargo clippy];
        buildEnvVars = {
        };
      in rec {
        packages.${name} = project.workspaceMembers.${name}.build;

        # `nix build`
        defaultPackage = packages.${name};

        # `nix run`
        apps.${name} = utils.lib.mkApp {
          inherit name;
          drv = packages.${name};
        };
        defaultApp = apps.${name};

        # `nix develop`
        devShells = {
          ci =
            pkgs.mkShell {
              inherit nativeBuildInputs;
              buildInputs = [pkgs.nodePackages.markdown-link-check pkgs.alejandra] ++ buildInputs;
              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            }
            // buildEnvVars;
          default =
            pkgs.mkShell
            {
              inherit buildInputs nativeBuildInputs;
              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            }
            // buildEnvVars;
        };
      }
    );
}
