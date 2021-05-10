use std::{fmt::Debug};


use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod hcloud;

trait ResourceSpec: Debug + PartialEq  {
    fn type_name(&self) -> &'static str;
    fn id(&self) -> &str;
}

trait OutputSpec: Debug + PartialEq + Clone + Sized + Serialize {}
impl<T> OutputSpec for T where T: Debug + PartialEq + Clone + Sized  + Serialize {}

trait Resource {
    type Spec : ResourceSpec;
    type Output : OutputSpec;

    fn spec(&self) -> &Self::Spec;
    fn output(&self) -> &Self::Output;
}

trait InspectableResource {
    fn id(&self) -> &str;
    fn spec_ron(&self) -> serde_json::Value;
    fn output_ron(&self) -> serde_json::Value;
}

trait ResourceListBuilder<R: ResourceSpec> {
    fn add(&mut self, r: R) -> &mut Self;
}

trait ResourceSpecList<R: ResourceSpec> {
    fn resources(&self) -> &[&R];
}

#[async_trait]
trait ResourceReconciler<R: Resource>: ResourceSpecList<R::Spec> {
    async fn fetch(&self) -> anyhow::Result<Vec<R>>;
    async fn create(&self, resource: &R::Spec) -> anyhow::Result<&R>;
    async fn destroy(&self, resource: &R) -> anyhow::Result<()>;
}

impl<R: Resource> ResourceReconciler<R> {
    async fn reconcile(&self) -> anyhow::Result<Vec<R>> {
        todo!("TODO")
    }
}