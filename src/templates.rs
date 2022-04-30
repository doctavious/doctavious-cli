use crate::utils::parse_enum;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use clap::ArgEnum;
use tera::{Context as TeraContext, Context, Function, Result as TeraResult, Tera, Value};
use crate::doctavious_error::{DoctaviousError, Result as DoctavousResult, EnumError};
use csv::ReaderBuilder;
use log::Record;
use indexmap::map::IndexMap;

/// Wrapper for [`tera`].
#[derive(Debug)]
pub struct Templates {
    tera: Tera,
}

impl Templates {
    /// Constructs a new instance.
    pub fn new() -> DoctavousResult<Self> {
        let mut tera = Tera::default();
        return Ok(Self { tera });
    }

    pub fn new_with_templates(templates: HashMap<&str, String>) -> DoctavousResult<Self> {
        let mut tera = Tera::default();
        for (k, v) in templates {
            if let Err(e) = tera.add_raw_template(k, v.as_str()) {
                return if let Some(error_source) = e.source() {
                    Err(DoctaviousError::TemplateParseError(error_source.to_string()))
                } else {
                    Err(DoctaviousError::TemplateError(e))
                };
            }
        }

        return Ok(Self { tera });
    }

    // TODO: probably makes sense to make this Into<&str, String>?
    /// Renders the template.
    pub fn render<S>(&self, template: &str, context: &S) -> DoctavousResult<String>
        where
            S: Serialize,
    {
        let tera_context = Context::from_serialize(context)?;
        return Ok(self.tera.render(template, &tera_context)?);
    }

    pub fn register_function<F: Function + 'static>(&mut self, name: &str, function: F) {
        self.tera.register_function(name, function)
    }

    pub fn one_off<S>(template: &str, context: &S, escape: bool) -> DoctavousResult<String>
        where
            S: Serialize
    {
        let tera_context = Context::from_serialize(context)?;
        return Ok(Tera::one_off(template, &tera_context, escape)?);
    }
}


// TODO: tests
#[cfg(test)]
mod tests {
    use std::{fs, env};
    use std::path::Path;
    use crate::output::Output;

    // TODO: invalid template should return valid error

}
