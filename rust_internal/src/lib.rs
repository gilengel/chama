use std::any::Any;

pub mod plugin;

pub trait GetPlugin<T> {
    fn get<S>(&self) -> Option<S>;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}