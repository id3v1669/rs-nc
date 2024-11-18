use std::collections::HashMap;

use iced::widget::{button, column, container, row, text, text_input, stack};
use iced::window::Id;
use iced::{event, Alignment, Element, Event, Length, Task as Command, Theme};
use iced_layershell::actions::{IcedNewMenuSettings, MenuDirection};
use iced_runtime::window::Action as WindowAction;
use iced_runtime::{task, Action};

use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use iced_layershell::MultiApplication;

use crate::data::nf_struct::NotificationAction;
use crate::notification::nf_handler::NotificationHandler;

pub async fn gen_ui() -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            size: Some((50, 50)),
                            exclusive_zone: 0,
                            anchor: Anchor::Bottom | Anchor::Left,
                            layer: Layer::Overlay,
                            margin: (10, 10, 10, 10),
                            keyboard_interactivity: KeyboardInteractivity::None,
            ..Default::default()
        },
        ..Default::default()
    };
    let _ = NotificationCenter::run(settings);

    Ok(())
}

struct NotificationCenter {
    ids: HashMap<iced::window::Id, WindowInfo>,
}

#[derive(Debug, Clone, PartialEq)]
struct PreCalc {
    font_size_name: u16,
    font_size_summary: u16,
    font_size_body: u16,
    image_size: f32,
    text_summary_paddings: iced::Padding,
    text_body_paddings: iced::Padding,
    text_paddings_block: iced::Padding,
}

#[derive(Debug, Clone, PartialEq)]
struct WindowInfo {
    notification: crate::data::nf_struct::Notification,
    precalc: PreCalc,
}

#[to_layer_message(multi, info_name = "WindowInfo")]
#[derive(Debug, Clone)]
pub enum Message {
    Close(Id),
    CloseByContentId(u32),
    TestMessage,
    MoveNotifications,
    Notify(crate::data::nf_struct::Notification)
}


impl NotificationCenter {
    async fn sleep_timer(sleep_time: u64) {
        tokio::time::sleep(std::time::Duration::from_secs(sleep_time)).await;
    }
    fn window_id(&self, notification_id: u32) -> Option<&iced::window::Id> {
        for (k, v) in self.ids.iter() {
            if notification_id == v.notification.notification_id {
                return Some(k);
            }
        }
        None
    }
    fn iced_container_style() -> iced::widget::container::Style {
        let config = crate::data::shared_data::CONFIG.lock().unwrap();
        return iced::widget::container::Style {
            text_color: Some(config.primary_text_color),
            border: iced::Border {
                color: config.border_color,
                width: config.border_width,
                radius: config.border_radius,
            },
            shadow: iced::Shadow {
                //has to be here as empty shadow is not allowed and no paddings yet to make it visible
                color: iced::Color::parse("#00000000").unwrap(),
                offset: iced::Vector { x: 0.0, y: 0.0 },
                blur_radius: 0.0,
            },
            background: Some(iced::Background::Color(config.background_color)),
        };
    }
}

impl MultiApplication for NotificationCenter {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type WindowInfo = WindowInfo;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
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
        String::from("NotificationCenter")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::batch([
            iced::Subscription::run(|| {
                iced::stream::channel(100, |sender| async move {
                    // setup the object server
                    let _connection = zbus::connection::Builder::session()
                        .expect("Failed to create zbus session connection")
                        .name("org.freedesktop.Notifications")
                        .expect("Failed to set name org.freedesktop.Notifications")
                        .serve_at("/org/freedesktop/Notifications", NotificationHandler::new(sender))
                        .expect("Failed to serve at /org/freedesktop/Notifications")
                        .build()
                        .await
                        .unwrap();
                    futures::future::pending::<()>().await;
                    unreachable!()
                })
            }),
            iced::event::listen_with(
                |event, _status, id| match event {
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)) => {
                        Some(Message::Close(id))
                    }
                    _ => None,
                },
            ),
        ])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        use iced::Event;
        match message {
            Message::Close(id) => {
                let config = crate::data::shared_data::CONFIG.lock().unwrap();
                let mut active_notifications = crate::data::shared_data::ACTIVE_NOTIFICATIONS.lock().unwrap();
                let info = self.id_info(id).unwrap();
                if let Some((&key, _)) = active_notifications.iter().find(|(_, &value)| value == info.notification.notification_id) {
                    let last = std::cmp::min(active_notifications.len() as i32, config.max_notifications-1);
                    for i in key..=last { 
                        if let Some(&next_value) = active_notifications.get(&(i+1)) {
                            active_notifications.insert(i, next_value);
                        }
                    }
                    active_notifications.remove(&last);
                }
                let mut active_notifications_count = active_notifications.len() as i32;
                Command::batch([
                    Command::done(Message::RemoveWindow(id)),
                    Command::done(Message::MoveNotifications),
                ])
            }
            Message::CloseByContentId(notification_id) => {
                let mut active_notifications = crate::data::shared_data::ACTIVE_NOTIFICATIONS.lock().unwrap();
                if let Some(id) = self.window_id(notification_id) {
                    return Command::done(Message::Close(*id))
                }
                Command::none()
            }
            Message::MoveNotifications => {
                let config = crate::data::shared_data::CONFIG.lock().unwrap();
                
                let mut active_notifications = crate::data::shared_data::ACTIVE_NOTIFICATIONS.lock().unwrap();
                let mut move_notifications: Vec<Command<Message>> = Vec::new();

                for (position_in_q, id_in_q) in active_notifications.clone() {
                    if let Some(id) = self.window_id(id_in_q) {
                        let offset: i32 = {
                            (config.height as i32 * (position_in_q-1))+(config.vertical_margin * (position_in_q-1)) + config.vertical_margin
                        };
                        move_notifications.push(Command::done(Message::MarginChange {
                            id: *id,
                            margin: (offset, config.horizontal_margin, config.vertical_margin, config.horizontal_margin),
                        }));
                    }
                }

                if move_notifications.len() > 0 {
                    return Command::batch(move_notifications)
                }
                Command::none()
                
            }
            Message::TestMessage => {
                println!("TestMessage");
                Command::none()
            }
            Message::Notify(notification) => {
                let id = notification.notification_id.clone();
                let config = crate::data::shared_data::CONFIG.lock().unwrap();
                let mut active_notifications = crate::data::shared_data::ACTIVE_NOTIFICATIONS.lock().unwrap();
                let mut active_notifications_count = active_notifications.len() as i32;
                for i in (1..=std::cmp::min(active_notifications_count, config.max_notifications-1)).rev() {
                    if let Some(&prev_value) = active_notifications.get(&i) {
                        active_notifications.insert(i+1, prev_value);
                    }
                }
                active_notifications.entry(1).and_modify(|value| *value = id).or_insert(id);
                let timeout = if config.respect_notification_timeout && notification.expire_timeout > 0 {
                    notification.expire_timeout
                } else {
                    config.local_expire_timeout
                };
                // precalculation of font sizes to avoid recalculating them every frame(view) update
                // TODO: add formulas here after figuring out propper grid layout and proportions
                let precalc = PreCalc {
                    font_size_name: 10,
                    font_size_summary: (config.height as f32 * 0.24) as u16,
                    font_size_body: (config.height as f32 * 0.17) as u16,
                    image_size: (config.height as f32)*0.75,
                    text_summary_paddings: iced::Padding {
                        top: 0.0,
                        bottom: 0.0,
                        left: (config.height as f32 * 0.05) + (config.height as f32 * 0.01),
                        right: 0.0,
                    },
                    text_body_paddings: iced::Padding {
                        top: 0.0,
                        bottom: 0.0,
                        left: config.height as f32 * 0.05,
                        right: 0.0,
                    },
                    text_paddings_block: iced::Padding {
                        top: 10.0,
                        bottom: 10.0,
                        left: config.height as f32 * 0.15,
                        right: 0.0,
                    },
                };

                Command::batch([
                    Command::done(Message::MoveNotifications),
                    Command::done(Message::NewLayerShell {
                        settings: NewLayerShellSettings {
                            size: Some((400, 100)),
                            exclusive_zone: None,
                            anchor: Anchor::Top | Anchor::Right,
                            layer: Layer::Overlay,
                            margin: Some((config.vertical_margin, config.horizontal_margin, config.vertical_margin, config.horizontal_margin)),
                            keyboard_interactivity: KeyboardInteractivity::None,
                            ..Default::default()
                        },
                        info: WindowInfo { notification: notification, precalc: precalc },
                    }),
                    Command::perform(Self::sleep_timer(timeout.try_into().unwrap()), move |_| Message::CloseByContentId(id)),
                ])
            }
            _ => unreachable!(),
        }
    }

    fn view(&self, id: Id) -> Element<Message> {
        if let Some(window_info) = self.id_info(id)
        {
            return iced::widget::container(
            iced::widget::row![
                iced::widget::svg(std::path::PathBuf::from("./assets/testing/linux.svg")) // home/user/myrepos/rs-nc/assets/testing/linux.svg
                    .width(iced::Length::Fixed(window_info.precalc.image_size as f32))
                    .height(iced::Length::Fixed(window_info.precalc.image_size as f32)),
                iced::widget::column![
                    iced::widget::column![
                        iced::widget::text(window_info.notification.summary.clone()).size(window_info.precalc.font_size_summary)
                            .align_x(iced::alignment::Horizontal::Left),
                        ]
                        .padding(window_info.precalc.text_summary_paddings),
                    iced::widget::column![
                        iced::widget::text(window_info.notification.body.clone()).size(window_info.precalc.font_size_body),
                        ]
                        .padding(window_info.precalc.text_body_paddings),
                    ]
                    .padding(window_info.precalc.text_paddings_block)
                ]
                .align_y(iced::alignment::Vertical::Center)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
            
            )
            .padding(10)
            .center(800)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_| NotificationCenter::iced_container_style())
            .into()
        }
        else {
            return iced::widget::container("ss")
            .padding(10)
            .center(800)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_| NotificationCenter::iced_container_style())
            .into()
        }
    }

    fn style(&self, _theme: &Self::Theme) -> iced_layershell::Appearance {
        iced_layershell::Appearance {
            background_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::TRANSPARENT,
        }
    }
}