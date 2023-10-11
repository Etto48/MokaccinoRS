use proc_macro::TokenStream;
use quote::quote;
use syn::{self, spanned::Spanned, DataEnum};

#[proc_macro_derive(Serializable)]
pub fn serializable_derive(input: TokenStream) -> TokenStream
{
    let ast = syn::parse(input).expect("Error during parsing");
    impl_serializable(&ast)
}

fn get_field_names(fields: &syn::Fields) -> Vec<syn::Ident>
{
    match fields
    {
        syn::Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let name = f.ident.clone();
                let ty = f.ty.clone();
                (name.expect("Named struct field left unnamed"), ty)
            });
            fields.clone().map(|(name, _ty)| name).collect()
        },
        syn::Fields::Unnamed(fields) => 
        {
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name = syn::Ident::new(&format!("f{}", i), f.span());
                let ty = f.ty.clone();
                (name, ty)
            });
            fields.clone().map(|(name, _ty)| name).collect()
        },
        syn::Fields::Unit => 
        {
            Vec::new()
        },
    }
}

fn get_field_types(fields: &syn::Fields) -> Vec<syn::Type>
{
    match fields
    {
        syn::Fields::Named(fields) => {
            let fields = fields.named.iter().map(|f| {
                let name = f.ident.clone();
                let ty = f.ty.clone();
                (name.expect("Named struct field left unnamed"), ty)
            });
            fields.clone().map(|(_name, ty)| ty).collect()
        },
        syn::Fields::Unnamed(fields) => 
        {
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name = syn::Ident::new(&format!("f{}", i), f.span());
                let ty = f.ty.clone();
                (name, ty)
            });
            fields.clone().map(|(_name, ty)| ty).collect()
        },
        syn::Fields::Unit => 
        {
            Vec::new()
        },
    }
}

fn build_serialize_body(fields: &syn::Fields, prepend_self: bool, use_ref: bool) -> proc_macro2::TokenStream
{
    let field_names = get_field_names(fields);
    match fields
    {
        syn::Fields::Named(_fields) => {
            match (prepend_self, use_ref)
            {
                (true,true) =>
                    quote!{
                        #(bytes.extend(Serializable::serialize(&self.#field_names));)*   
                    },
                (false,true) =>
                    quote!{
                        #(bytes.extend(Serializable::serialize(&#field_names));)*   
                    },
                (true,false) =>
                    quote!{
                        #(bytes.extend(Serializable::serialize(self.#field_names));)*   
                    },
                (false,false) =>
                    quote!{
                        #(bytes.extend(Serializable::serialize(#field_names));)*   
                    },
            }
        },
        syn::Fields::Unnamed(fields) => 
        {
            let fields = fields.unnamed.iter().enumerate().map(|(i, f)| {
                let name = syn::Ident::new(&format!("f{}", i), f.span());
                let ty = f.ty.clone();
                (name, ty)
            });
            let field_numbers = (0..fields.len()).map(syn::Index::from);
            match (prepend_self, use_ref)
            {
                (true, true) => 
                    quote! {
                        #(bytes.extend(Serializable::serialize(&self.#field_numbers));)*    
                    },
                (false, true) => 
                    quote! {
                        #(bytes.extend(Serializable::serialize(&#field_names));)*    
                    },
                (true, false) => 
                    quote! {
                        #(bytes.extend(Serializable::serialize(self.#field_numbers));)*    
                    },
                (false, false) => 
                    quote! {
                        #(bytes.extend(Serializable::serialize(#field_names));)*    
                    },
            }
        },
        syn::Fields::Unit => 
        {
            quote! {}
        },
    }
}

fn build_deserialize_body(fields: &syn::Fields) -> proc_macro2::TokenStream
{
    let field_names = get_field_names(fields);
    let field_types = get_field_types(fields);
    match fields
    {
        syn::Fields::Named(_fields) => {
            quote!{
                #(let (#field_names,len) = <#field_types as Serializable>::deserialize(&bytes[offset..])?;
                offset += len;)*
            }
        },
        syn::Fields::Unnamed(_fields) => 
        {
            quote! {
                #(let (#field_names,len) = <#field_types as Serializable>::deserialize(&bytes[offset..])?;
                offset += len;)*
            }
        },
        syn::Fields::Unit => 
        {
            quote! {}
        },
    }
}

fn build_constructor(fields: &syn::Fields, variation: Option<&syn::Ident>) -> proc_macro2::TokenStream
{
    let field_names = get_field_names(fields);
    match fields
    {
        syn::Fields::Named(_) => 
        {
            if let Some(variation) = variation
            {
                quote! {
                    Self::#variation {
                        #(#field_names),*
                    }
                }
            }
            else
            {
                quote! {
                    Self {
                        #(#field_names),*
                    }
                }
            }
        },
        syn::Fields::Unnamed(_) => 
        {
            if let Some(variation) = variation
            {
                quote! {
                    Self::#variation (
                        #(#field_names),*
                    )
                }
            }
            else
            {
                quote! {
                    Self (
                        #(#field_names),*
                    )
                }
            }
        },
        syn::Fields::Unit => 
        {
            if let Some(variation) = variation
            {
                quote! {
                    Self::#variation
                }
            }
            else
            {
                quote! {
                    Self
                }
            }
        },
    }
}

fn impl_serializable(ast: &syn::DeriveInput) -> TokenStream
{
    let name = &ast.ident;
    let gen = match &ast.data
    {
        syn::Data::Struct(syn::DataStruct{fields,..}) => 
        {
            let serialize_body = build_serialize_body(fields,true, true);
            let deserialize_body = build_deserialize_body(fields);
            let constructor_body = build_constructor(fields, None);
            quote !
            {
                impl Serializable for #name {
                    fn serialize(&self) -> Vec<u8> {
                        let mut bytes = Vec::new();
                        #serialize_body
                        bytes
                    }
                    fn deserialize(bytes: &[u8]) -> std::io::Result<(#name,usize)>{
                        let mut offset: usize = 0;
                        #deserialize_body
                        Ok((#constructor_body, offset))
                    }
                }
            }
        },
        syn::Data::Enum(DataEnum { variants , ..}) => {
            let variant_indices_0 = (0..variants.len()).map(syn::Index::from);
            let variant_indices_1 = (0..variants.len()).map(syn::Index::from);
            let variant_names = variants.iter().map(|v| v.ident.clone());
            
            let variant_fields = variants.iter().map(|v| v.fields.clone());
            let variant_fields_serialization = variant_fields.clone().map(|fields|
            {
                build_serialize_body(&fields, false, false)
            });
            let variant_fields_deserialization = variant_fields.clone().map(|fields|
            {
                build_deserialize_body(&fields)
            });

            let variant_names_and_fields = variant_names.zip(variant_fields.clone());
            let variant_constructors = variant_names_and_fields.clone().map(|(name, fields)|
            {
                build_constructor(&fields, Some(&name))
            });

            let variant_names_match = variant_names_and_fields.map(|(name, fields)|
            {
                let field_names = get_field_names(&fields);
                match fields
                {
                    syn::Fields::Named(_) => 
                    {
                        quote! {
                            Self::#name{
                                #(#field_names),*
                            }
                        }
                    },
                    syn::Fields::Unnamed(_) => 
                    {
                        quote! {
                            Self::#name(
                                #(#field_names),*
                            )
                        }
                    },
                    syn::Fields::Unit => 
                    {
                        quote! {
                            Self::#name
                        }
                    },
                }
            });
            
            quote!{
                impl Serializable for #name {
                    fn serialize(&self) -> Vec<u8>
                    {
                        let mut bytes = Vec::new();
                        match self {
                            #(#variant_names_match => {
                                bytes.push(#variant_indices_0);
                                #variant_fields_serialization
                            })*
                        }
                        bytes
                    }
                    fn deserialize(bytes: &[u8]) -> std::io::Result<(#name,usize)>
                    {
                        let mut offset: usize = 0;
                        if bytes.len() == 0 {
                            Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid data size"))
                        }
                        else
                        {
                            let variant_index = bytes[0];
                            offset += 1;
                            match variant_index {
                                #(#variant_indices_1 => {
                                    #variant_fields_deserialization
                                    Ok((#variant_constructors, offset))
                                })*
                                _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid variant index")),
                            }
                        }
                    }
                }
            }
        },
        syn::Data::Union(_) => unimplemented!("Unions are not supported"),
    };
    gen.into()
}