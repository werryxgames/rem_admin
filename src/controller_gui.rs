use crate::controller_gui::glib::clone;
use crate::server::Client;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use gtk4::{prelude::*, glib::{self, ControlFlow}};

fn refresh_clients(clients_m: Arc<Mutex<Vec<Client>>>, vbox_m: Rc<Mutex<gtk4::Box>>, prev_clients_m: Arc<Mutex<Vec<Client>>>) {
    let mut prev_clients = prev_clients_m.lock().unwrap();
    let clients = clients_m.lock().unwrap();
    let vbox = vbox_m.lock().unwrap();
    let mut refresh = false;

    if clients.len() != prev_clients.len() {
        refresh = true;
    } else {
        for client in clients.iter().enumerate() {
            if client.1.id != prev_clients[client.0].id {
                refresh = true;
                break;
            }
        }
    }

    if refresh {
        while let Some(row) = vbox.last_child() {
            vbox.remove(&row);
        }

        for client in clients.iter() {
            let b64_id = base64_light::base64_encode_bytes(&client.id.to_be_bytes());
            let label = gtk4::Label::builder()
            .label(format!("Client {}: '{}'", client.index, b64_id))
            .build();
            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            hbox.set_margin_start(8);
            hbox.set_margin_end(8);
            hbox.set_margin_top(8);
            hbox.set_margin_bottom(8);
            hbox.set_spacing(8);
            hbox.append(&label);
            let btn = gtk4::Button::builder()
            .label("Control")
            .hexpand(true)
            .build();
            hbox.append(&btn);
            vbox.append(&hbox);
        }

        prev_clients.clear();

        for client in clients.iter() {
            prev_clients.push((*client).clone());
        }
    }
}

fn idle_function(clients_m: Arc<Mutex<Vec<Client>>>, vbox: Rc<Mutex<gtk4::Box>>, prev_clients: Arc<Mutex<Vec<Client>>>) -> ControlFlow {
    refresh_clients(clients_m, vbox, prev_clients);
    ControlFlow::Continue
}

pub fn controller_gui_start(clients: Arc<Mutex<Vec<Client>>>) {
    let app = gtk4::Application::builder()
    .application_id("com.werryxgames.rem_admin")
    .build();
    app.connect_activate(clone!(@weak clients => move |app: &gtk4::Application| {
        let win = gtk4::ApplicationWindow::new(app);
        win.set_default_size(800, 600);
        win.set_title(Some("RemAdmin Controller"));
        let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        vbox.set_margin_start(8);
        vbox.set_margin_end(8);
        vbox.set_margin_top(8);
        vbox.set_margin_bottom(8);
        vbox.set_spacing(8);
        win.set_child(Some(&vbox));

        let vbox_clients = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        vbox_clients.set_spacing(8);
        let scrolled_clients = gtk4::ScrolledWindow::builder()
        .child(&vbox_clients)
        .has_frame(true)
        .vexpand(true)
        .build();
        vbox.append(&scrolled_clients);
        let vbox_arc = Rc::new(Mutex::new(vbox_clients));
        let prev_clients = Arc::new(Mutex::new(Vec::new()));
        refresh_clients(clients.clone(), vbox_arc.clone(), prev_clients.clone());
        let clients2 = clients.clone();
        let vbox_arc2 = vbox_arc.clone();
        let prev_clients2 = prev_clients.clone();

        let btn = gtk4::Button::with_label("Refresh");
        btn.connect_clicked(move |_| {
            refresh_clients(clients2.clone(), vbox_arc2.clone(), prev_clients2.clone());
        });

        let clients3 = clients.clone();
        let vbox_arc3 = vbox_arc.clone();
        let prev_clients3 = prev_clients.clone();
        glib::source::idle_add_local(move || {
            idle_function(clients3.clone(), vbox_arc3.clone(), prev_clients3.clone())
        });
        // TODO: Spawn thread that refreshes clients each X seconds
        vbox.append(&btn);
        win.present();
    }));
    let args: [&str; 0] = [];
    app.run_with_args(&args);
}
