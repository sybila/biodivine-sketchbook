/// Window context is an "god object" that owns all essential data structures of a single AEON
/// window. However, note that a window can actually have some of its components in "detached"
/// windows. In such cases, the window context also tracks the detached windows and delivers
/// necessary information to them (i.e. detached windows technically have their own context,
/// but their context is only a proxy for the "main" window context which holds the actual
/// state of all values).
pub struct WindowContext {}
