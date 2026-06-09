use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    parse2, Block, Expr, FieldValue, FnArg, Generics, Ident, LitBool, LitInt, Member, PatType,
    Receiver, ReturnType, Stmt, Token, Type,
};
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

define!(MessageSummary:
    message #(message*: Ident) and summary #(summary*: Ident)
);

#[test]
fn test_blank_separated_iter_capture() {
    let input = quote!(message hello world and summary short note);
    let res: MessageSummary = parse2(input).unwrap();

    let message: Vec<_> = res.message.iter().map(ToString::to_string).collect();
    let summary: Vec<_> = res.summary.iter().map(ToString::to_string).collect();

    assert_eq!(message, vec!["hello", "world"]);
    assert_eq!(summary, vec!["short", "note"]);
}

define!(BlankBracketMessageSummary:
    mod #(message*[]: Ident) as #(summary*[ ]: Ident)
);

#[test]
fn test_empty_bracket_iter_capture() {
    let input = quote!(mod hello world as short note);
    let res: BlankBracketMessageSummary = parse2(input).unwrap();

    let message: Vec<_> = res.message.iter().map(ToString::to_string).collect();
    let summary: Vec<_> = res.summary.iter().map(ToString::to_string).collect();

    assert_eq!(message, vec!["hello", "world"]);
    assert_eq!(summary, vec!["short", "note"]);
}

define!(AnonymousNestedTokens:
    #(*: #(token: Ident))
);

#[test]
fn test_anonymous_nested_iter_capture() {
    let input = quote!(alpha beta gamma);
    let res: AnonymousNestedTokens = parse2(input).unwrap();

    let tokens: Vec<_> = res.token.iter().map(ToString::to_string).collect();
    assert_eq!(tokens, vec!["alpha", "beta", "gamma"]);
}

define!(AnonymousNestedPairs:
    #(*: #(name: Ident) as #(ty: Type))
);

#[test]
fn test_anonymous_nested_iter_multi_field_capture() {
    let input = quote!(alpha as i32 beta as String);
    let res: AnonymousNestedPairs = parse2(input).unwrap();

    let names: Vec<_> = res.name.iter().map(ToString::to_string).collect();
    let types: Vec<_> = res.ty.iter().map(|ty| quote! {#ty}.to_string()).collect();

    assert_eq!(names, vec!["alpha", "beta"]);
    assert_eq!(types, vec!["i32", "String"]);
}

define!(AnonymousNestedBlankBracketPairs:
    #(*[]: #(name: Ident) as #(ty: Type))
);

#[test]
fn test_anonymous_nested_empty_bracket_iter_multi_field_capture() {
    let input = quote!(alpha as i32 beta as String);
    let res: AnonymousNestedBlankBracketPairs = parse2(input).unwrap();

    let names: Vec<_> = res.name.iter().map(ToString::to_string).collect();
    let types: Vec<_> = res.ty.iter().map(|ty| quote! {#ty}.to_string()).collect();

    assert_eq!(names, vec!["alpha", "beta"]);
    assert_eq!(types, vec!["i32", "String"]);
}

define!(AnonymousNestedCommaTokens:
    #(*[,]: #(token: Ident))
);

#[test]
fn test_anonymous_nested_iter_punctuated_capture() {
    let input = quote!(alpha, beta, gamma);
    let res: AnonymousNestedCommaTokens = parse2(input).unwrap();

    let tokens: Vec<_> = res.token.iter().map(ToString::to_string).collect();
    assert_eq!(tokens, vec!["alpha", "beta", "gamma"]);
    assert_eq!(res.token.len(), 3);
}

#[test]
fn test_anonymous_nested_iter_punctuated_trailing_capture() {
    let input = quote!(alpha, beta,);
    let res: AnonymousNestedCommaTokens = parse2(input).unwrap();

    let tokens: Vec<_> = res.token.iter().map(ToString::to_string).collect();
    assert_eq!(tokens, vec!["alpha", "beta"]);
    assert!(res.token.trailing_punct());
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

define!(CommandDescriptor:
    #(token*: CommandToken {
        Literal: Ident,
        Variable: < #(variable: Ident) >,
        PipeVariable: | #(variable: Ident) |,
        AtVariable: @ #(variable: Ident) @,
    })
);

#[test]
fn test_poly_capture_pattern_variant_trailing_comma() {
    let input = quote!(literal < angle > | pipe | @ at @);
    let res: CommandDescriptor = parse2(input).unwrap();

    assert_eq!(res.token.len(), 4);
    match &res.token[0] {
        CommandToken::Literal(ident) => assert_eq!(ident.to_string(), "literal"),
        _ => panic!("1st token should be Literal"),
    }
    match &res.token[1] {
        CommandToken::Variable { variable } => assert_eq!(variable.to_string(), "angle"),
        _ => panic!("2nd token should be Variable"),
    }
    match &res.token[2] {
        CommandToken::PipeVariable { variable } => assert_eq!(variable.to_string(), "pipe"),
        _ => panic!("3rd token should be PipeVariable"),
    }
    match &res.token[3] {
        CommandToken::AtVariable { variable } => assert_eq!(variable.to_string(), "at"),
        _ => panic!("4th token should be AtVariable"),
    }
}

define!(ComplexCommandDescriptorLiteral:
    #(token*: ComplexCommandTokenLiteral {
        Literal: Ident,
        Variable: <#(variable: Ident)#(descriptor?: ComplexDescriptorLiteral {
            Optional: #{?},
            Required: #{!}
        }): #(?: #{:} #(ret: Type))>:#(flag: TokenStream),
    })
);

#[test]
fn test_poly_capture_optional_nested_literal_colon_edge() {
    let input = quote!(<value?: : String>:flag);
    let res: ComplexCommandDescriptorLiteral = parse2(input).unwrap();

    assert_eq!(res.token.len(), 1);
    match &res.token[0] {
        ComplexCommandTokenLiteral::Variable {
            variable,
            descriptor,
            ret,
            flag,
        } => {
            assert_eq!(variable.to_string(), "value");
            assert!(matches!(
                descriptor,
                Some(ComplexDescriptorLiteral::Optional)
            ));
            let ret = ret.as_ref().expect("ret should be parsed");
            assert_eq!(quote! {#ret}.to_string(), "String");
            assert_eq!(quote! {#flag}.to_string(), "flag");
        }
        _ => panic!("token should be Variable"),
    }
}

define!(ComplexCommandDescriptorBare:
    #(token*: ComplexCommandTokenBare {
        Literal: Ident,
        Variable: <#(variable: Ident)#(descriptor?: ComplexDescriptorBare {
            Optional: #{?},
            Required: #{!}
        }): #(?: : #(ret: Type))>:#(flag: TokenStream),
    })
);

#[test]
fn test_poly_capture_optional_nested_bare_colon_edge() {
    let input = quote!(<value!: : String>:flag);
    let res: ComplexCommandDescriptorBare = parse2(input).unwrap();

    assert_eq!(res.token.len(), 1);
    match &res.token[0] {
        ComplexCommandTokenBare::Variable {
            variable,
            descriptor,
            ret,
            flag,
        } => {
            assert_eq!(variable.to_string(), "value");
            assert!(matches!(descriptor, Some(ComplexDescriptorBare::Required)));
            let ret = ret.as_ref().expect("ret should be parsed");
            assert_eq!(quote! {#ret}.to_string(), "String");
            assert_eq!(quote! {#flag}.to_string(), "flag");
        }
        _ => panic!("token should be Variable"),
    }
}

define!(
    #[derive(Debug)]
    pub CommandDescriptorWithOptionalFlag:
    #(token*: CommandTokenWithOptionalFlag {
        Literal: Ident,
        Variable: <#(variable: Ident)#(descriptor?: CommandDescriptorKind {
            Optional: #{?},
            Required: #{!}
        }) #(?: #{:} #(ty: Type))>#(?: #{:} #(flag: TokenStream)),
    })
);

define!(
    #[derive(Debug)]
    pub CliWithCommandDescriptor: #(executable: Ident) {
        #(sub_command*[,]: CommandWithDescriptor {
            SubCommand: CliWithCommandDescriptor,
            Arg: #(@: CommandDescriptorWithOptionalFlag)
        })
    }
);

#[test]
fn test_poly_capture_optional_flag_inside_recursive_enum_list() {
    let input = quote! {
        git {
            commit <message>:-m,
            clone <message: String>,
        }
    };
    let res: CliWithCommandDescriptor = parse2(input).unwrap();

    assert_eq!(res.executable.to_string(), "git");
    assert_eq!(res.sub_command.len(), 2);

    let CommandWithDescriptor::Arg(commit) = &res.sub_command[0] else {
        panic!("1st command should be an arg descriptor");
    };
    assert_eq!(commit.token.len(), 2);
    match &commit.token[1] {
        CommandTokenWithOptionalFlag::Variable {
            variable, ty, flag, ..
        } => {
            assert_eq!(variable.to_string(), "message");
            assert!(ty.is_none());
            assert_eq!(quote! {#flag}.to_string(), "- m");
        }
        _ => panic!("2nd commit token should be a variable"),
    }

    let CommandWithDescriptor::Arg(clone) = &res.sub_command[1] else {
        panic!("2nd command should be an arg descriptor");
    };
    assert_eq!(clone.token.len(), 2);
    match &clone.token[1] {
        CommandTokenWithOptionalFlag::Variable {
            variable, ty, flag, ..
        } => {
            assert_eq!(variable.to_string(), "message");
            let ty = ty.as_ref().expect("type should be parsed");
            assert_eq!(quote! {#ty}.to_string(), "String");
            assert!(flag.is_none());
        }
        _ => panic!("2nd clone token should be a variable"),
    }
}

#[test]
fn test_poly_capture_optional_type_with_generic_comma_in_recursive_enum_list() {
    let input = quote! {
        git {
            clone <message: HashMap<String, Vec<u8>>>
        }
    };
    let res: CliWithCommandDescriptor = parse2(input).unwrap();

    assert_eq!(res.sub_command.len(), 1);
    let CommandWithDescriptor::Arg(clone) = &res.sub_command[0] else {
        panic!("command should be an arg descriptor");
    };
    match &clone.token[1] {
        CommandTokenWithOptionalFlag::Variable { ty, flag, .. } => {
            let ty = ty.as_ref().expect("type should be parsed");
            assert_eq!(quote! {#ty}.to_string(), "HashMap < String , Vec < u8 > >");
            assert!(flag.is_none());
        }
        _ => panic!("2nd token should be a variable"),
    }
}

// 4. 关联捕获
define!(MyRoles: {
    #(roles*[,]: #(ident: Ident))
});

#[test]
fn test_named_nested_list() {
    let input = quote!({ a, b, c });
    let res: MyRoles = parse2(input).unwrap();
    assert_eq!(res.roles.len(), 3);
    // Check if the inner struct is accessible and correct
    assert_eq!(res.roles[0].ident.to_string(), "a");
    assert_eq!(res.roles[1].ident.to_string(), "b");
    assert_eq!(res.roles[2].ident.to_string(), "c");
}

define!(MyConfig: {
    #(items*[,]: #(pair: #(key: Ident): #(val: LitBool)))
});

#[test]
fn test_named_nested_complex() {
    let input = quote!({ a: true, b: false });
    let res: MyConfig = parse2(input).unwrap();
    assert_eq!(res.items.len(), 2);

    // Accessing nested struct fields
    assert_eq!(res.items[0].pair.key.to_string(), "a");
    assert!(res.items[0].pair.val.value);

    assert_eq!(res.items[1].pair.key.to_string(), "b");
    assert!(!res.items[1].pair.val.value);
}

define!(SingleWrapper: {
    #(inner: #(val: Ident))
});

#[test]
fn test_named_one_nested() {
    let input = quote!({ my_val });
    let res: SingleWrapper = parse2(input).unwrap();
    assert_eq!(res.inner.val.to_string(), "my_val");
}

define!(Mixed: {
    #(a: Ident)
    #(nested: #(b: Ident) #(c: Ident))
});

#[test]
fn test_mixed_nested() {
    let input = quote!({ x y z });
    let res: Mixed = parse2(input).unwrap();
    assert_eq!(res.a.to_string(), "x");
    assert_eq!(res.nested.b.to_string(), "y");
    assert_eq!(res.nested.c.to_string(), "z");
}

define!(pub Method:
    #(asyncness?: Token![async])
    #(unsafety?: Token![unsafe])
    #(name: Ident)#(?: <#(generic*[,]: Generics)>)(#(inputs*[,]: FnArg))#(output: ReturnType)#(block: Block)
);
define!(pub Property:
    pub #(name: Ident): #(ty: Type) #(?: = #(default: Expr))
);

define!(DeviceCofig:
    #(name: Ident) {
        #(config_items*[,]: Config {
            FieldValue,
            Method,
            Property
        })
    }
);

#[test]
fn test_device_config() {
    let input = quote! {
        DeviceA {
            transport: LocalSocket,
            batch,
            async get_name(&self) -> String {
                self.name
            },
            pub name: String = "device-a".to_string()
        }
    };
    let device_config = DeviceCofig::parse.parse2(input).unwrap();
    assert_eq!(device_config.name.to_string(), "DeviceA");
    if let Config::FieldValue(FieldValue {
        member: Member::Named(named),
        expr,
        ..
    }) = device_config.config_items.get(0).unwrap()
    {
        assert_eq!(named.to_string(), "transport");
        assert_eq!(quote! {#expr}.to_string(), "LocalSocket");
    } else {
        panic!("1st field should be FieldValue")
    }

    if let Config::FieldValue(FieldValue {
        member: Member::Named(named),
        colon_token: None,
        ..
    }) = device_config.config_items.get(1).unwrap()
    {
        assert_eq!(named.to_string(), "batch");
    } else {
        panic!("2nd field should be FieldValue without colon_token")
    }

    if let Config::Method(Method {
        asyncness: Some(_),
        unsafety: None,
        name,
        generic: None,
        inputs,
        output,
        block,
    }) = device_config.config_items.get(2).unwrap()
    {
        assert_eq!(name.to_string(), "get_name");
        let FnArg::Receiver(Receiver {
            reference: Some(_),
            mutability: None,
            colon_token: None,
            ..
        }) = inputs.get(0).unwrap()
        else {
            panic!("fn input should be receiver `&self`");
        };
        if let ReturnType::Type(_, ty) = output {
            assert_eq!(quote! {#ty}.to_string(), "String");
        } else {
            panic!("fn output should be `String`");
        }
        if let Some(Stmt::Expr(expr, None)) = block.stmts.first() {
            assert_eq!(quote! {#expr}.to_string(), "self . name");
        } else {
            panic!("fn body should be `self.name`")
        }
    } else {
        panic!("3rd field should be Method")
    }

    if let Config::Property(Property { name, ty, default }) =
        device_config.config_items.get(3).unwrap()
    {
        assert_eq!(name.to_string(), "name");
        assert_eq!(quote! {#ty}.to_string(), "String");
        let Some(expr) = default else {
            panic!(r#"default value should be `"device-a".to_string()`"#);
        };
        assert_eq!(quote! {#expr}.to_string(), r#""device-a" . to_string ()"#)
    }
}

define!(TransportInput:
    #(name: Ident)<#(adapter: Ident)>(#(args*[,]: PatType)) {
        #(fields*[,]: #(@: Ident) #(@?: :#(@: Type) #(?: = #(@: Expr))))
    }
);

#[test]
fn test_transport_input() {
    let input = quote! {
        TransportA<Adapter>(name: String) {
            name,
            version: i32,
            description: String = "transport-a"
        }
    };
    let transport_input = TransportInput::parse.parse2(input).unwrap();
    assert_eq!(transport_input.name.to_string(), "TransportA");
    assert_eq!(transport_input.adapter.to_string(), "Adapter");

    let PatType { pat, ty, .. } = &transport_input.args[0];
    assert_eq!(quote! {#pat}.to_string(), "name");
    assert_eq!(quote! {#ty}.to_string(), "String");

    if let Some((ident, None)) = transport_input.fields.get(0) {
        assert_eq!(ident.to_string(), "name");
    } else {
        panic!("1st field should be `name`");
    }
    if let Some((ident, Some((ty, None)))) = transport_input.fields.get(1) {
        assert_eq!(ident.to_string(), "version");
        assert_eq!(quote! {#ty}.to_string(), "i32");
    } else {
        panic!("2nd field should be `version: i32`");
    }
    if let Some((ident, Some((ty, Some(default_val))))) = transport_input.fields.get(2) {
        assert_eq!(ident.to_string(), "description");
        assert_eq!(quote! {#ty}.to_string(), "String");
        assert_eq!(quote! {#default_val}.to_string(), r#""transport-a""#);
    } else {
        panic!(r#"3rd field should be `description: String = "transport-a"`"#);
    }
}

define!(BacktrackTestParser:
    #(?: #(syn::Ident) #(syn::Token![,]))
    #(target_ident: syn::Ident) #(syn::Token![;])
);

#[test]
fn test_anonymous_optional_backtracking() {
    use syn::parse::Parser;
    let input_backtrack = quote! {
        Target;
    };

    let result = BacktrackTestParser::parse.parse2(input_backtrack);

    match result {
        Ok(output) => {
            assert_eq!(output.target_ident.to_string(), "Target",);
        }
        Err(e) => {
            panic!("Unexpected cursor position, error: {}", e);
        }
    }

    let input_full = quote! {
        Prefix, Target;
    };

    let result_full = BacktrackTestParser::parse.parse2(input_full).unwrap();
    assert_eq!(result_full.target_ident.to_string(), "Target");
}

define!(
    #[derive(Debug)]
    Device:
        #(name: Ident) {
            #(field*[,]: Field {
                Transport: transport: #(@: Expr), // transport: Transport("localhost:8080")
                Package: package: #(@: Expr), // package: package::name
                Batch: #{batch}, // batch
                Event: event #(name: Ident)#(body: EventBody {
                    Named: { #(field*[,]: PatType) }, // event Test { a: i16, b: bool }
                    Unamed: ( #(field*[,]: #(Type)) ) // event Test(i16, bool)
                }),
                Channel: channel #(@:Ident), // channel Test
            })
        }
);

#[test]
fn test_device() {
    use syn::parse::Parser;

    let input = quote! {
        MyDevice {
            transport: LocalSocket,
            package: my_package::name,
            batch,
            event Connected { peer_id: u32 },
            event Data(u8, bool),
            channel Status
        }
    };

    let device = Device::parse.parse2(input).unwrap();
    assert_eq!(device.name.to_string(), "MyDevice");
    assert_eq!(device.field.len(), 6);

    // 1. Transport: transport: <Expr>
    match device.field.iter().next().unwrap() {
        Field::Transport(expr) => {
            assert_eq!(quote! {#expr}.to_string(), "LocalSocket");
        }
        _ => panic!("1st field should be Transport"),
    }

    // 2. Package: package: <Expr>
    match device.field.iter().nth(1).unwrap() {
        Field::Package(expr) => {
            assert_eq!(quote! {#expr}.to_string(), "my_package :: name");
        }
        _ => panic!("2nd field should be Package"),
    }

    // 3. Batch: batch (literal keyword, no captures)
    match device.field.iter().nth(2).unwrap() {
        Field::Batch => {} // unit variant, no captures
        _ => panic!("3rd field should be Batch"),
    }

    // 4. Event with Named body: event Connected { peer_id: u32 }
    match device.field.iter().nth(3).unwrap() {
        Field::Event { name, body } => {
            assert_eq!(name.to_string(), "Connected");
            match body {
                EventBody::Named { field } => {
                    assert_eq!(field.len(), 1);
                    let PatType { pat, ty, .. } = field.first().unwrap();
                    assert_eq!(quote! {#pat}.to_string(), "peer_id");
                    assert_eq!(quote! {#ty}.to_string(), "u32");
                }
                _ => panic!("4th field body should be Named"),
            }
        }
        _ => panic!("4th field should be Event"),
    }

    // 5. Event with Unamed body: event Data(u8, bool)
    match device.field.iter().nth(4).unwrap() {
        Field::Event { name, body } => {
            assert_eq!(name.to_string(), "Data");
            match body {
                EventBody::Unamed { field } => {
                    assert_eq!(field.len(), 2);
                }
                _ => panic!("5th field body should be Unamed"),
            }
        }
        _ => panic!("5th field should be Event"),
    }

    // 6. Channel: channel <Ident>
    match device.field.iter().nth(5).unwrap() {
        Field::Channel(ident) => {
            assert_eq!(ident.to_string(), "Status");
        }
        _ => panic!("6th field should be Channel"),
    }
}

define!(
    #[derive(Debug)]
    Password:
        #{world is mine!}
);

#[test]
fn test_literal() {
    let input = quote! {
        world is mine!
    };
    let _ = Password::parse.parse2(input).expect("that's right");

    let input = quote! {
        word is mine?
    };
    let _ = Password::parse.parse2(input).expect_err("that's wrong");
}
