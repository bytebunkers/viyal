use formatter::format;

#[test]
fn test_formatter_class() {
    let raw = "class User{String name;int age;void sayHi(){print(\"hi\");}}";
    let expected = "class User {\n    String name;\n    int age;\n\n    void sayHi() {\n        print(\"hi\");\n    }\n}\n";
    let formatted = format(raw).unwrap();
    assert_eq!(formatted, expected);
}

#[test]
fn test_formatter_expressions() {
    let raw = "class A{void run(){foo(  a , b ) ; obj . prop; }}";
    let expected = "class A {\n    void run() {\n        foo(a, b);\n        obj.prop;\n    }\n}\n";
    let formatted = format(raw).unwrap();
    assert_eq!(formatted, expected);
}

