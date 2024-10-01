use iced::widget::{column, text};
use iced::window::Id;
use iced::{event, Alignment, Element, Event, Length, Task as Command, Theme};
use iced_runtime::window::Action as WindowAction;
use iced_runtime::{task, Action};

use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use iced_layershell::MultiApplication;


pub fn main() -> Result<(), iced_layershell::Error> {
    NotificationMulti::run(Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((400, 100)),
            margin: (10, 10, 10, 10),
            keyboard_interactivity: KeyboardInteractivity::None,
            ..Default::default()
        },
        ..Default::default()
    })
}

struct NotificationMulti {
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowInfo {
}

#[to_layer_message(multi, info_name = "WindowInfo", derives = "Debug Clone")]
enum Message {
    Close(Id),
    TextInput(String),
    IcedEvent(Event),
}

impl MultiApplication for NotificationMulti {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type WindowInfo = WindowInfo;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                text: "type something".to_string(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("Notification - Iced")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen().map(Message::IcedEvent)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(event) => {
                match event {
                    _ => {}
                }
                Command::none()
            }
            Message::Close(id) => task::effect(Action::Window(WindowAction::Close(id))),
            _ => unreachable!(),
        }
    }

    fn view(&self, _id: iced::window::Id) -> Element<Message> {
        iced::widget::container("text container")
            .padding(10)
            .center(800)
            .style(move |_| {

                iced::widget::container::Style {
                    //border: borders.border.rounded(iced::border::radius(10)),
                    text_color: Some(iced::Color::parse("#ff0000").unwrap()),
                    border: iced::Border{
                        color: iced::Color::parse("#ff00ff").unwrap(),
                        width: 2.0,
                        radius: iced::border::radius(10),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::parse("#ff0000").unwrap(),
                        offset: iced::Vector {
                            x: 10.0,
                            y: 10.0,
                        },
                        blur_radius: 15.0,
                    },
                    background: Some(iced::Background::Color(iced::Color::parse("#000000").unwrap()))
                }
            })
            .into()

    }

    fn theme(&self) -> Self::Theme {
        // Custom theme test1
        Theme::custom(
            //name: String,
            "CustomPalette".to_string(), 
            //palette: iced::theme::Palette,
            iced::theme::Palette {
                background: iced::Color::parse("#00000000").unwrap(),
                text: iced::Color::parse("#ffffff").unwrap(),
                primary: iced::Color::parse("#ff00ff").unwrap(),
                success: iced::Color::parse("#ffff00").unwrap(),
                danger: iced::Color::parse("#ff0000").unwrap(),
            }
        )
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        iced_layershell::Appearance {
            background_color: theme.palette().background,
            text_color: theme.palette().text,
        }
    }
}