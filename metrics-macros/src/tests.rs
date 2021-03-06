use syn::parse_quote;
use syn::{Expr, ExprPath};

use super::*;

#[test]
fn test_get_expanded_registration() {
    // Basic registration.
    let stream =
        get_expanded_registration("mytype", parse_quote! { "mykeyname" }, None, None, None);

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . register_mytype (",
        "metrics :: Key :: Owned (metrics :: KeyData :: from_static_name (& METRIC_NAME)) , ",
        "None , ",
        "None",
        ") ; ",
        "} }",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_get_expanded_registration_with_unit() {
    // Now with unit.
    let units: ExprPath = parse_quote! { metrics::Unit::Nanoseconds };
    let stream = get_expanded_registration(
        "mytype",
        parse_quote! { "mykeyname" },
        Some(Expr::Path(units)),
        None,
        None,
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . register_mytype (",
        "metrics :: Key :: Owned (metrics :: KeyData :: from_static_name (& METRIC_NAME)) , ",
        "Some (metrics :: Unit :: Nanoseconds) , ",
        "None",
        ") ; ",
        "} }",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_get_expanded_registration_with_description() {
    // And with description.
    let stream = get_expanded_registration(
        "mytype",
        parse_quote! { "mykeyname" },
        None,
        Some(parse_quote! { "flerkin" }),
        None,
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . register_mytype (",
        "metrics :: Key :: Owned (metrics :: KeyData :: from_static_name (& METRIC_NAME)) , ",
        "None , ",
        "Some (\"flerkin\")",
        ") ; ",
        "} }",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_get_expanded_registration_with_unit_and_description() {
    // And with unit and description.
    let units: ExprPath = parse_quote! { metrics::Unit::Nanoseconds };
    let stream = get_expanded_registration(
        "mytype",
        parse_quote! { "mykeyname" },
        Some(Expr::Path(units)),
        Some(parse_quote! { "flerkin" }),
        None,
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . register_mytype (",
        "metrics :: Key :: Owned (metrics :: KeyData :: from_static_name (& METRIC_NAME)) , ",
        "Some (metrics :: Unit :: Nanoseconds) , ",
        "Some (\"flerkin\")",
        ") ; ",
        "} }",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_get_expanded_callsite_fast_path_no_labels() {
    let stream = get_expanded_callsite(
        "mytype",
        "myop",
        parse_quote! {"mykeyname"},
        None,
        quote! { 1 },
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "static METRIC_KEY : metrics :: KeyData = metrics :: KeyData :: from_static_name (& METRIC_NAME) ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . myop_mytype (metrics :: Key :: Borrowed (& METRIC_KEY) , 1) ; ",
        "} }",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_get_expanded_callsite_fast_path_static_labels() {
    let labels = Labels::Inline(vec![(parse_quote! { "key1" }, parse_quote! { "value1" })]);
    let stream = get_expanded_callsite(
        "mytype",
        "myop",
        parse_quote! {"mykeyname"},
        Some(labels),
        quote! { 1 },
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "static METRIC_LABELS : [metrics :: Label ; 1usize] = [metrics :: Label :: from_static_parts (\"key1\" , \"value1\")] ; ",
        "static METRIC_KEY : metrics :: KeyData = metrics :: KeyData :: from_static_parts (& METRIC_NAME , & METRIC_LABELS) ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . myop_mytype (metrics :: Key :: Borrowed (& METRIC_KEY) , 1) ; ",
        "} ",
        "}",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_get_expanded_callsite_fast_path_dynamic_labels() {
    let labels = Labels::Inline(vec![(parse_quote! { "key1" }, parse_quote! { &value1 })]);
    let stream = get_expanded_callsite(
        "mytype",
        "myop",
        parse_quote! {"mykeyname"},
        Some(labels),
        quote! { 1 },
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . myop_mytype (metrics :: Key :: Owned (",
        "metrics :: KeyData :: from_parts (& METRIC_NAME [..] , vec ! [metrics :: Label :: new (\"key1\" , & value1)])",
        ") , 1) ; ",
        "} ",
        "}",
    );

    assert_eq!(stream.to_string(), expected);
}

/// If there are dynamic labels - generate a direct invocation.
#[test]
fn test_get_expanded_callsite_regular_path() {
    let stream = get_expanded_callsite(
        "mytype",
        "myop",
        parse_quote! {"mykeyname"},
        Some(Labels::Existing(parse_quote! { mylabels })),
        quote! { 1 },
    );

    let expected = concat!(
        "{ ",
        "static METRIC_NAME : [metrics :: SharedString ; 1] = [metrics :: SharedString :: const_str (\"mykeyname\")] ; ",
        "if let Some (recorder) = metrics :: try_recorder () { ",
        "recorder . myop_mytype (",
        "metrics :: Key :: Owned (metrics :: KeyData :: from_parts (& METRIC_NAME [..] , mylabels)) , ",
        "1",
        ") ; ",
        "} }",
    );

    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_key_to_quoted_no_labels() {
    let stream = key_to_quoted(None);
    let expected = "metrics :: KeyData :: from_static_name (& METRIC_NAME)";
    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_key_to_quoted_existing_labels() {
    let stream = key_to_quoted(Some(Labels::Existing(Expr::Path(
        parse_quote! { mylabels },
    ))));
    let expected = "metrics :: KeyData :: from_parts (& METRIC_NAME [..] , mylabels)";
    assert_eq!(stream.to_string(), expected);
}

/// Registration can only operate on static labels (i.e. labels baked into the
/// Key).
#[test]
fn test_key_to_quoted_inline_labels() {
    let stream = key_to_quoted(Some(Labels::Inline(vec![
        (parse_quote! {"mylabel1"}, parse_quote! { mylabel1 }),
        (parse_quote! {"mylabel2"}, parse_quote! { "mylabel2" }),
    ])));
    let expected = concat!(
        "metrics :: KeyData :: from_parts (& METRIC_NAME [..] , vec ! [",
        "metrics :: Label :: new (\"mylabel1\" , mylabel1) , ",
        "metrics :: Label :: new (\"mylabel2\" , \"mylabel2\")",
        "])"
    );
    assert_eq!(stream.to_string(), expected);
}

#[test]
fn test_key_to_quoted_inline_labels_empty() {
    let stream = key_to_quoted(Some(Labels::Inline(vec![])));
    let expected = concat!("metrics :: KeyData :: from_parts (& METRIC_NAME [..] , vec ! [])");
    assert_eq!(stream.to_string(), expected);
}
