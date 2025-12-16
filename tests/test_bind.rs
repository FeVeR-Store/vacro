use quote::quote;
use syn::{Ident, LitInt, Type};
use vacro::bind;

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

// 测试多态/枚举捕获 (新功能)
#[test]
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
