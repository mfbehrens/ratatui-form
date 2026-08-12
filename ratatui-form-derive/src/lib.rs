//! Derive macro for `ratatui-form`.
//!
//! `#[derive(FormModel)]` turns a struct into a form model: it generates a
//! [`FormModel`] impl with a `get_form` method that builds a form seeded with
//! the struct's current values, and a `TryFrom<TypedForm<T>>` impl that
//! converts the edited form back into the struct.
//!
//! Fields are addressed by their position in the form, in struct order
//! (skipped fields are excluded).
//!
//! Supported field types: `String`, `Option<String>`, `bool`, `std::net::Ipv4Addr`,
//! and the numeric types (`u8`..`u64`, `i8`..`i64`, `f32`, `f64`, and their size
//! variants). IPv4 fields are required and validated.
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
    GenericArgument, Ident, Lit, Meta, PathArguments, Token, Type, TypePath,
};

/// The number types supported as text fields.
const NUMBER_TYPES: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64",
];

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
            extractions.push(build_extraction(field, index, &attrs, &krate)?);
            index += 1;
        }

        assignments.push(quote!(#fident));
    }

    let form_model_impl = quote! {
        impl #krate::FormModel for #ident {
            fn get_form(&self) -> #krate::TypedForm<Self> {
                let mut form = #krate::TypedForm::<Self>::new(#title);
                #(#pushes)*
                form
            }
        }
    };

    let try_from_impl = quote! {
        impl std::convert::TryFrom<#krate::TypedForm<#ident>> for #ident {
            type Error = Vec<#krate::FormExtractError>;

            fn try_from(form: #krate::TypedForm<#ident>) -> Result<Self, Self::Error> {
                let mut errors: Vec<#krate::FormExtractError> = Vec::new();
                #(#extractions)*

                if errors.is_empty() {
                    Ok(Self { #(#assignments,)* })
                } else {
                    Err(errors)
                }
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

enum FieldKind {
    String,
    OptionString,
    Bool,
    Number,
    Ipv4,
}

fn field_kind(field: &Field, attrs: &FieldAttrs) -> syn::Result<FieldKind> {
    if attrs.skip {
        return Ok(FieldKind::String); // arbitrary; only used for the "supported" check below
    }

    let ty = &field.ty;
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return Err(unsupported(ty));
    };

    let segment = path
        .segments
        .last()
        .ok_or_else(|| syn::Error::new_spanned(ty, "expected a concrete field type"))?;
    let name = segment.ident.to_string();

    match name.as_str() {
        "String" if path.segments.len() == 1 => Ok(FieldKind::String),
        "bool" if path.segments.len() == 1 => Ok(FieldKind::Bool),
        "Option" => {
            let inner = option_inner(segment)?;
            if is_simple_type(inner, "String") {
                Ok(FieldKind::OptionString)
            } else {
                Err(unsupported(ty))
            }
        }
        n if NUMBER_TYPES.contains(&n) => Ok(FieldKind::Number),
        "Ipv4Addr" => Ok(FieldKind::Ipv4),
        _ => Err(unsupported(ty)),
    }
}

fn option_inner(segment: &syn::PathSegment) -> syn::Result<&Type> {
    match &segment.arguments {
        PathArguments::AngleBracketed(args) => match args.args.first() {
            Some(GenericArgument::Type(inner)) => Ok(inner),
            _ => Err(syn::Error::new_spanned(segment, "expected `Option<T>`")),
        },
        _ => Err(syn::Error::new_spanned(segment, "expected `Option<T>`")),
    }
}

fn is_simple_type(ty: &Type, name: &str) -> bool {
    if let Type::Path(TypePath { qself: None, path }) = ty {
        path.segments.len() == 1 && path.segments[0].ident == name
    } else {
        false
    }
}

fn unsupported(ty: &Type) -> syn::Error {
    syn::Error::new_spanned(
        ty,
        "unsupported field type for FormModel (supported: String, Option<String>, bool, std::net::Ipv4Addr, and numeric types)",
    )
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
    let kind = field_kind(field, attrs)?;

    let mut ts = match kind {
        FieldKind::String | FieldKind::OptionString | FieldKind::Number | FieldKind::Ipv4 => {
            quote!(#krate::TextInput::new(#label))
        }
        FieldKind::Bool => quote!(#krate::Checkbox::new(#label)),
    };

    if let Some(placeholder) = &attrs.placeholder {
        if !matches!(kind, FieldKind::Bool) {
            ts = quote!(#ts.placeholder(#placeholder));
        }
    }

    match kind {
        FieldKind::String | FieldKind::Bool => {
            if attrs.required {
                ts = quote!(#ts.required());
            }
        }
        FieldKind::Number => {
            ts = quote!(#ts.required().validator(Box::new(#krate::Numeric)));
        }
        FieldKind::Ipv4 => {
            ts = quote!(#ts.required().validator(Box::new(#krate::Ipv4)));
        }
        FieldKind::OptionString => {}
    }

    for validator in &attrs.validators {
        let validator = validator_expr(validator)?;
        ts = quote!(#ts.validator(Box::new(#validator)));
    }

    ts = match kind {
        FieldKind::String => quote!(#ts.initial_value(self.#field_ident.clone())),
        FieldKind::OptionString => {
            quote!(#ts.initial_value(self.#field_ident.clone().unwrap_or_default()))
        }
        FieldKind::Bool => quote!(#ts.checked(self.#field_ident)),
        FieldKind::Number => quote!(#ts.initial_value(self.#field_ident.to_string())),
        FieldKind::Ipv4 => quote!(#ts.initial_value(self.#field_ident.to_string())),
    };

    Ok(quote!(form.push(Box::new(#ts));))
}

fn build_extraction(
    field: &Field,
    index: usize,
    attrs: &FieldAttrs,
    krate: &TokenStream2,
) -> syn::Result<TokenStream2> {
    let Some(fident) = &field.ident else {
        return Err(syn::Error::new_spanned(field, "unnamed struct field"));
    };
    let kind = field_kind(field, attrs)?;
    let idx = syn::Index::from(index);

    let missing = quote! {
        errors.push(#krate::FormExtractError {
            field_index: #idx,
            message: "field not found in form".to_string(),
        });
    };

    Ok(match kind {
        FieldKind::String => quote! {
            let #fident: String = match form.value_str(#idx) {
                Some(value) => value,
                None => { #missing Default::default() }
            };
        },
        FieldKind::OptionString => quote! {
            let #fident: Option<String> = match form.value_str(#idx) {
                Some(value) if value.is_empty() => None,
                Some(value) => Some(value),
                None => None,
            };
        },
        FieldKind::Bool => quote! {
            let #fident: bool = match form.value_bool(#idx) {
                Some(value) => value,
                None => {
                    errors.push(#krate::FormExtractError {
                        field_index: #idx,
                        message: "expected a boolean".to_string(),
                    });
                    false
                }
            };
        },
        FieldKind::Number => {
            let ty = &field.ty;
            quote! {
                let #fident: #ty = match form.value_str(#idx) {
                    Some(value) => match value.parse::<#ty>() {
                        Ok(n) => n,
                        Err(_) => {
                            errors.push(#krate::FormExtractError {
                                field_index: #idx,
                                message: "expected a number".to_string(),
                            });
                            Default::default()
                        }
                    },
                    None => { #missing Default::default() }
                };
            }
        }
        FieldKind::Ipv4 => {
            let ty = &field.ty;
            quote! {
                let #fident: #ty = match form.value_str(#idx) {
                    Some(value) => match value.parse::<#ty>() {
                        Ok(ip) => ip,
                        Err(_) => {
                            errors.push(#krate::FormExtractError {
                                field_index: #idx,
                                message: "expected an IPv4 address".to_string(),
                            });
                            std::net::Ipv4Addr::UNSPECIFIED
                        }
                    },
                    None => { #missing std::net::Ipv4Addr::UNSPECIFIED }
                };
            }
        }
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
