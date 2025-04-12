use grepdef::{Args, SearchResult, SearchResultFormat};
use rstest::rstest;
use std::num::NonZero;

mod common;

#[rstest]
fn search_returns_matching_js_function_line_with_args_new() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected = vec![common::get_expected_search_result_for_file_type("js")];
    let actual = common::do_search(Args::new(
        query,
        Some("js".into()),
        Some(vec![file_path]),
        true,
    ));
    assert_eq!(expected, actual);
}

#[rstest]
fn search_returns_nothing_for_no_results() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("thisFunctionDoesNotExist");
    let file_type_string = String::from("js");
    let expected: Vec<SearchResult> = vec![];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_thread() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected = vec![common::get_expected_search_result_for_file_type("js")];
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.threads = Some(NonZero::new(1).unwrap());
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
fn to_grep_formats_message_without_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!("{}:{}", expected_result.file_path, expected_result.text);
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = false;
    args.no_color = true;
    let actual = common::do_search(args);
    for result in actual {
        assert_eq!(expected, result.to_grep());
    }
}

#[rstest]
fn to_grep_from_callback_formats_message_without_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!("{}:{}", expected_result.file_path, expected_result.text);
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = false;
    args.no_color = true;
    let actual = common::do_search_callback(args);
    for result in actual {
        assert_eq!(expected, result.to_grep());
    }
}

#[rstest]
fn to_grep_formats_message_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{}:{}:{}",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    let actual = common::do_search(args);
    for result in actual {
        assert_eq!(expected, result.to_grep());
    }
}

#[rstest]
fn to_grep_from_callback_formats_message_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{}:{}:{}",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    let actual = common::do_search_callback(args);
    for result in actual {
        assert_eq!(expected, result.to_grep());
    }
}

#[rstest]
fn search_and_format_returns_formatted_string_for_grep_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{}:{}:{}",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    args.format = Some(SearchResultFormat::Grep);
    let actual = common::do_search_format(args);
    for result in actual {
        assert_eq!(expected, result);
    }
}

#[rstest]
fn search_and_format_callback_provides_formatted_string_for_grep_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{}:{}:{}",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    args.format = Some(SearchResultFormat::Grep);
    let actual = common::do_search_format_callback(args);
    for result in actual {
        assert_eq!(expected, result);
    }
}

#[rstest]
fn search_and_format_returns_formatted_string_for_grep_without_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!("{}:{}", expected_result.file_path, expected_result.text);
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = false;
    args.no_color = true;
    args.format = Some(SearchResultFormat::Grep);
    let actual = common::do_search_format(args);
    for result in actual {
        assert_eq!(expected, result);
    }
}

#[rstest]
fn search_and_format_returns_formatted_string_for_json_list_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{{\"event_type\":\"MATCH\",\"file_path\":\"{}\",\"line_number\":{},\"text\":\"{}\"}},",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    args.format = Some(SearchResultFormat::JsonList);
    let actual = common::do_search_format(args);
    for (i, result) in actual.iter().enumerate() {
        match i {
            0 => assert_eq!("[", String::from(result)),
            1 => assert_eq!("{\"event_type\":\"START\"},", String::from(result)),
            3 => assert_eq!("{\"event_type\":\"END\"}", String::from(result)),
            4 => assert_eq!("]", String::from(result)),
            _ => assert_eq!(expected, String::from(result)),
        }
    }
}

#[rstest]
fn search_and_format_returns_formatted_string_for_json_per_match_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{{\"file_path\":\"{}\",\"line_number\":{},\"text\":\"{}\"}}",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    args.format = Some(SearchResultFormat::JsonPerMatch);
    let actual = common::do_search_format(args);
    for result in actual {
        assert_eq!(expected, result);
    }
}

#[rstest]
fn search_and_format_returns_formatted_string_for_json_per_match_without_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{{\"file_path\":\"{}\",\"line_number\":null,\"text\":\"{}\"}}",
        expected_result.file_path, expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = false;
    args.no_color = true;
    args.format = Some(SearchResultFormat::JsonPerMatch);
    let actual = common::do_search_format(args);
    for result in actual {
        assert_eq!(expected, result);
    }
}

#[rstest]
fn search_and_format_callback_provides_formatted_string_for_json_per_match_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{{\"file_path\":\"{}\",\"line_number\":{},\"text\":\"{}\"}}",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    args.format = Some(SearchResultFormat::JsonPerMatch);
    let actual = common::do_search_format_callback(args);
    for result in actual {
        assert_eq!(expected, result);
    }
}

#[rstest]
fn search_and_format_callback_provides_formatted_string_for_json_list_with_number() {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected_result = common::get_expected_search_result_for_file_type("js");
    let expected = format!(
        "{{\"event_type\":\"MATCH\",\"file_path\":\"{}\",\"line_number\":{},\"text\":\"{}\"}},",
        expected_result.file_path,
        expected_result.line_number.unwrap(),
        expected_result.text
    );
    let file_type_string = String::from("js");
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.line_number = true;
    args.no_color = true;
    args.format = Some(SearchResultFormat::JsonList);
    let actual = common::do_search_format_callback(args);
    for (i, result) in actual.iter().enumerate() {
        match i {
            0 => assert_eq!("[", String::from(result)),
            1 => assert_eq!("{\"event_type\":\"START\"},", String::from(result)),
            3 => assert_eq!("{\"event_type\":\"END\"}", String::from(result)),
            4 => assert_eq!("]", String::from(result)),
            _ => assert_eq!(expected, String::from(result)),
        }
    }
}

#[rstest]
fn search_returns_matching_js_function_line_with_two_files() {
    let file_path = format!(
        "{} {}",
        common::get_default_fixture_for_file_type_string("js").unwrap(),
        common::get_default_fixture_for_file_type_string("php").unwrap()
    );
    let query = String::from("parseQuery");
    let file_type_string = String::from("js");
    let expected = vec![common::get_expected_search_result_for_file_type("js")];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_file_one_directory_matching_on_directory() {
    let file_path =
        String::from("./tests/fixtures/ ./tests/fixtures/by-language/ignored-fixture.js");
    let query = String::from("parseQuery");
    let file_type_string = String::from("php");
    let expected = vec![common::get_expected_search_result_for_file_type("php")];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
fn search_returns_matching_js_function_line_with_one_file_one_directory_matching_on_file() {
    let file_path = format!(
        "./src {}",
        common::get_default_fixture_for_file_type_string("js").unwrap()
    );
    let query = String::from("parseQuery");
    let file_type_string = String::from("js");
    let expected = vec![common::get_expected_search_result_for_file_type("js")];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
#[case(String::from("parseQuery"), String::from("js"))]
#[case(String::from("parseQuery"), String::from("php"))]
#[case(String::from("query_db"), String::from("rs"))]
fn search_returns_matching_function_line_guessing_file_type_from_file_name(
    #[case] query: String,
    #[case] file_type_string: String,
) {
    let file_path =
        common::get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let expected = vec![common::get_expected_search_result_for_file_type(
        file_type_string.as_str(),
    )];
    let args = common::make_args(query, Some(file_path), None);
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
#[case(String::from("parseQuery"), String::from("js"))]
#[case(String::from("parseQuery"), String::from("javascript"))]
#[case(String::from("parseQuery"), String::from("jsx"))]
#[case(String::from("parseQuery"), String::from("javascript.jsx"))]
#[case(String::from("parseQuery"), String::from("javascriptreact"))]
#[case(String::from("parseQuery"), String::from("php"))]
#[case(String::from("query_db"), String::from("rs"))]
#[case(String::from("query_db"), String::from("rust"))]
fn search_returns_matching_function_line(#[case] query: String, #[case] file_type_string: String) {
    let file_path =
        common::get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let expected = vec![common::get_expected_search_result_for_file_type(
        file_type_string.as_str(),
    )];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
#[case(String::from("js"))]
#[case(String::from("ts"))]
#[case(String::from("jsx"))]
#[case(String::from("tsx"))]
#[case(String::from("javascript"))]
#[case(String::from("javascript.jsx"))]
#[case(String::from("javascriptreact"))]
#[case(String::from("typescript"))]
#[case(String::from("typescript.tsx"))]
#[case(String::from("typescriptreact"))]
fn search_returns_matching_js_function_line_with_filetype_alias(#[case] file_type_string: String) {
    let file_path = common::get_default_fixture_for_file_type_string("js").unwrap();
    let query = String::from("parseQuery");
    let expected = vec![common::get_expected_search_result_for_file_type("js")];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
#[case(String::from("queryDb"), String::from("js"), 1)]
#[case(String::from("makeQuery"), String::from("js"), 4)]
#[case(String::from("parseQuery"), String::from("js"), 7)]
#[case(String::from("objectWithFunctionShorthand"), String::from("js"), 15)]
#[case(String::from("shorthandFunction"), String::from("js"), 16)]
#[case(String::from("longhandFunction"), String::from("js"), 25)]
#[case(String::from("longhandArrowFunction"), String::from("js"), 34)]
#[case(String::from("longhandProperty"), String::from("js"), 43)]
#[case(String::from("queryDb"), String::from("jsx"), 1)]
#[case(String::from("makeQuery"), String::from("jsx"), 4)]
#[case(String::from("parseQuery"), String::from("jsx"), 7)]
#[case(String::from("objectWithFunctionShorthand"), String::from("jsx"), 15)]
#[case(String::from("shorthandFunction"), String::from("jsx"), 16)]
#[case(String::from("longhandFunction"), String::from("jsx"), 25)]
#[case(String::from("longhandArrowFunction"), String::from("jsx"), 34)]
#[case(String::from("longhandProperty"), String::from("jsx"), 43)]
#[case(String::from("queryDbTS"), String::from("ts"), 1)]
#[case(String::from("makeQueryTS"), String::from("ts"), 4)]
#[case(String::from("parseQueryTS"), String::from("ts"), 7)]
#[case(String::from("objectWithFunctionShorthandTS"), String::from("ts"), 15)]
#[case(String::from("shorthandFunctionTS"), String::from("ts"), 16)]
#[case(String::from("longhandFunctionTS"), String::from("ts"), 25)]
#[case(String::from("longhandArrowFunctionTS"), String::from("ts"), 34)]
#[case(String::from("longhandPropertyTS"), String::from("ts"), 43)]
#[case(String::from("AnInterface"), String::from("ts"), 59)]
#[case(String::from("AType"), String::from("ts"), 63)]
#[case(String::from("TypeDefObject"), String::from("ts"), 66)]
#[case(String::from("TypeDefSimple"), String::from("ts"), 72)]
#[case(String::from("queryDb"), String::from("php"), 2)]
#[case(String::from("parseQuery"), String::from("php"), 6)]
#[case(String::from("Foo"), String::from("php"), 11)]
#[case(String::from("Bar"), String::from("php"), 14)]
#[case(String::from("Zoom"), String::from("php"), 17)]
#[case(String::from("MyEnum"), String::from("php"), 20)]
#[case(String::from("doSomething"), String::from("php"), 24)]
#[case(String::from("doSomethingAbsolute"), String::from("php"), 26)]
#[case::php_global_constant(String::from("GLOBALCONSTANT"), String::from("php"), 30)]
#[case::php_global_define_single(String::from("GLOBALDEFINESINGLE"), String::from("php"), 31)]
#[case::php_global_define_double(String::from("GLOBALDEFINEDOUBLE"), String::from("php"), 32)]
#[case::php_global_define_nospace(
    String::from("GLOBALDEFINESINGLE_NOSPACE"),
    String::from("php"),
    34
)]
#[case::php_global_define_leadspace(
    String::from("GLOBALDEFINESINGLE_LEADSPACE"),
    String::from("php"),
    35
)]
#[case::php_global_class_constant(String::from("MYCONSTANT"), String::from("php"), 38)]
#[case(String::from("query_db"), String::from("rs"), 1)]
#[case(String::from("public_func"), String::from("rs"), 6)]
#[case(String::from("Wrapper"), String::from("rs"), 4)]
#[case(String::from("ContainerWithoutBlock"), String::from("rs"), 9)]
#[case(String::from("ContainerWithBlock"), String::from("rs"), 11)]
#[case(String::from("FileType"), String::from("rs"), 19)]
#[case(String::from("search_file"), String::from("rs"), 29)]
fn search_returns_expected_line_number_for_file_type(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_number: usize,
) {
    let file_path =
        common::get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    let actual = common::do_search(args);
    assert_eq!(1, actual.len(), "Did not find exactly one match");
    let first_actual = actual.get(0).expect("Search failed for test");
    assert_eq!(
        line_number,
        first_actual.line_number.unwrap(),
        "Match found but it has the wrong line number"
    );
}

#[rstest]
#[case::php_non_unique_method(String::from("similarMethod"), String::from("php"), [48, 51, 54])]
fn search_returns_expected_line_number_group_for_file_type(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_numbers: [usize; 3],
) {
    let file_path =
        common::get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    let actual = common::do_search(args);
    assert_eq!(
        line_numbers.len(),
        actual.len(),
        "Did not find expected number of matches"
    );
    for (i, result) in actual.iter().enumerate() {
        assert_eq!(
            line_numbers[i],
            result.line_number.unwrap(),
            "Match found but it has the wrong line number"
        );
    }
}

#[rstest]
#[case::php_non_unique_method_limit_1(String::from("similarMethod"), String::from("php"), [48, 51, 54], 1)]
#[case::php_non_unique_method_limit_2(String::from("similarMethod"), String::from("php"), [48, 51, 54], 2)]
#[case::php_non_unique_method_limit_3(String::from("similarMethod"), String::from("php"), [48, 51, 54], 3)]
fn search_returns_limited_expected_line_number_group_for_file_type(
    #[case] query: String,
    #[case] file_type_string: String,
    #[case] line_numbers: [usize; 3],
    #[case] limit: usize,
) {
    let file_path =
        common::get_default_fixture_for_file_type_string(file_type_string.as_str()).unwrap();
    let mut args = common::make_args(query, Some(file_path), Some(file_type_string));
    args.limit = Some(limit);
    let actual = common::do_search(args);
    assert_eq!(
        limit,
        actual.len(),
        "Did not find expected number of matches"
    );
    for (i, result) in actual.iter().enumerate() {
        assert_eq!(
            line_numbers[i],
            result.line_number.unwrap(),
            "Match found but it has the wrong line number"
        );
    }
}

#[rstest]
fn search_returns_matching_js_function_line_for_recursive() {
    let file_path = String::from("./tests");
    let query = String::from("parseQuery");
    let file_type_string = String::from("js");
    let expected = vec![
        common::get_expected_search_result_for_file_type("js"),
        common::get_expected_search_result_for_file_type("jsx"),
    ];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    let actual = common::do_search(args);
    println!("expected {:?}", expected);
    println!("actual   {:?}", actual);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_js_function_line_for_recursive_default_path() {
    let query = String::from("parseQuery");
    let file_type_string = String::from("js");
    let expected = vec![
        common::get_expected_search_result_for_file_type("js"),
        common::get_expected_search_result_for_file_type("jsx"),
    ];
    let args = common::make_args(query, None, Some(file_type_string));
    let actual = common::do_search(args);
    println!("expected {:?}", expected);
    println!("actual   {:?}", actual);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_ts_function_line_for_recursive() {
    let file_path = String::from("./tests/fixtures");
    let query = String::from("parseQueryTS");
    let file_type_string = String::from("ts");
    let expected = vec![
        common::get_expected_search_result_for_file_type("ts"),
        common::get_expected_search_result_for_file_type("tsx"),
    ];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    let actual = common::do_search(args);
    println!("expected {:?}", expected);
    println!("actual   {:?}", actual);
    assert!(actual.iter().all(|item| expected.contains(item)));
    assert!(expected.iter().all(|item| actual.contains(item)));
}

#[rstest]
fn search_returns_matching_php_function_line_guessing_file_type_from_directory() {
    let file_path = String::from("./tests/fixtures/only-php/other-php-fixture.php");
    let query = String::from("otherPhpFunction");
    let line_number = Some(3);
    let expected = vec![SearchResult {
        event_type: grepdef::SearchEventType::NONE,
        file_path: file_path.clone(),
        line_number,
        text: String::from("function otherPhpFunction() {"),
    }];
    let args = common::make_args(query, Some(String::from("./tests/fixtures/only-php")), None);
    assert_eq!(expected, common::do_search(args));
}

#[rstest]
#[case(String::from("parseQuery"), String::from("js"))]
#[case(String::from("parseQuery"), String::from("php"))]
#[case(String::from("query_db"), String::from("rs"))]
fn search_returns_matching_function_line_for_recursive(
    #[case] query: String,
    #[case] file_type_string: String,
) {
    let file_path = String::from("./tests/fixtures");
    let expected = vec![common::get_expected_search_result_for_file_type(
        file_type_string.as_str(),
    )];
    let args = common::make_args(query, Some(file_path), Some(file_type_string));
    let actual = common::do_search(args);
    println!("expected {:?}", expected);
    println!("actual   {:?}", actual);
    // Note that there may be more results than was expected, but we're ok with that here.
    assert!(expected.iter().all(|item| actual.contains(item)));
}
