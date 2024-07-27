import { type Event, emit, listen } from '@tauri-apps/api/event'
import { dialog, invoke } from '@tauri-apps/api'
import {
  Monotonicity,
  Essentiality,
  DataCategory,
  type DynamicProperty,
  type StaticProperty,
  type DynamicPropertyType,
  type StaticPropertyType
} from './html/util/data-interfaces'

/* Names of relevant events that communicate with the Tauri backend. */

const AEON_ACTION = 'aeon-action'
const AEON_VALUE = 'aeon-value'
const AEON_REFRESH = 'aeon-refresh'

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

    /** The whole sketch instance. */
    sketchRefreshed: Observable<SketchData>
    /** Refresh the whole sketch. */
    refreshSketch: () => void

    /** Export the sketch data to a file. */
    exportSketch: (path: string) => void
    /** Import the sketch data from a file. */
    importSketch: (path: string) => void
    /** Set the sketch to a "default" mode, essentially emptying it and starting anew. */
    newSketch: () => void
    /** The whole modified sketch instance (after importing or starting a new sketch). */
    sketchReplaced: Observable<SketchData>

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
      /** Create new variable with given ID and name.
       * If only ID string is given, it is used for both ID and name.
       * Variable is placed on a given position. */
      addVariable: (varId: string, varName?: string, position?: LayoutNodeDataPrototype | LayoutNodeDataPrototype[]) => void
      /** Create new variable with automatically generated ID and name.
       * Variable is placed on a given position. */
      addDefaultVariable: (position?: LayoutNodeDataPrototype | LayoutNodeDataPrototype[]) => void
      /** VariableData of a removed variable. */
      variableRemoved: Observable<VariableData>
      /** Remove a variable with given ID. */
      removeVariable: (varId: string) => void
      /** VariableData (with updated `name`) for a renamed variable. */
      variableNameChanged: Observable<VariableData>
      /** Set a name of variable with given ID to a given new name. */
      setVariableName: (varId: string, newName: string) => void
      /** Object with `original_id` of a variable and its `new_id`. */
      variableIdChanged: Observable<VariableIdUpdateData>
      /** Set an ID of variable with given original ID to a new id. */
      setVariableId: (originalId: string, newId: string) => void
      /** VariableData (with updated `update_fn`) for a variable with modified update function. */
      variableUpdateFnChanged: Observable<VariableData>
      /** Set an expression of update function for variable with given ID. */
      setVariableUpdateFn: (varId: string, newExpression: string) => void

      /** Uninterpreted function-related setter events: */

      /** UninterpretedFnData for a newly created uninterpreted function. */
      uninterpretedFnCreated: Observable<UninterpretedFnData>
      /** Create new uninterpreted function with given arity, ID, and name.
       * If name is not given, ID string is used for both ID and name. */
      addUninterpretedFn: (uninterpretedFnId: string, arity: number, uninterpretedFnName?: string) => void
      /** Create new uninterpreted function with automatically generated ID and name, and with zero arity & no
       * additional constraints. */
      addDefaultUninterpretedFn: () => void
      /** UninterpretedFnData of a removed uninterpreted function. */
      uninterpretedFnRemoved: Observable<UninterpretedFnData>
      /** Remove uninterpreted function with given ID. */
      removeUninterpretedFn: (uninterpretedFnId: string) => void
      /** UninterpretedFnData (with updated `name`) for a renamed uninterpreted function. */
      uninterpretedFnNameChanged: Observable<UninterpretedFnData>
      /** Set name of uninterpreted function with given ID. */
      setUninterpretedFnName: (uninterpretedFnId: string, newName: string) => void
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
      /** Object with `original_id` of an uninterpreted function and its `new_id`. */
      uninterpretedFnIdChanged: Observable<UninterpretedFnIdUpdateData>
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
      /** Create a new dataset with given ID, variables, observations, and category. */
      addDataset: (id: string, variables: string[], observations: ObservationData[], category: DataCategory) => void
      /** Create a new empty dataset of unspecified category. */
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
      /** DatasetData of a fully modified dataset. */
      datasetContentChanged: Observable<DatasetData>
      /** Set content (variables, observations - everything) of dataset with given ID. */
      setDatasetContent: (id: string, newContent: DatasetData) => void
      /** DatasetMetaData (with updated `category`) of a modified dataset. */
      datasetCategoryChanged: Observable<DatasetMetaData>
      /** Set category of dataset with given ID. */
      setDatasetCategory: (id: string, newCategory: DataCategory) => void
      /** DatasetMetaData (with updated `variables`) of a modified dataset. */
      datasetVariableChanged: Observable<DatasetMetaData>
      /** Set variable's ID within a specified dataset. */
      setDatasetVariable: (datasetId: string, originalId: string, newId: string) => void
      /** DatasetData (with updated both `variables` and `observations`) of a modified dataset. */
      datasetVariableRemoved: Observable<DatasetMetaData>
      /** Remove variable from a specified dataset (removing "a column" of a dataset's table). */
      removeDatasetVariable: (datasetId: string, varId: string) => void

      /** ObservationData for a newly pushed observation (also contains corresponding dataset ID). */
      observationPushed: Observable<ObservationData>
      /** Push a new observation with into a specified dataset. If observation data are not provided,
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
      /** ObservationData for an observation with modified content (also contains corresponding dataset ID). */
      observationContentChanged: Observable<ObservationData>
      /** Modify a content of a particular observation.  */
      setObservationContent: (datasetId: string, observation: ObservationData) => void
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
      /** Create a new dynamic property with given ID, variables, of given `variant` (with corresponding data). */
      addDynamic: (id: string, name: string, variant: DynamicProperty) => void
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
      /** Create a new static property with given ID, variables, of given `variant` (with corresponding data). */
      addStatic: (id: string, name: string, variant: StaticProperty) => void
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
    }
  }

  /** The events regarding the analysis workflow. */
  analysis: {
    /** Sketch data transfered from backend (useful to initiate the state). */
    sketchRefreshed: Observable<SketchData>
    /** Ask for a sketch data (might be useful to update analysis window, or if automatic sketch
     * transfer from backend does not work). */
    refreshSketch: () => void

    /** Start the full inference analysis. */
    startFullInference: () => void
    /** Note that inference analysis was started. */
    inferenceStarted: Observable<boolean>
    /** Inference analysis results. */
    inferenceResultsReceived: Observable<InferenceResults>

    /** Start the static check analysis. */
    startStaticCheck: () => void
    /** Note that static check analysis was started. */
    staticCheckStarted: Observable<boolean>
    /** Static check analysis results. */
    staticCheckResultsReceived: Observable<StaticCheckResults>
  }

  /** The information about errors occurring when processing events on backend. */
  error: {
    /** Error message provided by backend. */
    errorReceived: Observable<string>
  }

  /** Events for creating new sessions. */
  new_session: {
    /** Create a new analysis session. */
    createNewAnalysisSession: () => void
  }
}

/** An object representing all relevant parts of the whole sketch. */
export interface SketchData {
  model: ModelData
  datasets: DatasetData[]
  dyn_properties: DynamicProperty[]
  stat_properties: StaticProperty[]
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
  update_fn: string
}

/** An object representing basic information regarding a model's uninterpreted function. */
export interface UninterpretedFnData {
  id: string
  name: string
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
  dataset: string
  values: string // string with `0`/`1`/`*`, for instance: "0001**110"
}

/** An object representing all information regarding a whole dataset. */
export interface DatasetData {
  id: string
  observations: ObservationData[]
  variables: string[]
  category: DataCategory
}

/**
 * An object representing basic "metadata" information regarding a dataset.
 * Specifically, does not contain information about dataset's observations.
 * */
export interface DatasetMetaData {
  id: string
  variables: [string]
  category: DataCategory
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

/** An object representing all information regarding inference analysis results. */
export interface InferenceResults {
  numSatNetworks: number
  computationTime: number
}

/** An object representing all information regarding static check analysis results. */
export interface StaticCheckResults {
  numSatNetworks: number
  computationTime: number
}

/** A function that is notified when a state value changes. */
export type OnStateValue<T> = (value: T) => void

/**
 * An object that can be emitted through `AeonEvents` as a user action.
 *
 * The `path` property specifies which state item is affected by the event, while
 * the `payload` represents the contents of the event. Multiple events can be
 * joined together to create a "compound" event that is treated as
 * a single user action in the context of the undo-redo stack.
 */
export interface AeonEvent {
  path: string[]
  payload: string | null
}

/**
 * One observable event. This does not necessarily maps to a single state item.
 */
class Observable<T> {
  path: string[]
  listeners: Array<OnStateValue<T>> = []

  constructor (path: string[]) {
    this.path = path
    aeonEvents.setEventListener(path, this.#acceptPayload.bind(this))
  }

  /**
   * Register a listener to this specific observable state item.
   *
   * The listener is notified right away with the current value of the state item.
   *
   * @param listener The listener function which should be invoked when the value
   * of this state item changes.
   * @returns `true` if the listener was added, `false` if it was already registered.
   */
  addEventListener (listener: OnStateValue<T>): boolean {
    if (this.listeners.includes(listener)) {
      return false
    }
    this.listeners.push(listener)
    return true
  }

  /**
     * Unregister a listener previously added through `addEventListener`.
     *
     * @param listener The listener to be unregistered.
     * @returns `true` if the listener was removed, `false` if it was not registered.
     */
  removeEventListener (listener: OnStateValue<T>): boolean {
    const index = this.listeners.indexOf(listener)
    if (index > -1) {
      this.listeners.splice(index, 1)
      return true
    }
    return false
  }

  /**
   * Accept a payload value coming from the backend.
   * @param payload The actual JSON-encoded payload value.
   */
  #acceptPayload (payload: string | null): void {
    payload = payload ?? 'null'
    try {
      const value = JSON.parse(payload)
      for (const listener of this.listeners) {
        this.#notifyListener(listener, value)
      }
    } catch (error) {
      const path = JSON.stringify(this.path)
      dialog.message(
                `Cannot dispatch event ${path} with payload ${payload}: ${String(error)}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    }
  }

  #notifyListener (listener: OnStateValue<T>, value: T): void {
    try {
      listener(value)
    } catch (error) {
      const path = JSON.stringify(this.path)
      dialog.message(
                `Cannot handle event ${path} with value ${String(value)}: ${String(value)}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    }
  }
}

/**
 * One item of the observable application state of type `T`. The UI can listen to
 * the value changes.
 *
 * This structure assumes the state is not directly user editable. That is, the user cannot
 * directly write value of type `T` into this item. However, it could be still editable
 * indirectly through other items or triggers.
 */
class ObservableState<T> {
  path: string[]
  lastValue: T
  listeners: Array<OnStateValue<T>> = []

  constructor (path: string[], initial: T) {
    this.path = path
    this.lastValue = initial
    aeonEvents.setEventListener(path, this.#acceptPayload.bind(this))
  }

  /**
     * Register a listener to this specific observable state item.
     *
     * The listener is notified right away with the current value of the state item.
     *
     * @param listener The listener function which should be invoked when the value
     * of this state item changes.
     * @param notifyNow If set to `false`, the listener is not notified with the current
     * value of the state item upon registering.
     * @returns `true` if the listener was added, `false` if it was already registered.
     */
  addEventListener (listener: OnStateValue<T>, notifyNow = true): boolean {
    if (this.listeners.includes(listener)) {
      return false
    }
    this.listeners.push(listener)
    if (notifyNow) {
      this.#notifyListener(listener, this.lastValue)
    }
    return true
  }

  /**
     * Unregister a listener previously added through `addEventListener`.
     *
     * @param listener The listener to be unregistered.
     * @returns `true` if the listener was removed, `false` if it was not registered.
     */
  removeEventListener (listener: OnStateValue<T>): boolean {
    const index = this.listeners.indexOf(listener)
    if (index > -1) {
      this.listeners.splice(index, 1)
      return true
    }
    return false
  }

  /**
     * Re-emit the latest state of this item to all event listeners.
     */
  refresh (): void {
    aeonEvents.refresh(this.path)
  }

  /**
     * The last value that was emitted for this observable state.
     */
  value (): T {
    return this.lastValue
  }

  /**
     * Accept a payload value coming from the backend.
     * @param payload The actual JSON-encoded payload value.
     */
  #acceptPayload (payload: string | null): void {
    payload = payload ?? 'null'
    try {
      const value = JSON.parse(payload)
      this.lastValue = value
      for (const listener of this.listeners) {
        this.#notifyListener(listener, value)
      }
    } catch (error) {
      const path = JSON.stringify(this.path)
      dialog.message(
                `Cannot dispatch event ${path} with payload ${payload}: ${String(error)}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    }
  }

  #notifyListener (listener: OnStateValue<T>, value: T): void {
    try {
      listener(value)
    } catch (error) {
      const path = JSON.stringify(this.path)
      dialog.message(
                `Cannot handle event ${path} with value ${String(value)}: ${String(value)}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    }
  }
}

/**
 * A version of `ObservableState` which supports direct mutation.
 */
class MutableObservableState<T> extends ObservableState<T> {
  /**
     * Emit a user action which triggers value mutation on the backend. This should then
     * result in a new value being observable through the registered event listeners.
     *
     * @param value New value.
     */
  emitValue (value: T): void {
    aeonEvents.emitAction(this.set(value))
  }

  /**
     * Create a `set` event for the provided `value` and this observable item. Note that the event
     * is not emitted automatically, but needs to be sent through the `AeonEvents`. You can use
     * `this.emitValue` to create and emit the event as one action.
     *
     * The reason why you might want to create the event but not emit it is that events can be
     * bundled to create complex reversible user actions.
     *
     * @param value New value.
     * @returns An event that can be emitted through `AeonEvents`.
     */
  set (value: T): AeonEvent {
    return {
      path: this.path,
      payload: JSON.stringify(value)
    }
  }
}

/**
 * A (reasonably) type-safe wrapper around AEON related tauri events that communicate the state
 * of the application between the frontend and the backend.
 *
 * Each event path only supports one event listener. If you need more listenerers, you likely
 * want to wrap the path into an `ObservableState` object.
 */
class AeonEvents {
  /** Uniquely identifies one "session". Multiple windows can share a session. */
  sessionId: string
  listeners: any

  constructor (sessionId: string) {
    this.sessionId = sessionId
    this.listeners = {}
    listen(AEON_VALUE, this.#onStateChange.bind(this)).catch((e) => {
      console.error(e)
    })
  }

  /** Request a retransmission of state data managed at the provided path. */
  refresh (path: string[]): void {
    emit(AEON_REFRESH, {
      session: this.sessionId,
      path
    }).catch((error) => {
      dialog.message(
                `Cannot refresh [${JSON.stringify(path)}]: ${String(error)}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    })
  }

  /**
     * Emit one or more aeon events as a single user action.
     *
     * Assuming all events are reversible, the item will be saved as a single undo-redo entry.
     */
  emitAction (events: AeonEvent | AeonEvent[]): void {
    if (!(events instanceof Array)) {
      events = [events]
    }
    emit(AEON_ACTION, {
      session: this.sessionId,
      events
    }).catch((error) => {
      dialog.message(
                `Cannot process events [${JSON.stringify(events)}]: ${error}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    })
  }

  /**
     * Set an event listener that will be notified when the specified path changes.
     *
     * Note that only one listener is supported for each path. If you want more listeners,
     * consider the `ObservableState` class.
     */
  setEventListener (
    path: string[],
    listener: (payload: string | null) => void
  ): void {
    let listeners = this.listeners
    while (path.length > 1) {
      const key = path[0]
      path = path.slice(1)
      if (!(key in listeners)) {
        listeners[key] = {}
      }
      listeners = listeners[key]
    }
    listeners[path[0]] = listener
  }

  /**
     * React to (possibly multiple) value changes.
     *
     * Note that if multiple values changed, the listeners are notified
     * in sequence.
     */
  #onStateChange (event: Event<AeonEvent[]>): void {
    for (const e of event.payload) {
      this.#notifyValueChange(e)
    }
  }

  #notifyValueChange (event: AeonEvent): void {
    // Find listener residing at the specified path, or return if no listener exists.
    let listener = this.listeners
    let path = event.path
    while (path.length > 0) {
      const key = path[0]
      path = path.slice(1)
      if (!(key in listener)) {
        console.log('Event ignored.', event.path)
        return
      }
      listener = listener[key]
    }

    // Emit event.
    try {
      listener(event.payload)
    } catch (error) {
      dialog.message(
                `Cannot handle event [${JSON.stringify(event.path)}] with payload ${String(event.payload)}: ${String(error)}`,
                { title: 'Internal app error', type: 'error' }
      ).catch((e) => {
        console.error(e)
      })
    }
  }
}

/**
 * A singleton object which implements access to Tauri events.
 */
export const aeonEvents = new AeonEvents(await invoke('get_session_id'))

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
    createNewAnalysisSession (): void {
      aeonEvents.emitAction({
        path: ['new-analysis-session'],
        payload: null
      })
    }
  },
  sketch: {
    sketchRefreshed: new Observable<SketchData>(['sketch', 'get_whole_sketch']),
    refreshSketch (): void {
      aeonEvents.refresh(['sketch', 'get_whole_sketch'])
    },
    exportSketch (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'export_sketch'],
        payload: path
      })
    },
    sketchReplaced: new Observable<SketchData>(['sketch', 'set_all']),
    importSketch (path: string): void {
      aeonEvents.emitAction({
        path: ['sketch', 'import_sketch'],
        payload: path
      })
    },
    newSketch (): void {
      aeonEvents.emitAction({
        path: ['sketch', 'new_sketch'],
        payload: null
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
      variableNameChanged: new Observable<VariableData>(['sketch', 'model', 'variable', 'set_name']),
      variableIdChanged: new Observable<VariableIdUpdateData>(['sketch', 'model', 'variable', 'set_id']),
      variableUpdateFnChanged: new Observable<VariableData>(['sketch', 'model', 'variable', 'set_update_fn']),

      uninterpretedFnCreated: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'add']),
      uninterpretedFnRemoved: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'remove']),
      uninterpretedFnNameChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_name']),
      uninterpretedFnArityChanged: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'set_arity']),
      uninterpretedFnArityIncremented: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'increment_arity']),
      uninterpretedFnArityDecremented: new Observable<UninterpretedFnData>(['sketch', 'model', 'uninterpreted_fn', 'decrement_arity']),
      uninterpretedFnIdChanged: new Observable<UninterpretedFnIdUpdateData>(['sketch', 'model', 'uninterpreted_fn', 'set_id']),
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

      addVariable (
        varId: string = '',
        varName: string = '',
        position: LayoutNodeDataPrototype | LayoutNodeDataPrototype[] = []
      ): void {
        if (varName === undefined) {
          varName = varId
        }

        if (!Array.isArray(position)) {
          position = [position]
        }

        // First action creates the variable, either default or given.
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', 'add'],
          payload: JSON.stringify({ variable: { id: varId, name: varName, update_fn: '' }, layouts: position })
        })
      },
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
      setVariableName (varId: string, newName: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'variable', varId, 'set_name'],
          payload: newName
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
          payload: newExpression.toString()
        })
      },
      addUninterpretedFn (uninterpretedFnId: string, arity: number, uninterpretedFnName: string = ''): void {
        if (uninterpretedFnName === '') {
          uninterpretedFnName = uninterpretedFnId
        }
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', 'add'],
          payload: JSON.stringify({
            id: uninterpretedFnId,
            name: uninterpretedFnName,
            arguments: Array(arity).fill([Monotonicity.UNSPECIFIED, Essentiality.UNKNOWN]),
            expression: ''
          })
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
      setUninterpretedFnName (uninterpretedFnId: string, newName: string): void {
        aeonEvents.emitAction({
          path: ['sketch', 'model', 'uninterpreted_fn', uninterpretedFnId, 'set_name'],
          payload: newName
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
      datasetCategoryChanged: new Observable<DatasetMetaData>(['sketch', 'observations', 'set_category']),
      datasetVariableChanged: new Observable<DatasetMetaData>(['sketch', 'observations', 'set_var_id']),
      datasetVariableRemoved: new Observable<DatasetMetaData>(['sketch', 'observations', 'remove_var']),

      observationPushed: new Observable<ObservationData>(['sketch', 'observations', 'push_obs']),
      observationPopped: new Observable<ObservationData>(['sketch', 'observations', 'pop_obs']),
      observationRemoved: new Observable<ObservationData>(['sketch', 'observations', 'remove_obs']),
      observationIdChanged: new Observable<ObservationIdUpdateData>(['sketch', 'observations', 'set_obs_id']),
      observationContentChanged: new Observable<ObservationData>(['sketch', 'observations', 'set_obs_content']),

      addDataset (id: string, variables: string[], observations: ObservationData[], category: DataCategory = DataCategory.UNSPECIFIED): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', 'add'],
          payload: JSON.stringify({
            id,
            variables,
            observations,
            category
          })
        })
      },
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
      setDatasetCategory (id: string, newCategory: DataCategory): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', id, 'set_category'],
          payload: JSON.stringify(newCategory)
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
      setObservationContent (datasetId: string, observation: ObservationData): void {
        aeonEvents.emitAction({
          path: ['sketch', 'observations', datasetId, observation.id, 'set_content'],
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

      addDynamic (id: string, name: string, variant: DynamicProperty): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'dynamic', 'add'],
          payload: JSON.stringify({
            id,
            name,
            variant
          })
        })
      },
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
      addStatic (id: string, name: string, variant: StaticProperty): void {
        aeonEvents.emitAction({
          path: ['sketch', 'properties', 'static', 'add'],
          payload: JSON.stringify({
            id,
            name,
            variant
          })
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
    sketchRefreshed: new Observable<SketchData>(['analysis', 'get_sketch']),
    refreshSketch (): void {
      aeonEvents.refresh(['analysis', 'get_sketch'])
    },

    inferenceResultsReceived: new Observable<InferenceResults>(['analysis', 'inference_results']),
    staticCheckResultsReceived: new Observable<StaticCheckResults>(['analysis', 'static_results']),
    inferenceStarted: new Observable<boolean>(['analysis', 'inference_running']),
    staticCheckStarted: new Observable<boolean>(['analysis', 'static_running']),

    startFullInference (): void {
      aeonEvents.emitAction({
        path: ['analysis', 'run_inference'],
        payload: null
      })
    },
    startStaticCheck (): void {
      aeonEvents.emitAction({
        path: ['analysis', 'run_static'],
        payload: null
      })
    }
  }
}
