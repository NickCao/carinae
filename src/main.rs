#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("carinae/include/nix.hh");
        fn nixOpenStore(uri: String) -> String;
    }
}

fn main() {
    println!("{}", crate::ffi::nixOpenStore("daemon".to_string()));
}
