use actix::prelude::*;

pub struct ContainerManager;

impl Actor for ContainerManager {
    type Context = Context<Self>;
}
