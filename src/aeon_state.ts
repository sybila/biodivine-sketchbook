import {
  Observable,
  ObservableState,
  MutableObservableState,
  aeonEvents
} from './aeon_events'

import {
  type Monotonicity,
  type Essentiality,
  type DynamicProperty,
  type StaticProperty,
  type DynamicPropertyType,
  type StaticPropertyType
} from './html/util/data-interfaces'

import {
  type InferenceStatusReport,
  type InferenceResults
} from './html/util/analysis-interfaces'

/** An object representing all relevant parts of the whole sketch. */
export interface SketchData {
  model: ModelData
  datasets: DatasetData[]
  dyn_properties: DynamicProperty[]
  stat_properties: StaticProperty[]
  annotation: string
}

/** An object representing all relevant parts of a model. */
export interface ModelData {
  variables: VariableData[]
  regulations: RegulationData[]
  uninterpreted_fns: UninterpretedFnData[]
  layouts: LayoutData[]
}

/** An object representing basic information regarding a model variable. */
export interface VariableData {
  id: string
  name: string
  annotation: string
  update_fn: string
}

/** An object representing basic information regarding a model's uninterpreted function. */
export interface UninterpretedFnData {
  id: string
  name: string
  annotation: string
  arguments: Array<[Monotonicity, Essentiality]>
  expression: string
}

/** An object representing basic information regarding a model regulation. */
export interface RegulationData {
  regulator: string
  target: string
  sign: Monotonicity
  essential: Essentiality
}

/** An object representing basic information regarding a model layout. */
export interface LayoutData {
  id: string
  name: string
  nodes: LayoutNodeData[]
}

/** An object representing basic information regarding a node in a layout. */
export interface LayoutNodeData {
  layout: string
  variable: string
  px: number
  py: number
}

/**
 * The same as `LayoutNodeData`, but does not have a fixed variable ID because
 * it is associated with a variable that does not have an ID yet.
 */
export interface LayoutNodeDataPrototype {
  layout: string
  px: number
  py: number
}

/** An object representing basic information regarding an observation (in a particular dataset). */
export interface ObservationData {
  id: string
  name: string
  annotation: string
  dataset: string
  values: string // string with `0`/`1`/`*`, for instance: "0001**110"
}

/** An object representing all information regarding a whole dataset. */
export interface DatasetData {
  id: string
  name: string
  annotation: string
  observations: ObservationData[]
  variables: string[]
}

/**
 * An object representing basic "metadata" information regarding a dataset.
 * Specifically, does not contain information about dataset's observations.
 * */
export interface DatasetMetaData {
  id: string
  name: string
  annotation: string
  variables: string[]
}

/** An object representing information needed for loading a dataset. */
export interface DatasetLoadData { path: string, id: string }

/** An object representing information needed for variable id change. */
export interface VariableIdUpdateData { original_id: string, new_id: string }

/** An object representing information needed for uninterpreted function's id change. */
export interface UninterpretedFnIdUpdateData { original_id: string, new_id: string }

/** An object representing information needed for observation's id change. */
export interface ObservationIdUpdateData { original_id: string, new_id: string, metadata: string }

/** An object representing information needed for dataset's id change. */
export interface DatasetIdUpdateData { original_id: string, new_id: string }

/** An object representing information needed for dynamic property's id change. */
export interface DynPropIdUpdateData { original_id: string, new_id: string }

/** An object representing information needed for static property's id change. */
export interface StatPropIdUpdateData { original_id: string, new_id: string }

/**
 * A type-safe representation of the state managed by an Aeon session.
 *
 * Under the hood, this uses `AeonEvents` to implement actions and listeners.
 */
interface AeonState {

  /** Access to the internal state of the undo-redo stack. */
  undoStack: {
    /** True if the stack has actions that can be undone. */
    canUndo: ObservableState<boolean>
    /** True if the stack has actions that can be redone. */
    canRedo: ObservableState<boolean>
    /** Try to undo an action. Emits an error if no actions can be undone. */
    undo: () => void
    /** Try to redo an action. Emits an error if no actions can be redone. */
    redo: () => void
  }

  /** The state of the main navigation tab-bar. */
  tabBar: {
    /** Integer ID of the currently active tab. */
    active: MutableObservableState<number>
    /** A *sorted* list of IDs of all pinned tabs. */
    pinned: MutableObservableState<number[]>
    /** Pins a single tab if not currently pinned. */
    pin: (id: number) => void
    /** Unpins a single tab if currently pinned. */
    unpin: (id: number) => void
  }

  sketch: {
    /** First, some general events regarding the whole sketch: */

    /** The refresh of the whole sketch instance. */
    sketchRefreshed: Observable<SketchData>
    /** Refresh the whole sketch. */
    refreshSketch: () => void

    /** Export the sketch data to a file in the custom JSON format. */
    exportSketch: (path: string) => void
    /** Export the sketch data to a file in the extended AEON format. */
    exportAeon: (path: string) => void
    /** Export the network PNG to a file. */
    exportNetworkPng: (path: string, pngBase64: string) => void
    /** Import the sketch data from a special sketch JSON file. */
    importSketch: (path: string) => void
    /** Import the sketch data from a AEON file. */
    importAeon: (path: string) => void
    /** Import model from a SBML file. */
    importSbml: (path: string) => void
    /** Set the sketch to a "default" mode, essentially emptying it and starting anew. */
    newSketch: () => void
    /** The whole replaced sketch instance (after importing or starting a new sketch). */
    sketchReplaced: Observable<SketchData>
    /** Set annotation of the whole sketch. */
    setAnnotation: (annotation: string) => void
    /** Annotation of the whole sketch was changed. */
    annotationChanged: Observable<string>
    /** Run the explicit consistency check on the sketch. */
    checkConsistency: () => void
    /** Results of an explicit consistency check (a summary message). */
    consistencyResults: Observable<string>

    /** The state of the main model. */
    model: {
      /** Refresh events: */

      /** The whole model instance. */
      modelRefreshed: Observable<ModelData>
      /** Refresh the whole model. */
      refreshModel: () => void
      /** List of model variables. */
      variablesRefreshed: Observable<VariableData[]>
      /** Refresh the variables. */
      refreshVariables: () => void
      /** List of model uninterpreted functions. */
      uninterpretedFnsRefreshed: Observable<UninterpretedFnData[]>
      /** Refresh the uninterpreted functions. */
      refreshUninterpretedFns: () => void
      /** List of model regulations. */
      regulationsRefreshed: Observable<RegulationData[]>
      /** Refresh the regulations. */
      refreshRegulations: () => void
      /** List of model layouts. */
      layoutsRefreshed: Observable<LayoutData[]>
      /** Refresh the layouts. */
      refreshLayouts: () => void
      /** List of nodes in a given layout. */
      layoutNodesRefreshed: Observable<LayoutNodeData[]>
      /** Refresh the nodes in a given layout. */
      refreshLayoutNodes: (layoutId: string) => void

      /** Variable-related setter events: */

      /** VariableData for a newly created variable. */
      variableCreated: Observable<VariableData>
      /** Create new variable with automatically generated ID and name.
       * Variable is placed on a given position. */
      addDefaultVariable: (position?: LayoutNodeDataPrototype | LayoutNodeDataPrototype[]) => void
      /** VariableData of a removed variable. */
      variableRemoved: Observable<VariableData>
      /** Remove a variable with given ID. */
      removeVariable: (varId: string) => void
      /** VariableData (with updated name or annotation) for a modified variable. */
      variableDataChanged: Observable<VariableData>
      /** Set a variable data for a var with given ID. */
      setVariableData: (varId: string, variableData: VariableData) => void
      /** ModelData after variable's ID is changed.
       * Since variable ID change can affect many parts of the model (update fns, regulations, ...), we
       * get the whole model data at once. */
      variableIdChanged: Observable<ModelData>
      /** Set an ID of variable with given original ID to a new id. */
      setVariableId: (originalId: string, newId: string) => void
      /** VariableData (with updated `update_fn`) for a variable with modified update function. */
      variableUpdateFnChanged: Observable<VariableData>
      /** Set an expression of update function for variable with given ID. */
      setVariableUpdateFn: (varId: string, newExpression: string) => void

      /** Uninterpreted function-related setter events: */

      /** UninterpretedFnData for a newly created uninterpreted function. */
      uninterpretedFnCreated: Observable<UninterpretedFnData>
      /** Create new uninterpreted function with automatically generated ID and name, and with
       * zero arity & no additional constraints. */
      addDefaultUninterpretedFn: () => void
      /** UninterpretedFnData of a removed uninterpreted function. */
      uninterpretedFnRemoved: Observable<UninterpretedFnData>
      /** Remove uninterpreted function with given ID. */
      removeUninterpretedFn: (uninterpretedFnId: string) => void
      /** UninterpretedFnData (with updated name or annotation) for a modified uninterpreted function. */
      uninterpretedFnDataChanged: Observable<UninterpretedFnData>
      /** Set data of uninterpreted function with given ID. */
      setUninterpretedFnData: (uninterpretedFnId: string, fnData: UninterpretedFnData) => void
      /** UninterpretedFnData (with updated `arity`) for a modified uninterpreted function. */
      uninterpretedFnArityChanged: Observable<UninterpretedFnData>
      /** Set arity of uninterpreted function with given ID. */
      setUninterpretedFnArity: (uninterpretedFnId: string, newArity: number) => void
      /** UninterpretedFnData (with incremented `arity`) for a modified uninterpreted function. */
      uninterpretedFnArityIncremented: Observable<UninterpretedFnData>
      /** Increment arity of uninterpreted function with given ID. */
      incrementUninterpretedFnArity: (uninterpretedFnId: string) => void
      /** UninterpretedFnData (with decremented `arity`) for a modified uninterpreted function. */
      uninterpretedFnArityDecremented: Observable<UninterpretedFnData>
      /** Decrement arity of uninterpreted function with given ID. */
      decrementUninterpretedFnArity: (uninterpretedFnId: string) => void
      /** ModelData after function's ID is changed.
       * Since function ID change can affect many parts of the model (update fns, other uninterpreted fns, ...), we
       * get the whole model data at once. */
      uninterpretedFnIdChanged: Observable<ModelData>
      /** Set an ID of an uninterpreted function with given original ID to a new id. */
      setUninterpretedFnId: (originalId: string, newId: string) => void
      /** UninterpretedFnData (with updated `expression`) for a modified uninterpreted function. */
      uninterpretedFnExpressionChanged: Observable<UninterpretedFnData>
      /** Set an expression of uninterpreted function with given ID. */
      setUninterpretedFnExpression: (uninterpretedFnId: string, newExpression: string) => void
      /** UninterpretedFnData (with updated `monotonicity` for one of the args) for a modified uninterpreted function. */
      uninterpretedFnMonotonicityChanged: Observable<UninterpretedFnData>
      /** Set a monotonicity of uninterpreted function with given ID. */
      setUninterpretedFnMonotonicity: (uninterpretedFnId: string, idx: number, monotonicity: Monotonicity) => void
      /** UninterpretedFnData (with updated `essentiality` for one of the args) for a modified uninterpreted function. */
      uninterpretedFnEssentialityChanged: Observable<UninterpretedFnData>
      /** Set an essentiality of uninterpreted function with given ID. */
      setUninterpretedFnEssentiality: (uninterpretedFnId: string, idx: number, essentiality: Essentiality) => void

      /** Regulation-related setter events: */

      /** RegulationData for a newly created regulation. */
      regulationCreated: Observable<RegulationData>
      /** Create new regulation specified by all of its four components. */
      addRegulation: (regulatorId: string, targetId: string, sign: string, essential: string) => void
      /** RegulationData of a removed regulation. */
      regulationRemoved: Observable<RegulationData>
      /** Create regulation specified by its regulator and target. */
      removeRegulation: (regulatorId: string, targetId: string) => void
      /** RegulationData (with updated `sign`) of a modified regulation. */
      regulationSignChanged: Observable<RegulationData>
      /** Set sign of a regulation specified by its regulator and target. */
      setRegulationSign: (regulatorId: string, targetId: string, newSign: Monotonicity) => void
      /** RegulationData (with updated `essentiality`) of a modified regulation. */
      regulationEssentialityChanged: Observable<RegulationData>
      /** Set essentiality of a regulation specified by its regulator and target. */
      setRegulationEssentiality: (regulatorId: string, targetId: string, newEssentiality: Essentiality) => void

      /** Layout-related setter events: */

      /** LayoutData for a newly created layout. */
      layoutCreated: Observable<LayoutData>
      /** Create new layout with given ID and name. */
      addLayout: (layoutId: string, layoutName: string) => void
      /** LayoutData of a removed layout. */
      layoutRemoved: Observable<LayoutData>
      /** Remove a layout with given ID. */
      removeLayout: (layoutId: string) => void
      /** LayoutNodeData (with new `px` and `py`) for a modified layout node. */
      nodePositionChanged: Observable<LayoutNodeData>
      /** Change a position of a variable in a layout to new coordinates. */
      changeNodePosition: (layoutId: string, varId: string, newX: number, newY: number) => void
    }

    /** The state of the observations and datasets. */
    observations: {
      /** Refresh events: */

      /** List of all datasets. */
      datasetsRefreshed: Observable<DatasetData[]>
      /** Refresh all the datasets. */
      refreshDatasets: () => void
      /** A particular dataset. */
      singleDatasetRefreshed: Observable<DatasetData>
      /** Refresh a single dataset with a given ID. */
      refreshSingleDataset: (id: string) => void
      /** A particular refreshed observation. */
      observationRefreshed: Observable<ObservationData>
      /** Refresh a single observation from a specified dataset. */
      refreshSingleObservation: (datasetId: string, observationId: string) => void

      /** Events to edit datasets or observations: */

      /** DatasetData for a newly created dataset. */
      datasetCreated: Observable<DatasetData>
      /** Create a new empty dataset. */
      addDefaultDataset: () => void
      /** DatasetData for a newly loaded dataset (from a csv file).
       *  This is intentionally different than `datasetCreated`, since loaded datasets might require some processing. */
      datasetLoaded: Observable<DatasetData>
      /** Load a new dataset from a CSV file. */
      loadDataset: (path: string) => void
      /** DatasetData of a removed dataset. */
      datasetRemoved: Observable<DatasetData>
      /** Remove dataset with given ID. */
      removeDataset: (id: string) => void
      /** Object with `original_id` of a dataset and its `new_id`. */
      datasetIdChanged: Observable<DatasetIdUpdateData>
      /** Set ID of dataset with given original ID to a new id. */
      setDatasetId: (originalId: string, newId: string) => void
      /** DatasetMetaData of a modified dataset (could be modified name or annotation). */
      datasetMetadataChanged: Observable<DatasetMetaData>
      /** Set metadata (could be name or annotation) of a dataset with given ID. */
      setDatasetMetadata: (id: string, metadata: DatasetMetaData) => void
      /** DatasetData of a fully modified dataset. */
      datasetContentChanged: Observable<DatasetData>
      /** Set content (variables, observations - everything) of dataset with given ID. */
      setDatasetContent: (id: string, newContent: DatasetData) => void
      /** DatasetMetaData (with updated `variables`) of a modified dataset. */
      datasetVariableChanged: Observable<DatasetMetaData>
      /** Set variable's ID within a specified dataset. */
      setDatasetVariable: (datasetId: string, originalId: string, newId: string) => void
      /** DatasetData (with updated both `variables` and `observations`) of a modified dataset. */
      datasetVariableRemoved: Observable<DatasetData>
      /** Remove variable from a specified dataset (removing "a column" of a dataset's table). */
      removeDatasetVariable: (datasetId: string, varId: string) => void
      /** DatasetData (with updated both `variables` and `observations`) of a modified dataset. */
      datasetVariableAdded: Observable<DatasetData>
      /** Add (placeholder) variable to a specified dataset (adding an empty column to a dataset's table). */
      addDatasetVariable: (datasetId: string) => void
      /** Export dataset with given ID to a given file. */
      exportDataset: (id: string, path: string) => void

      /** ObservationData for a newly pushed observation (also contains corresponding dataset ID). */
      observationPushed: Observable<ObservationData>
      /** Push a new observation into a specified dataset. If observation data are not provided,
       * the observation is newly generated on backend (with unspecified values). */
      pushObservation: (datasetId: string, observation?: ObservationData) => void
      /** ObservationData for a popped (removed from the end) observation (also contains corresponding dataset ID). */
      observationPopped: Observable<ObservationData>
      /** Pop (remove) observation from the end of a specified dataset. */
      popObservation: (datasetId: string) => void
      /** ObservationData for a removed observation (also contains corresponding dataset ID). */
      observationRemoved: Observable<ObservationData>
      /** Remove any observation from a specified dataset. */
      removeObservation: (datasetId: string, observationId: string) => void
      /** Object with `original_id` of a observation, its `new_id`. Dataset's ID is in the field `metadata`. */
      observationIdChanged: Observable<ObservationIdUpdateData>
      /** Set ID of observation (in a specified dataset) with given original ID to a new id. */
      setObservationId: (datasetId: string, originalId: string, newId: string) => void
      /** ObservationData for a modified observation. */
      observationDataChanged: Observable<ObservationData>
      /** Modify a particular observation (could be name, annotations, values).  */
      setObservationData: (datasetId: string, observation: ObservationData) => void
    }

    /** The state of the dynamic and static properties. */
    properties: {
      /** Refresh events: */

      /** List of all dynamic properties. */
      dynamicPropsRefreshed: Observable<DynamicProperty[]>
      /** Refresh all dynamic properties. */
      refreshDynamicProps: () => void
      /** List of all static properties. */
      staticPropsRefreshed: Observable<StaticProperty[]>
      /** Refresh all static properties. */
      refreshStaticProps: () => void

      /** Events regarding dynamic properties. */

      /** Newly created dynamic property. */
      dynamicCreated: Observable<DynamicProperty>
      /** Create a new default dynamic property of given variant. */
      addDefaultDynamic: (variant: DynamicPropertyType) => void
      /** Data of a removed dynamic property. */
      dynamicRemoved: Observable<DynamicProperty>
      /** Remove dynamic property with given ID. */
      removeDynamic: (id: string) => void
      /** Data of a modified dynamic property. */
      dynamicContentChanged: Observable<DynamicProperty>
      /** Set content of dynamic property with given ID. */
      setDynamicContent: (id: string, newContent: DynamicProperty) => void
      /** Object with `original_id` of a dynamic prop and its `new_id`. */
      dynamicIdChanged: Observable<DynPropIdUpdateData>
      /** Set ID of dynamic property with given original ID to a new id. */
      setDynamicId: (originalId: string, newId: string) => void

      /** Events regarding static properties. */

      /** Newly created static property. */
      staticCreated: Observable<StaticProperty>
      /** Create a new default static property with given ID and variant. */
      addDefaultStatic: (variant: StaticPropertyType) => void
      /** Data of a removed static property. */
      staticRemoved: Observable<StaticProperty>
      /** Remove static property with given ID. */
      removeStatic: (id: string) => void
      /** Data of a modified static property. */
      staticContentChanged: Observable<StaticProperty>
      /** Set content of static property with given ID. */
      setStaticContent: (id: string, newContent: StaticProperty) => void
      /** Object with `original_id` of a static prop and its `new_id`. */
      staticIdChanged: Observable<StatPropIdUpdateData>
      /** Set ID of static property with given original ID to a new id. */
      setStaticId: (originalId: string, newId: string) => void
      /** List of all `StaticProperty` after variable's ID is changed.
       * Since var ID change can affect any property, we "refresh" all data at once. */
      allStaticUpdated: Observable<StaticProperty[]>
    }
  }

  /** The events regarding the inference analysis. */
  analysis: {
    /** State related events. */

    /** Sketch data transfered from backend (useful to initiate the state). */
    sketchRefreshed: Observable<SketchData>
    /** Ask for a sketch data (might be useful to update inference analysis window, or
     * if automatic sketch transfer from backend does not work). */
    refreshSketch: () => void

    /** Inference computation related events. */

    /** Start the full inference. */
    startFullInference: () => void
    /** Start the inference with static properties only. */
    startStaticInference: () => void
    /** Start the inference with dynamic properties only. */
    startDynamicInference: () => void
    /** Information that async inference was successfully started. */
    inferenceStarted: Observable<boolean>
    /** Fully reset the inference and start again. The same sketch will be used. */
    resetInference: () => void
    /** Information that inference was reset. */
    inferenceReset: Observable<boolean>
    /** Ping backend to see if the results are ready. Can be used regardless of
     * what inference type is running. */
    pingForInferenceResults: () => void
    /** Update message from the inference solver. Can be multi-line. */
    computationUpdated: Observable<InferenceStatusReport[]>
    /** Error message from the inference solver. */
    computationErrorReceived: Observable<string>

    /** Inference results related events. */

    /** Inference results. */
    inferenceResultsReceived: Observable<InferenceResults>
    /** Sample given number of Boolean networks from the results, either dereministically
     * or randomly. The networks are saved in a zip archive at given path. */
    sampleNetworks: (count: number, seed: number | null, path: string) => void
    /** Dump archive with results (including the sketch, the converted aeon BN used for inference, and
     * a BDD with all satisfying colors) to the given path. */
    dumpFullResults: (path: string) => void
  }

  /** The information about errors occurring when processing events on backend. */
  error: {
    /** Generic error, with a message provided by backend. */
    errorReceived: Observable<string>
  }

  /** Events for creating new sessions. */
  new_session: {
    /** Create a new inference session. */
    createNewInferenceSession: () => void
  }
}
/**
 * A singleton state management object for the current Aeon session.
 */
export const aeonState: AeonState = {
  undoStack: {
    canUndo: new ObservableState<boolean>(['undo_stack', 'can_undo'], false),
    canRedo: new ObservableState<boolean>(['undo_stack', 'can_redo'], false),
    undo () {
      aeonEvents.emitAction({
        path: ['undo_stack', 'undo'],
        payload: null
      })
    },
    redo () {
      aeonEvents.emitAction({
        path: ['undo_stack', 'redo'],
        payload: null
      })
    }
  },
  tabBar: {
    active: new MutableObservableState<number>(['tab_bar', 'active'], 0),
    pinned: new MutableObservableState<number[]>(['tab_bar', 'pinned'], []),
    pin (id: number): void {
      const value = this.pinned.value()
      if (!value.includes(id)) {
        value.push(id)
        value.sort((a, b) => a - b)
        this.pinned.emitValue(value)
      }
    },
    unpin (id: number): void {
      const value = this.pinned.value()
      const index = value.indexOf(id)
      if (index > -1) {
        value.splice(index, 1)
        this.pinned.emitValue(value)
      }
    }
  },
  error: {
    errorReceived: new Observable<string>(['error'])
  },
  new_session: {
    createNewInferenceSession (): void {
      aeonEvents.emitAction({
        path: ['new-inference-session'],
        payload: null
      })
    }
  },
  sketch: {
    sketchRefreshed: new Observable<SketchData>(['sketch', 'get_whole_sketch']),
    consistencyResults: new Observable<string>(['sketch', 'consistency_results']),
    sketchReplaced: new Observable<SketchData>(['sketch', 'set_all']),
    annotationChanged: new Observable<string>(['sketch', 'set_annotation']),

    refreshSketch (): void {
      aeonEvents.refresh(['sketch', 'get_whole_sketch'])
    },
    exportSketch (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'export_sketch'],
        payload: path
      })
    },
    exportAeon (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'export_aeon'],
        payload: path
      })
    },
    exportNetworkPng (path: string, pngBase64: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'export_png'],
        payload: JSON.stringify({ path, png: pngBase64 })
      })
    },
    importSketch (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'import_sketch'],
        payload: path
      })
    },
    importAeon (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'import_aeon'],
        payload: path
      })
    },
    importSbml (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'import_sbml'],
        payload: path
      })
    },
    newSketch (): void {
      aeonEvents.emitAction({
        path: ['sketch', 'new_sketch'],
        payload: null
      })
    },
    checkConsistency (): void {
      aeonEvents.emitAction({
        path: ['sketch', 'check_consistency'],
        payload: null
      })
    },
    setAnnotation (annotation: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'set_annotation'],
        payload: annotation
      })
    },

    model: {
      modelRefreshed: new Observable<ModelData>(['sketch', 'model', 'get_whole_model']),
      refreshModel (): void {
        aeonEvents.refresh(['sketch', 'model', 'get_whole_model'])
      },
      variablesRefreshed: new Observable<VariableData[]>(['sketch', 'model', 'get_variables']),
      refreshVariables (): void {
        aeonEvents.refresh(['sketch', 'model', 'get_variables'])
      },
      uninterpretedFnsRefreshed: new Observable<UninterpretedFnData[]>(['sketch', 'model', 'get_uninterpreted_fns']),
      refreshUninterpretedFns (): void {
        aeonEvents.refresh(['sketch', 'model', 'get_uninterpreted_fns'])
      },
      regulationsRefreshed: new Observable<RegulationData[]>(['sketch', 'model', 'get_regulations']),
      refreshRegulations (): void {
        aeonEvents.refresh(['sketch', 'model', 'get_regulations'])
      },
      layoutsRefreshed: new Observable<LayoutData[]>(['sketch', 'model', 'get_layouts']),
      refreshLayouts (): void {
        aeonEvents.refresh(['sketch', 'model', 'get_layouts'])
      },
      layoutNodesRefreshed: new Observable<LayoutNodeData[]>(['sketch', 'model', 'get_layout_nodes']),
      refreshLayoutNodes (layoutId: string): void {
        aeonEvents.refresh(['sketch', 'model', 'get_layout_nodes', layoutId])
      },

      variableCreated: new Observable<VariableData>(['sketch', 'model', 'variable', 'add']),
      variableRemoved: new Observable<VariableData>(['sketch', 'model', 'variable', 'remove']),
      variableDataChanged: new Observable<VariableData>(['sketch', 'model', 'variable', 'set_data']),
      variableIdChanged: new Observable<ModelData>(['sketch', 'model', 'variable', 'set_id']),
      variableUpdateFnChanged: new Observable<VariableData>(['sketch', 'model', 'variable', 'set_update_fn']),

      uninterpretedFnCreated: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'add']),
      uninterpretedFnRemoved: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'remove']),
      uninterpretedFnDataChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_data']),
      uninterpretedFnArityChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_arity']),
      uninterpretedFnArityIncremented: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'increment_arity']),
      uninterpretedFnArityDecremented: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'decrement_arity']),
      uninterpretedFnIdChanged: new Observable<ModelData>(['sketch', 'model', 'uninterpreted_fn', 'set_id']),
      uninterpretedFnExpressionChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_expression']),
      uninterpretedFnMonotonicityChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_monotonicity']),
      uninterpretedFnEssentialityChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_essentiality']),

      regulationCreated: new Observable<RegulationData>(['sketch', 'model', 'regulation', 'add']),
      regulationRemoved: new Observable<RegulationData>(['sketch', 'model', 'regulation', 'remove']),
      regulationSignChanged: new Observable<RegulationData>(['sketch', 'model', 'regulation', 'set_sign']),
      regulationEssentialityChanged: new Observable<RegulationData>(['sketch', 'model', 'regulation', 'set_essentiality']),

      layoutCreated: new Observable<LayoutData>(['sketch', 'model', 'layout', 'add']),
      layoutRemoved: new Observable<LayoutData>(['sketch', 'model', 'layout', 'remove']),
      nodePositionChanged: new Observable<LayoutNodeData>(['sketch', 'model', 'layout', 'update_position']),

      addDefaultVariable (position: LayoutNodeDataPrototype | LayoutNodeDataPrototype[] = []): void {
        if (!Array.isArray(position)) {
          position = [position]
        }

        // First action creates the variable, either default or given.
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', 'add_default'],
          payload: JSON.stringify(position)
        })
      },
      removeVariable (varId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', varId, 'remove'],
          payload: null
        })
      },
      setVariableData (varId: string, variableData: VariableData): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', varId, 'set_data'],
          payload: JSON.stringify(variableData)
        })
      },
      setVariableId (originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', originalId, 'set_id'],
          payload: newId
        })
      },
      setVariableUpdateFn (varId: string, newExpression: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', varId, 'set_update_fn'],
          payload: newExpression
        })
      },
      addDefaultUninterpretedFn (): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', 'add_default'],
          payload: null
        })
      },
      removeUninterpretedFn (uninterpretedFnId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'remove'],
          payload: null
        })
      },
      setUninterpretedFnData (uninterpretedFnId: string, fnData: UninterpretedFnData): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'set_data'],
          payload: JSON.stringify(fnData)
        })
      },
      setUninterpretedFnArity (uninterpretedFnId: string, newArity: number): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'set_arity'],
          payload: newArity.toString()
        })
      },
      incrementUninterpretedFnArity (uninterpretedFnId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'increment_arity'],
          payload: null
        })
      },
      decrementUninterpretedFnArity (uninterpretedFnId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'decrement_arity'],
          payload: null
        })
      },
      setUninterpretedFnId (originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', originalId, 'set_id'],
          payload: newId
        })
      },
      setUninterpretedFnExpression (uninterpretedFnId: string, newExpression: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'set_expression'],
          payload: newExpression.toString()
        })
      },
      setUninterpretedFnMonotonicity (uninterpretedFnId: string, idx: number, monotonicity: Monotonicity): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'set_monotonicity'],
          payload: JSON.stringify({ idx, monotonicity })
        })
      },
      setUninterpretedFnEssentiality (uninterpretedFnId: string, idx: number, essentiality: Essentiality): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'set_essentiality'],
          payload: JSON.stringify({ idx, essentiality })
        })
      },
      addRegulation (regulatorId: string, targetId: string, sign: string, essential: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'regulation', 'add'],
          payload: JSON.stringify({
            regulator: regulatorId,
            target: targetId,
            sign,
            essential
          })
        })
      },
      removeRegulation (regulatorId: string, targetId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'regulation', regulatorId, targetId, 'remove'],
          payload: null
        })
      },
      setRegulationSign (regulatorId: string, targetId: string, newSign: Monotonicity): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'regulation', regulatorId, targetId, 'set_sign'],
          payload: JSON.stringify(newSign)
        })
      },
      setRegulationEssentiality (regulatorId: string, targetId: string, newEssentiality: Essentiality): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'regulation', regulatorId, targetId, 'set_essentiality'],
          payload: JSON.stringify(newEssentiality)
        })
      },
      addLayout (layoutId: string, layoutName: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'layout', 'add'],
          payload: JSON.stringify({ id: layoutId, name: layoutName })
        })
      },
      removeLayout (layoutId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'layout', layoutId, 'remove'],
          payload: null
        })
      },
      changeNodePosition (layoutId: string, varId: string, newX: number, newY: number): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'layout', layoutId, 'update_position'],
          payload: JSON.stringify({ layout: layoutId, variable: varId, px: newX, py: newY })
        })
      }
    },
    observations: {
      datasetsRefreshed: new Observable<DatasetData[]>(['sketch', 'observations', 'get_all_datasets']),
      refreshDatasets (): void {
        aeonEvents.refresh(['sketch', 'observations', 'get_all_datasets'])
      },
      singleDatasetRefreshed: new Observable<DatasetData>(['sketch', 'observations', 'get_dataset']),
      refreshSingleDataset (id: string): void {
        aeonEvents.refresh(['sketch', 'observations', 'get_dataset', id])
      },
      observationRefreshed: new Observable<ObservationData>(['observations', 'get_observation']),
      refreshSingleObservation (datasetId: string, observationId: string): void {
        aeonEvents.refresh(['sketch', 'observations', 'get_observation', datasetId, observationId])
      },

      datasetCreated: new Observable<DatasetData>(['sketch', 'observations', 'add']),
      datasetLoaded: new Observable<DatasetData>(['sketch', 'observations', 'load']),
      datasetRemoved: new Observable<DatasetData>(['sketch', 'observations', 'remove']),
      datasetIdChanged: new Observable<DatasetIdUpdateData>(['sketch', 'observations', 'set_id']),
      datasetContentChanged: new Observable<DatasetData>(['sketch', 'observations', 'set_content']),
      datasetMetadataChanged: new Observable<DatasetMetaData>(['sketch', 'observations', 'set_metadata']),
      datasetVariableChanged: new Observable<DatasetMetaData>(['sketch', 'observations', 'set_var_id']),
      datasetVariableRemoved: new Observable<DatasetData>(['sketch', 'observations', 'remove_var']),
      datasetVariableAdded: new Observable<DatasetData>(['sketch', 'observations', 'add_var']),

      observationPushed: new Observable<ObservationData>(['sketch', 'observations', 'push_obs']),
      observationPopped: new Observable<ObservationData>(['sketch', 'observations', 'pop_obs']),
      observationRemoved: new Observable<ObservationData>(['sketch', 'observations', 'remove_obs']),
      observationIdChanged: new Observable<ObservationIdUpdateData>(['sketch', 'observations', 'set_obs_id']),
      observationDataChanged: new Observable<ObservationData>(['sketch', 'observations', 'set_obs_data']),

      addDefaultDataset (): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', 'add_default'],
          payload: null
        })
      },
      loadDataset (path: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', 'load'],
          payload: path
        })
      },
      removeDataset (id: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', id, 'remove'],
          payload: null
        })
      },
      setDatasetId (originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', originalId, 'set_id'],
          payload: newId
        })
      },
      setDatasetContent (id: string, newContent: DatasetData): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', id, 'set_content'],
          payload: JSON.stringify(newContent)
        })
      },
      setDatasetMetadata (id: string, metadata: DatasetMetaData): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', id, 'set_metadata'],
          payload: JSON.stringify(metadata)
        })
      },
      setDatasetVariable (datasetId: string, originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, 'set_variable'],
          payload: JSON.stringify({ original_id: originalId, new_id: newId })
        })
      },
      removeDatasetVariable (datasetId: string, varId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, 'remove_var'],
          payload: varId
        })
      },
      addDatasetVariable (datasetId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, 'add_var'],
          payload: null
        })
      },
      exportDataset (id: string, path: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', id, 'export'],
          payload: path
        })
      },
      pushObservation (datasetId: string, observation?: ObservationData): void {
        if (observation === undefined) {
          aeonEvents.emitAction({
            path: ['sketch', 'observations', datasetId, 'push_empty_obs'],
            payload: null
          })
        } else {
          aeonEvents.emitAction({
            path: ['sketch', 'observations', datasetId, 'push_obs'],
            payload: JSON.stringify(observation)
          })
        }
      },
      popObservation (datasetId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, 'pop_obs'],
          payload: null
        })
      },
      removeObservation (datasetId: string, observationId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, observationId, 'remove'],
          payload: null
        })
      },
      setObservationId (datasetId: string, originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, originalId, 'set_id'],
          payload: newId
        })
      },
      setObservationData (datasetId: string, observation: ObservationData): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, observation.id, 'set_data'],
          payload: JSON.stringify(observation)
        })
      }
    },
    properties: {
      dynamicPropsRefreshed: new Observable<DynamicProperty[]>(['sketch', 'properties', 'get_all_dynamic']),
      refreshDynamicProps (): void {
        aeonEvents.refresh(['sketch', 'properties', 'get_all_dynamic'])
      },
      staticPropsRefreshed: new Observable<StaticProperty[]>(['sketch', 'properties', 'get_all_static']),
      refreshStaticProps (): void {
        aeonEvents.refresh(['sketch', 'properties', 'get_all_static'])
      },

      dynamicCreated: new Observable<DynamicProperty>(['sketch', 'properties', 'dynamic', 'add']),
      dynamicContentChanged: new Observable<DynamicProperty>(['sketch', 'properties', 'dynamic', 'set_content']),
      dynamicRemoved: new Observable<DynamicProperty>(['sketch', 'properties', 'dynamic', 'remove']),
      dynamicIdChanged: new Observable<DynPropIdUpdateData>(['sketch', 'properties', 'dynamic', 'set_id']),
      staticCreated: new Observable<StaticProperty>(['sketch', 'properties', 'static', 'add']),
      staticContentChanged: new Observable<StaticProperty>(['sketch', 'properties', 'static', 'set_content']),
      staticRemoved: new Observable<StaticProperty>(['sketch', 'properties', 'static', 'remove']),
      staticIdChanged: new Observable<StatPropIdUpdateData>(['sketch', 'properties', 'static', 'set_id']),
      allStaticUpdated: new Observable<StaticProperty[]>(['sketch', 'properties', 'static', 'set_var_id_everywhere']),

      addDefaultDynamic (variant: DynamicPropertyType): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'dynamic', 'add_default'],
          payload: JSON.stringify(variant)
        })
      },
      setDynamicContent (id: string, newContent: DynamicProperty): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'dynamic', id, 'set_content'],
          payload: JSON.stringify(newContent)
        })
      },
      removeDynamic (id: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'dynamic', id, 'remove'],
          payload: null
        })
      },
      setDynamicId (originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'dynamic', originalId, 'set_id'],
          payload: newId
        })
      },
      addDefaultStatic (variant: StaticPropertyType): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'static', 'add_default'],
          payload: JSON.stringify(variant)
        })
      },
      setStaticContent (id: string, newContent: StaticProperty): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'static', id, 'set_content'],
          payload: JSON.stringify(newContent)
        })
      },
      removeStatic (id: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'static', id, 'remove'],
          payload: null
        })
      },
      setStaticId (originalId: string, newId: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'static', originalId, 'set_id'],
          payload: newId
        })
      }
    }
  },
  analysis: {
    sketchRefreshed: new Observable<SketchData>(['inference', 'get_sketch']),
    inferenceReset: new Observable<boolean>(['inference', 'inference_reset']),

    refreshSketch (): void {
      aeonEvents.refresh(['inference', 'get_sketch'])
    },
    resetInference () {
      aeonEvents.emitAction({
        path: ['inference', 'reset_inference'],
        payload: null
      })
    },
    sampleNetworks (count: number, seed: number | null, path: string): void {
      aeonEvents.emitAction({
        path: ['inference', 'sample_networks'],
        payload: JSON.stringify({ count, seed, path })
      })
    },
    dumpFullResults (path: string): void {
      aeonEvents.emitAction({
        path: ['inference', 'dump_full_results'],
        payload: path
      })
    },

    inferenceResultsReceived: new Observable<InferenceResults>(['inference', 'inference_results']),
    inferenceStarted: new Observable<boolean>(['inference', 'inference_running']),
    computationUpdated: new Observable<InferenceStatusReport[]>(['inference', 'computation_update']),
    computationErrorReceived: new Observable<string>(['inference', 'inference_error']),

    startFullInference (): void {
      aeonEvents.emitAction({
        path: ['inference', 'run_full_inference'],
        payload: null
      })
    },
    startStaticInference (): void {
      aeonEvents.emitAction({
        path: ['inference', 'run_static_inference'],
        payload: null
      })
    },
    startDynamicInference (): void {
      aeonEvents.emitAction({
        path: ['inference', 'run_dynamic_inference'],
        payload: null
      })
    },
    pingForInferenceResults (): void {
      aeonEvents.emitAction({
        path: ['inference', 'get_inference_results'],
        payload: null
      })
    }
  }
}
