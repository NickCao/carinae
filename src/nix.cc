#include "carinae/include/nix.hh"

namespace carinae {
std::shared_ptr<nix::Store> nixOpenStore(rust::String uri) {
  return nix::openStore(std::string(uri));
}

rust::String nixStoreDir(std::shared_ptr<nix::Store> store) {
  return store->storeDir;
}

NixPathInfo nixPathInfoFromHashPart(std::shared_ptr<nix::Store> store,
                                    rust::String hash) {
  auto path = store->queryPathFromHashPart(std::string(hash));
  if (path) {
    auto pathinfo = store->queryPathInfo(*path);

    rust::Vec<rust::String> sigs;
    for (auto sig : pathinfo->sigs)
      sigs.push_back(rust::String(sig));

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
  return NixPathInfo{};
}

struct RustSink : nix::Sink {
public:
  rust::Box<NarContext> *ctx;
  rust::Fn<bool(NarContext &ctx, rust::Vec<rust::u8>)> *send;
  bool status;
  RustSink(rust::Box<NarContext> *ctx,
           rust::Fn<bool(NarContext &ctx, rust::Vec<rust::u8>)> *send,
           bool status)
      : ctx(ctx), send(send), status(status){};
  ~RustSink() {}
  void operator()(std::string_view data) {
    rust::Vec<rust::u8> ser;
    for (auto d : data)
      ser.push_back(d);
    status = (*send)(**ctx, ser);
  };
  bool good() { return status; }
};

void nixNarFromHashPart(
    std::shared_ptr<nix::Store> store, rust::String hash,
    rust::Box<NarContext> ctx,
    rust::Fn<bool(NarContext &ctx, rust::Vec<rust::u8>)> send) {
  auto path = store->queryPathFromHashPart(std::string(hash));
  if (path) {
    auto sink = RustSink{&ctx, &send, true};
    store->narFromPath(*path, sink);
  }
}

} // namespace carinae