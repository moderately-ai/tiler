fn main() {
    let _ = {
        #[cfg(target_os = "macos")]
        const _: () = {
            compile_error!("selected macOS artifact family could not be built");
        };
        "semantic-fallback"
    };
}
