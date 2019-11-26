use crate::neoapi::register_functionality;
use crate::utils::{react_failure, react_halo, react_skull, react_success};
use rhai::{Any, Engine, EvalAltResult, Scope};
use serenity::model::prelude::Message;
use serenity::prelude::Context;

pub struct Script {
    pub source_msg: Message,
    pub engine: rhai::Engine,
    pub enabled: bool,
}

unsafe impl Send for Script {}
unsafe impl Sync for Script {}

impl Script {
    pub fn new(source: &str, context: &Context, source_msg: Message) -> Option<Self> {
        let mut engine = Engine::new();
        register_functionality(&mut engine, context, &source_msg);

        match engine.eval::<()>(source) {
            Ok(_) => {
                let _ = source_msg.react(context, react_success());
                Some(Script {
                    engine,
                    source_msg,
                    enabled: true,
                })
            }
            Err(e) => {
                let _ = source_msg.reply(context, format!("Fehler: {}", e));
                let _ = source_msg.react(context, react_failure());
                None
            }
        }
    }

    pub fn notify(&mut self, context: &Context, function: &str, args: Vec<Box<dyn Any>>) {
        if !self.enabled {
            return;
        }

        let mut scope = Scope::new();
        let mut counter = 0;
        for arg_value in args.into_iter() {
            let arg_name = format!("__argument{}", counter);
            scope.push((arg_name, arg_value));
            counter += 1;
        }

        let call_args = scope
            .iter()
            .map(|arg| arg.0.clone())
            .collect::<Vec<_>>()
            .join(",");
        let call = format!("{}({});", function, call_args);

        match self.engine.eval_with_scope::<()>(&mut scope, &call) {
            Ok(_) => {
                let _ = self.source_msg.react(context, react_halo());
            }
            Err(EvalAltResult::ErrorFunctionNotFound(_)) => (),
            Err(e) => {
                println!("execution error: {}", e);
                let _ = self.source_msg.react(context, react_skull());
            }
        }
    }

    pub fn set_status(&mut self, enabled: bool, ctx: &Context) {
        if self.enabled == enabled {
            return;
        }

        let _ = self.source_msg.delete_reactions(&ctx.clone());
        self.enabled = enabled;

        let _ = match enabled {
            true => self.source_msg.react(&ctx.clone(), react_success()),
            false => self.source_msg.react(&ctx.clone(), react_failure()),
        };
    }
}
