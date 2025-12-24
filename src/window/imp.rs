use gtk::{
    CompositeTemplate,
    glib::{self, subclass::InitializingObject},
    subclass::prelude::*,
};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/xyz/exefer/ns-emu-cheats-downloader/ui/window.ui")]
pub struct Window {
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub icon: TemplateChild<gtk::Picture>,
    #[template_child]
    pub search_entry: TemplateChild<gtk::SearchEntry>,
    #[template_child]
    pub titles_view: TemplateChild<gtk::ColumnView>,
    #[template_child]
    pub id_column: TemplateChild<gtk::ColumnViewColumn>,
    #[template_child]
    pub name_column: TemplateChild<gtk::ColumnViewColumn>,
    #[template_child]
    pub release_date_column: TemplateChild<gtk::ColumnViewColumn>,
    #[template_child]
    pub publisher_column: TemplateChild<gtk::ColumnViewColumn>,
    #[template_child]
    pub selected_title_label: TemplateChild<gtk::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "NecdWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for Window {}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
