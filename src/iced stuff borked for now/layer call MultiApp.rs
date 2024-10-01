use std::collections::HashMap;

use iced::widget::{button, column, container, row, text, text_input};
use iced::window::Id;
use iced::{event, Alignment, Element, Event, Length, Task as Command, Theme};
use iced_layershell::actions::{IcedNewMenuSettings, MenuDirection};
use iced_runtime::window::Action as WindowAction;
use iced_runtime::{task, Action};

use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use iced_layershell::MultiApplication;



#[tokio::main]
async fn main() -> Result<(), iced_layershell::Error> {

    loop {
        println!("Please input height of the window");
        let mut heightin = String::new();
        std::io::stdin().read_line(&mut heightin).unwrap();
        println!("You typed: {}", heightin);
        let height: u32 = heightin.trim().parse().unwrap();
        let mut widthin = String::new();
        println!("Please input width of the window");
        std::io::stdin().read_line(&mut widthin).unwrap();
        println!("You typed: {}", widthin);
        let width: u32 = widthin.trim().parse().unwrap();
        // call genUi function without waiting for response
        tokio::spawn(async move {
            genUi(width, height).await.unwrap();
        });
    }

    Ok(())
}

pub async fn genUi(width: u32, height: u32) -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((width, height)),
            margin: (10, 10, 10, 10),
            keyboard_interactivity: KeyboardInteractivity::None,
            binded_output_name: Some("test".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    println!("before run");
    tokio::spawn(async move {
        Counter::run(settings)
    });
    println!("after run");
    //let window_id = Counter::window_id(&WindowInfo::Counter).unwrap();
    //println!("window_id {:?}", window_id);
    std::future::pending::<()>().await;
    Ok(())
}

struct Counter {
    value: i32,
    text: String,
    ids: HashMap<iced::window::Id, WindowInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowInfo {
    Counterc,
}

#[to_layer_message(multi, info_name = "WindowInfo")]
#[derive(Debug, Clone)]
enum Message {
    Close(Id),
    IcedEvent(Event),
}

impl Counter {
    fn window_id(&self, info: &WindowInfo) -> Option<&iced::window::Id> {
        for (k, v) in self.ids.iter() {
            if info == v {
                return Some(k);
            }
        }
        None
    }
}

impl MultiApplication for Counter {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type WindowInfo = WindowInfo;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                value: 0,
                text: "type something".to_string(),
                ids: HashMap::new(),
            },
            Command::none(),
        )
    }

    fn id_info(&self, id: iced::window::Id) -> Option<Self::WindowInfo> {
        self.ids.get(&id).cloned()
    }

    fn set_id_info(&mut self, id: iced::window::Id, info: Self::WindowInfo) {
        self.ids.insert(id, info);
    }

    fn remove_id(&mut self, id: iced::window::Id) {
        self.ids.remove(&id);
    }

    fn namespace(&self) -> String {
        String::from("Counter - Iced")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen().map(Message::IcedEvent)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use iced::keyboard;
        use iced::keyboard::key::Named;
        use iced::Event;
        match message {
            Message::IcedEvent(event) => {
                match event {
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)) => {
                        println!("Right mouse button pressed");
                        //print ids info from hash map
                        println!("ids {:?}", self.ids.len());
                        for (k, v) in self.ids.iter() {
                            println!("k {:?} v {:?}", k, v);
                        }
                    }
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                        println!("Left mouse button pressed");
                        
                        iced::Task::done(Message::NewLayerShell {
                            settings: NewLayerShellSettings {
                                size: Some((500, 700)),
                                exclusive_zone: None,
                                anchor: Anchor::Left | Anchor::Bottom,
                                layer: Layer::Top,
                                margin: None,
                                keyboard_interactivity: KeyboardInteractivity::None,
                                use_last_output: false,
                            },
                            info: WindowInfo::Counterc,
                        });
                    }
                    _ => {}
                }
                Command::none()
            }
            Message::Close(id) => task::effect(Action::Window(WindowAction::Close(id))),
            _ => unreachable!(),
        }
    }

    fn view(&self, id: Id) -> Element<Message> {
        
        iced::widget::container("text container")
            .padding(10)
            .center(800)
            .into()
    }
}