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
        _ => return err(&input.ident, "Expected Enum").into(),
    };

    let impl_display = variants.iter().map(|v| {
        let variant = &v.ident;
        quote! {
            #instruction::#variant(ins) => write!(f, "{}", ins),
        }
    });

    let impl_convert_from_for_instruction = variants.iter()
        .filter(|v| &v.ident != "Any")
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
        .filter(|v| &v.ident != "Any")
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
        .filter(|v| &v.ident != "Any")
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
        .filter(|v| &v.ident != "Any")
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
        &format!("{}Builder", struct_ident), 
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
        _ => return err(struct_ident, "Expected Struct with named fields").into(),
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
            quote! { #name: #ty }
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

            fn check_build(&mut self) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
                Ok(#struct_ident {
                    #(#builder_check_build_field)*
                })
            }

            pub fn build(&mut self) -> std::result::Result<#instruction_name, std::boxed::Box<dyn std::error::Error>> {
                let instruction_builder = self.check_build()?;
                let value = instruction_builder.#value_method();
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

fn err<T: quote::ToTokens>(t: T, msg: &str) -> proc_macro2::TokenStream {
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

fn get_attr(attr: &Vec<syn::Attribute>, struct_ident: &syn::Ident) -> Result<AttrData, proc_macro2::TokenStream> {
    if attr.len() != 1 {
        return Err(err(struct_ident, EXPECT_ATTR_TEMPLATE));
    }

    if let syn::Meta::List( ref metalist ) = &attr[0].meta {
        let tokenstream = &mut metalist.tokens.clone().into_iter();

        verify_attr_ident(tokenstream.next(), "instruction_name", metalist)?;
        verify_attr_punct(tokenstream.next(), '=', metalist)?;

        let instruction_name = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(err(metalist, EXPECT_ATTR_TEMPLATE)),
        };

        verify_attr_punct(tokenstream.next(), ',', metalist)?;
        verify_attr_ident(tokenstream.next(), "value_method", metalist)?;
        verify_attr_punct(tokenstream.next(), '=', metalist)?;

        let value_method = match tokenstream.next() {
            Some(TokenTree::Ident(ref i)) => i.clone(),
            _ => return Err(err(metalist, EXPECT_ATTR_TEMPLATE)),
        };

        Ok(AttrData { instruction_name, value_method })
    } else {
        return Err(err(struct_ident, EXPECT_ATTR_TEMPLATE));
    }
}

fn verify_attr_ident<T: quote::ToTokens>(token: Option<TokenTree>, expected_ident: &str, span: T) -> Result<(), proc_macro2::TokenStream> {
    match token {
        Some(TokenTree::Ident(ref i)) => {
            if i != expected_ident {
                return Err(err(span, EXPECT_ATTR_TEMPLATE));
            }
        },
        _ => return Err(err(span, EXPECT_ATTR_TEMPLATE)),

    }
    Ok(())
}

fn verify_attr_punct<T: quote::ToTokens>(token: Option<TokenTree>, expected_punct: char, span: T) -> Result<(), proc_macro2::TokenStream> {
    match token {
        Some(TokenTree::Punct(ref p)) => {
            if p.as_char() != expected_punct {
                return Err(err(span, EXPECT_ATTR_TEMPLATE));
            }
        },
        _ => return Err(err(span, EXPECT_ATTR_TEMPLATE)),

    }
    Ok(())
}
