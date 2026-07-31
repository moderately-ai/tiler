//! `tensor!` rejects input it has no grammar for, spanned at the offending
//! token, rather than expanding to something it guessed.

fn main() {
    let _region = tiler::tensor!(y = softmax(x));
}
