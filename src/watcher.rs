use crate::context::StringList;
use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::ffi::CString;

pub struct FileWatcher {
    inner: Box<dyn notify::Watcher>,
    recv: std::sync::mpsc::Receiver<String>,
}

fn event_kind_to_string(kind: notify::EventKind) -> &'static str {
    match kind {
        notify::EventKind::Create(_) => "create",
        notify::EventKind::Access(_) => "access",
        notify::EventKind::Modify(_) => "modify",
        notify::EventKind::Remove(_) => "remove",
        _ => "other",
    }
}

fn event_to_string(evt: notify::Event) -> String {
    let paths: Vec<String> = evt
        .paths
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let pathstr = paths.join(";");

    format!("{}:{}", event_kind_to_string(evt.kind), pathstr)
}

impl FileWatcher {
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = std::sync::mpsc::channel();

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, Error>| match res {
                Ok(evt) => {
                    tx.send(event_to_string(evt)).expect("Send error.");
                }
                Err(e) => {
                    tx.send(e.to_string()).expect("Send error");
                }
            },
            Config::default(),
        )
        .map_err(|e| e.to_string())?;

        Ok(Self {
            inner: Box::new(watcher),
            recv: rx,
        })
    }

    pub fn unwatch(&mut self, path: String) -> Result<(), String> {
        self.inner.unwatch(path.as_ref()).map_err(|e| e.to_string())
    }

    pub fn watch(&mut self, path: String, recursive: bool) -> Result<(), String> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        self.inner
            .watch(path.as_ref(), mode)
            .map_err(|e| e.to_string())
    }

    pub fn poll_events(&mut self) -> StringList {
        let mut events = StringList::new();
        while let Ok(evt) = self.recv.try_recv() {
            events.push(
                CString::new(evt)
                    .expect("It should be impossible for a std::String to contain null bytes!"),
            );
        }
        events
    }
}
