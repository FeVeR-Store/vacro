use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, TokenStreamExt};
use syn::{
    braced, custom_keyword,
    parse::{discouraged::Speculative, Parse},
    parse_quote,
    spanned::Spanned,
    token::Brace,
    Ident, Token, Type,
};

pub enum HelpField {
    Help(TokenStream),
    Error(TokenStream),
    #[cfg(feature = "parser")]
    Example(TokenStream),
}

custom_keyword!(error);
custom_keyword!(help);
custom_keyword!(example);

impl Into<HelpField> for error {
    fn into(self) -> HelpField {
        HelpField::Error(TokenStream::new())
    }
}

impl Into<HelpField> for help {
    fn into(self) -> HelpField {
        HelpField::Help(TokenStream::new())
    }
}
#[cfg(feature = "parser")]
impl Into<HelpField> for example {
    fn into(self) -> HelpField {
        HelpField::Example(TokenStream::new())
    }
}

impl HelpField {
    pub fn set_inner(&mut self, inner: TokenStream) {
        match self {
            Self::Error(e) => e.append_all(inner),
            Self::Help(e) => e.append_all(inner),
            #[cfg(feature = "parser")]
            Self::Example(e) => e.append_all(inner),
        }
    }
}

impl Parse for HelpField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut field: HelpField = if input.peek(error) {
            input.parse::<error>()?.into()
        } else if input.peek(help) {
            input.parse::<help>()?.into()
        } else if input.peek(example) {
            #[cfg(not(feature = "parser"))]
            return Err(
                input.error("`example` field is only valid when the parser feature is enabled")
            );
            #[cfg(feature = "parser")]
            input.parse::<example>()?.into()
        } else {
            let token: Ident = input.parse()?;
            return Err(input.error(format!(
                "expect `error`, `help`, find {}",
                token.to_string()
            )));
        };
        let _colon: Token![:] = input.parse()?;
        let fork = input.fork();
        let mut tokens = TokenStream::new();
        while !fork.peek(Token![,]) && !fork.is_empty() {
            tokens.append(fork.parse::<TokenTree>()?);
        }
        input.advance_to(&fork);
        field.set_inner(tokens);
        Ok(field)
    }
}

pub struct Help {
    alias: Ident,
    _colon: Token![:],
    ty: Type,
    _brace: Brace,
    error: TokenStream,
    help: TokenStream,
    #[cfg(feature = "parser")]
    example: TokenStream,
}

impl Parse for Help {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let alias = input.parse()?;
        let _colon = input.parse()?;
        let ty = input.parse()?;

        let content;
        let _brace = braced!(content in input);
        let fields = content.parse_terminated(HelpField::parse, Token![,])?;

        let brace_span = _brace.span.span();

        let mut error = None;
        let mut help = None;
        #[cfg(feature = "parser")]
        let mut example = None;

        fields.iter().for_each(|f| {
            match f {
                HelpField::Error(inner) => error = Some(inner.clone()),
                HelpField::Help(inner) => help = Some(inner.clone()),
                #[cfg(feature = "parser")]
                HelpField::Example(inner) => example = Some(inner.clone()),
            };
        });

        Ok(Self {
            alias,
            _colon,
            ty,
            _brace,
            error: error.ok_or(syn::Error::new(brace_span, "expected error field"))?,
            help: help.ok_or(syn::Error::new(brace_span, "expected help field"))?,
            #[cfg(feature = "parser")]
            example: example.ok_or(syn::Error::new(brace_span, "expected example field"))?,
        })
    }
}

pub fn help_impl(input: TokenStream) -> TokenStream {
    let Help {
        alias,
        ty,
        error,
        help,
        #[cfg(feature = "parser")]
        example,
        ..
    } = parse_quote!(#input);

    let to_token_impl = if cfg!(feature = "quote") {
        quote! {
            impl ::quote::ToTokens for #alias {
                fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
                    ::quote::ToTokens::to_tokens(&self.0, tokens);
                }
            }
        }
    } else {
        quote! {}
    };
    let example_token = quote! {#example}.to_string();
    let parser_help_impl = if cfg!(feature = "parser") {
        let pkg = if cfg!(feature = "standalone") {
            quote! {::vacro_parser}
        } else {
            quote! {::vacro::parser}
        };
        quote! {
            impl #pkg::__private::CustomHelp for #alias {
                fn custom_message() -> ::std::string::String {
                    ::std::string::String::from(#example_token)
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        struct #alias(#ty);

        impl std::ops::Deref for #alias {
            type Target = #ty;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #to_token_impl
        #parser_help_impl

        impl ::syn::parse::Parse for #alias {
            fn parse(input_stream: ::syn::parse::ParseStream) -> ::syn::Result<Self> {
                ::std::result::Result::Ok(#alias(input_stream.parse::<#ty>().map_err(|err| {
                    let err = err.to_string();
                    let input = input_stream.to_string();
                    let full_msg = format!("{}\nhelp: {}\n\n", format!(#error), format!(#help));
                    ::syn::Error::new(input_stream.span(), full_msg)
                })?))
            }
        }
    }
}
