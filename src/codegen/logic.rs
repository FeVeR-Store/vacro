mod capture;
mod input;
mod keyword;
mod pattern;

use crate::ast::keyword::KeywordMap;

pub struct Compiler {
    /// 用于收集所有的关键字定义，最后统一生成在头部
    keyword_map: KeywordMap,
    /// 临时变量计数器
    temp_counter: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            keyword_map: KeywordMap::new(),
            temp_counter: 0,
        }
    }

    /// 获取唯一的临时变量名
    fn next_temp_ident(&mut self) -> syn::Ident {
        let i = self.temp_counter;
        self.temp_counter += 1;
        quote::format_ident!("_temp_{}", i)
    }
}
