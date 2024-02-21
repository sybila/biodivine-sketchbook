import { type Event, emit, listen } from '@tauri-apps/api/event'
import { dialog, invoke } from '@tauri-apps/api'
import { type Monotonicity } from './html/util/data-interfaces'

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

  /** The state of the main model. */
  model: {
    /** Refresh events. */

    /** List of model variables. */
    variablesRefreshed: Observable<[VariableData]>
    /** Refresh the variables. */
    refreshVariables: () => void
    /** List of model uninterpreted functions. */
    uninterpretedFnsRefreshed: Observable<[UninterpretedFnData]>
    /** Refresh the uninterpretedFns. */
    refreshUninterpretedFns: () => void
    /** List of model regulations. */
    regulationsRefreshed: Observable<[RegulationData]>
    /** Refresh the regulations. */
    refreshRegulations: () => void
    /** List of model layouts. */
    layoutsRefreshed: Observable<[LayoutData]>
    /** Refresh the layouts. */
    refreshLayouts: () => void
    /** List of nodes in a given layout. */
    layoutNodesRefreshed: Observable<[LayoutNodeData]>
    /** Refresh the nodes in a given layout. */
    refreshLayoutNodes: (layoutId: string) => void

    /** Variable-related setter events */

    /** VariableData for a newly created variable. */
    variableCreated: Observable<VariableData>
    /** Create new variable with given ID and name. If only ID string is given, it is used for both ID and name. */
    addVariable: (varId: string, varName?: string, position?: LayoutNodeDataPrototype | LayoutNodeDataPrototype[]) => void
    /** VariableData of a removed variable. */
    variableRemoved: Observable<VariableData>
    /** Remove a variable with given ID. */
    removeVariable: (varId: string) => void
    /** VariableData (with a new `name`) for a renamed variable. */
    variableNameChanged: Observable<VariableData>
    /** Set a name of variable with given ID to a given new name. */
    setVariableName: (varId: string, newName: string) => void
    /** Object with `original_id` of a variable and its `new_id`. */
    variableIdChanged: Observable<VariableIdUpdateData>
    /** Set an ID of variable with given original ID to a new id. */
    setVariableId: (originalId: string, newId: string) => void

    /** Uninterpreted function-related setter events */

    /** UninterpretedFnData for a newly created uninterpreted function. */
    uninterpretedFnCreated: Observable<UninterpretedFnData>
    /** Create new uninterpreted function with given arity, ID, and name.
     * If name is not given, ID string is used for both ID and name. */
    addUninterpretedFn: (uninterpretedFnId: string, arity: number, uninterpretedFnName?: string) => void
    /** UninterpretedFnData of a removed uninterpreted function. */
    uninterpretedFnRemoved: Observable<UninterpretedFnData>
    /** Remove uninterpreted function with given ID. */
    removeUninterpretedFn: (uninterpretedFnId: string) => void
    /** UninterpretedFnData (with a new `name`) for a renamed uninterpreted function. */
    uninterpretedFnNameChanged: Observable<UninterpretedFnData>
    /** Set a name of uninterpreted function with given ID. */
    setUninterpretedFnName: (uninterpretedFnId: string, newName: string) => void
    /** UninterpretedFnData (with a new `arity`) for a modified uninterpreted function. */
    uninterpretedFnArityChanged: Observable<UninterpretedFnData>
    /** Set an arity of uninterpreted function with given ID. */
    setUninterpretedFnArity: (uninterpretedFnId: string, newArity: number) => void
    /** Object with `original_id` of a uninterpreted function and its `new_id`. */
    uninterpretedFnIdChanged: Observable<UninterpretedFnIdUpdateData>
    /** Set an ID of uninterpreted function with given original ID to a new id. */
    setUninterpretedFnId: (originalId: string, newId: string) => void

    /** Regulation-related setter events */

    /** RegulationData for a newly created regulation. */
    regulationCreated: Observable<RegulationData>
    /** Create new regulation specified by all of its four components. */
    addRegulation: (regulatorId: string, targetId: string, sign: string, essential: string) => void
    /** RegulationData of a removed regulation. */
    regulationRemoved: Observable<RegulationData>
    /** Create regulation specified by its regulator and target. */
    removeRegulation: (regulatorId: string, targetId: string) => void
    /** RegulationData (with a new `sign`) of a regulation that has its sign changed. */
    regulationSignChanged: Observable<RegulationData>
    /** Set sign of a regulation specified by its regulator and target. */
    setRegulationSign: (regulatorId: string, targetId: string, newSign: string) => void
    /** RegulationData (with a new `essentiality`) of a regulation that has its essentiality changed. */
    regulationEssentialityChanged: Observable<RegulationData>
    /** Set essentiality of a regulation specified by its regulator and target. */
    setRegulationEssentiality: (regulatorId: string, targetId: string, newEssentiality: string) => void

    /** Layout-related setter events */

    /** LayoutData for a newly created layout. */
    layoutCreated: Observable<LayoutData>
    /** Create a new layout with given ID and name. */
    addLayout: (layoutId: string, layoutName: string) => void
    /** LayoutData of a removed layout. */
    layoutRemoved: Observable<LayoutData>
    /** Remove a layout with given ID. */
    removeLayout: (layoutId: string) => void
    /** LayoutNodeData (with a new `px` and `py`) for a layout node that had its position changed. */
    nodePositionChanged: Observable<LayoutNodeData>
    /** Change a position of a variable in a layout to new coordinates. */
    changeNodePosition: (layoutId: string, varId: string, newX: number, newY: number) => void
  }
}

/** An object representing basic information regarding a model variable. */
export interface VariableData {
  id: string
  name: string
}

/** An object representing basic information regarding a model's uninterpreted function. */
export interface UninterpretedFnData {
  id: string
  name: string
  arity: number
}

/** An object representing basic information regarding a model regulation. */
export interface RegulationData {
  regulator: string
  target: string
  sign: Monotonicity
  essential: string
}

/** An object representing basic information regarding a model layout. */
export interface LayoutData {
  id: string
  name: string
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

/** An object representing information needed for variable id change. */
export interface VariableIdUpdateData { original_id: string, new_id: string }

/** An object representing information needed for uninterpreted function's id change. */
export interface UninterpretedFnIdUpdateData { original_id: string, new_id: string }

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
  model: {
    variablesRefreshed: new Observable<[VariableData]>(['model', 'get_variables']),
    refreshVariables (): void {
      aeonEvents.refresh(['model', 'get_variables'])
    },
    uninterpretedFnsRefreshed: new Observable<[UninterpretedFnData]>(['model', 'get_uninterpreted_fns']),
    refreshUninterpretedFns (): void {
      aeonEvents.refresh(['model', 'get_uninterpreted_fns'])
    },
    regulationsRefreshed: new Observable<[RegulationData]>(['model', 'get_regulations']),
    refreshRegulations (): void {
      aeonEvents.refresh(['model', 'get_regulations'])
    },
    layoutsRefreshed: new Observable<[LayoutData]>(['model', 'get_layouts']),
    refreshLayouts (): void {
      aeonEvents.refresh(['model', 'get_layouts'])
    },
    layoutNodesRefreshed: new Observable<[LayoutNodeData]>(['model', 'get_layout_nodes']),
    refreshLayoutNodes (layoutId: string): void {
      aeonEvents.refresh(['model', 'get_layout_nodes', layoutId])
    },

    variableCreated: new Observable<VariableData>(['model', 'variable', 'add']),
    variableRemoved: new Observable<VariableData>(['model', 'variable', 'remove']),
    variableNameChanged: new Observable<VariableData>(['model', 'variable', 'set_name']),
    variableIdChanged: new Observable<VariableIdUpdateData>(['model', 'variable', 'set_id']),

    uninterpretedFnCreated: new Observable<UninterpretedFnData>(['model', 'uninterpreted_fn', 'add']),
    uninterpretedFnRemoved: new Observable<UninterpretedFnData>(['model', 'uninterpreted_fn', 'remove']),
    uninterpretedFnNameChanged: new Observable<UninterpretedFnData>(['model', 'uninterpreted_fn', 'set_name']),
    uninterpretedFnArityChanged: new Observable<UninterpretedFnData>(['model', 'uninterpreted_fn', 'set_arity']),
    uninterpretedFnIdChanged: new Observable<UninterpretedFnIdUpdateData>(['model', 'uninterpreted_fn', 'set_id']),

    regulationCreated: new Observable<RegulationData>(['model', 'regulation', 'add']),
    regulationRemoved: new Observable<RegulationData>(['model', 'regulation', 'remove']),
    regulationSignChanged: new Observable<RegulationData>(['model', 'regulation', 'set_sign']),
    regulationEssentialityChanged: new Observable<RegulationData>(['model', 'regulation', 'set_essentiality']),

    layoutCreated: new Observable<LayoutData>(['model', 'layout', 'add']),
    layoutRemoved: new Observable<LayoutData>(['model', 'layout', 'remove']),
    nodePositionChanged: new Observable<LayoutNodeData>(['model', 'layout', 'update_position']),

    addVariable (
      varId: string,
      varName: string = '',
      position: LayoutNodeDataPrototype | LayoutNodeDataPrototype[] = []
    ): void {
      if (varName === '') {
        varName = varId
      }
      const actions = []
      // First action creates the variable.
      actions.push({
        path: ['model', 'variable', 'add'],
        payload: JSON.stringify({ id: varId, name: varName })
      })
      // Subsequent actions position it in one or more layouts.
      if (!Array.isArray(position)) {
        position = [position]
      }
      for (const p of position) {
        actions.push({
          path: ['model', 'layout', p.layout, 'update_position'],
          payload: JSON.stringify({
            layout: p.layout,
            variable: varId,
            px: p.px,
            py: p.py
          })
        })
      }
      aeonEvents.emitAction(actions)
    },
    removeVariable (varId: string): void {
      aeonEvents.emitAction({
        path: ['model', 'variable', varId, 'remove'],
        payload: null
      })
    },
    setVariableName (varId: string, newName: string): void {
      aeonEvents.emitAction({
        path: ['model', 'variable', varId, 'set_name'],
        payload: newName
      })
    },
    setVariableId (originalId: string, newId: string): void {
      aeonEvents.emitAction({
        path: ['model', 'variable', originalId, 'set_id'],
        payload: newId
      })
    },
    addUninterpretedFn (uninterpretedFnId: string, arity: number, uninterpretedFnName: string = ''): void {
      if (uninterpretedFnName === '') {
        uninterpretedFnName = uninterpretedFnId
      }
      aeonEvents.emitAction({
        path: ['model', 'uninterpreted_fn', 'add'],
        payload: JSON.stringify({ id: uninterpretedFnId, arity, name: uninterpretedFnName })
      })
    },
    removeUninterpretedFn (uninterpretedFnId: string): void {
      aeonEvents.emitAction({
        path: ['model', 'uninterpreted_fn', uninterpretedFnId, 'remove'],
        payload: null
      })
    },
    setUninterpretedFnName (uninterpretedFnId: string, newName: string): void {
      aeonEvents.emitAction({
        path: ['model', 'uninterpreted_fn', uninterpretedFnId, 'set_name'],
        payload: newName
      })
    },
    setUninterpretedFnArity (uninterpretedFnId: string, newArity: number): void {
      aeonEvents.emitAction({
        path: ['model', 'uninterpreted_fn', uninterpretedFnId, 'set_name'],
        payload: newArity.toString()
      })
    },
    setUninterpretedFnId (originalId: string, newId: string): void {
      aeonEvents.emitAction({
        path: ['model', 'uninterpreted_fn', originalId, 'set_id'],
        payload: newId
      })
    },
    addRegulation (regulatorId: string, targetId: string, sign: string, essential: string): void {
      aeonEvents.emitAction({
        path: ['model', 'regulation', 'add'],
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
        path: ['model', 'regulation', regulatorId, targetId, 'remove'],
        payload: null
      })
    },
    setRegulationSign (regulatorId: string, targetId: string, newSign: string): void {
      aeonEvents.emitAction({
        path: ['model', 'regulation', regulatorId, targetId, 'set_sign'],
        payload: JSON.stringify(newSign)
      })
    },
    setRegulationEssentiality (regulatorId: string, targetId: string, newEssentiality: string): void {
      aeonEvents.emitAction({
        path: ['model', 'regulation', regulatorId, targetId, 'set_essentiality'],
        payload: JSON.stringify(newEssentiality)
      })
    },
    addLayout (layoutId: string, layoutName: string): void {
      aeonEvents.emitAction({
        path: ['model', 'layout', 'add'],
        payload: JSON.stringify({ id: layoutId, name: layoutName })
      })
    },
    removeLayout (layoutId: string): void {
      aeonEvents.emitAction({
        path: ['model', 'layout', layoutId, 'remove'],
        payload: null
      })
    },
    changeNodePosition (layoutId: string, varId: string, newX: number, newY: number): void {
      aeonEvents.emitAction({
        path: ['model', 'layout', layoutId, 'update_position'],
        payload: JSON.stringify({ layout: layoutId, variable: varId, px: newX, py: newY })
      })
    }
  }
}
