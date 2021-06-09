use clap::{load_yaml, App};

use ecs::{Entities, Resources, Storage, EventQueue, WorldEvent};

use super::Error;
use crate::{components::{Camera, Info, Model, Renderable, Status, UiModel}, resources::{SceneGraph}, CommandTrait};
use serde::{Deserialize, Serialize};
use term_table::{TableBuilder, TableStyle};
use term_table::row::Row;
use term_table::table_cell::TableCell;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntitiesCommand;

impl CommandTrait for EntitiesCommand {
    fn name(&self) -> &'static str {
        "entities"
    }

    fn description(&self) -> &'static str {
        "Provides access to entities within the world"
    }

    fn run(&self, res: &Resources, args: &[String]) -> anyhow::Result<()> {
        let app_yaml = load_yaml!("entities.yaml");
        let matches = App::from_yaml(app_yaml).get_matches_from_safe(args)?;
        let (subcommand, scm) = matches.subcommand();

        if subcommand == "info" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("info"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res.borrow::<Entities>().try_get(index).ok_or(Error::EntityNotFound(index))?;

            if let Some(ic) = res.borrow_components::<Info>().get(&entity) {
                println!("Name: {}, Description: {}", ic.name(), ic.description());
            }

            if let Some(sc) = res.borrow_components::<Status>().get(&entity) {
                println!("Enabled: {}, Visible: {}", sc.enabled(), sc.visible());
            }

            if let Some(mc) = res.borrow_components::<Model>().get(&entity) {
                println!("LOCAL - Position: {:?}, Orientation: {}, Scale: {:?}", mc.position().coords, mc.orientation(), mc.scale());
            }

            if let Some(sgmc) = res.borrow::<SceneGraph<Model>>().get(&entity) {
                println!("GLOBAL - Position: {:?}, Orientation: {}, Scale: {:?}", sgmc.position().coords, sgmc.orientation(), sgmc.scale());
            }

            if let Some(umc) = res.borrow_components::<UiModel>().get(&entity) {
                println!("UI LOCAL - Position: {:?}, Depth: {}, Scale: {:?}", umc.position().coords, umc.depth(), umc.scale());
            }

            if let Some(sgumc) = res.borrow::<SceneGraph<UiModel>>().get(&entity) {
                println!("UI GLOBAL - Position: {:?}, Depth: {}, Scale: {:?}", sgumc.position().coords, sgumc.depth(), sgumc.scale());
            }

            let mut other_components = String::from("Other components:");
            if res.borrow_components::<Camera>().has(&entity) {
                other_components.push_str(" CAMERA");
            }

            if res.borrow_components::<Renderable>().has(&entity) {
                other_components.push_str(" RENDERABLE");
            }
            println!("{}", other_components);
        } else if subcommand == "count" {
            let entities = res.borrow::<Entities>();
            let statuses = res.borrow_components::<Status>();

            let total_count = entities.len();
            let ev_count = statuses.iter().filter(|s| s.enabled() && s.visible()).count();
            let eh_count = statuses.iter().filter(|s| s.enabled() && !s.visible()).count();
            let dh_count = statuses.iter().filter(|s| !s.enabled() && !s.visible()).count();
            let dv_count = statuses.iter().filter(|s| !s.enabled() && s.visible()).count();
            let no_status = entities.iter().filter(|e| !statuses.has(e)).count();

            let table = TableBuilder::new()
                .style(TableStyle::simple())
                .rows(vec![
                    Row::new(vec![TableCell::new("Loaded entities"), TableCell::new(format!("Total {}", total_count)), TableCell::new(format!("No status {}", no_status))]),
                    Row::new(vec![TableCell::new(""), TableCell::new("Enabled"), TableCell::new("Disabled")]),
                    Row::new(vec![TableCell::new("Visible"), TableCell::new(ev_count), TableCell::new(dv_count)]),
                    Row::new(vec![TableCell::new("Hidden"), TableCell::new(eh_count), TableCell::new(dh_count)]),
                    Row::new(vec![TableCell::new("Sub total"), TableCell::new(ev_count + eh_count), TableCell::new(dv_count + dh_count)]),
                ])
                .build();

            println!("{}", table.render());
        } else if subcommand == "list" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("list"))?;

            let show_all = scm.is_present("all");
            let show_disabled = scm.is_present("disabled");
            let show_hidden = scm.is_present("hidden");

            let entities = res.borrow::<Entities>();
            let infos = res.borrow_components::<Info>();
            let statuses = res.borrow_components::<Status>();

            for entity in entities.iter() {
                if show_all || statuses.get(entity).map_or(false, |s| (s.enabled() || show_disabled) && (s.visible() || show_hidden)) {
                    println!("{}: {}", entity.idx(), infos.get(entity).map_or("(no name)", |i| i.name()));
                }
            }

        } else if subcommand == "create" {
            res.borrow_mut::<EventQueue<WorldEvent>>().send(WorldEvent::CreateEntity);
        } else if subcommand == "destroy" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("destroy"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res.borrow::<Entities>().try_get(index).ok_or(Error::EntityNotFound(index))?;

            res.borrow_mut::<EventQueue<WorldEvent>>().send(WorldEvent::DestroyEntity(entity))
        }

        Ok(())
    }
}
