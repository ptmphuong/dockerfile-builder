mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(InstructionInit)]
pub fn instruction_init(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let instruction = &input.ident;

    let variants = match input.data {
        syn::Data::Enum(syn::DataEnum { ref variants, .. }, ..) => variants,
        _ => return utils::make_err(&input.ident, "Expected Enum").into(),
    };

    let impl_display = variants.iter().map(|v| {
        let variant = &v.ident;
        quote! {
            #instruction::#variant(ins) => write!(f, "{}", ins),
        }
    });

    let impl_convert_from_for_instruction =
        variants.iter().filter(|v| &v.ident != "ANY").map(|v| {
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
            let variant_lower = &variant.to_string().to_lowercase();
            let builder_name = format!("{}Builder", utils::make_title_case(variant_lower.to_string()));

            let doc_definition = format!("`{}` instruction. ", variant);
            let doc_instruction = format!(r#"Create a new `{}` from string literal using `::from()`:
```
# use dockerfile_builder::instruction::{};
let {} = {}::from("some instruction value");
```"#, variant, variant, variant_lower, variant);
            let doc_link_builder = format!("* See how `{}` can be built with `{}` [here](crate::instruction_builder::{})", variant, builder_name, builder_name);
            let doc_link_reference = format!("* Link to Dockerfile Reference [here](https://docs.docker.com/engine/reference/builder/#{})", variant_lower);
            quote! {
                #[doc = #doc_definition]
                ///
                #[doc = #doc_instruction]
                ///
                #[doc = #doc_link_builder]
                ///
                #[doc = #doc_link_reference]
                #[derive(Debug, Clone, Eq, PartialEq)]
                pub struct #variant {
                    pub value: String,
                }
            }
        }
    );

    let impl_convert_from_for_variant = variants.iter().filter(|v| &v.ident != "ANY").map(|v| {
        let variant = &v.ident;
        let gen_doc = format!("Construct a new {} instruction from raw string", variant);
        quote! {
            impl<T> std::convert::From<T> for #variant where T: Into<String> {
                #[doc = #gen_doc]
                fn from(value: T) -> Self {
                    #variant {
                        value: value.into(),
                    }
                }
            }
        }
    });

    let impl_display_for_variant = variants.iter().filter(|v| &v.ident != "ANY").map(|v| {
        let variant = &v.ident;
        let variant_string = &variant.to_string().to_uppercase();
        quote! {
            impl std::fmt::Display for #variant {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{} {}", #variant_string, self.value)
                }
            }
        }
    });

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
    }
    .into()
}

#[proc_macro_derive(InstructionBuilder, attributes(instruction_builder))]
pub fn instruction_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_ident = &input.ident;
    let builder_ident = syn::Ident::new(&format!("{}Inner", struct_ident), struct_ident.span());

    let attr = &input.attrs;
    let (instruction_name, value_method) = match utils::get_attr(attr, struct_ident) {
        Ok(ad) => (ad.instruction_name, ad.value_method),
        Err(e) => return e.into(),
    };

    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => return utils::make_err(struct_ident, "Expected Struct with named fields").into(),
    };

    let builder_empty = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name: std::option::Option::None, }
    });

    // Add Option wrapper if the given type isn't already Option.
    let builder_field = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if utils::is_type("Option", ty) {
            quote! { #name: #ty, }
        } else {
            quote! { #name: std::option::Option<#ty>, }
        }
    });

    let builder_set_method = fields.iter().map(|f| {
        let name = &f.ident;

        let original_ty = &f.ty;

        // Custom set method for Vec<String> or Option<Vec<String>
        // These methods can accept Vec<T> where T: Into<String> as argument.
        if utils::is_type_vec_string(original_ty) || utils::is_type_option_vec_string(original_ty)
        {
            return quote! {
                pub fn #name<T: Into<String>>(&mut self, #name: Vec<T>) -> &mut Self {
                    let converted = #name.into_iter().map(|s| s.into()).collect::<Vec<String>>();
                    self.#name = Some(converted);
                    self
                }
            };
        }

        // Custom set method for String or Option<String>
        // These methods can accept Vec<T> where T: Into<String> as argument.
        if utils::is_type_option_string(original_ty) || utils::is_type("String", original_ty) {
            return quote! {
                pub fn #name<T: Into<String>>(&mut self, #name: T) -> &mut Self {
                    self.#name = Some(#name.into());
                    self
                }
            };
        }

        // Defaut set method.
        // If original type is Option<inner> => set type is inner
        // Else set type is original type
        let set_ty = utils::inner_type("Option", original_ty).unwrap_or(original_ty);
        quote! {
            pub fn #name(&mut self, #name: #set_ty) -> &mut Self {
                self.#name = Some(#name);
                self
            }
        }
    });

    let builder_set_each_method = fields.iter().map(|f| {
        if f.attrs.is_empty() {
            return None;
        }

        if f.attrs.len() != 1 {
            return utils::make_err(&f.ident, utils::EXPECT_EACH_ATTR_TEMPLATE).into();
        }

        let each_ident_result = if let Some(field_ident) = &f.ident {
            utils::get_each_attr(&f.attrs, field_ident)
        } else {
            return utils::make_err(&f.ident, "Expect field ident").into();
        };

        let each_ident = match each_ident_result {
            Ok(i) => i,
            Err(e) => return e.into(),
        };

        let name = &f.ident;
        let original_ty = &f.ty;

        // Custom set each method for Vec<String> or Option<Vec<String>>
        // These method accept T where T:Into<String> as argument
        if utils::is_type_vec_string(original_ty) || utils::is_type_option_vec_string(original_ty) {
            return Some(quote! {
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
            });
        }

        let set_ty = if let Some(inner_ty) = utils::inner_type("Vec", original_ty) {
            inner_ty
        } else {
            return utils::make_err(
                f,
                r#"Fields must have Vec type to use the "each" attribute"#,
            )
            .into();
        };

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
    });

    let builder_check_build_field = fields.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;
        if utils::is_type("Option", ty) {
            quote! {
                #name: self.#name.clone(),
            }
        } else {
            quote! {
                #name: self.#name.clone()
                    .ok_or(concat!(stringify!(#name), " is required for ", stringify!(#struct_ident)))?,
            }
        }
    });

    let builder_impl_note = format!("Builder implementation for [`{}`]", struct_ident);

    quote! {
        impl #struct_ident {
            /// Builder implementation
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_empty)*
                }
            }
        }

        pub struct #builder_ident {
            #(#builder_field)*
        }

        #[doc = #builder_impl_note]
        ///
        /// This code is automatically generated by the `InstructionBuilder` macro
        impl #builder_ident {
            #(#builder_set_method)*
            #(#builder_set_each_method)*

            pub fn check_build(&mut self) -> eyre::Result<#struct_ident, String> {
                Ok(#struct_ident {
                    #(#builder_check_build_field)*
                })
            }

            pub fn build(&mut self) -> eyre::Result<#instruction_name, String> {
                let instruction_builder = self.check_build()?;
                let value = instruction_builder.#value_method()?;
                Ok(
                    #instruction_name {
                        value,
                    }
                )
            }
        }
    }
    .into()
}
