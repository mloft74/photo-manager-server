/// TODO: Create an event system that can be used across threads.
/// - Trait to send events (will most likely only consist of a single method)
/// - Trait to add listeners
/// - Might need a trait for handling TCP socket connections, but that sounds like it might be the responsibility of a separate piece of code to handle
/// - Arc<Mutex<*EventSystemType*>> to make it work across multiple threads (see how the screensaver does this for reference)
const TODO: () = ();
