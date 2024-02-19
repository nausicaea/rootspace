use clap::{load_yaml, App};
use ecs::{Entities, EventQueue, Index, Resources, Storage, WorldEvent};
use rose_tree::Hierarchy;
use serde::{Deserialize, Serialize};
use term_table::{row::Row, table_cell::TableCell, TableBuilder, TableStyle};

use super::{CommandTrait, Error};
use crate::components::{Camera, Info, Model, Renderable, Status, UiModel};

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
            let entity = res
                .borrow::<Entities>()
                .get(index)
                .ok_or(Error::EntityNotFound(index))?;

            if let Some(ic) = res.borrow_components::<Info>().get(&entity) {
                println!("Name: {}, Description: {}", ic.name(), ic.description());
            } else {
                println!("No description found");
            }

            if let Some(sc) = res.borrow_components::<Status>().get(&entity) {
                println!("LOCAL - Enabled: {}, Visible: {}", sc.enabled(), sc.visible());
            } else {
                println!("No status information found");
            }

            let mut other_components = String::new();
            if res.borrow_components::<Model>().contains(&entity) {
                other_components.push_str(" MODEL");
            }
            if res.borrow_components::<UiModel>().contains(&entity) {
                other_components.push_str(" UI-MODEL");
            }
            if res.borrow_components::<Camera>().contains(&entity) {
                other_components.push_str(" CAMERA");
            }
            if res.borrow_components::<Renderable>().contains(&entity) {
                other_components.push_str(" RENDERABLE");
            }
            if other_components.len() > 0 {
                println!("Other components:{}", other_components);
            } else {
                println!("No other components found");
            }
        } else if subcommand == "table" {
            let entities = res.borrow::<Entities>();
            let statuses = res.borrow_components::<Status>();

            // FIXME: This does not account for hierarchical statuses
            let total_count = entities.len();
            let ev_count = statuses.iter().filter(|s| s.enabled() && s.visible()).count();
            let eh_count = statuses.iter().filter(|s| s.enabled() && !s.visible()).count();
            let dh_count = statuses.iter().filter(|s| !s.enabled() && !s.visible()).count();
            let dv_count = statuses.iter().filter(|s| !s.enabled() && s.visible()).count();
            let no_status = entities.iter().filter(|e| !statuses.contains(e)).count();

            let table = TableBuilder::new()
                .style(TableStyle::simple())
                .rows(vec![
                    Row::new(vec![
                        TableCell::new("Loaded entities"),
                        TableCell::new(format!("Total {}", total_count)),
                        TableCell::new(format!("No status {}", no_status)),
                    ]),
                    Row::new(vec![
                        TableCell::new(""),
                        TableCell::new("Enabled"),
                        TableCell::new("Disabled"),
                    ]),
                    Row::new(vec![
                        TableCell::new("Visible"),
                        TableCell::new(ev_count),
                        TableCell::new(dv_count),
                    ]),
                    Row::new(vec![
                        TableCell::new("Hidden"),
                        TableCell::new(eh_count),
                        TableCell::new(dh_count),
                    ]),
                    Row::new(vec![
                        TableCell::new("Sub total"),
                        TableCell::new(ev_count + eh_count),
                        TableCell::new(dv_count + dh_count),
                    ]),
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
                // FIXME: This does not account for hierarchical statuses
                if show_all
                    || statuses.get(entity).map_or(false, |s| {
                        (s.enabled() || show_disabled) && (s.visible() || show_hidden)
                    })
                {
                    let info = infos.get(entity);

                    println!(
                        "Entity {}: {}, {}",
                        entity,
                        info.map_or("(no name)", |i| i.name()),
                        info.map_or("(no description)", |i| i.description()),
                    );
                }
            }
        } else if subcommand == "create" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("create"))?;
            let parent = scm.value_of("parent");
            let parent = if let Some(parent) = parent {
                let parent = parent.parse::<usize>()?;
                let parent = res
                    .borrow::<Entities>()
                    .get(parent)
                    .ok_or(Error::EntityNotFound(parent))?;

                Some(parent)
            } else {
                None
            };

            let entity = res.borrow_mut::<Entities>().create();

            if let Some(parent) = parent {
                res.borrow_mut::<Hierarchy<Index>>().insert_child(&parent, entity);
                println!("Created the entity {} with parent {}", entity, parent);
            } else {
                res.borrow_mut::<Hierarchy<Index>>().insert(entity);
                println!("Created the entity {}", entity);
            }

            res.borrow_mut::<EventQueue<WorldEvent>>()
                .send(WorldEvent::EntityCreated(entity));
        } else if subcommand == "destroy" {
            let scm = scm.ok_or(Error::NoSubcommandArguments("destroy"))?;
            let index = scm.value_of("index").ok_or(Error::NoIndexSpecified)?;

            let index: usize = index.parse()?;
            let entity = res
                .borrow::<Entities>()
                .get(index)
                .ok_or(Error::EntityNotFound(index))?;

            if res.borrow::<Hierarchy<Index>>().has_children(&entity) {
                return Err(Into::into(Error::CannotDestroyEntity(index)));
            }

            res.borrow_mut::<Hierarchy<Index>>().remove(&entity);
            res.borrow_components_mut::<Info>().remove(&entity);
            res.borrow_components_mut::<Status>().remove(&entity);
            res.borrow_components_mut::<Model>().remove(&entity);
            res.borrow_components_mut::<UiModel>().remove(&entity);
            res.borrow_components_mut::<Camera>().remove(&entity);
            res.borrow_components_mut::<Renderable>().remove(&entity);
            res.borrow_mut::<Entities>().destroy(entity);
            println!("Destroyed the entity {}", entity);
        }

        Ok(())
    }
}
