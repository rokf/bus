
extern crate csv;
extern crate gtk;

use gtk::prelude::*;

use gtk::{
    ListBox,
    ListBoxRow,
    Label,
    Box,
    Orientation,
    Entry,
    Button
};

use std::fs::File;
use std::cell::RefCell;
use std::rc::Rc;

pub fn check_for_file(filename : &str) -> bool {
    match File::open(filename) {
        Ok(f) => {
            println!("File {} exists.", filename);
            true
        },
        Error => {
            match File::create(filename) {
                Ok(f) => {
                    println!("Has been created.");
                },
                Error => {
                    println!("Couldn't be created.");
                }
            }
            false
        },
    }
}

// pub fn load_favorites(fav_ref : &mut Vec<(String,String)>, filename : &str) {
pub fn load_favorites(fav_ref : Rc<RefCell<Vec<(String,String)>>>, filename : &str) {
    let mut rdr = csv::Reader::from_file(filename).unwrap();
    rdr = rdr.has_headers(false);
    let mut mutfavr = fav_ref.borrow_mut();
    for record in rdr.decode() {
        let (from, to): (String, String) = record.unwrap();
        println!("loading: {} {}",from,to);
        mutfavr.push((from, to));
    }
}

pub fn write_favorites(fav_ref :& Rc<RefCell<Vec<(String,String)>>>, filename : &str) {
    let mut wtr = csv::Writer::from_file(filename).unwrap();
    let mut mutfavr = fav_ref.borrow_mut();
    for record in mutfavr.iter() {
        let res = wtr.encode(record);
        // println!("res: {:?}",res);
        // let res = wtr.write(record.iter());
    }
}

pub fn append_favorite(lb : &ListBox, from : &str, to : &str, e1: &Entry, e2: &Entry) {
    let r = ListBoxRow::new();
    let l_from = Label::new(Some(from));
    let l_to = Label::new(Some(to));
    let rb = Box::new(Orientation::Vertical,0);
    let fc = "".to_string() + from;
    let tc = "".to_string() + to;
    r.connect_activate({
        let e1 = e1.clone();
        let e2 = e2.clone();
        move |_| {
            e1.set_text(&fc);
            e2.set_text(&tc);
        }
    });
    rb.add(&l_from); rb.add(&l_to);
    r.add(&rb);
    lb.insert(&r,-1);
}

pub fn update_fav_box(fav_ref : &Vec<(String,String)>, fav_box_ref : &ListBox, e1: &Entry, e2: &Entry) {
    for child in fav_box_ref.get_children() {
        fav_box_ref.remove(&child);
    }
    for f in fav_ref {
        append_favorite(fav_box_ref, &f.0, &f.1, e1, e2);
    }
    fav_box_ref.show_all();
}
