//! Derive macro for `ratatui-form`.
//!
//! `#[derive(FormModel)]` turns a struct into a form model: it generates a
//! [`FormModel`] impl with a `get_form` method that builds a form seeded with
//! the struct's current values, and a `TryFrom<Form<T>>` impl that
//! converts the edited form back into the struct.
//!
//! Fields are addressed by their position in the form, in struct order
//! (skipped fields are excluded).
//!
//! Field construction and extraction are delegated to the
//! [`FormValue`](https://docs.rs/ratatui-form/latest/ratatui_form/trait.FormValue.html)
//! trait in the `ratatui-form` crate, so any type can be used as a field as
//! long as `FormValue` is implemented for it. The library implements it for
//! `String`, `Option<String>`, `bool`, `std::net::Ipv4Addr`,
//! `std::net::Ipv6Addr`, and the numeric types (`u8`..`u64`, `i8`..`i64`,
//! `f32`, `f64`, and their size variants); implement it yourself to add
//! completely custom types.
//!
//! Field attributes:
//! - `#[form(label = "…")]` — display label (defaults to the humanized name)
//! - `#[form(placeholder = "…")]` — text input placeholder
//! - `#[form(required)]` — mark the field as required
//! - `#[form(validate = path)]` — a `fn(&str) -> bool` validator (repeatable)
//! - `#[form(skip)]` — exclude the field from the form; restored via `Default`
//!
//! Struct attribute:
//! - `#[form(title = "…")]` — the form title (defaults to the struct name)

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, Data, DeriveInput, Expr, Field, Fields,
    Ident, Lit, Meta, Token,
};

/// Path used to refer to the `ratatui-form` crate in generated code.
fn crate_path() -> TokenStream2 {
    match crate_name("ratatui-form") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        // Within ratatui-form's own examples/tests the derive is "itself", but
        // those are separate crates; reference the library by its lib name.
        Ok(FoundCrate::Itself) | Err(_) => quote!(::ratatui_form),
    }
}

/// Derives `FormModel` for a struct.
#[proc_macro_derive(FormModel, attributes(form))]
pub fn derive_form_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_form_model(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn impl_form_model(input: &DeriveInput) -> syn::Result<TokenStream2> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "FormModel does not support generic structs",
        ));
    }

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "FormModel only supports structs with named fields",
                ))
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "FormModel only supports structs",
            ))
        }
    };

    let ident = &input.ident;
    let krate = crate_path();
    let title = match parse_struct_title(&input.attrs)? {
        Some(title) => title,
        None => humanize(&ident.to_string()),
    };

    let mut pushes: Vec<TokenStream2> = Vec::new();
    let mut extractions: Vec<TokenStream2> = Vec::new();
    let mut assignments: Vec<TokenStream2> = Vec::new();
    let mut index = 0;

    for field in fields {
        let Some(fident) = &field.ident else {
            return Err(syn::Error::new_spanned(field, "unnamed struct field"));
        };
        let attrs = parse_field_attrs(&field.attrs)?;

        if attrs.skip {
            let ty = &field.ty;
            extractions.push(quote!(let #fident: #ty = Default::default();));
        } else {
            let label = attrs
                .label
                .clone()
                .unwrap_or_else(|| humanize(&fident.to_string()));
            pushes.push(build_field(field, &label, &attrs, &krate)?);
            extractions.push(build_extraction(field, index, ident, &krate)?);
            index += 1;
        }

        assignments.push(quote!(#fident));
    }

    let form_model_impl = quote! {
        impl #krate::FormModel for #ident {
            fn get_form(&self) -> #krate::Form<Self> {
                let mut form = #krate::Form::<Self>::new(#title);
                #(#pushes)*
                form
            }
        }
    };

    let try_from_impl = quote! {
        impl std::convert::TryFrom<#krate::Form<#ident>> for #ident {
            type Error = Vec<#krate::FormExtractError>;

            fn try_from(form: #krate::Form<#ident>) -> Result<Self, Self::Error> {
                #(#extractions)*
                Ok(Self { #(#assignments,)* })
            }
        }
    };

    Ok(quote! {
        #form_model_impl
        #try_from_impl
    })
}

fn parse_struct_title(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    for attr in attrs {
        if !attr.path().is_ident("form") {
            continue;
        }
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            if let Meta::NameValue(nv) = &meta {
                if nv.path.is_ident("title") {
                    return expr_to_string(&nv.value).map(Some);
                }
            }
        }
    }
    Ok(None)
}

#[derive(Default)]
struct FieldAttrs {
    label: Option<String>,
    placeholder: Option<String>,
    required: bool,
    validators: Vec<Expr>,
    skip: bool,
}

fn parse_field_attrs(attrs: &[Attribute]) -> syn::Result<FieldAttrs> {
    let mut out = FieldAttrs::default();

    for attr in attrs {
        if !attr.path().is_ident("form") {
            continue;
        }
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("required") => out.required = true,
                Meta::Path(path) if path.is_ident("skip") => out.skip = true,
                Meta::NameValue(nv) if nv.path.is_ident("label") => {
                    out.label = Some(expr_to_string(&nv.value)?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("placeholder") => {
                    out.placeholder = Some(expr_to_string(&nv.value)?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("validate") => {
                    out.validators.push(nv.value);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unsupported #[form(...)] attribute",
                    ))
                }
            }
        }
    }

    Ok(out)
}

fn expr_to_string(expr: &Expr) -> syn::Result<String> {
    if let Expr::Lit(lit) = expr {
        if let Lit::Str(s) = &lit.lit {
            return Ok(s.value());
        }
    }
    Err(syn::Error::new_spanned(
        expr,
        "expected a string literal, e.g. `label = \"Name\"`",
    ))
}

fn build_field(
    field: &Field,
    label: &str,
    attrs: &FieldAttrs,
    krate: &TokenStream2,
) -> syn::Result<TokenStream2> {
    let Some(field_ident) = &field.ident else {
        return Err(syn::Error::new_spanned(field, "unnamed struct field"));
    };

    let mut validators: Vec<TokenStream2> = Vec::new();
    for validator in &attrs.validators {
        let validator = validator_expr(validator)?;
        validators.push(quote!(Box::new(#validator)));
    }

    let placeholder = match &attrs.placeholder {
        Some(placeholder) => quote!(::core::option::Option::Some(#placeholder.to_string())),
        None => quote!(::core::option::Option::None),
    };
    let required = attrs.required;

    Ok(quote! {
        form.push(#krate::FormValue::form_field(
            #krate::FieldSpec {
                label: #label.to_string(),
                placeholder: #placeholder,
                required: #required,
                validators: ::std::vec![#(#validators),*],
            },
            &self.#field_ident,
        ));
    })
}

fn build_extraction(
    field: &Field,
    index: usize,
    model: &Ident,
    krate: &TokenStream2,
) -> syn::Result<TokenStream2> {
    let Some(fident) = &field.ident else {
        return Err(syn::Error::new_spanned(field, "unnamed struct field"));
    };
    let ty = &field.ty;
    let idx = syn::Index::from(index);

    Ok(quote! {
        let #fident: #ty = match #krate::FormValue::form_extract::<#model>(&form, #idx) {
            Ok(value) => value,
            Err(err) => return Err(::std::vec![err]),
        };
    })
}

fn validator_expr(expr: &Expr) -> syn::Result<TokenStream2> {
    if let Expr::Path(_) = expr {
        Ok(quote!(#expr as fn(&str) -> bool))
    } else {
        Err(syn::Error::new_spanned(
            expr,
            "validate must be a function path, e.g. `validate = is_valid_email`",
        ))
    }
}

fn humanize(name: &str) -> String {
    let mut out = String::new();
    let mut capitalize = true;
    for c in name.chars() {
        if c == '_' {
            out.push(' ');
            capitalize = true;
        } else if capitalize {
            out.extend(c.to_uppercase());
            capitalize = false;
        } else {
            out.push(c);
        }
    }
    out
}
