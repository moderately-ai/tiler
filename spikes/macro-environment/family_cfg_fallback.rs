fn main() {
    let selected = {
        #[cfg(target_os = "ios")]
        const _: () = {
            compile_error!("selected iOS artifact family could not be built");
        };
        "semantic-fallback"
    };
    if selected != "semantic-fallback" {
        eprintln!("nonmatching family did not select the semantic fallback");
        std::process::exit(1);
    }
}
