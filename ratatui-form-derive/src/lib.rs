//! Derive macro for `ratatui-form`.
//!
//! `#[derive(TypedForm)]` turns a struct into a form model: it generates a
//! submodule containing a `Fields` struct that holds typed form input controls,
//! implements `FormFields`, `TypedForm`, and `TryFrom<Form<Fields>>` to convert
//! the edited form back into the struct.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Attribute, Data, DeriveInput, Expr, Fields, Ident,
    Lit, Meta, Token,
};

/// Path used to refer to the `ratatui-form` crate in generated code.
fn crate_path() -> TokenStream2 {
    match crate_name("ratatui-form") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
        Ok(FoundCrate::Itself) | Err(_) => quote!(::ratatui_form),
    }
}

/// Derives `TypedForm` for a struct.
#[proc_macro_derive(TypedForm, attributes(form))]
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
            "TypedForm does not support generic structs",
        ));
    }

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    input,
                    "TypedForm only supports structs with named fields",
                ))
            }
        },
        Data::Enum(_) | Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "TypedForm only supports structs",
            ))
        }
    };

    let ident = &input.ident;
    let krate = crate_path();
    let title = match parse_struct_title(&input.attrs)? {
        Some(title) => title,
        None => humanize(&ident.to_string()),
    };

    let mod_name = format!("{}_form", to_snake_case(&ident.to_string()));
    let mod_ident = Ident::new(&mod_name, ident.span());

    let mut struct_fields: Vec<TokenStream2> = Vec::new();
    let mut mut_refs: Vec<TokenStream2> = Vec::new();
    let mut immut_refs: Vec<TokenStream2> = Vec::new();
    let mut field_inits: Vec<TokenStream2> = Vec::new();
    let mut extractions: Vec<TokenStream2> = Vec::new();
    let mut assignments: Vec<TokenStream2> = Vec::new();
    let mut index = 0usize;

    for field in fields {
        let Some(fident) = &field.ident else {
            return Err(syn::Error::new_spanned(field, "unnamed struct field"));
        };
        let attrs = parse_field_attrs(&field.attrs)?;
        let ty = &field.ty;

        if attrs.skip {
            assignments.push(quote!(#fident: ::core::default::Default::default()));
        } else {
            let label = attrs
                .label
                .clone()
                .unwrap_or_else(|| humanize(&fident.to_string()));

            let spec_expr = build_field_spec(label, &attrs)?;

            struct_fields.push(quote!(pub #fident: <#ty as #krate::FieldType>::BaseFieldType));
            mut_refs.push(quote!(&mut self.#fident));
            immut_refs.push(quote!(&self.#fident));
            field_inits.push(
                quote!(#fident: <#ty as #krate::FieldType>::form_field(#spec_expr, &self.#fident)),
            );

            let fname_str = fident.to_string();
            let idx_lit = syn::Index::from(index);

            extractions.push(quote! {
                let #fident = match <#ty as #krate::FieldType>::form_extract(&form.fields.#fident) {
                    Ok(value) => ::core::option::Option::Some(value),
                    Err(msg) => {
                        errors.push(#krate::FormExtractError::new(#idx_lit, #fname_str, msg));
                        ::core::option::Option::None
                    }
                };
            });

            assignments.push(quote!(#fident: #fident.unwrap()));
            index += 1;
        }
    }

    let form_fields_mod = quote! {
        #[allow(non_snake_case)]
        pub mod #mod_ident {
            use super::*;

            pub struct Fields {
                #(#struct_fields,)*
            }

            impl #krate::FormFields for Fields {
                fn fields_mut(&mut self) -> ::std::vec::Vec<&mut dyn #krate::BasicFieldType> {
                    ::std::vec![#(#mut_refs),*]
                }

                fn fields(&self) -> ::std::vec::Vec<&dyn #krate::BasicFieldType> {
                    ::std::vec![#(#immut_refs),*]
                }
            }
        }
    };

    let form_model_impl = quote! {
        impl #krate::TypedForm for #ident {
            type Fields = #mod_ident::Fields;

            fn get_form(&self) -> #krate::Form<Self::Fields> {
                let fields = #mod_ident::Fields {
                    #(#field_inits,)*
                };
                #krate::Form::new(#title, fields)
            }
        }
    };

    let try_from_impl = quote! {
        impl ::std::convert::TryFrom<#krate::Form<#mod_ident::Fields>> for #ident {
            type Error = ::std::vec::Vec<#krate::FormExtractError>;

            fn try_from(form: #krate::Form<#mod_ident::Fields>) -> Result<Self, Self::Error> {
                let mut errors = ::std::vec::Vec::new();

                #(#extractions)*

                if !errors.is_empty() {
                    return Err(errors);
                }

                Ok(Self {
                    #(#assignments,)*
                })
            }
        }
    };

    Ok(quote! {
        #form_fields_mod
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

fn build_field_spec(label: String, attrs: &FieldAttrs) -> syn::Result<TokenStream2> {
    let krate = crate_path();
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
        #krate::FieldAttributes {
            label: #label.to_string(),
            placeholder: #placeholder,
            required: #required,
            validators: ::std::vec![#(#validators),*],
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

fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn humanize(name: &str) -> String {
    name.split('_')
        .filter(|word| !word.is_empty())
        .map(humanize_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn humanize_word(word: &str) -> String {
    let is_initialism = word.len() <= 3 && word.chars().all(|c| c.is_ascii_lowercase());
    if is_initialism {
        word.to_ascii_uppercase()
    } else {
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{humanize, to_snake_case};

    #[test]
    fn converts_snake_case() {
        assert_eq!(to_snake_case("Signup"), "signup");
        assert_eq!(to_snake_case("ServerConfig"), "server_config");
    }

    #[test]
    fn separates_snake_case_words() {
        assert_eq!(humanize("full_name"), "Full Name");
    }

    #[test]
    fn capitalizes_single_words() {
        assert_eq!(humanize("name"), "Name");
        assert_eq!(humanize("email"), "Email");
    }

    #[test]
    fn uppercases_short_initialisms() {
        assert_eq!(humanize("ip"), "IP");
        assert_eq!(humanize("id"), "ID");
        assert_eq!(humanize("ip_address"), "IP Address");
        assert_eq!(humanize("api_key"), "API KEY");
    }

    #[test]
    fn leaves_longer_words_alone() {
        assert_eq!(humanize("ipv6"), "Ipv6");
        assert_eq!(humanize("user"), "User");
    }

    #[test]
    fn handles_empty_and_underscores() {
        assert_eq!(humanize(""), "");
        assert_eq!(humanize("_"), "");
        assert_eq!(humanize("a_b_c"), "A B C");
    }
}
