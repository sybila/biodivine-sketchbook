import { Event, emit, listen } from '@tauri-apps/api/event'
import { dialog, invoke } from '@tauri-apps/api';

/* Names of relevant events that communicate with the Tauri backend. */

const AEON_ACTION = 'aeon-action'
const AEON_VALUE = 'aeon-value'
const AEON_REFRESH = 'aeon-refresh'


/**
 * A type-safe representation of the state managed by an Aeon session.
 * 
 * Under the hood, this uses `AeonEvents` to implement actions and listeners.
 */
type AeonState = {

    /** Access to the internal state of the undo-redo stack. */
    undo_stack: {
        /** True if the stack has actions that can be undone. */
        can_undo: ObservableState<boolean>,
        /** True if the stack has actions that can be redone. */
        can_redo: ObservableState<boolean>,
        /** Try to undo an action. Emits an error if no actions can be undone. */
        undo(): void,
        /** Try to redo an action. Emits an error if no actions can be redone. */
        redo(): void,        
    },

    /** The state of the main navigation tab-bar. */
    tab_bar: {
        /** Integer ID of the currently active tab. */
        active: MutableObservableState<number>,
        /** A *sorted* list of IDs of all pinned tabs. */
        pinned: MutableObservableState<number[]>,
        /** Pins a single tab if not currently pinned. */
        pin(id: number): void,
        /** Unpins a single tab if currently pinned. */
        unpin(id: number): void,
    }

}

/** A function that is notified when a state value changes. */
type OnStateValue<T> = (value: T) => void;

/**
 * An object that can be emitted through `AeonEvents` as a user action.
 * 
 * The `path` property specifies which state item is affected by the event, while 
 * the `payload` represents the contents of the event. Multiple events can be
 * joined together to create a "compound" event that is treated as 
 * a single user action in the context of the undo-redo stack.
 */
type AeonEvent = {
    path: string[],
    payload: string | null,
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
    last_value: T
    listeners: OnStateValue<T>[] = []    

    constructor(path: string[], initial: T) {
        this.path = path;
        this.last_value = initial;
        aeon_events.setEventListener(path, this.#acceptPayload.bind(this));
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
    addEventListener(listener: OnStateValue<T>, notifyNow = true) {
        if (this.listeners.includes(listener)) {
            return false;
        }
        this.listeners.push(listener);
        if (notifyNow) {
            this.#notifyListener(listener, this.last_value);
        }
        return true;
    }

    /**
     * Unregister a listener previously added through `addEventListener`.
     * 
     * @param listener The listener to be unregistered.
     * @returns `true` if the listener was removed, `false` if it was not registered.
     */
    removeEventListener(listener: OnStateValue<T>) {
        const index = this.listeners.indexOf(listener);
        if (index > -1) {
            this.listeners.splice(index, 1);
            return true;
        }
        return false;
    }

    /**
     * Re-emit the latest state of this item to all event listeners.
     */
    refresh() {
        aeon_events.refresh(this.path);
    }

    /**
     * The last value that was emitted for this observable state.
     */
    value(): T {
        return this.last_value;
    }

    /**
     * Accept a payload value coming from the backend.
     * @param payload The actual JSON-encoded payload value.
     */
    #acceptPayload(payload: string | null) {        
        try {
            const value = JSON.parse(payload === null ? "null" : payload);
            this.last_value = value;
            for (let listener of this.listeners) {
                this.#notifyListener(listener, value);                
            }
        } catch (error) {
            dialog.message(
                `Cannot dispatch event [${this.path}] with payload ${payload}: ${error}`, 
                { title: "Internal app error", type: "error" }
            );
        }
    }

    #notifyListener(listener: OnStateValue<T>, value: T) {
        try {
            listener(value);
        } catch (error) {
            dialog.message(
                `Cannot handle event [${this.path}] with value ${value}: ${error}`, 
                { title: "Internal app error", type: "error" }
            );
        }
    }
}

/**
 * A version of `ObservableState` which supports direct mutation.
 */
class MutableObservableState<T> extends ObservableState<T> {
    constructor(path: string[], initial: T) {
        super(path, initial)
    }

    /**
     * Emit a user action which triggers value mutation on the backend. This should then 
     * result in a new value being observable through the registered event listeners.
     * 
     * @param value New value.
     */
    emitValue(value: T) {
        aeon_events.emitAction(this.set(value))
    }

    /**
     * Create a `set` event for the provided `value` and this observable item. Note that the event
     * is not emitted automatically, but needs to be sent through the `aeon_events`. You can use 
     * `this.emitValue` to create and emit the event as one action. 
     * 
     * The reason why you might want to create the event but not emit it is that events can be 
     * bundled to create complex reversible user actions.
     * 
     * @param value New value.
     * @returns An event that can be emitted through `AeonEvents`.
     */
    set(value: T): AeonEvent {
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
    session_id: string
    listeners: any

    constructor(session_id: string) {
        this.session_id = session_id;
        this.listeners = {}
        listen(AEON_VALUE, this.#onStateChange.bind(this))
    }

    /** Request a retransmission of state data managed at the provided path. */
    refresh(path: string[]) {        
        emit(AEON_REFRESH, {
            session: this.session_id,
            path: path,            
        }).catch((error) => {
            dialog.message(
                `Cannot refresh [${path}]: ${error}`, 
                { title: "Internal app error", type: "error" }
            );
        });
    }

    /** 
     * Emit one or more aeon events as a single user action. 
     * 
     * Assuming all events are reversible, the item will be saved as a single undo-redo entry.          
     */
    emitAction(events: AeonEvent | AeonEvent[]) {
        if (!(events instanceof Array)) {
            events = [events]
        }
        emit(AEON_ACTION, {
            session: this.session_id,
            events: events
        }).catch((error) => {
            dialog.message(
                `Cannot process events [${events}]: ${error}`, 
                { title: "Internal app error", type: "error" }
            );
        });                
    }

    /**
     * Set an event listener that will be notified when the specified path changes.
     * 
     * Note that only one listener is supported for each path. If you want more listeners,
     * consider the `ObservableState` class.
     */
    setEventListener(
        path: string[], 
        listener: (payload: string | null) => void
    ) {        
        let listeners = this.listeners
        while (path.length > 1) {
            let key = path[0]
            path = path.slice(1)
            if (!(key in listeners)) {
                listeners[key] = {}                
            }
            listeners = listeners[key]
        }
        listeners[path[0]] = listener;
    }

    /**
     * React to (possibly multiple) value changes. 
     * 
     * Note that if multiple values changed, the listeners are notified
     * in sequence.
     */
    #onStateChange(event: Event<AeonEvent[]>) {
        for (let e of event.payload) {
            this.#notifyValueChange(e)
        }        
    }

    #notifyValueChange(event: AeonEvent) {
        // Find listener residing at the specified path, or return if no listener exists.
        let listener = this.listeners;
        let path = event.path;
        while (path.length > 0) {
            let key = path[0]
            path = path.slice(1)
            if (!(key in listener)) {
                console.log("Event ignored.", event.path);
                return;
            }
            listener = listener[key];
        }

        // Emit event.
        try {
            listener(event.payload);
        } catch (error) {
            dialog.message(
                `Cannot handle event [${event.path}] with payload ${event.payload}: ${error}`, 
                { title: "Internal app error", type: "error" }
            );
        }
    }

}

/**
 * A singleton object which implements access to Tauri events.
 */
export let aeon_events = new AeonEvents(await invoke('get_session_id'));

/**
 * A singleton state management object for the current Aeon session.
 */
export let aeon_state: AeonState = {
    undo_stack: {
        can_undo: new ObservableState<boolean>(["undo_stack", "can_undo"], false),
        can_redo: new ObservableState<boolean>(["undo_stack", "can_redo"], false),
        undo() {
            aeon_events.emitAction({
                path: ["undo_stack", "undo"],
                payload: null,
            })
        },
        redo() {
            aeon_events.emitAction({
                path: ["undo_stack", "redo"],
                payload: null,
            })
        },
    },    
    tab_bar: {
        active: new MutableObservableState<number>(["tab_bar", "active"], 0),
        pinned: new MutableObservableState<number[]>(["tab_bar", "pinned"], []),
        pin (id: number): void {
            const value = this.pinned.value();
            if (!value.includes(id)) {
                value.push(id);
                value.sort();
                this.pinned.emitValue(value);
            }
        },
        unpin (id: number): void {
            const value = this.pinned.value();
            const index = value.indexOf(id);
            if (index > -1) {
                value.splice(index, 1);
                this.pinned.emitValue(value);
            }
        }
    }
};