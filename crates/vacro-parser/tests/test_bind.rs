use quote::quote;
use std::str::FromStr;
use syn::{Ident, LitBool, LitInt, Type};
use vacro_parser::bind;

use proc_macro2::TokenStream;

// 测试最基础的具名捕获
#[test]
fn test_basic_capture() {
    let input = quote!(my_var);

    // capture! 会生成类似:
    // struct Output { name: Ident }
    // ... parse logic ...
    // Result<Output>
    bind!(
        let res = (input -> #(name: Ident));
    );

    assert!(res.is_ok());
    let output = res.unwrap();
    assert_eq!(output.name.to_string(), "my_var");
}

// 测试行内元组捕获
#[test]
fn test_inline_capture() {
    let input = quote!( my_var: i32 );

    // 期望解析: Ident, Punct(:), Type
    bind!(
        let res = (input -> #(@: Ident) : #(@: Type));
    );

    assert!(res.is_ok());
    let (name, ty) = res.unwrap(); // capture! 生成元组
    assert_eq!(name.to_string(), "my_var");
    assert_eq!(quote! {#ty}.to_string(), "i32");
}

// 测试多态/枚举捕获
#[test]
#[allow(dead_code)]
fn test_enum_capture() {
    // 情况 1: 输入是 Ident
    let input1 = quote!(MyIdent);
    bind! {
       let res1 = (input1 -> #(val: TestEnum1 {
           Var1: Ident,
           Var2: LitInt
       }));
    };

    let output1 = res1.unwrap();
    // 验证生成的枚举变体
    match output1.val {
        TestEnum1::Var1(id) => assert_eq!(id.to_string(), "MyIdent"),
        _ => panic!("Expected Var1"),
    }

    // 情况 2: 输入是 LitInt
    let input2 = quote!(123);
    bind!(
        let res2 = (input2 -> #(val: TestEnum2 {
            Var1: Ident,
            Var2: LitInt
        }));
    );

    let output2 = res2.unwrap();
    match output2.val {
        TestEnum2::Var2(lit) => assert_eq!(lit.base10_digits(), "123"),
        _ => panic!("Expected Var2"),
    }

    // 测试迭代捕获
    let input3 = quote! {123, my_ident, 456};
    bind!(
        let res3 = (input3 -> #(val*[,]: TestEnum3 {
            Var1: Ident,
            Var2: LitInt
        }));
    );
    let output3 = res3.unwrap();
    match &output3.val[0] {
        TestEnum3::Var1(_id) => panic!("Expected Var2"),
        TestEnum3::Var2(lit) => assert_eq!(lit.base10_digits(), "123"),
    }
    match &output3.val[1] {
        TestEnum3::Var1(id) => assert_eq!(id.to_string(), "my_ident"),
        TestEnum3::Var2(_lit) => panic!("Expected Var1"),
    }
    match &output3.val[2] {
        TestEnum3::Var1(_id) => panic!("Expected Var2"),
        TestEnum3::Var2(lit) => assert_eq!(lit.base10_digits(), "456"),
    }
}

// 测试自定义关键字与符号
#[test]
fn test_custom_keyword_symbol() {
    // quote 会分词为 <- >，因此手动构建注入
    let sym = TokenStream::from_str("<->").unwrap();
    let input = quote!( pair my_var1 #sym my_var2 );

    // 期望解析: CustomKeyword(pair) Ident, CustomSymbol(<->), Ident
    bind!(
        let res = (input -> pair #(@: Ident) <-> #(@: Ident));
    );

    // assert!(res.is_ok());
    let (var1, var2) = res.unwrap(); // capture! 生成元组
    assert_eq!(var1.to_string(), "my_var1");
    assert_eq!(var2.to_string(), "my_var2");
}

// 测试解析失败的情况
#[test]
fn test_capture_fail() {
    let input = quote!(123); // 给一个整数

    // 期望解析 Ident
    bind!(
        let res = (input -> #(_name: Ident));
    );

    assert!(res.is_err()); // 应该报错
}

// 4. 关联捕获

#[test]
fn test_named_nested_list() {
    let input = quote!({ a, b, c });
    bind! {
        let res = (input -> {
            #(roles*[,]: #(ident: Ident))
        }).unwrap();
    }
    assert_eq!(res.roles.len(), 3);
    // Check if the inner struct is accessible and correct
    assert_eq!(res.roles[0].ident.to_string(), "a");
    assert_eq!(res.roles[1].ident.to_string(), "b");
    assert_eq!(res.roles[2].ident.to_string(), "c");
}

#[test]
fn test_named_nested_complex() {
    let input = quote!({ a: true, b: false });
    bind! {
        let res = (input -> {
            #(items*[,]: #(pair: #(key: Ident): #(val: LitBool)))
        }).unwrap();
    }
    assert_eq!(res.items.len(), 2);

    // Accessing nested struct fields
    assert_eq!(res.items[0].pair.key.to_string(), "a");
    assert!(res.items[0].pair.val.value);

    assert_eq!(res.items[1].pair.key.to_string(), "b");
    assert!(!res.items[1].pair.val.value);
}

#[test]
fn test_named_one_nested() {
    let input = quote!({ my_val });
    bind! {
        let res = (input -> {
            #(inner: #(val: Ident))
        }).unwrap();
    }
    assert_eq!(res.inner.val.to_string(), "my_val");
}

#[test]
fn test_mixed_nested() {
    let input = quote!({ x y z });
    bind! {
        let res = (input -> {
            #(a: Ident)
            #(nested: #(b: Ident) #(c: Ident))
        }).unwrap();
    }
    assert_eq!(res.a.to_string(), "x");
    assert_eq!(res.nested.b.to_string(), "y");
    assert_eq!(res.nested.c.to_string(), "z");
}
