use crate::doctavious_error::{DoctaviousError, Result as DoctavousResult};
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use tera::{Context, Function, Tera};

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
    pub fn render<S>(
        &self,
        template: &str,
        context: &S,
    ) -> DoctavousResult<String>
    where
        S: Serialize,
    {
        let tera_context = Context::from_serialize(context)?;
        return Ok(self.tera.render(template, &tera_context)?);
    }

    pub fn register_function<F: Function + 'static>(
        &mut self,
        name: &str,
        function: F,
    ) {
        self.tera.register_function(name, function)
    }

    pub fn one_off<S>(
        template: &str,
        context: &S,
        escape: bool,
    ) -> DoctavousResult<String>
    where
        S: Serialize,
    {
        let tera_context = Context::from_serialize(context)?;
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
