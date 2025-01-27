{ mk, localCrates, serdeWith, versions }:

mk {
  package.name = "sel4-kernel-loader-payload-types";
  dependencies = {
    serde = serdeWith [ "derive" ] // { optional = true; };
    heapless = { version = versions.heapless; features = [ "serde" ]; };
    num-traits = { version = versions.num-traits; default-features = false; };
  };
  nix.local.dependencies = with localCrates; [
    sel4-platform-info-types
  ];
}
