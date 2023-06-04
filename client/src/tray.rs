enum Message {
    Quit,
    Green,
    Red,
    Open,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    None,
    Open,
    Exit,
}

pub struct Menu {
    item: tray_item::TrayItem,
    receiver: std::sync::mpsc::Receiver<Message>,
    exit: bool,
}

impl Menu {
    pub fn new() -> Self {
        let mut tray = tray_item::TrayItem::new(
            "Lumin",
            tray_item::IconSource::Resource("name-of-icon-in-rc-file"),
        )
        .unwrap();

        let (sender, receiver) = std::sync::mpsc::channel();

        let green_tx = sender.clone();
        tray.add_menu_item("Open", move || {
            green_tx.send(Message::Open).unwrap();
        })
        .unwrap();

        // tray.add_menu_item("Hello", || {
        //     println!("Hello!");
        // })
        // .unwrap();

        tray.inner_mut().add_separator().unwrap();

        let red_tx = sender.clone();
        tray.add_menu_item("Red", move || {
            red_tx.send(Message::Red).unwrap();
        })
        .unwrap();

        let green_tx = sender.clone();
        tray.add_menu_item("Green", move || {
            green_tx.send(Message::Green).unwrap();
        })
        .unwrap();

        tray.inner_mut().add_separator().unwrap();

        let quit_tx = sender.clone();
        tray.add_menu_item("Quit", move || {
            quit_tx.send(Message::Quit).unwrap();
        })
        .unwrap();

        Menu {
            item: tray,
            receiver,
            exit: false,
        }
    }

    pub fn update(&mut self) -> Command {
        if self.exit {
            return Command::Exit;
        }
        if let Ok(msg) = self.receiver.try_recv() {
            match msg {
                Message::Quit => {
                    println!("Quit");
                    self.exit = true;
                    return Command::Exit;
                }
                Message::Red => {
                    println!("Red");
                    self.item
                        .set_icon(tray_item::IconSource::Resource("another-name-from-rc-file"))
                        .unwrap();
                }
                Message::Green => {
                    println!("Green");
                    self.item
                        .set_icon(tray_item::IconSource::Resource("name-of-icon-in-rc-file"))
                        .unwrap()
                }
                Message::Open => {
                    println!("Opening app");
                    return Command::Open;
                }
                _ => {}
            }
        }
        Command::None
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
