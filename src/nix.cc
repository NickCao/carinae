#include "carinae/include/nix.hh"

namespace carinae {
std::shared_ptr<nix::Store> nixOpenStore(rust::Str uri) {
  return nix::openStore(std::string(uri));
}

rust::String nixStoreDir(std::shared_ptr<nix::Store> store) {
  return store->storeDir;
}

NixPathInfo nixPathInfoFromHashPart(std::shared_ptr<nix::Store> store,
                                    rust::Str hash, rust::Str key) {
  auto path = store->queryPathFromHashPart(std::string(hash));
  if (!path)
    throw std::invalid_argument("error: path invalid");
  auto pathinfo = store->queryPathInfo(*path);
  rust::Vec<rust::String> sigs;
  for (auto sig : pathinfo->sigs)
    sigs.push_back(rust::String(sig));
  if (!key.empty()) {
    sigs.push_back(nix::SecretKey(std::string(key)).signDetached(pathinfo->fingerprint(*store)));
  }
  return NixPathInfo{
      store->printStorePath(pathinfo->path),
      pathinfo->deriver ? std::string(pathinfo->deriver->to_string()) : "",
      pathinfo->narHash.to_string(nix::Base32, true),
      nix::concatStringsSep(" ", pathinfo->shortRefs()),
      pathinfo->narSize,
      sigs,
      pathinfo->ca ? nix::renderContentAddress(pathinfo->ca) : "",
  };
}

struct RustSink : nix::Sink {
 public:
  rust::Box<NarContext>* ctx;
  rust::Fn<bool(NarContext& ctx, rust::Slice<const rust::u8>)>* send;
  bool status;
  RustSink(rust::Box<NarContext>* ctx,
           rust::Fn<bool(NarContext& ctx, rust::Slice<const rust::u8>)>* send,
           bool status)
      : ctx(ctx), send(send), status(status){};
  void operator()(std::string_view data) {
    status = (*send)(**ctx, rust::Slice((const rust::u8*) data.data(), data.size()));
  };
  bool good() { return status; }
};

void nixNarFromHashPart(
    std::shared_ptr<nix::Store> store,
    rust::Str hash,
    rust::Box<NarContext> ctx,
    rust::Fn<bool(NarContext& ctx, rust::Slice<const rust::u8>)> send) {
  auto path = store->queryPathFromHashPart(std::string(hash));
  if (!path)
    throw std::invalid_argument("error: path invalid");
  auto sink = RustSink{&ctx, &send, true};
  store->narFromPath(*path, sink);
}

}  // namespace carinae