#pragma once
#define SYSTEM "x86_64-linux"

#include "rust/cxx.h"
#include "nix/nar-info.hh"
#include "nix/store-api.hh"
#include "nix/crypto.hh"
#include "carinae/src/main.rs.h"

namespace carinae {
typedef std::shared_ptr<nix::Store> Store;
Store nixOpenStore(rust::Str);
rust::String nixStoreDir(Store);
NixPathInfo nixPathInfoFromHashPart(Store, rust::Str, rust::Str);
void nixNarFromHashPart(
    Store,
    rust::Str,
    rust::Box<NarContext>,
    rust::Fn<bool(NarContext&, rust::Slice<const rust::u8>)>);
}  // namespace carinae
