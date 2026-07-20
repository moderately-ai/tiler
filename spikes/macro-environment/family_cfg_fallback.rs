fn main() {
    let selected = {
        #[cfg(target_os = "ios")]
        const _: () = {
            compile_error!("selected iOS artifact family could not be built");
        };
        "semantic-fallback"
    };
    assert_eq!(selected, "semantic-fallback");
}
