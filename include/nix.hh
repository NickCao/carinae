#pragma once
#define SYSTEM "x86_64-linux"
#include "carinae/src/main.rs.h"
#include "nix/nar-info.hh"
#include "nix/store-api.hh"
#include "rust/cxx.h"

namespace carinae {
struct NixPathInfo;
std::shared_ptr<nix::Store> nixOpenStore(rust::String uri);
rust::String nixStoreDir(std::shared_ptr<nix::Store> store);
NixPathInfo nixPathInfoFromHashPart(std::shared_ptr<nix::Store> store,
                                    rust::String hash);
void nixNarFromHashPart(
    std::shared_ptr<nix::Store>, rust::String hash, rust::Box<NarContext> ctx,
    rust::Fn<bool(NarContext &ctx, rust::Vec<rust::u8>)> send);
} // namespace carinae