{
  description = "Nix cache copy";

  inputs.nru.url = "github:voidcontext/nix-rust-utils?ref=refs/tags/v0.6.0";

  outputs = {nru, ...}:
    nru.lib.mkOutputs ({pkgsUnstable, ...}: {
      crate = {src = ./.;};
      buildInputs = [pkgsUnstable.cocogitto pkgsUnstable.cargo-edit];
    });
}
