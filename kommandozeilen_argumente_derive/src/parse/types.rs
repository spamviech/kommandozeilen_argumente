//! Helper types and code-generation utilities for `#[derive(Parse)]`.

mod item;
mod names;
mod settings;

pub(in crate::parse) use item::*;
pub(in crate::parse) use names::*;
pub(in crate::parse) use settings::*;

#[cfg(test)]
mod test {
    use quote::quote;
    use venial::parse_item;

    use super::item::try_into_parse_item;

    #[test]
    #[expect(clippy::indexing_slicing, reason = "Simplify testing of the first/second element.")]
    fn parse_item_collects_struct_fields_and_enum_variants() {
        let struct_item = try_into_parse_item(
            quote!(
                struct Arguments {
                    verbose: bool,
                }
            ),
            parse_item(quote!(
                struct Arguments {
                    verbose: bool,
                }
            ))
            .expect("struct parses as a venial item"),
        )
        .expect("struct is supported");
        assert_eq!(struct_item.name.to_string(), "Arguments");
        assert_eq!(struct_item.variants.len(), 1);
        assert!(struct_item.variants[0].0.is_none());

        let enum_item = try_into_parse_item(
            quote!(
                enum Arguments {
                    File { path: String },
                    Url { url: String },
                }
            ),
            parse_item(quote!(
                enum Arguments {
                    File { path: String },
                    Url { url: String },
                }
            ))
            .expect("enum parses as a venial item"),
        )
        .expect("enum is supported");
        assert_eq!(enum_item.name.to_string(), "Arguments");
        assert_eq!(enum_item.variants.len(), 2);
        assert_eq!(
            enum_item.variants[0].0.as_ref().map(ToString::to_string),
            Some(String::from("File"))
        );
        assert_eq!(
            enum_item.variants[1].0.as_ref().map(ToString::to_string),
            Some(String::from("Url"))
        );
    }
}
