//! # Html Render Plugins

//pub mod app_context_script;
//mod entry;
//mod is_granted;
//mod translate;
mod utils;

use {
    //self::{app_context_script::*, entry::*, is_granted::*, translate::*, utils::*},
    tera::Tera,
    utils::*
};

pub(super) fn register_modules(tera: &mut Tera) {
    //     tera.register_function("entry_link_tags", entry_link_tags);
    //     tera.register_function("entry_script_tags", entry_script_tags);
    //     tera.register_function("app_context_script", app_context_script);
    //     tera.register_function("is_granted", is_granted);
    //    tera.register_filter("t", translate);
    tera.register_function("dbg", dbg);
    tera.register_function("debug", debug);
    tera.register_function("is_debug", is_debug);
    tera.register_tester("is_null", is_null);
}
