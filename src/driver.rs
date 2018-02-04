use std::io;

use futures;
use futures::prelude::*;
use futures_stream_select_all::select_all;
use serde_yaml::Value;
use tokio_core::reactor::Handle;

use actions::Action;
use actions::command::{ACTION_NAME as COMMAND_ACTION_NAME, CommandAction};
use context::{Context, TriggerBehavior};
use triggers::{Activity, Trigger};
use triggers::wifi::{TRIGGER_NAME as WIFI_TRIGGER_NAME, WifiTrigger};

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

            if (ctx.trigger_behavior == TriggerBehavior::And && activity_counter == ctx.triggers.len()) ||
                (ctx.trigger_behavior == TriggerBehavior::Or && activity_counter > 0 && prev_act_counter == 0) {
                for action in actions.iter_mut() {
                    let _ = action.enter();
                }
            } else {
                for action in actions.iter_mut() {
                    let _ = action.leave();
                }
            }

            futures::finished(())
        })
        .map(|_| ())
        .map_err(|_| ());

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