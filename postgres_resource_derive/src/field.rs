use syn::{
    parse::{Parse, ParseStream, Result},
    punctuated::Punctuated,
    token, Attribute, Ident,
};

#[derive(Debug, PartialEq)]
enum FieldAttr {
    Optional,
    ForeignKey,
}

#[derive(Debug)]
pub struct Field {
    attr: Vec<FieldAttr>,
    pub name: Ident,
    ty: Ident,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr = Self::parse_attr(&input)?;
        let name: Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let ty = input.parse()?;

        Ok(Field { attr, name, ty })
    }
}

impl Field {
    fn parse_attr(input: &ParseStream) -> Result<Vec<FieldAttr>> {
        let mut result = Vec::new();
        if let Some(attrs) = input.call(Attribute::parse_outer).ok() {
            attrs.iter().for_each(|attr| {
                let ident = &attr.path.segments[0].ident;

                match &ident.to_string()[..] {
                    "optional" => result.push(FieldAttr::Optional),
                    "fk" => result.push(FieldAttr::ForeignKey),
                    _ => panic!("Invalid attribute for field"),
                }
            });
        }
        Ok(result)
    }

    pub fn ty(&self) -> proc_macro2::TokenStream {
        let ty = &self.ty;
        let mut ty_tokens = quote!(#ty);

        if self.attr.contains(&FieldAttr::Optional) {
            ty_tokens = quote!(Option<#ty>);
        }

        ty_tokens
    }

    pub fn fk(&self) -> bool {
        self.attr.contains(&FieldAttr::ForeignKey)
    }
}
