
extern crate gtk;
extern crate hyper;
extern crate json;
extern crate requests;

mod opts;
mod favorites;

use opts::OPTIONS;

use gtk::prelude::*;
use gtk::Orientation;

use requests::{get};

use std::rc::Rc;
use std::cell::RefCell;

use gtk::{
    Button,
    Window,
    WindowType,
    HeaderBar,
    Box,
    ScrolledWindow,
    ListBox,
    ListBoxRow,
    Label,
    ListStore,
    Calendar,
    Entry,
    EntryCompletion,
    CellRendererText,
    TreeView,
    TreeViewColumn,
    StyleContext,
    CssProvider,
    STYLE_PROVIDER_PRIORITY_APPLICATION
};

fn repair(station : &str) -> String {
    station.replace(" ","+")
}

fn update(list_box: &ListBox, from : &str, to : &str, date : &str) {
    println!("{} {} {}", from, to, date);
    for child in list_box.get_children() {
        list_box.remove(&child);
    }
    let url = "http://www.apms.si/response.ajax.php?com=voznired&task=get&datum=".to_string() +
            date +
            "&postaja_od=" +
            &repair(from) +
            "&postaja_do=" +
            &repair(to);
    println!("FULL URL: {}",url);
    let res = get(&url).unwrap();
    let txt = res.text().unwrap();
    let parsed = json::parse(txt).unwrap();
    for vi in 0..parsed.len() {
        let odhod = &parsed[vi]["odhod"].to_string();
        let prihod = &parsed[vi]["prihod"].to_string();
        let voznja = &parsed[vi]["voznja"].to_string();
        let lbox_row = ListBoxRow::new();
        let lbl1 = Label::new(Some(odhod));
        let lbl2 = Label::new(Some(prihod));
        let lbl3 = Label::new(Some(voznja));
        let bx = Box::new(Orientation::Horizontal, 10);
        bx.add(&lbl1); bx.add(&lbl2); bx.add(&lbl3);
        lbox_row.add(&bx);
        list_box.insert(&lbox_row, -1);
    }
    list_box.show_all();
}

fn update_ls(ls : &ListStore, from : &str, to : &str, date : &str) {
    ls.clear();
    let url = "http://www.apms.si/response.ajax.php?com=voznired&task=get&datum=".to_string() +
            date +
            "&postaja_od=" +
            &repair(from) +
            "&postaja_do=" +
            &repair(to);
    let res = get(&url).unwrap();
    let txt = res.text().unwrap();
    let parsed = json::parse(txt).unwrap();
    for vi in 0..parsed.len() {
        let odhod = &parsed[vi]["odhod"].to_string();
        let prihod = &parsed[vi]["prihod"].to_string();
        let voznja = &parsed[vi]["voznja"].to_string();
        let iter = ls.append();
        ls.set(&iter,&[0,1,2],&[&odhod.to_value(),&prihod.to_value(),&voznja.to_value()]);
    }
}

macro_rules! clone {
    ($($n:ident),+ ; $b:expr) => (
        {
        $(let $n = $n.clone();)+
                $b
        }
    );
}

fn main() {
    if gtk::init().is_err() { println!("Failed to initialize GTK."); return; }

    let data_liststore = ListStore::new(&[gtk::Type::String,
        gtk::Type::String,
        gtk::Type::String
    ]);
    let tview = TreeView::new_with_model(&data_liststore);

    // make columns
    for (i, v) in ["Odhod","Prihod","Voznja"].iter().enumerate() {
        let col = TreeViewColumn::new();
        col.set_title(v);
        let renderer = CellRendererText::new();
        col.pack_start(&renderer, true);
        col.add_attribute(&renderer, "text", i as i32);
        tview.append_column(&col);
    }

    let mut favs : Rc<RefCell<Vec<(String,String)>>> = Rc::new(RefCell::new(vec![]));
    favorites::check_for_file("apmsfav.txt");
    {
        let fav_c = favs.clone();
        favorites::load_favorites(fav_c, "apmsfav.txt");
    }


    let window = Window::new(WindowType::Toplevel);
    window.set_title("Bus");
    window.set_default_size(800, 600);

    // CSS
    let w_display = window.get_display().unwrap();
    let w_screen = w_display.get_screen(0);
    let css_style_provider = CssProvider::new();
    // let css_file = include_str!("style.css");
    let css_file = "GtkHeaderBar GtkButton { color: #0077BE; }";
    css_style_provider.load_from_data(css_file).unwrap();
    StyleContext::add_provider_for_screen(&w_screen
        , &css_style_provider
        , STYLE_PROVIDER_PRIORITY_APPLICATION
    );

    let button = Button::new_from_icon_name("media-playlist-shuffle-symbolic", 1);
    let fav_button = Button::new_from_icon_name("emblem-favorite-symbolic", 1);

    let s_entry1 = Entry::new();
    let s_entry2 = Entry::new();

    let ecomp1 = EntryCompletion::new();
    let ecomp2 = EntryCompletion::new();
    ecomp1.set_text_column(0);
    ecomp2.set_text_column(0);
    let liststore = ListStore::new(&[gtk::Type::String]);

    for opt in OPTIONS {
        let iter = liststore.append();
        liststore.set(&iter,&[0],&[&opt.to_value()]);
    }

    ecomp1.set_model(Some(&liststore));
    ecomp2.set_model(Some(&liststore));
    s_entry1.set_completion(Some(&ecomp1));
    s_entry2.set_completion(Some(&ecomp2));

    let header = HeaderBar::new();
    header.set_show_close_button(true);
    header.pack_start(&s_entry1);
    header.pack_start(&button);
    header.pack_start(&s_entry2);
    header.pack_end(&fav_button);
    window.set_titlebar(Some(&header));

    let scrl = ScrolledWindow::new(None,None);
    scrl.set_margin_right(10);
    // scrl.add(&lbox);
    scrl.add(&tview);
    scrl.set_hexpand(true);

    let bx = Box::new(Orientation::Horizontal, 0);
    bx.set_margin_top(10);
    bx.set_margin_left(10);
    bx.set_margin_right(10);
    bx.set_margin_bottom(10);
    bx.pack_start(&scrl,true,true,0);

    let sidebar = Box::new(Orientation::Vertical,0);
    sidebar.set_hexpand(false);

    let calendar = Calendar::new();
    sidebar.pack_start(&calendar,false,false,0);

    let fav_box = ListBox::new();
    let fav_scroll = ScrolledWindow::new(None,None);
    // fav_box.set_vexpand(true);
    fav_scroll.add(&fav_box);
    // fav_scroll.set_vexpand(true);
    sidebar.pack_start(&fav_scroll, false, false, 0);

    bx.add(&sidebar);
    window.add(&bx);
    window.show_all();

    { // dirty
        let favs = favs.clone();
        let mut favcell = favs.borrow_mut();
        favorites::update_fav_box(&favcell, &fav_box, &s_entry1, &s_entry2);
    }

    calendar.connect_day_selected_double_click(clone!(s_entry1,s_entry2,data_liststore; move |c| {
        let (year,month,day) = c.get_date(); // FIXME
        let date_str = day.to_string() + "." + &(month+1).to_string() + "." + &year.to_string();
        let from = s_entry1.get_text().unwrap();
        let to = s_entry2.get_text().unwrap();
        // update(&lbox, &from, &to, &date_str);
        update_ls(&data_liststore, &from, &to, &date_str);
    }));

    button.connect_clicked({
        let s_entry1 = s_entry1.clone();
        let s_entry2 = s_entry2.clone();
        move |_| {
            let tmp = &s_entry1.get_text().unwrap();
            s_entry1.set_text(&s_entry2.get_text().unwrap());
            s_entry2.set_text(&tmp);
        }
    });

    fav_button.connect_clicked({
        let s_entry1 = s_entry1.clone();
        let s_entry2 = s_entry2.clone();
        let fav_box = fav_box.clone();
        let mut favs = favs.clone();
        move |_| {
            let mut favcell = favs.borrow_mut(); // magic
            let text1 = s_entry1.get_text().unwrap();
            let text2 = s_entry2.get_text().unwrap();
            favcell.push((text1, text2));
            favorites::update_fav_box(&favcell, &fav_box, &s_entry1, &s_entry2);
        }
    });

    window.connect_delete_event({
        let favs = favs.clone();
        move |_, _| {
            favorites::write_favorites(&favs,"apmsfav.txt");
            gtk::main_quit();
            Inhibit(false)
        }
    });

    gtk::main();
}
