use quote::quote;
use syn::{parse2, Ident, LitInt};
use vacro_parser::define;

// 1. 基础测试：最简单的结构体定义
// 定义一个名为 Simple 的解析器，格式为 "kw" + Ident
define!(Simple:
    kw
    #(name: Ident)
);

#[test]
fn test_simple_define() {
    let input = quote!( kw hello );
    let res: Simple = parse2(input).unwrap();
    assert_eq!(res.name.to_string(), "hello");
}

// 2. 复杂测试：包含可选、重复和嵌套
// 模拟函数签名：fn name ( arg, arg ) -> ret
define!(FuncSig:
    fn
    #(name: Ident)
    ( #(args*[,]: Ident) )
    #(?: -> #(ret: Ident))
);

#[test]
fn test_complex_struct() {
    // Case A: 完整形式
    let input_full = quote!( fn my_func (a, b, c) -> bool );
    let res_full: FuncSig = parse2(input_full).unwrap();

    assert_eq!(res_full.name.to_string(), "my_func");
    assert_eq!(res_full.args.len(), 3);
    assert_eq!(res_full.args[1].to_string(), "b");
    assert!(res_full.ret.is_some());
    assert_eq!(res_full.ret.unwrap().to_string(), "bool");

    // Case B: 缺省形式 (没有返回值)
    let input_short = quote!( fn run() );
    let res_short: FuncSig = parse2(input_short).unwrap();

    assert_eq!(res_short.name.to_string(), "run");
    assert!(res_short.args.is_empty());
    assert!(res_short.ret.is_none());
}

// 3. 多态测试 (Enum Generation)
// 测试 define! 是否能正确生成并使用枚举
// 格式：#(data: PolyEnum { Id: Ident, Num: LitInt })
define!(PolyWrapper:
    start
    #(data: PolyEnum {
        Id: Ident,
        Num: LitInt
    })
    end
);

#[test]
fn test_define_enum_generation() {
    // 分支 1: Ident
    let input1 = quote!( start my_id end );
    let res1: PolyWrapper = parse2(input1).unwrap();

    match res1.data {
        PolyEnum::Id(id) => assert_eq!(id.to_string(), "my_id"),
        _ => panic!("Expected Id variant"),
    }

    // 分支 2: LitInt
    let input2 = quote!( start 123 end );
    let res2: PolyWrapper = parse2(input2).unwrap();

    match res2.data {
        PolyEnum::Num(n) => assert_eq!(n.base10_digits(), "123"),
        _ => panic!("Expected Num variant"),
    }
}
