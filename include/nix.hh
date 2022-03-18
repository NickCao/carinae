#pragma once
#define SYSTEM "x86_64-linux"

#include "rust/cxx.h"
#include "nix/nar-info.hh"
#include "nix/store-api.hh"
#include "nix/crypto.hh"
#include "carinae/src/main.rs.h"

namespace carinae {
std::shared_ptr<nix::Store> nixOpenStore(rust::String);
rust::String nixStoreDir(std::shared_ptr<nix::Store>);
NixPathInfo nixPathInfoFromHashPart(std::shared_ptr<nix::Store>, rust::String, rust::String);
void nixNarFromHashPart(std::shared_ptr<nix::Store>,
                        rust::String,
                        rust::Box<NarContext>,
                        rust::Fn<bool(NarContext&, rust::Vec<rust::u8>)>);
}  // namespace carinae
