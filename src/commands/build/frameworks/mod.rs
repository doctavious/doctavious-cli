use crate::commands::build::framework::FrameworkSupport;
use crate::commands::build::frameworks::antora::Antora;
use crate::commands::build::frameworks::astro::Astro;
use crate::commands::build::frameworks::docfx::DocFx;
use crate::commands::build::frameworks::docusaurus_v2::DocusaurusV2;
use crate::commands::build::frameworks::eleventy::Eleventy;
use crate::commands::build::frameworks::gatsby::Gatsby;
use crate::commands::build::frameworks::hexo::Hexo;
use crate::commands::build::frameworks::hugo::Hugo;
use crate::commands::build::frameworks::jekyll::Jekyll;
use crate::commands::build::frameworks::mdbook::MDBook;
use crate::commands::build::frameworks::mkdocs::MKDocs;
use crate::commands::build::frameworks::nextjs::NextJS;
use crate::commands::build::frameworks::nuxtjs::NuxtJS;
use crate::commands::build::frameworks::sphinx::Sphinx;
use crate::commands::build::frameworks::sveltekit::SvelteKit;
use crate::commands::build::frameworks::vitepress::VitePress;
use crate::commands::build::frameworks::vuepress::VuePress;

mod antora;
mod astro;
mod docfx;
mod docusaurus_v2;
mod eleventy;
mod gatsby;
mod hugo;
mod jekyll;
mod hexo;
mod mdbook;
mod mkdocs;
mod nextjs;
mod nuxtjs;
mod nuxt_v3;
mod sphinx;
mod statiq;
mod sveltekit;
mod vitepress;
mod vuepress;


// I wish Box<dyn> hasnt necessary and maybe its not with a different structure
// but I'm at a loss for how how to structure these frameworks and allow fn overrides,
// so I suppose this will have to work until I or someone else comes up with something better
pub fn get_frameworks() -> Vec<Box<dyn FrameworkSupport>> {
    let mut frameworks = Vec::<Box<dyn FrameworkSupport>>::new();
    frameworks.push(Box::new(Antora::default()));
    frameworks.push(Box::new(Astro::default()));
    frameworks.push(Box::new(DocFx::default()));
    frameworks.push(Box::new(DocusaurusV2::default()));
    frameworks.push(Box::new(Eleventy::default()));
    frameworks.push(Box::new(Gatsby::default()));
    frameworks.push(Box::new(Hexo::default()));
    frameworks.push(Box::new(Hugo::default()));
    frameworks.push(Box::new(Jekyll::default()));
    frameworks.push(Box::new(MDBook::default()));
    frameworks.push(Box::new(MKDocs::default()));
    frameworks.push(Box::new(NextJS::default()));
    frameworks.push(Box::new(NuxtJS::default()));
    frameworks.push(Box::new(Sphinx::default()));
    frameworks.push(Box::new(SvelteKit::default()));
    frameworks.push(Box::new(VitePress::default()));
    frameworks.push(Box::new(VuePress::default()));
    frameworks
}
