#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::env;
use eframe::egui;
use riven::{RiotApi};
use riven::consts::PlatformRoute;
use riven::models::champion_mastery_v4::ChampionMastery;
use riven::models::summoner_v4::Summoner;
use riven::models::clash_v1::Player;

fn main() -> Result<(), eframe::Error> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 1000.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Summoner Requester v0.1",
        options,
        Box::new(|_cc| Box::new(MyApp::new(_cc))),
    )
}

struct MyApp {
    summoner_id: String,
    summoner_name: String,
    tmp_summoner_name: String,
    summoner_level: i64,
    team_players_len: usize,
    team_summoner_ids: Vec<String>,
    requested: bool
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            summoner_id: "".to_owned(),
            summoner_name: "".to_owned(),
            tmp_summoner_name: "Enter Summoner Name".to_owned(),
            summoner_level: 69,
            team_players_len: 0,
            team_summoner_ids: vec!["".to_owned()],
            requested: false
        }
    }
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {

                let args: Vec<String> = env::args().collect();
                dbg!(&args);
                let api_key = &args[1]; // "RGAPI-01234567-89ab-cdef-0123-456789abcdef";
                let riot_api = RiotApi::new(api_key);

                ui.heading("Summoner Requester");
                ui.horizontal(|ui| {
                    let name_label = ui.label("Summoner Name: ");
                    ui.text_edit_singleline(&mut self.tmp_summoner_name)
                        .labelled_by(name_label.id);
                });
                if ui.button("Request Mastery").clicked() {
                     let summoner: Summoner = get_summoner(&riot_api,&self.tmp_summoner_name).await;
                    self.summoner_id = summoner.id;
                    self.summoner_name = summoner.name;
                    self.summoner_level = summoner.summoner_level;
                }
                ui.label(format!("Name '{}', Level {}", self.summoner_name, self.summoner_level));

                if self.summoner_id != "" && !self.requested {
                    let team_players = get_players_of_team_by_summoner(&riot_api, &self.summoner_id, &self.summoner_name).await;
                    self.team_players_len = team_players.len();
                    for player in team_players {
                        ui.label(format!("----------------------------------------"));

                        let team_summoner = riot_api.summoner_v4().get_by_summoner_id(PlatformRoute::EUW1, &player.summoner_id).await
                            .expect("no team summoner found");
                        ui.label(format!("Name '{}', Level {}, Position {}", team_summoner.name,team_summoner.summoner_level, player.position));
                        let masteries = get_player_masteries(&riot_api, &team_summoner.id).await;

                        for (i, mastery) in masteries.iter().take(10).enumerate() {
                            ui.label(format!("{: >2}) {: <9}    {: >7} ({})", i + 1,
                                mastery.champion_id.name().unwrap_or("UNKNOWN"),
                                mastery.champion_points, mastery.champion_level));
                        }
                    }
                    requested = true;
                }
            });
        });
    }
}

async fn get_summoner(riot_api: &RiotApi, name: &str) -> Summoner{

    // Get summoner data.
    return riot_api.summoner_v4()
        .get_by_summoner_name(PlatformRoute::EUW1, name).await
        .expect("Summoner not found")
        .expect("Summoner with name not found");
}


async fn get_players_of_team_by_summoner(riot_api: &RiotApi, summoner_id: &String, summoner_name: &String) -> Vec<Player> {

    let team_player = riot_api.clash_v1().get_players_by_summoner(PlatformRoute::EUW1, summoner_id).await
        .expect(&format!("Could not find team for {}",summoner_name));

    let team_id = team_player[0].team_id
                    .clone().expect("team not found");

    let team = riot_api.clash_v1().get_team_by_id(PlatformRoute::EUW1, &team_id).await
        .expect("no team found")
        .expect("team with team id not found");

    team.players
}

async fn get_player_masteries(riot_api: &RiotApi, id: &String) -> Vec<ChampionMastery> {
    // Get summoner data.
    let summoner = riot_api.summoner_v4()
        .get_by_summoner_id(PlatformRoute::EUW1, id).await
        .expect("Get summoner failed.");

    // Get champion mastery data.
    let masteries = riot_api.champion_mastery_v4()
        .get_all_champion_masteries(PlatformRoute::EUW1, &summoner.id).await
        .expect("Get champion masteries failed.");

    masteries
}
