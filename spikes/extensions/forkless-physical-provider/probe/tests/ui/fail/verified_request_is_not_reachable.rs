// Reserved subject 1 of 5: the verified target request.
//
// `ImplementationContext` carries the verified request the region belongs to,
// and a provider cannot read it. That is what stops a provider re-deriving the
// host's normalization and disagreeing with it: the request-subject binding
// compares a proposed body against the *compiler's* normalization, so a second
// implementation of it in a provider crate would be a second answer to a
// question that must have one.
//
// What a provider reads instead is stated positively rather than left to be
// inferred from this refusal, and the compiling contrast beside it names all of
// it: `pass/provider_vocabulary_is_publicly_reachable.rs`.

use tiler_compiler::physical_provider::ImplementationContext;

fn read(context: &ImplementationContext<'_>) {
    let _ = context.request();
}

fn main() {
    let _: fn(&ImplementationContext<'_>) = read;
}
