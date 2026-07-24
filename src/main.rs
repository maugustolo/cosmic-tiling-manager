use cosmic::app::{Core, Settings};
use cosmic::iced::{Length, Task};
use cosmic::widget::{text, scrollable, Column, Row, text_input, icon, container};
use cosmic::widget::button;
use cosmic::{Application, Element};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct WindowInfo {
    id: String,
    app_id: String,
    title: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct WindowRule {
    appid: String,
    title: String,
    enabled: bool,
}

fn get_open_windows() -> Vec<WindowInfo> {
    let output = Command::new("cosmic-ext-window-helper")
        .arg("state")
        .output();
        
    if let Ok(out) = output {
        if let Ok(json) = String::from_utf8(out.stdout) {
            return serde_json::from_str(&json).unwrap_or_default();
        }
    }
    vec![]
}

fn rules_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".config/cosmic/com.system76.CosmicSettings.WindowRules/v1/tiling_exception_custom")
}

fn read_rules() -> Vec<WindowRule> {
    let path = rules_path();
    if let Ok(content) = fs::read_to_string(&path) {
        ron::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    }
}

fn write_rules(rules: &[WindowRule]) {
    let path = rules_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    // Configura o RON para exportar de forma limpa, semelhante ao padrão do COSMIC
    let pretty = ron::ser::PrettyConfig::default();
    if let Ok(content) = ron::ser::to_string_pretty(rules, pretty) {
        let _ = fs::write(path, content);
    }
}

struct TilingApp {
    core: Core,
    windows: Vec<WindowInfo>,
    rules: Vec<WindowRule>,
    search_query: String,
}

#[derive(Clone, Debug)]
enum Message {
    Refresh,
    AddRule(String),
    RemoveRule(String),
    SearchChanged(String),
    ExportRules,
    ImportRules,
    RulesImported(Option<Vec<WindowRule>>),
    FileOperationDone,
}

impl Application for TilingApp {
    type Executor = cosmic::iced::executor::Default;
    type Message = Message;
    type Flags = ();

    const APP_ID: &'static str = "com.system76.CosmicTilingManager";

    fn core(&self) -> &Core { &self.core }
    fn core_mut(&mut self) -> &mut Core { &mut self.core }

    fn init(core: Core, _flags: ()) -> (Self, Task<cosmic::Action<Message>>) {
        (
            TilingApp {
                core,
                windows: get_open_windows(),
                rules: read_rules(),
                search_query: String::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<cosmic::Action<Message>> {
        match message {
            Message::Refresh => {
                self.windows = get_open_windows();
                self.rules = read_rules();
                Task::none()
            }
            Message::AddRule(app_id) => {
                if !self.rules.iter().any(|r| r.appid == app_id) {
                    self.rules.push(WindowRule {
                        appid: app_id,
                        title: ".*".to_string(),
                        enabled: true,
                    });
                    write_rules(&self.rules);
                }
                Task::none()
            }
            Message::RemoveRule(app_id) => {
                self.rules.retain(|r| r.appid != app_id);
                write_rules(&self.rules);
                Task::none()
            }
            Message::SearchChanged(query) => {
                self.search_query = query;
                Task::none()
            }
            Message::ExportRules => {
                let rules = self.rules.clone();
                Task::perform(
                    async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .set_title("Export Rules Backup")
                            .set_file_name("tiling_rules.ron")
                            .add_filter("RON", &["ron"])
                            .save_file()
                            .await 
                        {
                            let pretty = ron::ser::PrettyConfig::default();
                            if let Ok(content) = ron::ser::to_string_pretty(&rules, pretty) {
                                let _ = std::fs::write(handle.path(), content);
                            }
                        }
                    },
                    |_| Message::FileOperationDone
                ).map(Into::into)
            }
            Message::ImportRules => {
                Task::perform(
                    async {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .set_title("Import Rules Backup")
                            .add_filter("RON", &["ron"])
                            .pick_file()
                            .await
                        {
                            if let Ok(content) = std::fs::read_to_string(handle.path()) {
                                if let Ok(new_rules) = ron::from_str::<Vec<WindowRule>>(&content) {
                                    return Some(new_rules);
                                }
                            }
                        }
                        None
                    },
                    Message::RulesImported
                ).map(Into::into)
            }
            Message::RulesImported(Some(new_rules)) => {
                self.rules = new_rules;
                write_rules(&self.rules);
                Task::none()
            }
            Message::RulesImported(None) | Message::FileOperationDone => {
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut open_col = Column::new().spacing(15).push(text("Open Windows").size(20));
        for w in &self.windows {
            open_col = open_col.push(
                Row::new().spacing(10).align_y(cosmic::iced::Alignment::Center)
                    .push(icon::from_name(w.app_id.as_str()).size(24))
                    .push(text(&w.app_id).width(Length::Fill))
                    .push(button::text("Float").on_press(Message::AddRule(w.app_id.clone())))
            );
        }

        let mut rule_col = Column::new().spacing(15)
            .push(text("Active Exceptions").size(20));
            
        let filtered_rules = self.rules.iter().filter(|r| r.appid.to_lowercase().contains(&self.search_query.to_lowercase()));
            
        for r in filtered_rules {
            rule_col = rule_col.push(
                Row::new().spacing(10).align_y(cosmic::iced::Alignment::Center)
                    .push(icon::from_name(r.appid.as_str()).size(24))
                    .push(text(&r.appid).width(Length::Fill))
                    .push(button::text("Remove").on_press(Message::RemoveRule(r.appid.clone())))
            );
        }
        
        let left_panel = container(scrollable(open_col))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(20)
            .class(cosmic::theme::Container::Secondary);
            
        let right_panel = container(scrollable(rule_col))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .padding(20);

        let content = Row::new().spacing(30)
            .push(left_panel)
            .push(right_panel);
            
        let header_text = Column::new().spacing(5)
            .push(text("COSMIC Tiling Exceptions Manager").size(32))
            .push(text("Manage applications that should bypass the COSMIC tiling system and open in floating mode.").size(16));
            
        let search_box = text_input("Search active rules...", &self.search_query)
            .on_input(Message::SearchChanged)
            .width(Length::Fixed(300.0));
            
        let header = Row::new()
            .spacing(20)
            .align_y(cosmic::iced::Alignment::Center)
            .push(header_text.width(Length::Fill))
            .push(search_box);
            
        let toolbar = Row::new()
            .spacing(15)
            .push(button::text("Refresh Windows").on_press(Message::Refresh))
            .push(button::text("Export Backup").on_press(Message::ExportRules))
            .push(button::text("Import Backup").on_press(Message::ImportRules));

        Column::new().spacing(20).padding(30)
            .push(header)
            .push(toolbar)
            .push(content)
            .into()
    }
}

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<TilingApp>(Settings::default(), ())
}
