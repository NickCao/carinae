#include "carinae/include/nix.hh"

namespace carinae {
void init() {
  nix::initLibStore();
}

Store openStore(rust::Str uri) {
  return nix::openStore(std::string(uri));
}

rust::String storeDir(Store store) {
  return store->storeDir;
}

PathInfo queryPathInfoFromHashPart(Store store, rust::Str hash, rust::Str key) {
  auto path = store->queryPathFromHashPart(std::string(hash));
  if (!path)
    throw std::invalid_argument("error: path invalid");
  auto pathinfo = store->queryPathInfo(*path);
  rust::Vec<rust::String> sigs;
  for (auto sig : pathinfo->sigs)
    sigs.push_back(rust::String(sig));
  if (!key.empty()) {
    sigs.push_back(nix::SecretKey(std::string(key))
                       .signDetached(pathinfo->fingerprint(*store)));
  }
  return PathInfo{
      store->printStorePath(pathinfo->path),
      pathinfo->deriver ? std::string(pathinfo->deriver->to_string()) : "",
      pathinfo->narHash.to_string(nix::Base32, true),
      nix::concatStringsSep(" ", pathinfo->shortRefs()),
      pathinfo->narSize,
      sigs,
      pathinfo->ca ? nix::renderContentAddress(pathinfo->ca) : "",
  };
}

void narFromHashPart(Store store,
                     rust::Str hash,
                     rust::Box<NarContext> ctx,
                     rust::Fn<bool(NarContext& ctx, rust::Vec<uint8_t>)> send) {
  auto path = store->queryPathFromHashPart(std::string(hash));
  if (!path)
    throw std::invalid_argument("error: path invalid");
  auto sink = nix::LambdaSink([&ctx, &send](std::string_view data) {
    rust::Vec<uint8_t> d;
    std::copy(data.begin(), data.end(), std::back_inserter(d));
    if (!(send)(*ctx, d))
      throw std::invalid_argument("error: cannot send nar");
  });
  store->narFromPath(*path, sink);
}

rust::String getBuildLog(Store store, rust::Str path) {
  auto & logstore = nix::require<nix::LogStore>(*store);
  auto storepath = nix::StorePath(std::string(path));
  auto log = logstore.getBuildLog(storepath);
  if (!log)
    throw std::invalid_argument("error: no log for path");
  return *log;
}

}  // namespace carinae
