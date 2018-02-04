use std::io;

use futures::future;
use futures::prelude::*;
use futures_stream_select_all::select_all;
use serde_yaml::Value;
use tokio_core::reactor::Handle;

use actions::Action;
use actions::command::{ACTION_NAME as COMMAND_ACTION_NAME, CommandAction};
use context::{Context, TriggerBehavior};
use triggers::{Activity, Trigger};
use triggers::wifi::{TRIGGER_NAME as WIFI_TRIGGER_NAME, WifiTrigger};

/// Drives the given context listening for evidence sources and
/// executing actions as required.
pub fn drive(ctx: Context, handle: Handle) -> io::Result<Box<Future<Item = (), Error = ()>>> {
    let mut actions = ctx.actions.iter()
        .map(|(key, config)| get_action(key, config))
        .collect::<io::Result<Vec<Box<Action>>>>()?;

    let h = handle.clone();
    let triggers = ctx.triggers.iter()
        .map(|(key, config)| get_trigger(key, config))
        .collect::<io::Result<Vec<Box<Trigger>>>>()?
        .into_iter()
        .map(|mut t| t.listen(h.clone()));

    let mut activity_counter = 0;
    let driver = select_all(triggers)
        .for_each(move |act| {
            let prev_act_counter = activity_counter;
            match act {
                Activity::Active => activity_counter += 1,
                Activity::Inactive => activity_counter -= 1,
            }

            let is_all_active_and = ctx.trigger_behavior == TriggerBehavior::And &&
                activity_counter == ctx.triggers.len();
            let is_or_and_has_active = ctx.trigger_behavior == TriggerBehavior::Or &&
                activity_counter > 0 && prev_act_counter == 0;

            if is_all_active_and || is_or_and_has_active {
                let enter_all = actions.iter_mut()
                    .map(|act| act.enter())
                    .collect::<Vec<_>>();
                let fut = future::join_all(enter_all)
                    .map(|_| ());

                Box::new(fut) as Box<Future<Item = (), Error = io::Error>>
            } else {
                let leave_all = actions.iter_mut()
                    .map(|act| act.leave())
                    .collect::<Vec<_>>();
                let fut = future::join_all(leave_all)
                    .map(|_| ());

                Box::new(fut) as Box<Future<Item = (), Error = io::Error>>
            }
        })
        .map_err(|err| eprintln!("Experienced error while driving context: {:?}.", err));

    Ok(Box::new(driver))
}

fn get_action(name: &str, config: &Value) -> io::Result<Box<Action>> {
    match name.trim() {
        COMMAND_ACTION_NAME => Ok(Box::new(CommandAction::from_config(config)?)),

        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown action name '{}'.", name).as_ref(),
        ))
    }
}

fn get_trigger(name: &str, config: &Value) -> io::Result<Box<Trigger>> {
    match name.trim() {
        WIFI_TRIGGER_NAME => Ok(Box::new(WifiTrigger::from_config(config)?)),

        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown trigger name '{}'.", name).as_ref(),
        ))
    }
}