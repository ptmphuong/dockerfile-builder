use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(InstructionInit)]
pub fn instruction_init(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let instruction = &input.ident;

    let variants = match input.data {
        syn::Data::Enum(syn::DataEnum { ref variants, .. }, .. ) => variants,
        _ => return make_err(&input.ident, "Expected Enum").into(),
    };

    let impl_display = variants.iter().map(|v| {
        let variant = &v.ident;
        quote! {
            #instruction::#variant(ins) => write!(f, "{}", ins),
        }
    });

    let impl_convert_from_for_instruction = variants.iter()
        .filter(|v| &v.ident != "ANY")
        .map(|v| {
            let variant = &v.ident;
            quote! {
                impl std::convert::From<#variant> for #instruction {
                    fn from(instruction: #variant) -> Self {
                        Instruction::#variant(instruction)
                    }
                }
        }
    });

    let variant_init = variants.iter()
        .filter(|v| &v.ident != "ANY")
        .map(|v| {
            let variant = &v.ident;
            quote! {
                #[derive(Debug, Clone, Eq, PartialEq)]
                pub struct #variant {
                    pub value: String,
                }
            }
        }
    );

    let impl_convert_from_for_variant = variants.iter()
        .filter(|v| &v.ident != "ANY")
        .map(|v| {
            let variant = &v.ident;
            quote! {
                impl<T> std::convert::From<T> for #variant where T: Into<String> {
                    fn from(value: T) -> Self {
                        #variant { 
                            value: value.into(),
                        }
                    }
                }
            }
        }
    );

    let impl_display_for_variant = variants.iter()
        .filter(|v| &v.ident != "ANY")
        .map(|v| {
            let variant = &v.ident;
            let variant_string = &variant.to_string().to_uppercase();
            quote! {
                impl std::fmt::Display for #variant {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        write!(f, "{} {}", #variant_string, self.value)
                    }
                }
            }
        }
    );

    quote! {
        impl std::fmt::Display for #instruction {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#impl_display)*
                }
            } 
        }

        #(#impl_convert_from_for_instruction)*

        #(#variant_init
          #impl_convert_from_for_variant
          #impl_display_for_variant
        )*
    }.into()
}

#[proc_macro_derive(InstructionBuilder, attributes(instruction_builder))]
pub fn instruction_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_ident = &input.ident;
    let builder_ident = syn::Ident::new(
        &format!("{}Inner", struct_ident), 
        struct_ident.span()
    );

    let attr = &input.attrs;
    let (instruction_name, value_method) = match get_attr(attr, struct_ident) {
        Ok(ad) => (ad.instruction_name, ad.value_method),
        Err(e) => return e.into(),
    };

    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => return make_err(struct_ident, "Expected Struct with named fields").into(),
    };

    let builder_empty = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: std::option::Option::None, }
    });

    // Add Option wrapper if the given type isn't already Option.
    let builder_field = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if inner_type("Option", ty).is_some() {
            quote! { #name: #ty, }
        } else {
            quote! { #name: std::option::Option<#ty>, }
        }
    });

    let builder_set_method = fields.iter().map(|f| {
        let name = &f.ident;

        let original_ty = &f.ty;
        let inner_ty = inner_type("Option", original_ty);
        let set_ty = inner_ty.unwrap_or(original_ty);

        if is_type("String", set_ty) {
            quote! { 
                pub fn #name<T: Into<String>>(&mut self, #name: T) -> &mut Self {
                    self.#name = Some(#name.into());
                    self
                }
            }

        } else {
            quote! { 
                pub fn #name(&mut self, #name: #set_ty) -> &mut Self {
                    self.#name = Some(#name);
                    self
                }
            }
        }
    });

    let builder_set_each_method = fields.iter().map(|f| {
        if !&f.attrs.is_empty() {
            if f.attrs.len() != 1 {
                return make_err(&f.ident, EXPECT_EACH_ATTR_TEMPLATE).into();
            }

            let each_ident_result = get_each_attr(&f.attrs, &f.ident.clone().unwrap());

            let each_ident = match each_ident_result {
                Ok(i) => i,
                Err(e) => return e.into(),
            };

            let name = &f.ident;
            let original_ty = &f.ty;
            let inner_ty = inner_type("Vec", original_ty);
            if inner_ty.is_none() {
                return make_err(f, r#"Fields must have Vec type to use the "each"" attribute"#).into();
            }
            let set_ty = inner_ty.unwrap();

            if is_type("String", set_ty) {
                Some(quote! { 
                    pub fn #each_ident<T: Into<String>>(&mut self, #each_ident: T) -> &mut Self {
                        let arg = #each_ident.into();
                        if self.#name.is_none() {
                            self.#name = Some(vec![]);
                        }
                        if let Some(ref mut vector) = self.#name {
                            vector.push(arg);
                        } else {
                            unreachable!();
                        }
                        self
                    }
                })
            } else {
                Some(quote! { 
                    pub fn #each_ident(&mut self, #each_ident: #set_ty) -> &mut Self {
                        if self.#name.is_none() {
                            self.#name = vec![];
                        }
                        if let Some(ref mut vector) = self.#name {
                            vector.push(#each_ident);
                        } else {
                            unreachable!();
                        }
                        self
                    }
                })
            }
        } else {
            None
        }
    });

    let builder_check_build_field = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if inner_type("Option", ty).is_some() {
            quote! { 
                #name: self.#name.clone(),
            }
        } else {
            quote! { 
                #name: self.#name.clone()
                    .ok_or(concat!(stringify!(#name), " is not set for ", stringify!(#struct_ident)))?,
            }
        }
    }); 


    quote! {
        impl #struct_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_empty)*
                }
            }
        }

        pub struct #builder_ident {
            #(#builder_field)*
        }

        impl #builder_ident {
            #(#builder_set_method)*
            #(#builder_set_each_method)*

            fn check_build(&mut self) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
                Ok(#struct_ident {
                    #(#builder_check_build_field)*
                })
            }

            pub fn build(&mut self) -> std::result::Result<#instruction_name, std::boxed::Box<dyn std::error::Error>> {
                let instruction_builder = self.check_build()?;
                let value = instruction_builder.#value_method()?;
                Ok(
                    #instruction_name {
                        value,
                    }
                )
            }
        }
    }.into()
}

fn inner_type<'a>(wrapper_ty: &str, ty: &'a syn::Type) -> std::option::Option<&'a syn::Type> {
    if let syn::Type::Path(ref p) = ty {
       if p.path.segments.len() != 1 || p.path.segments[0].ident != wrapper_ty {
           return std::option::Option::None;
       }

       if let syn::PathArguments::AngleBracketed(ref inner_ty_args) = p.path.segments[0].arguments {
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

fn is_type(ty_name: &str, ty: &syn::Type) -> bool {
    if let syn::Type::Path(ref p) = ty {
       if p.path.segments.len() == 1 && p.path.segments[0].ident == ty_name {
           return true;
       }
    }
    false
}

fn make_err<T: quote::ToTokens>(t: T, msg: &str) -> proc_macro2::TokenStream {
    syn::Error::new_spanned(t, msg).to_compile_error()
}

struct AttrData {
    instruction_name: syn::Ident,
    value_method: syn::Ident,
}

const EXPECT_ATTR_TEMPLATE: &str = r#"Expected 
#[instruction_builder(
    instruction_name = ..., 
    value_method = ...
)]"#;

const EXPECT_EACH_ATTR_TEMPLATE: &str = r#"Expected 
#[instruction_builder(each = ...)]"#;

fn get_each_attr(attr: &Vec<syn::Attribute>, struct_ident: &syn::Ident) -> Result<syn::Ident, proc_macro2::TokenStream> {
    if attr.len() != 1 {
        return Err(make_err(struct_ident, EXPECT_EACH_ATTR_TEMPLATE));
    }
    if let syn::Meta::List( ref metalist ) = &attr[attr.len() - 1].meta {
        let tokenstream = &mut metalist.tokens.clone().into_iter();

        verify_attr_ident(tokenstream.next(), "each", metalist, EXPECT_EACH_ATTR_TEMPLATE)?;
        verify_attr_punct(tokenstream.next(), '=', metalist, EXPECT_EACH_ATTR_TEMPLATE)?;

        let each_ident = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(make_err(metalist, EXPECT_EACH_ATTR_TEMPLATE)),
        };

        Ok(each_ident)
    } else {
        return Err(make_err(struct_ident, EXPECT_EACH_ATTR_TEMPLATE));
    }
}

fn get_attr(attr: &Vec<syn::Attribute>, struct_ident: &syn::Ident) -> Result<AttrData, proc_macro2::TokenStream> {
    if attr.is_empty() {
        return Err(make_err(struct_ident, EXPECT_ATTR_TEMPLATE));
    }

    // first attr can be doc
    if let syn::Meta::List( ref metalist ) = &attr[attr.len() - 1].meta {
        let tokenstream = &mut metalist.tokens.clone().into_iter();

        verify_attr_ident(tokenstream.next(), "instruction_name", metalist, EXPECT_ATTR_TEMPLATE)?;
        verify_attr_punct(tokenstream.next(), '=', metalist, EXPECT_ATTR_TEMPLATE)?;

        let instruction_name = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(make_err(metalist, EXPECT_ATTR_TEMPLATE)),
        };

        verify_attr_punct(tokenstream.next(), ',', metalist, EXPECT_ATTR_TEMPLATE)?;
        verify_attr_ident(tokenstream.next(), "value_method", metalist, EXPECT_ATTR_TEMPLATE)?;
        verify_attr_punct(tokenstream.next(), '=', metalist, EXPECT_ATTR_TEMPLATE)?;

        let value_method = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(make_err(metalist, EXPECT_ATTR_TEMPLATE)),
        };

        Ok(AttrData { instruction_name, value_method })
    } else {
        return Err(make_err(struct_ident, EXPECT_ATTR_TEMPLATE));
    }
}

fn verify_attr_ident<T: quote::ToTokens>(token: Option<TokenTree>, expected_ident: &str, span: T, err_msg: &str) -> Result<(), proc_macro2::TokenStream> {
    match token {
        Some(TokenTree::Ident(ref i)) => {
            if i != expected_ident {
                return Err(make_err(span, err_msg));
            }
        },
        _ => return Err(make_err(span, err_msg)),

    }
    Ok(())
}

fn verify_attr_punct<T: quote::ToTokens>(token: Option<TokenTree>, expected_punct: char, span: T, err_msg: &str) -> Result<(), proc_macro2::TokenStream> {
    match token {
        Some(TokenTree::Punct(ref p)) => {
            if p.as_char() != expected_punct {
                return Err(make_err(span, err_msg));
            }
        },
        _ => return Err(make_err(span, err_msg)),

    }
    Ok(())
}
