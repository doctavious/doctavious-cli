use crate::doctavious_error::{DoctaviousError, Result as DoctavousResult};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use serde_json::{to_value, Value};
use tera::{Context, Function, Tera};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TemplateContext {
    data: BTreeMap<String, Value>,
}

impl TemplateContext {
    /// Initializes an empty context
    pub fn new() -> Self {
        Self { data: BTreeMap::new() }
    }

    /// Takes a serde-json `Value` and convert it into a `Context` with no overhead/cloning.
    pub fn from_value(obj: Value) -> DoctavousResult<Self> {
        match obj {
            Value::Object(m) => {
                let mut data = BTreeMap::new();
                for (key, value) in m {
                    data.insert(key, value);
                }
                Ok(TemplateContext { data })
            }
            _ => Err(DoctaviousError::Msg(
                String::from("Creating a Context from a Value/Serialize requires it being a JSON object"),
            )),
        }
    }

    /// Converts the `val` parameter to `Value` and insert it into the context.
    ///
    /// Panics if the serialization fails.
    ///
    /// ```rust
    /// # use templates::TemplateContext;
    /// let mut context = templates::TemplateContext::new();
    /// context.insert("number_users", &42);
    /// ```
    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.data.insert(key.into(), to_value(val).unwrap());
    }

    /// Takes something that impl Serialize and create a context with it.
    /// Meant to be used if you have a hashmap or a struct and don't want to insert values
    /// one by one in the context.
    pub fn from_serialize(value: impl Serialize) -> DoctavousResult<Self> {
        let obj = to_value(value)?;
        TemplateContext::from_value(obj)
    }

    /// Returns the value at a given key index.
    pub fn get(&self, index: &str) -> Option<&Value> {
        self.data.get(index)
    }

    /// Remove a key from the context, returning the value at the key if the key was previously
    /// inserted into the context.
    pub fn remove(&mut self, index: &str) -> Option<Value> {
        self.data.remove(index)
    }

    /// Checks if a value exists at a specific index.
    pub fn contains_key(&self, index: &str) -> bool {
        self.data.contains_key(index)
    }
}

/// Wrapper for [`tera`].
#[derive(Debug)]
pub struct Templates {
    tera: Tera,
}

impl Templates {
    /// Constructs a new instance.
    pub fn new() -> DoctavousResult<Self> {
        let tera = Tera::default();
        return Ok(Self { tera });
    }

    pub fn new_with_templates(
        templates: HashMap<&str, String>,
    ) -> DoctavousResult<Self> {
        let mut tera = Tera::default();
        for (k, v) in templates {
            if let Err(e) = tera.add_raw_template(k, v.as_str()) {
                return if let Some(error_source) = e.source() {
                    Err(DoctaviousError::TemplateParseError(
                        error_source.to_string(),
                    ))
                } else {
                    Err(DoctaviousError::TemplateError(e))
                };
            }
        }

        return Ok(Self { tera });
    }

    // TODO: probably makes sense to make this Into<&str, String>?
    /// Renders the template.
    pub fn render(
        &self,
        template: &str,
        context: &TemplateContext,
    ) -> DoctavousResult<String> {
        let tera_context = Context::from_serialize(&context.data)?;
        return Ok(self.tera.render(template, &tera_context)?);
    }

    pub fn register_function<F: Function + 'static>(
        &mut self,
        name: &str,
        function: F,
    ) {
        self.tera.register_function(name, function)
    }

    pub fn one_off(
        template: &str,
        context: &TemplateContext,
        escape: bool,
    ) -> DoctavousResult<String> {
        let tera_context = Context::from_serialize(&context.data)?;
        println!("{:?}", tera_context);
        return Ok(Tera::one_off(template, &tera_context, escape)?);
    }
}

// TODO: tests
#[cfg(test)]
mod tests {
    use crate::output::Output;
    use std::path::Path;
    use std::{env, fs};

    // TODO: invalid template should return valid error
}
