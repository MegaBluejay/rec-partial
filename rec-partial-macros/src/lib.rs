use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, DeriveInput, Error, Field, Fields, Ident, Meta, Path, Token, Type, WherePredicate,
    parenthesized, parse_quote, punctuated::Punctuated, spanned::Spanned,
};

#[proc_macro_derive(HasPartial, attributes(partial))]
pub fn derive_has_partial(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    doit(input.into())
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn doit(tokens: TokenStream) -> syn::Result<TokenStream> {
    let mut input: DeriveInput = syn::parse2(tokens)?;

    let mut derive_paths: Punctuated<Path, Token![,]> = Punctuated::new();
    let mut all = vec![];

    for attr in input.attrs.drain(..) {
        if !attr.path().is_ident("partial") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("derive") {
                let content;
                parenthesized!(content in meta.input);
                let paths = Punctuated::<Path, Token![,]>::parse_terminated(&content)?;
                derive_paths.extend(paths.into_pairs());

                return Ok(());
            }

            if meta.path.is_ident("all") {
                let content;
                parenthesized!(content in meta.input);
                let meta: Meta = content.parse()?;
                all.push(parse_quote! {
                    #[#meta]
                });

                return Ok(());
            }

            Err(meta.error("unrecognized partial"))
        })?;
    }

    let derive = if derive_paths.is_empty() {
        None
    } else {
        Some(parse_quote! {
            #[derive(#derive_paths)]
        })
    };

    input.attrs.extend(derive);

    let mut tys = HashSet::new();

    match &mut input.data {
        syn::Data::Struct(s) => {
            do_fields(&mut s.fields, &mut tys, all.as_ref());
        }
        syn::Data::Enum(e) => {
            for v in &mut e.variants {
                v.attrs.clear();
                do_fields(&mut v.fields, &mut tys, all.as_ref());
            }
        }
        syn::Data::Union(u) => {
            return Err(syn::Error::new(u.union_token.span(), "union not supported"));
        }
    }

    input
        .generics
        .make_where_clause()
        .predicates
        .extend(tys.into_iter().map(|ty| {
            let pred: WherePredicate = parse_quote! { #ty: ::rec_partial::HasPartial };
            pred
        }));

    let partial_ident = Ident::new(&format!("Partial{}", input.ident), input.ident.span());
    let ident = std::mem::replace(&mut input.ident, partial_ident);
    let partial_ident = &input.ident;

    let (imp, ty, whr) = input.generics.split_for_impl();

    Ok(quote! {
        #input

        impl #imp ::rec_partial::HasPartial for #ident #ty #whr {
            type Partial = #partial_ident #ty;
        }
    })
}

fn do_fields(fields: &mut Fields, tys: &mut HashSet<Type>, all: &[Attribute]) {
    match fields {
        Fields::Named(n) => {
            for field in &mut n.named {
                do_field(field, tys, all);
            }
        }
        Fields::Unnamed(u) => {
            for field in &mut u.unnamed {
                do_field(field, tys, all);
            }
        }
        Fields::Unit => {}
    }
}

fn do_field(field: &mut Field, tys: &mut HashSet<Type>, all: &[Attribute]) {
    field.attrs.clear();
    field.attrs.extend(all.iter().cloned());

    let og_ty = &field.ty;
    let new_ty = parse_quote! {
        ::core::option::Option<<#og_ty as ::rec_partial::HasPartial>::Partial>
    };
    tys.insert(std::mem::replace(&mut field.ty, new_ty));
}
