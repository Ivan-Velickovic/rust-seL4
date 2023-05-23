{ mk, localCrates, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "banscii-artist-interface-types";
  dependencies = {
    inherit (versions) zerocopy;
    num_enum = { version = versions.num_enum; default-features = false; };
  };
  nix.meta.skip = true;
}
