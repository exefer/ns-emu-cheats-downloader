use super::TinfoilTitle;
use gtk::glib::{self, Properties};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

#[derive(Default, Properties)]
#[properties(wrapper_type = super::TinfoilTitleObject)]
pub struct TinfoilTitleObject {
    #[property(name = "id", get, set, type = String, member = id)]
    #[property(name = "name", get, set, type = String, member = name)]
    #[property(name = "release-date", get, set, type = Option<String>, member = release_date)]
    #[property(name = "publisher", get, set, type = Option<String>, member = publisher)]
    pub data: RefCell<TinfoilTitle>,
    #[property(name = "name-lowercase", get, set)]
    pub name_lowercase: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for TinfoilTitleObject {
    const NAME: &'static str = "TinfoilTitleObject";
    type Type = super::TinfoilTitleObject;
}

#[glib::derived_properties]
impl ObjectImpl for TinfoilTitleObject {}
