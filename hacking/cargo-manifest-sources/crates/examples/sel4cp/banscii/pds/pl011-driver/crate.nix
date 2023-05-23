{ mk, localCrates, versions }:

mk {
  nix.meta.labels = [ "leaf" ];
  nix.meta.requirements = [ "sel4" ];
  package.name = "banscii-pl011-driver";
  nix.local.dependencies = with localCrates; [
    sel4cp
    banscii-pl011-driver-interface-types
  ];
  dependencies = {
    inherit (versions) tock-registers;
    sel4cp.default-features = false;
    inherit (versions) heapless;
  };
  nix.meta.skip = true;
}
