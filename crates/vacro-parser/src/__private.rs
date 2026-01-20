use std::any::type_name;
use std::marker::PhantomData;
use std::ops::Deref;

// 优先级标签
pub struct PriorityHigh;
pub struct PriorityLow;

impl Deref for PriorityHigh {
    type Target = PriorityLow;
    fn deref(&self) -> &Self::Target {
        &PriorityLow
    }
}

// 代理对象
pub struct HelpQuery<T>(PhantomData<T>);
impl<T> HelpQuery<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

// help! 实现的接口
pub trait CustomHelp {
    fn custom_message() -> String;
}

// 默认逻辑的 Trait
pub trait HelpImplDefault {
    // 注意：参数接收 &PriorityLow
    fn get_message(&self, _p: &PriorityLow) -> String;
}

// 为所有 HelpQuery<T> 实现默认逻辑
impl<T> HelpImplDefault for HelpQuery<T> {
    fn get_message(&self, _: &PriorityLow) -> String {
        let name = type_name::<T>().split("::").last().unwrap_or("Unknown");
        format!("<{}>", name)
    }
}

// 自定义逻辑的 Trait
pub trait HelpImplCustom {
    // 参数接收&PriorityHigh, 在没有实现自定义逻辑时，会进行deref拿到&PriorityLow，调用默认实现
    fn get_message(&self, _p: &PriorityHigh) -> String;
}

// 只有当 T: CustomHelp 时，HelpQuery<T> 才拥有这个 Trait
impl<T: CustomHelp> HelpImplCustom for HelpQuery<T> {
    fn get_message(&self, _: &PriorityHigh) -> String {
        <T as CustomHelp>::custom_message()
    }
}
