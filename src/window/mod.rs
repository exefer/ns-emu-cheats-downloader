mod imp;

use crate::tinfoil_title_object::{TinfoilTitle, TinfoilTitleObject};
use curl::easy::Easy;
use gtk::{
    gdk, gio,
    glib::{self, subclass::prelude::*},
    prelude::*,
};
use std::rc::Rc;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    pub fn setup_titles(&self, titles: Vec<TinfoilTitle>) {
        let store = gio::ListStore::new::<TinfoilTitleObject>();

        let title_objects: Vec<TinfoilTitleObject> = titles
            .into_iter()
            .map(TinfoilTitleObject::from_title)
            .collect();
        store.extend_from_slice(&title_objects);

        let filter = gtk::CustomFilter::new(|_| true);

        let filter_model = gtk::FilterListModel::new(Some(store), Some(filter.clone()));

        let sorter = self.imp().titles_view.sorter();
        let sort_model = gtk::SortListModel::new(Some(filter_model), sorter);
        sort_model.set_incremental(true);

        let selection_model = gtk::SingleSelection::new(Some(sort_model));
        self.imp().titles_view.set_model(Some(&selection_model));

        self.setup_column(&self.imp().id_column, |obj| obj.id());
        self.setup_column(&self.imp().name_column, |obj| obj.name());
        self.setup_column(&self.imp().release_date_column, |obj| {
            obj.release_date().unwrap_or_default()
        });
        self.setup_column(&self.imp().publisher_column, |obj| {
            obj.publisher().unwrap_or_default()
        });

        self.imp().search_entry.connect_search_changed(glib::clone!(
            #[weak]
            filter,
            move |entry| {
                let search_text = entry.text().as_str().to_lowercase();

                if search_text.is_empty() {
                    filter.set_filter_func(|_| true);
                } else {
                    let search_text_rc: Rc<str> = Rc::from(search_text.as_str());
                    filter.set_filter_func(move |obj| {
                        obj.downcast_ref::<TinfoilTitleObject>()
                            .is_some_and(|title_obj| {
                                let name = title_obj.name_lowercase();
                                name.contains(search_text_rc.as_ref())
                            })
                    });
                }
            }
        ));

        let titles_view = self.imp().titles_view.get();
        let stack = self.imp().stack.get();
        let selected_title_label = self.imp().selected_title_label.get();
        let icon = self.imp().icon.get();

        titles_view.connect_activate(glib::clone!(
            #[weak]
            titles_view,
            #[weak]
            stack,
            #[weak]
            selected_title_label,
            #[weak]
            icon,
            move |_, position| {
                if let Some(model) = titles_view.model()
                    && let Some(item) = model.item(position)
                    && let Some(title_obj) = item.downcast_ref::<TinfoilTitleObject>()
                {
                    let title_id = title_obj.id();

                    selected_title_label.set_label(&format!(
                        "ID: {}\nName: {}\nRelease Date: {}\nPublisher: {}",
                        title_id,
                        title_obj.name(),
                        title_obj.release_date().unwrap(),
                        title_obj.publisher().unwrap()
                    ));
                    stack.set_visible_child_name("cheat-manager");

                    let mut easy = Easy::new();
                    let mut image_bytes = Vec::new();
                    easy.url(&format!("https://tinfoil.io/ti/{}/512/512", title_id))
                        .unwrap();
                    {
                        let mut transfer = easy.transfer();
                        transfer
                            .write_function(|data| {
                                image_bytes.extend_from_slice(data);
                                Ok(data.len())
                            })
                            .unwrap();
                        transfer.perform().unwrap();
                    }
                    if let Ok(texture) =
                        gdk::Texture::from_bytes(&glib::Bytes::from_owned(image_bytes))
                    {
                        icon.set_paintable(Some(&texture));
                    }
                }
            }
        ));
    }

    fn setup_column<F>(&self, column: &gtk::ColumnViewColumn, get_text: F)
    where
        F: Fn(&TinfoilTitleObject) -> String + 'static + Clone,
    {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(move |_, item| {
            let label = gtk::Label::new(None);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_single_line_mode(true);
            item.downcast_ref::<gtk::ListItem>()
                .expect("item is not ListItem")
                .set_child(Some(&label));
        });

        factory.connect_bind(glib::clone!(
            #[strong]
            get_text,
            move |_, item| {
                let list_item = item.downcast_ref::<gtk::ListItem>().unwrap();
                let title_obj = list_item
                    .item()
                    .and_downcast::<TinfoilTitleObject>()
                    .unwrap();
                let label = list_item.child().and_downcast::<gtk::Label>().unwrap();

                let text = get_text(&title_obj);
                label.set_label(&text);
            }
        ));

        column.set_factory(Some(&factory));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let item1 = obj1.downcast_ref::<TinfoilTitleObject>().unwrap();
            let item2 = obj2.downcast_ref::<TinfoilTitleObject>().unwrap();

            let text1 = get_text(item1);
            let text2 = get_text(item2);

            match text1.cmp(&text2) {
                std::cmp::Ordering::Less => gtk::Ordering::Smaller,
                std::cmp::Ordering::Equal => gtk::Ordering::Equal,
                std::cmp::Ordering::Greater => gtk::Ordering::Larger,
            }
        });

        column.set_sorter(Some(&sorter));
    }
}
