use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper, SessionState};
use crate::app::DynError;
use crate::debug;
use crate::sketchbook::Sketch;
use serde::{Deserialize, Serialize};

/// Object encompassing all of the state components of the Analysis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisState {
    /// Boolean network sketch to run the analysis on.
    sketch: Sketch,
    // TODO
}

impl AnalysisState {
    pub fn new(sketch: Sketch) -> AnalysisState {
        AnalysisState { sketch }
    }
}

impl SessionHelper for AnalysisState {}

impl SessionState for AnalysisState {
    fn perform_event(&mut self, event: &Event, at_path: &[&str]) -> Result<Consumed, DynError> {
        debug!(
            "Events for analysis not implemented! Can't process {:?} at {:?}",
            event, at_path
        );
        todo!()
    }

    fn refresh(&self, full_path: &[String], at_path: &[&str]) -> Result<Event, DynError> {
        debug!(
            "Refresh for analysis not implemented! Can't process {:?} {:?}",
            full_path, at_path
        );
        todo!()
    }
}
