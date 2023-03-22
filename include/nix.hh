#pragma once
#define SYSTEM "x86_64-linux"

#include "rust/cxx.h"
#include "nix/nar-info.hh"
#include "nix/store-api.hh"
#include "nix/store-cast.hh"
#include "nix/log-store.hh"
#include "nix/crypto.hh"
#include "carinae/src/main.rs.h"

namespace carinae {
typedef std::shared_ptr<nix::Store> Store;
void init();
Store openStore(rust::Str);
rust::String storeDir(Store);
PathInfo queryPathInfoFromHashPart(Store, rust::Str, rust::Str);
void narFromHashPart(Store,
                     rust::Str,
                     rust::Box<NarContext>,
                     rust::Fn<bool(NarContext&, rust::Vec<uint8_t>)>);
rust::String getBuildLog(Store, rust::Str);
}  // namespace carinae
