

// TODO: can also access fields like the following. Make sure to include in docs
// {%- for row in data %}
// | {{- row['status'] }} | {{ row['RFD'] }} |
// {%- endfor -%}

use std::collections::HashMap;
use std::path::PathBuf;
use csv::ReaderBuilder;
use indexmap::IndexMap;
use serde_json::{json, to_value, Value};
use crate::TemplateExtension;
use crate::templates::Templates;
use std::string::String;
use serde_json::value::Value as Json;


pub(crate) fn toc_template(extension: TemplateExtension) ->  &'static str {
    return match extension {
        TemplateExtension::Markdown => {
            r#"<!-- snippet::markdown_toc -->
{% if headers -%}
  | {{ headers | join(sep=" | ") }} |
|
    {%- for i in range(end=headers|length) -%}
        --- |
    {%- endfor -%}
{%- endif -%}
{% for row in data %}
|
    {%- for key, value in row %}
        {{- value }} |
    {%- endfor -%}
{%- endfor %}
<!-- end::markdown_toc -->"#
        }
        TemplateExtension::Asciidoc => {
            r#"<!-- snippet::asciidoc_toc -->
|===
{% if headers -%}
  | {{ headers | join(sep=" | ") }}
{%- endif %}
{%- for row in data %}
{% for key, value in row %}
| {{ value }}
{%- endfor -%}
{%- endfor %}
<!-- end::asciidoc_toc -->"#
        }
    }
}

// TODO: this should likely live somewhere else. somewhere that could be used by adr and rfds
// toc from Vec<&str> list of content
// toc from Vec<PathBuf> list of files
// toc from list<string> (headers) and Vec<HashMap> (data)
pub(crate) fn render_toc(path: PathBuf, template: &str) -> String {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path).unwrap();

    let mut context: HashMap<&str, Value> = HashMap::new();

    let headers = rdr.headers().unwrap().clone();
    let headers_vec: Vec<String> = headers.deserialize(None).unwrap();
    context.insert("headers", json!(&headers_vec));

    let mut output: Vec<IndexMap<String,String>> = Vec::new();
    for record in rdr.records() {
        // let record: IndexMap<String,Option<String>> = row.unwrap().deserialize(Some(&headers)).unwrap();
        // output.push(record);
        // let record: Record = row.unwrap().deserialize(Some(&headers)).unwrap();
        let mut map: IndexMap<String, String> = IndexMap::new();
        for row in record.iter() {
            for (pos, field) in row.into_iter().enumerate() {
                println!("{} / {}", pos, field);
                map.insert(headers_vec.get(pos).unwrap().to_string(), field.to_string());
            }
        }
        output.push(map);
    }
    // to_value(val).unwrap()
    context.insert("data", to_value(&output).unwrap());
    println!("{:?}", &output);
    println!("{:?}", json!(&context));

    return Templates::one_off(template, &context, false).unwrap();
}


// r#"<!-- snippet::markdown_toc -->
// {{# if headers }}
//   | {{ headers | join(sep=" | ") }}
// |
//     {%- for i in range(end=headers|length) -%}
//         --- |
//     {%- endfor -%}
// {%- endif -%}
// {% for row in data %}
// |
//     {%- for key, value in row %}
//         {{- value }} |
//     {%- endfor -%}
// {%- endfor -%}
// {# end::markdown_toc #}"#


// implement via bare function
// fn toc(h: &Helper, _: &Handlebars, c: &Context, rc: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
//     //           let param = h.param(0).ok_or(RenderError::new("param not found"))?;
//     let param = h.param(0).unwrap();
//
//     let context = c.data().as_object().unwrap();
//     out.write("<!-- snippet::markdown_toc -->")?;
//     let headers =  context.get("headers").unwrap().as_array().unwrap();
//     let t: Vec<&str> = headers.into_iter().map(|v| v.as_str().unwrap()).collect(); //.join(" | ");
//     out.write(("| ".to_owned() + t.join(" | ").as_str()).as_ref())?;
//     // out.write("| " + t.as_str());
//     out.write("|")?;
//     for n in 1..=headers.len() {
//         out.write("--- |")?;
//     }
//     let data =  context.get("data").unwrap().as_object().unwrap();
//     for (k, v) in data {
//         out.write(("| ".to_owned() + &v.to_string() + " |").as_ref())?;
//     }
//     out.write("<!-- end::markdown_toc -->")?;
//
//     Ok(())
// }

// #[derive(Clone, Copy)]
// struct JoinHelper;
//
// impl HelperDef for JoinHelper {
//     fn call_inner<'reg: 'rc, 'rc>(
//         &self,
//         h: &Helper<'reg, 'rc>,
//         r: &'reg Handlebars<'reg>,
//         _: &'rc Context,
//         _: &mut RenderContext<'reg, 'rc>,
//     ) -> Result<ScopedJson<'reg, 'rc>, RenderError> {
//         println!("help....");
//
//         let collection_value = h
//             .param(0)
//             .ok_or_else(|| RenderError::new("Param not found for helper \"lookup\""))?;
//         let index = h
//             .param(1)
//             .ok_or_else(|| RenderError::new("Insufficient params for helper \"lookup\""))?;
//
//         let value = match *collection_value.value() {
//             Json::Array(ref v) => index
//                 .value()
//                 .as_u64()
//                 .and_then(|u| v.get(u as usize))
//                 .unwrap_or(&Json::Null),
//             Json::Object(ref m) => index
//                 .value()
//                 .as_str()
//                 .and_then(|k| m.get(k))
//                 .unwrap_or(&Json::Null),
//             _ => &Json::Null,
//         };
//
//
//         println!("{:?}", collection_value);
//         println!("{:?}", index);
//
//         // let collection = serde_json::to_string(h.param(0).unwrap().value()).unwrap();
//         let collection: Vec<&str> = h.param(0).unwrap().value().as_array().unwrap()
//             .iter()
//             .map(|v| v.as_str().unwrap())
//             .collect();
//         println!("collection: {:?}", collection);
//         let separator = h.param(1).unwrap().value().as_str().unwrap();
//
//         // out.write(collection.join(separator).as_ref());
//
//         // Ok(value.clone().into())
//         Ok(ScopedJson::from(json!(collection.join(separator))))
//     }
// }
//
// static JOIN_HELPER: JoinHelper = JoinHelper;
//
// #[derive(Clone, Copy)]
// struct RepeatHelper;
// impl HelperDef for RepeatHelper {
//     fn call<'reg: 'rc, 'rc>(
//         &self,
//         h: &Helper<'reg, 'rc>,
//         r: &'reg Handlebars<'reg>,
//         ctx: &'rc Context,
//         rc: &mut RenderContext<'reg, 'rc>,
//         out: &mut dyn Output,
//     ) -> HelperResult {
//         println!("repeat please");
//         let param = h.param(0).ok_or(RenderError::new("param not found"))?;
//         for _ in 1..=param.value().as_i64().unwrap() {
//             h.template().unwrap().render(r, ctx, rc, out)?;
//         }
//
//         Ok(())
//     }
// }
//
// static REPEAT_HELPER: RepeatHelper = RepeatHelper;

#[cfg(test)]
mod tests {
    use std::{fs, env};
    use std::collections::HashMap;
    use std::path::Path;
    // use handlebars::Handlebars;
    // use crate::commands::design_decisions::toc::{JoinHelper, RepeatHelper, toc_template};
    use crate::commands::design_decisions::toc::{toc_template};

    use crate::output::Output;
    use crate::templates::TemplateExtension;
    use std::string::String;

    #[test]
    fn markdown_toc() {
        let toc = super::render_toc(
            Path::new("./tests/resources/sample_csv.csv").to_path_buf(),
            toc_template(TemplateExtension::Markdown)
        );
        let expected = r#"<!-- snippet::markdown_toc -->
| status | RFD |
|--- |--- |
|published |[RFD 1 Something](./rfd/0001/README.md) |
|draft |[RFD 2 Another Thing](./rfd/0002/README.md) |
<!-- end::markdown_toc -->"#;
        assert_eq!(expected, toc);
    }

    #[test]
    fn asciidoc_toc() {
        let toc = super::render_toc(
            Path::new("./tests/resources/sample_csv.csv").to_path_buf(),
            toc_template(TemplateExtension::Asciidoc)
        );

        let expected = r#"<!-- snippet::asciidoc_toc -->
|===
| status | RFD

| published
| [RFD 1 Something](./rfd/0001/README.md)

| draft
| [RFD 2 Another Thing](./rfd/0002/README.md)
<!-- end::asciidoc_toc -->"#;
        assert_eq!(expected, toc);
    }

    // #[test]
    // fn join() {
    //     println!("what...");
    //     let mut handlebars = Handlebars::new();
    //     handlebars.register_helper("join", Box::new(JoinHelper));
    //     assert!(handlebars
    //         .register_template_string("t0", "{{join headers \"|\"}}")
    //         .is_ok());
    //
    //     let mut data = HashMap::new();
    //     data.insert("headers", vec!["header_1", "header_2"]);
    //     let r0 = handlebars.render("t0", &data);
    //     assert_eq!(r0.ok().unwrap(), "header_1|header_2".to_string());
    // }

    // #[test]
    // fn repeat() {
    //     let mut handlebars = Handlebars::new();
    //     handlebars.register_helper("repeat", Box::new(RepeatHelper));
    //     assert!(handlebars
    //         .register_template_string("t0", "{{#repeat 2}}hello {{/repeat}}")
    //         .is_ok());
    //
    //     let r0 = handlebars.render("t0", &String::from(""));
    //     assert_eq!(r0.ok().unwrap(), "hello hello ".to_string());
    // }
}

