
use std::{collections::BTreeMap, iter::FromIterator};

use anyhow::Context;
use ::hcloud::{
    apis::{configuration::Configuration, servers_api::ListServersParams},
    models::{Server},
};

/// This is just some exploration code for now...
pub async fn reconcile(
    configuration: &Configuration,
    wanted_servers: &[Server],
) -> anyhow::Result<()> {
    let mut servers: Vec<Server> = Vec::with_capacity(wanted_servers.len());

    let mut next_page = None;
    loop {
        let listed_servers = hcloud::apis::servers_api::list_servers(
            configuration,
            ListServersParams {
                label_selector: Some("managed_by=reconcise".to_string()),
                page: next_page,
                ..Default::default()
            },
        )
        .await
        .with_context(|| "while listing servers")?;

        let meta: Option<Box<hcloud::models::Meta>> = listed_servers.meta;
        next_page = meta.and_then(|m| m.pagination).and_then(|p| p.next_page);
        if next_page.is_none() {
            break;
        }
        servers.extend(listed_servers.servers.into_iter());
    }

    let wanted: BTreeMap<&str, &Server> = servers_by_name(wanted_servers);
    let actual: BTreeMap<&str, &Server> = servers_by_name(&servers);

    let mut wanted_iter = wanted.iter();
    let mut wanted_item = wanted_iter.next();
    let mut actual_iter = actual.iter();
    let mut actual_item = actual_iter.next();

    let mut add = Vec::new();
    let mut remove = Vec::new();
    let mut sync = Vec::new();

    loop {
        match (wanted_item, actual_item) {
            (Some((wn, wanted_server)), Some((an, actual_server))) => match wn.cmp(an) {
                std::cmp::Ordering::Less => {
                    add.push(wanted_server);
                    wanted_item = wanted_iter.next();
                }
                std::cmp::Ordering::Equal => {
                    remove.push(actual_server);
                    actual_item = actual_iter.next();
                }
                std::cmp::Ordering::Greater => {
                    sync.push((wanted_server, actual_server));
                    actual_item = actual_iter.next();
                    wanted_item = wanted_iter.next();
                }
            },
            (Some((_, wanted_server)), None) => {
                add.push(wanted_server);
                wanted_item = wanted_iter.next();
            }
            (None, Some((_, actual_server))) => {
                remove.push(actual_server);
                actual_item = actual_iter.next();
            }
            (None, None) => break,
        }
    }

    Ok(())
}

fn servers_by_name<'a, B: FromIterator<(&'a str, &'a Server)>>(servers: &'a [Server]) -> B {
    servers
        .iter()
        .map(|server| (server.name.as_str(), server))
        .collect()
}
