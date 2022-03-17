#pragma once
#include "carinae/include/nix.hh"
using namespace nix;

rust::String nixOpenStore(rust::String uri) {
  return openStore(std::string(uri))->name();
}
