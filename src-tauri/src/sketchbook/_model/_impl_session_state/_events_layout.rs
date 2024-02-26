use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::{AeonError, DynError};
use crate::sketchbook::layout::{LayoutId, NodePosition};
use crate::sketchbook::simplified_structs::{LayoutData, LayoutNodeData};
use crate::sketchbook::ModelState;

use std::str::FromStr;

/// Implementation for events related to `layouts` of the model.
impl ModelState {
    /// Perform event of adding a new `layout` component to this `ModelState`.
    pub(super) fn event_add_layout(&mut self, event: &Event) -> Result<Consumed, DynError> {
        let component_name = "model/layout";

        // get payload components (json for LayoutData)
        let payload = Self::clone_payload_str(event, component_name)?;
        let layout_data = LayoutData::from_str(payload.as_str())?;
        let layout_id_str = layout_data.id;
        let layout_id = self.generate_layout_id(&layout_id_str);
        let name = layout_data.name;

        // perform the event, prepare the state-change variant (path and payload stay the same)
        // todo: decide on how to add layouts - now we just make a copy of the default layout
        self.add_layout_copy(layout_id, name.as_str(), &Self::get_default_layout_id())?;
        let state_change = event.clone();

        // prepare the reverse event (it is a remove event, so IDs are in path and payload is empty)
        let reverse_path = ["model", "layout", layout_id_str.as_str(), "remove"];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(Consumed::Reversible {
            state_change,
            perform_reverse: (event.clone(), reverse_event),
        })
    }

    /// Perform event of modifying or removing existing `layout` component of this `ModelState`.
    pub(super) fn event_modify_layout(
        &mut self,
        event: &Event,
        at_path: &[&str],
        layout_id: LayoutId,
    ) -> Result<Consumed, DynError> {
        let component_name = "model/layout";

        if Self::starts_with("update_position", at_path).is_some() {
            // get payload components (json containing "var_id", "new_x" and "new_y")
            let payload = Self::clone_payload_str(event, component_name)?;
            let new_node_data = LayoutNodeData::from_str(payload.as_str())?;
            let var_id = self.get_var_id(new_node_data.variable.as_str())?;
            let new_x = new_node_data.px;
            let new_y = new_node_data.py;
            let new_position = NodePosition(new_x, new_y);

            let orig_pos = self.get_node_position(&layout_id, &var_id)?.clone();
            let orig_pos_data = LayoutNodeData::new(
                layout_id.to_string(),
                var_id.to_string(),
                orig_pos.0,
                orig_pos.1,
            );
            let new_pos_data =
                LayoutNodeData::new(layout_id.to_string(), var_id.to_string(), new_x, new_y);

            if new_position == orig_pos {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move ID from path to payload)
            self.update_node_position(&layout_id, &var_id, new_x, new_y)?;
            let state_change_path = ["model", "layout", "update_position"];
            let state_change = Event::build(&state_change_path, Some(&new_pos_data.to_string()));

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(orig_pos_data.to_string());
            Ok(Consumed::Reversible {
                state_change,
                perform_reverse: (event.clone(), reverse_event),
            })
        } else if Self::starts_with("remove", at_path).is_some() {
            // check that payload is really empty
            if event.payload.is_some() {
                let message = "Payload must be empty for layout removing.".to_string();
                return AeonError::throw(message);
            }

            let layout = self.get_layout(&layout_id)?;
            let layout_data = LayoutData::from_layout(&layout_id, layout);

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.remove_layout(&layout_id)?;
            let state_change_path = ["model", "layout", "remove"];
            let state_change = Event::build(&state_change_path, Some(&layout_data.to_string()));

            // todo make reversible in the future?
            Ok(Consumed::Irreversible {
                state_change,
                reset: true,
            })
        } else {
            Self::invalid_path_error_specific(at_path, component_name)
        }
    }

    /// Perform events related to `layouts` component of this `ModelState`.
    pub(super) fn perform_layout_event(
        &mut self,
        event: &Event,
        at_path: &[&str],
    ) -> Result<Consumed, DynError> {
        let component_name = "model/layout";

        // there is either adding of a new layout, or editing/removing of an existing one
        // when adding new layout, the `at_path` is just ["add"]
        // when editing existing layout, the `at_path` is ["layout_id", "<action>"]

        if Self::starts_with("add", at_path).is_some() {
            Self::assert_path_length(at_path, 1, component_name)?;
            self.event_add_layout(event)
        } else {
            Self::assert_path_length(at_path, 2, component_name)?;
            let layout_id_str = at_path.first().unwrap();
            let layout_id = self.get_layout_id(layout_id_str)?;

            self.event_modify_layout(event, &at_path[1..], layout_id)
        }
    }
}
