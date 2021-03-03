//
// Copyright 2021 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use proc_macro2::TokenStream as TokenStream2;
use quote::*;
use syn::spanned::Spanned;
use syn::*;
use syn_mid::{FnArg, Pat, PatType, Signature};
use unzip3::Unzip3;

use crate::ResultKind;

pub(crate) fn bridge_fn(name: String, sig: &Signature, result_kind: ResultKind) -> TokenStream2 {
    let name = format_ident!("Java_org_signal_client_internal_Native_{}", name);

    let (env_arg, output) = match (result_kind, &sig.output) {
        (ResultKind::Regular, ReturnType::Default) => (quote!(), quote!()),
        (ResultKind::Regular, ReturnType::Type(_, ref ty)) => {
            (quote!(), quote!(-> jni_result_type!(#ty)))
        }
        (ResultKind::Void, _) => (quote!(), quote!()),
        (ResultKind::Buffer, ReturnType::Type(_, _)) => (quote!(&env,), quote!(-> jni::jbyteArray)),
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
                ty,
            }) => (
                name.ident.clone(),
                quote!(#(#attrs)* #name #colon_token jni_arg_type!(#ty)),
                quote! {
                    let mut #name = <#ty as jni::ArgTypeInfo>::borrow(&env, #name)?;
                    let #name = <#ty as jni::ArgTypeInfo>::load_from(&env, &mut #name)?
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
            env: jni::JNIEnv,
            _class: jni::JClass,
            #(#input_args),*
        ) #output {
            jni::run_ffi_safe(&env, || {
                #(#input_processing);*;
                let __result = #orig_name(#env_arg #(#input_names),*);
                #await_if_needed;
                jni::ResultTypeInfo::convert_into(__result, &env)
            })
        }
    }
}

pub(crate) fn name_from_ident(ident: &Ident) -> String {
    ident.to_string().replace("_", "_1")
}
