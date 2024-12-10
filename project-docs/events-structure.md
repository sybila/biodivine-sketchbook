
# Overview

The communication between backend and frontend components of the app happens mainly through custom events.
This document provides an overview of the events that can be processed by the backend state API. 
Events are structured in a hierarchical manner to simplify navigation and understanding.

---
## Undo Stack
- **Path**: `['undo_stack', 'undo']`
  - **Description**: Undo the last action.
  - **Payload**: None
- **Path**: `['undo_stack', 'redo']`
  - **Description**: Redo the last undone action.
  - **Payload**: None


---
## Tab Bar
- **Path**: `['tab_bar', 'active']`
  - **Description**: Set the active tab.
  - **Payload**: `{ "tabId": "number" }`
- **Path**: `['tab_bar', 'pin']`
  - **Description**: Pin a tab.
  - **Payload**: `{ "tabId": "number" }`
- **Path**: `['tab_bar', 'unpin']`
  - **Description**: Unpin a tab.
  - **Payload**: `{ "tabId": "number" }`


---
## Sketch
- **Path**: `['sketch', 'refresh']`
  - **Description**: Refresh the whole sketch.
  - **Payload**: None
- **Path**: `['sketch', 'export_sketch']`
  - **Description**: Export the sketch data to a custom JSON format.
  - **Payload**: `{ "path": "string" }`
- **Path**: `['sketch', 'export_aeon']`
  - **Description**: Export the sketch data to an extended AEON format.
  - **Payload**: `{ "path": "string" }`
- **Path**: `['sketch', 'import_sketch']`
  - **Description**: Import sketch data from a JSON file.
  - **Payload**: `{ "path": "string" }`
- **Path**: `['sketch', 'import_aeon']`
  - **Description**: Import sketch data from an AEON file.
  - **Payload**: `{ "path": "string" }`
- **Path**: `['sketch', 'import_sbml']`
  - **Description**: Import a model from an SBML file.
  - **Payload**: `{ "path": "string" }`
- **Path**: `['sketch', 'new_sketch']`
  - **Description**: Set the sketch to a default state, clearing all data.
  - **Payload**: None
- **Path**: `['sketch', 'set_annotation']`
  - **Description**: Set a new annotation for the sketch.
  - **Payload**: `{ "annotation": "string" }`
- **Path**: `['sketch', 'check_consistency']`
  - **Description**: Run a consistency check on the sketch.
  - **Payload**: None

---

### Model Events

#### Model Refresh Events

- **Path**: `['sketch', 'model', 'refresh']`
  - **Description**: Refresh the entire model.
  - **Payload**: None
- **Path**: `['sketch', 'model', 'variables', 'refresh']`
  - **Description**: Refresh the model variables.
  - **Payload**: None
- **Path**: `['sketch', 'model', 'uninterpreted_fns', 'refresh']`
  - **Description**: Refresh the uninterpreted functions.
  - **Payload**: None
- **Path**: `['sketch', 'model', 'regulations', 'refresh']`
  - **Description**: Refresh the regulations.
  - **Payload**: None
- **Path**: `['sketch', 'model', 'layouts', 'refresh']`
  - **Description**: Refresh the layouts.
  - **Payload**: None

#### Model Variable Events
- **Path**: `['sketch', 'model', 'variable', 'add_default']`
  - **Description**: Add a new variable at the specified position.
  - **Payload**:
    ```json
    {
      "position": [{ "layout": "string", "px": "number", "py": "number" }]
    }
    ```
- **Path**: `['sketch', 'model', 'variable', 'remove']`
  - **Description**: Remove a variable by ID.
  - **Payload**: `{ "varId": "string" }`
- **Path**: `['sketch', 'model', 'variable', 'set_data']`
  - **Description**: Update the data of a variable by ID.
  - **Payload**:
    ```json
    {
      "varId": "string",
      "data": { "name": "string", "annotation": "string" }
    }
    ```
- **Path**: `['sketch', 'model', 'variable', 'set_update_fn']`
  - **Description**: Set the update function for a variable.
  - **Payload**: `{ "varId": "string", "expression": "string" }`

#### Model Functions Events

- **Path**: `['sketch', 'model', 'uninterpreted_fn', 'add_default']`
  - **Description**: Add a new uninterpreted function.
  - **Payload**: None
- **Path**: `['sketch', 'model', 'uninterpreted_fn', 'remove']`
  - **Description**: Remove an uninterpreted function by ID.
  - **Payload**: `{ "fnId": "string" }`
- **Path**: `['sketch', 'model', 'uninterpreted_fn', 'set_data']`
  - **Description**: Update data of an uninterpreted function.
  - **Payload**:
    ```json
    {
      "fnId": "string",
      "data": { "name": "string", "annotation": "string", "expression": "string" }
    }
    ```

#### Model Regulations Events
- **Path**: `['sketch', 'model', 'regulation', 'add']`
  - **Description**: Add a new regulation.
  - **Payload**:
    ```json
    {
      "regulator": "string",
      "target": "string",
      "sign": "string",
      "essential": "string"
    }
    ```
- **Path**: `['sketch', 'model', 'regulation', 'remove']`
  - **Description**: Remove a regulation by specifying regulator and target.
  - **Payload**:
    ```json
    {
      "regulator": "string",
      "target": "string"
    }
    ```

#### Model Layout Events
- **Path**: `['sketch', 'model', 'layout', 'add']`
  - **Description**: Add a new layout with a specified ID and name.
  - **Payload**:
    ```json
    {
      "layoutId": "string",
      "name": "string"
    }
    ```
- **Path**: `['sketch', 'model', 'layout', 'remove']`
  - **Description**: Remove a layout by ID.
  - **Payload**: `{ "layoutId": "string" }`
- **Path**: `['sketch', 'model', 'layout', 'update_position']`
  - **Description**: Update the position of a node in the layout.
  - **Payload**:
    ```json
    {
      "layoutId": "string",
      "varId": "string",
      "px": "number",
      "py": "number"
    }
    ```

---

### Observations and Datasets Events

#### Refresh Events

- **Path**: `['sketch', 'observations', 'refresh']`
  - **Description**: Refresh all datasets.
  - **Payload**: None
- **Path**: `['sketch', 'observations', 'refresh_dataset']`
  - **Description**: Refresh a single dataset by ID.
  - **Payload**: `{ "id": "string" }`
- **Path**: `['sketch', 'observations', 'refresh_observation']`
  - **Description**: Refresh a single observation in a specified dataset.
  - **Payload**: `{ "datasetId": "string", "observationId": "string" }`


#### Dataset Events
- **Path**: `['sketch', 'observations', 'add_default']`
  - **Description**: Add a new empty dataset.
  - **Payload**: None
- **Path**: `['sketch', 'observations', 'load']`
  - **Description**: Load a dataset from a CSV file.
  - **Payload**: `{ "path": "string" }`
- **Path**: `['sketch', 'observations', 'remove']`
  - **Description**: Remove a dataset by ID.
  - **Payload**: `{ "id": "string" }`
- **Path**: `['sketch', 'observations', 'set_id']`
  - **Description**: Update a dataset's ID.
  - **Payload**: `{ "originalId": "string", "newId": "string" }`
- **Path**: `['sketch', 'observations', 'set_metadata']`
  - **Description**: Update the metadata (name, annotation) of a dataset.
  - **Payload**: `{ "id": "string", "metadata": { "name": "string", "annotation": "string" } }`
- **Path**: `['sketch', 'observations', 'set_content']`
  - **Description**: Update the content (variables and observations) of a dataset.
  - **Payload**: `{ "id": "string", "content": { "variables": [...], "observations": [...] } }`
- **Path**: `['sketch', 'observations', 'add_var']`
  - **Description**: Add a placeholder variable to a dataset.
  - **Payload**: `{ "datasetId": "string" }`
- **Path**: `['sketch', 'observations', 'remove_var']`
  - **Description**: Remove a variable from a dataset by ID.
  - **Payload**: `{ "datasetId": "string", "varId": "string" }`
- **Path**: `['sketch', 'observations', 'set_var_id']`
  - **Description**: Update the ID of a variable in a dataset.
  - **Payload**: `{ "datasetId": "string", "originalId": "string", "newId": "string" }`

#### Observations Events
- **Path**: `['sketch', 'observations', 'push_obs']`
  - **Description**: Add a new observation to a dataset.
  - **Payload**: `{ "datasetId": "string", "observation": { "id": "string", "values": "string" } }`
- **Path**: `['sketch', 'observations', 'pop_obs']`
  - **Description**: Remove the last observation from a dataset.
  - **Payload**: `{ "datasetId": "string" }`
- **Path**: `['sketch', 'observations', 'remove_obs']`
  - **Description**: Remove an observation from a dataset.
  - **Payload**: `{ "datasetId": "string", "observationId": "string" }`
- **Path**: `['sketch', 'observations', 'set_obs_id']`
  - **Description**: Update the ID of an observation.
  - **Payload**: `{ "datasetId": "string", "originalId": "string", "newId": "string" }`
- **Path**: `['sketch', 'observations', 'set_obs_data']`
  - **Description**: Update the data of an observation.
  - **Payload**:
    ```json
    {
      "datasetId": "string",
      "observation": {
        "id": "string",
        "name": "string",
        "annotation": "string",
        "values": "string"
      }
    }
    ```


---

### Properties Events

#### Refresh Events
- **Path**: `['sketch', 'properties', 'dynamic', 'refresh']`
  - **Description**: Refresh all dynamic properties.
  - **Payload**: None
- **Path**: `['sketch', 'properties', 'static', 'refresh']`
  - **Description**: Refresh all static properties.
  - **Payload**: None


#### Dynamic Properties Events
- **Path**: `['sketch', 'properties', 'dynamic', 'add_default']`
  - **Description**: Add a new dynamic property of the specified variant.
  - **Payload**: `{ "variant": "string" }`
- **Path**: `['sketch', 'properties', 'dynamic', 'remove']`
  - **Description**: Remove a dynamic property by ID.
  - **Payload**: `{ "id": "string" }`
- **Path**: `['sketch', 'properties', 'dynamic', 'set_content']`
  - **Description**: Update the content of a dynamic property.
  - **Payload**:
    ```json
    {
      "id": "string",
      "newContent": {
        "type": "string",
        "name": "string",
        "value": "any"
      }
    }
    ```
- **Path**: `['sketch', 'properties', 'dynamic', 'set_id']`
  - **Description**: Update the ID of a dynamic property.
  - **Payload**: `{ "originalId": "string", "newId": "string" }`

#### Static Properties Events
- **Path**: `['sketch', 'properties', 'static', 'add_default']`
  - **Description**: Add a new static property of the specified variant.
  - **Payload**: `{ "variant": "string" }`
- **Path**: `['sketch', 'properties', 'static', 'remove']`
  - **Description**: Remove a static property by ID.
  - **Payload**: `{ "id": "string" }`
- **Path**: `['sketch', 'properties', 'static', 'set_content']`
  - **Description**: Update the content of a static property.
  - **Payload**:
    ```json
    {
      "id": "string",
      "newContent": {
        "type": "string",
        "name": "string",
        "value": "any"
      }
    }
    ```
- **Path**: `['sketch', 'properties', 'static', 'set_id']`
  - **Description**: Update the ID of a static property.
  - **Payload**: `{ "originalId": "string", "newId": "string" }`


---
## Inference Session Events

#### State Related Events
- **Path**: `['inference', 'refresh_sketch']`
  - **Description**: Request the current sketch data from the backend.
  - **Payload**: None

#### Inference Computation Events
- **Path**: `['inference', 'start_full_inference']`
  - **Description**: Start a full inference analysis.
  - **Payload**: None
- **Path**: `['inference', 'start_static_inference']`
  - **Description**: Start an inference analysis using static properties only.
  - **Payload**: None
- **Path**: `['inference', 'start_dynamic_inference']`
  - **Description**: Start an inference analysis using dynamic properties only.
  - **Payload**: None
- **Path**: `['inference', 'reset_inference']`
  - **Description**: Reset the current inference and start again using the same sketch.
  - **Payload**: None
- **Path**: `['inference', 'ping_for_results']`
  - **Description**: Check if inference results are ready.
  - **Payload**: None

#### Inference Results Events
- **Path**: `['inference', 'sample_networks']`
  - **Description**: Sample Boolean networks from the inference results.
  - **Payload**:
    ```json
    {
      "count": "number",
      "seed": "number|null",
      "path": "string"
    }
    ```
- **Path**: `['inference', 'dump_results']`
  - **Description**: Save the inference results to a specified path, including the sketch and other related data.
  - **Payload**: `{ "path": "string" }`


---
## Error Events
- **Path**: `['error']`
  - **Description**: Receive a generic error message from the backend.
  - **Payload**:
    ```json
    {
      "message": "string"
    }
    ```


---
## New Session Events
- **Path**: `['new-inference-session']`
  - **Description**: Create a new inference session.
  - **Payload**: None

---

