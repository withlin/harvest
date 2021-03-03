use common::Result;
use event::{obj::Dispatch, EventHandler};
use serde::{Deserialize, Serialize};
use std::convert::AsRef;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use strum::AsRefStr;

#[derive(AsRefStr, Debug)]
pub enum Event {
    #[strum(serialize = "Add")]
    Add,
    #[strum(serialize = "Delete")]
    Delete,
    #[strum(serialize = "Update")]
    Update,
}

pub trait GetPod {
    fn get(&self) -> Option<&Pod>;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pod {
    // on this the uuid path is unique identifier
    pub uuid: String,
    pub offset: usize,
    pub namespace: String,
    pub pod_name: String,
    pub container_name: String,
    pub upload: bool,
}

impl GetPod for Pod {
    fn get(&self) -> Option<&Pod> {
        Some(self)
    }
}

type PodList = Vec<Pod>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PodListMarshaller(PodList);

impl PodListMarshaller {
    pub fn to_json(&self) -> String {
        match serde_json::to_string(&self.0) {
            Ok(contents) => contents,
            Err(_) => "".to_owned(),
        }
    }
}

pub struct Database {
    // pod key is the pod path uuid
    pods: HashMap<String, Pod>,
    // pod op registry and handle events
    event_handler: Dispatch<Pod>,
}

impl Database {
    pub fn new(event_handler: Dispatch<Pod>) -> Self {
        Self {
            pods: HashMap::new(),
            event_handler,
        }
    }

    pub fn all(&self) -> PodListMarshaller {
        PodListMarshaller(
            self.pods
                .iter()
                .map(|(_, v)| v.clone())
                .collect::<Vec<Pod>>(),
        )
    }

    pub fn get(&self, uuid: String) -> Option<&Pod> {
        self.pods.get(&*uuid)
    }

    pub fn incr_offset_by_uuid(&mut self, uuid: String, incr_size: usize) {
        self.pods.get_mut(&uuid).unwrap().offset += incr_size
    }

    pub fn get_by_pod(&self, namespace: String, pod: String) -> Option<&Pod> {
        match self
            .pods
            .iter()
            .find(|(_, v)| v.namespace == namespace && v.pod_name == pod)
        {
            None => None,
            Some((_, pod)) => Some(pod),
        }
    }

    pub fn put(&mut self, pod: Pod) -> Result<()> {
        self.event_handler
            .dispatch(Event::Add.as_ref().to_string(), pod.clone());
        self.pods.insert(pod.uuid.clone(), pod);
        Ok(())
    }

    pub fn delete_by_namespace_pod(&mut self, namespace: String, pod: String) -> Result<()> {
        let uuid = self
            .pods
            .iter()
            .find(|(_, v)| v.namespace == namespace && v.pod_name == pod);

        if let Some((_uuid, _)) = uuid {
            return self.delete(_uuid.to_owned());
        }

        Ok(())
    }

    pub fn delete(&mut self, uuid: String) -> Result<()> {
        match self.pods.get(&*uuid) {
            Some(pod) => {
                self.event_handler
                    .dispatch(Event::Delete.as_ref().to_string(), pod.clone());
                self.pods.remove(&*uuid);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn update(&mut self, uuid: String, pod: Pod) -> Result<()> {
        self.event_handler
            .dispatch(Event::Update.as_ref().to_string(), pod.clone());
        self.pods.insert(uuid, pod);
        Ok(())
    }
}

pub fn new_sync_database(db: Database) -> Arc<Mutex<Database>> {
    Arc::new(Mutex::new(db))
}

#[cfg(test)]
mod tests {
    use crate::Event;

    #[test]
    fn event_it_works() {
        assert_eq!(Event::Add.as_ref(), "Add");
        assert_eq!(Event::Delete.as_ref(), "Delete");
        assert_eq!(Event::Update.as_ref(), "Update");
    }
}
