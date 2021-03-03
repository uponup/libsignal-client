//
// Copyright 2021 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use heck::SnakeCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::spanned::Spanned;
use syn::*;
use syn_mid::{FnArg, Pat, PatType, Signature};
use unzip3::Unzip3;

use crate::ResultKind;

pub(crate) fn bridge_fn(name: String, sig: &Signature, result_kind: ResultKind) -> TokenStream2 {
    let name = format_ident!("signal_{}", name);

    let (output_args, env_arg, output_processing) = match (result_kind, &sig.output) {
        (ResultKind::Regular, ReturnType::Default) => (quote!(), quote!(), quote!()),
        (ResultKind::Regular, ReturnType::Type(_, ref ty)) => (
            quote!(out: *mut ffi_result_type!(#ty),), // note the trailing comma
            quote!(),
            quote!(ffi::write_result_to(out, __result)?),
        ),
        (ResultKind::Void, ReturnType::Default) => (quote!(), quote!(), quote!()),
        (ResultKind::Void, ReturnType::Type(_, _)) => (quote!(), quote!(), quote!(__result?;)),
        (ResultKind::Buffer, ReturnType::Type(_, _)) => (
            quote!(
                out: *mut *const libc::c_uchar,
                out_len: *mut libc::size_t, // note the trailing comma
            ),
            quote!(ffi::Env,), // note the trailing comma
            quote!(ffi::write_bytearray_to(out, out_len, __result?)?),
        ),
        (ResultKind::Buffer, ReturnType::Default) => {
            return Error::new(
                sig.paren_token.span,
                "missing result type for bridge_fn_buffer",
            )
            .to_compile_error()
        }
    };

    let await_if_needed = sig.asyncness.map(|_| {
        quote! {
            let __result = expect_ready(__result);
        }
    });

    let (input_names, input_args, input_processing): (Vec<_>, Vec<_>, Vec<_>) = sig
        .inputs
        .iter()
        .skip(if result_kind.has_env() { 1 } else { 0 })
        .map(|arg| match arg {
            FnArg::Receiver(tokens) => (
                Ident::new("self", tokens.self_token.span),
                Error::new(tokens.self_token.span, "cannot have 'self' parameter")
                    .to_compile_error(),
                quote!(),
            ),
            FnArg::Typed(PatType {
                attrs,
                pat: box Pat::Ident(name),
                colon_token,
                ty:
                    ty
                    @
                    box Type::Reference(TypeReference {
                        elem: box Type::Slice(_),
                        ..
                    }),
            }) => {
                let size_arg = format_ident!("{}_len", name.ident);
                (
                    name.ident.clone(),
                    quote!(
                        #(#attrs)* #name #colon_token ffi_arg_type!(#ty),
                        #size_arg: libc::size_t
                    ),
                    quote!(
                        let #name = <#ty as ffi::SizedArgTypeInfo>::convert_from(#name, #size_arg)?
                    ),
                )
            }
            FnArg::Typed(PatType {
                attrs,
                pat: box Pat::Ident(name),
                colon_token,
                ty,
            }) => (
                name.ident.clone(),
                quote!(#(#attrs)* #name #colon_token ffi_arg_type!(#ty)),
                quote! {
                    let mut #name = <#ty as ffi::ArgTypeInfo>::borrow(#name)?;
                    let #name = <#ty as ffi::ArgTypeInfo>::load_from(&mut #name)?
                },
            ),
            FnArg::Typed(PatType { pat, .. }) => (
                Ident::new("unexpected", pat.span()),
                Error::new(pat.span(), "cannot use patterns in paramater").to_compile_error(),
                quote!(),
            ),
        })
        .unzip3();

    let orig_name = sig.ident.clone();

    quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #name(
            #output_args
            #(#input_args),*
        ) -> *mut ffi::SignalFfiError {
            ffi::run_ffi_safe(|| {
                #(#input_processing);*;
                let __result = #orig_name(#env_arg #(#input_names),*);
                #await_if_needed;
                #output_processing;
                Ok(())
            })
        }
    }
}

pub(crate) fn name_from_ident(ident: &Ident) -> String {
    ident.to_string().to_snake_case()
}
