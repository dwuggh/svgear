use anyhow::Result;

pub trait Painter {
    fn paint(&self, content: &str) -> Result<String>;
}
