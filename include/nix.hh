#pragma once
#define SYSTEM "x86_64-linux"

#include "rust/cxx.h"
#include "nix/nar-info.hh"
#include "nix/store-api.hh"
#include "nix/crypto.hh"
#include "carinae/src/main.rs.h"

namespace carinae {
std::shared_ptr<nix::Store> nixOpenStore(rust::Str);
rust::String nixStoreDir(std::shared_ptr<nix::Store>);
NixPathInfo nixPathInfoFromHashPart(std::shared_ptr<nix::Store>, rust::Str, rust::Str);
void nixNarFromHashPart(std::shared_ptr<nix::Store>,
                        rust::Str,
                        rust::Box<NarContext>,
                        rust::Fn<bool(NarContext&, rust::Vec<rust::u8>)>);
}  // namespace carinae
