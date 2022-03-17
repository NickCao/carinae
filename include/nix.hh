#pragma once
#define SYSTEM "x86_64-linux"
#include "nix/store-api.hh"
#include "rust/cxx.h"

rust::String nixOpenStore(rust::String uri);
