use proc_macro2::TokenTree;

pub(crate) fn inner_type<'a>(
    wrapper_ty: &str,
    ty: &'a syn::Type,
) -> std::option::Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper_ty {
            return std::option::Option::None;
        }

        if let syn::PathArguments::AngleBracketed(ref inner_ty_args) = p.path.segments[0].arguments
        {
            if inner_ty_args.args.len() != 1 {
                return std::option::Option::None;
            }

            let arg = inner_ty_args.args.first().unwrap();
            if let syn::GenericArgument::Type(ref inner_ty) = arg {
                return std::option::Option::Some(inner_ty);
            }
        }
    }
    std::option::Option::None
}

pub(crate) fn is_type(ty_name: &str, ty: &syn::Type) -> bool {
    if let syn::Type::Path(ref p) = ty {
        if p.path.segments.len() == 1 && p.path.segments[0].ident == ty_name {
            return true;
        }
    }
    false
}

pub(crate) fn is_type_vec_string(ty: &syn::Type) -> bool {
    if let Some(inner_of_vec) = inner_type("Vec", ty) {
        return is_type("String", inner_of_vec);
    }
    false
}

pub(crate) fn is_type_option_vec_string(ty: &syn::Type) -> bool {
    if let Some(inner_of_option) = inner_type("Option", ty) {
        if let Some(inner_of_option_vec) = inner_type("Vec", inner_of_option) {
            return is_type("String", inner_of_option_vec);
        }
    }
    false
}

pub(crate) fn is_type_option_string(ty: &syn::Type) -> bool {
    if let Some(inner_of_option) = inner_type("Option", ty) {
        return is_type("String", inner_of_option);
    }
    false
}

pub(crate) fn make_err<T: quote::ToTokens>(t: T, msg: &str) -> proc_macro2::TokenStream {
    syn::Error::new_spanned(t, msg).to_compile_error()
}

pub(crate) struct AttrData {
    pub(crate) instruction_name: syn::Ident,
    pub(crate) value_method: syn::Ident,
}

const EXPECT_ATTR_TEMPLATE: &str = r#"Expected 
#[instruction_builder(
    instruction_name = <name>, 
    value_method = <method>,
)]"#;

pub(crate) const EXPECT_EACH_ATTR_TEMPLATE: &str = r#"Expected 
#[instruction_builder(each = <arg>)]"#;

pub(crate) fn get_each_attr(
    attr: &Vec<syn::Attribute>,
    struct_ident: &syn::Ident,
) -> eyre::Result<syn::Ident, proc_macro2::TokenStream> {
    if attr.len() != 1 {
        return Err(make_err(struct_ident, EXPECT_EACH_ATTR_TEMPLATE));
    }
    if let syn::Meta::List(ref metalist) = &attr[attr.len() - 1].meta {
        let tokenstream = &mut metalist.tokens.clone().into_iter();

        verify_attr_ident(
            tokenstream.next(),
            "each",
            metalist,
            EXPECT_EACH_ATTR_TEMPLATE,
        )?;
        verify_attr_punct(tokenstream.next(), '=', metalist, EXPECT_EACH_ATTR_TEMPLATE)?;

        let each_ident = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(make_err(metalist, EXPECT_EACH_ATTR_TEMPLATE)),
        };

        Ok(each_ident)
    } else {
        Err(make_err(struct_ident, EXPECT_EACH_ATTR_TEMPLATE))
    }
}

pub(crate) fn get_attr(
    attr: &Vec<syn::Attribute>,
    struct_ident: &syn::Ident,
) -> eyre::Result<AttrData, proc_macro2::TokenStream> {
    if attr.is_empty() {
        return Err(make_err(struct_ident, EXPECT_ATTR_TEMPLATE));
    }

    // first attr can be doc
    if let syn::Meta::List(ref metalist) = &attr[attr.len() - 1].meta {
        let tokenstream = &mut metalist.tokens.clone().into_iter();

        verify_attr_ident(
            tokenstream.next(),
            "instruction_name",
            metalist,
            EXPECT_ATTR_TEMPLATE,
        )?;
        verify_attr_punct(tokenstream.next(), '=', metalist, EXPECT_ATTR_TEMPLATE)?;

        let instruction_name = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(make_err(metalist, EXPECT_ATTR_TEMPLATE)),
        };

        verify_attr_punct(tokenstream.next(), ',', metalist, EXPECT_ATTR_TEMPLATE)?;
        verify_attr_ident(
            tokenstream.next(),
            "value_method",
            metalist,
            EXPECT_ATTR_TEMPLATE,
        )?;
        verify_attr_punct(tokenstream.next(), '=', metalist, EXPECT_ATTR_TEMPLATE)?;

        let value_method = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(make_err(metalist, EXPECT_ATTR_TEMPLATE)),
        };

        Ok(AttrData {
            instruction_name,
            value_method,
        })
    } else {
        Err(make_err(struct_ident, EXPECT_ATTR_TEMPLATE))
    }
}

pub(crate) fn verify_attr_ident<T: quote::ToTokens>(
    token: Option<TokenTree>,
    expected_ident: &str,
    span: T,
    err_msg: &str,
) -> eyre::Result<(), proc_macro2::TokenStream> {
    match token {
        Some(TokenTree::Ident(ref i)) => {
            if i != expected_ident {
                return Err(make_err(span, err_msg));
            }
        }
        _ => return Err(make_err(span, err_msg)),
    }
    Ok(())
}

pub(crate) fn verify_attr_punct<T: quote::ToTokens>(
    token: Option<TokenTree>,
    expected_punct: char,
    span: T,
    err_msg: &str,
) -> eyre::Result<(), proc_macro2::TokenStream> {
    match token {
        Some(TokenTree::Punct(ref p)) => {
            if p.as_char() != expected_punct {
                return Err(make_err(span, err_msg));
            }
        }
        _ => return Err(make_err(span, err_msg)),
    }
    Ok(())
}

pub(crate) fn make_title_case(s: String) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
