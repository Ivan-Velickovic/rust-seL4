{ mk, localCrates, coreLicense, meAsAuthor, versions }:

mk {
  nix.meta.requirements = [ "sel4" ];
  package.name = "sel4-sys";
  package.build = "build/main.rs";
  nix.local.dependencies = with localCrates; [
    sel4-config
    sel4-bitfield-types
  ];
  nix.local.build-dependencies = with localCrates; [
    sel4-build-env
    sel4-rustfmt-helper
    sel4-bitfield-parser
    sel4-config
    sel4-config-data
  ];
  package.license = coreLicense;
  package.authors = [ meAsAuthor ];
  dependencies = {
    inherit (versions) log;
  };
  build-dependencies = {
    bindgen = "0.64.0";
    xmltree = "0.10.3";
    glob = "0.3.0";
    regex = "1.7.0";
    inherit (versions) proc-macro2;
    inherit (versions) quote;
    inherit (versions) syn;
  };
  features = {
    wrappers = [];
  };
}