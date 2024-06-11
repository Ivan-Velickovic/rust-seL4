#
# Copyright 2023, Colias Group, LLC
# Copyright 2023, Galois, Inc.
#
# SPDX-License-Identifier: BSD-2-Clause
#

{ mk, versions, localCrates, serdeWith, authors }:

mk {
  package.name = "sel4-hal-adapters";
  package.authors = with authors; [
    nspin
    "Ben Hamlin <hamlinb@galois.com>"
  ];
  dependencies = {
    inherit (versions) log;
    serde = serdeWith [];
  } // (with localCrates; {
    inherit sel4-microkit-message;
    sel4-microkit = sel4-microkit // { default-features = false; };
  });
}
