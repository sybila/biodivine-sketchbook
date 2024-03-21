use crate::app::event::Event;
use crate::app::state::{Consumed, SessionHelper};
use crate::app::DynError;
use crate::sketchbook::data_structs::{LayoutData, LayoutNodeData};
use crate::sketchbook::event_utils::{make_reversible, make_state_change};
use crate::sketchbook::layout::NodePosition;
use crate::sketchbook::{LayoutId, ModelState};

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

        // perform the event
        self.add_layout_copy(layout_id, &name, &Self::get_default_layout_id())?;

        // prepare the state-change and reverse event (which is a remove event)
        let reverse_path = ["model", "layout", &layout_id_str, "remove"];
        let reverse_event = Event::build(&reverse_path, None);
        Ok(make_reversible(event.clone(), event, reverse_event))
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
            let new_position = NodePosition(new_node_data.px, new_node_data.py);

            let orig_pos = self.get_node_position(&layout_id, &var_id)?.clone();
            let orig_pos_data =
                LayoutNodeData::new(layout_id.as_str(), var_id.as_str(), orig_pos.0, orig_pos.1);
            let new_pos_data = LayoutNodeData::new(
                layout_id.as_str(),
                var_id.as_str(),
                new_node_data.px,
                new_node_data.py,
            );

            if new_position == orig_pos {
                return Ok(Consumed::NoChange);
            }

            // perform the event, prepare the state-change variant (move ID from path to payload)
            self.update_node_position(&layout_id, &var_id, new_node_data.px, new_node_data.py)?;
            let state_change =
                make_state_change(&["model", "layout", "update_position"], &new_pos_data);

            // prepare the reverse event
            let mut reverse_event = event.clone();
            reverse_event.payload = Some(orig_pos_data.to_string());
            Ok(make_reversible(state_change, event, reverse_event))
        } else if Self::starts_with("remove", at_path).is_some() {
            Self::assert_payload_empty(event, component_name)?;

            let layout = self.get_layout(&layout_id)?;
            let layout_data = LayoutData::from_layout(&layout_id, layout);

            // perform the event, prepare the state-change variant (move id from path to payload)
            self.remove_layout(&layout_id)?;
            let state_change = make_state_change(&["model", "layout", "remove"], &layout_data);

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
