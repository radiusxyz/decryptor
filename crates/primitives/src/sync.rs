//! Wrapper module for thread-safe synchronization primitives.
use crate::error::PrimitiveError;
use crossbeam::atomic::AtomicCell;
use serde::{
    de::{Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    thread::{self, ThreadId},
};

/// Re-exports
#[doc(inline)]
pub use crossbeam::channel;

/// Thread-safe shareable pointer that supports transparent
/// deserialization and serialization of inner data. Internally,
/// it wraps std::sync::Arc.
///
/// # Caveat
///
/// A deserialized type from the type into which the original type
/// has been serialized and the original type DO NOT share the
/// reference count.
///
/// # Example
///
/// ```
/// use primitives::sync::Arc;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Deserialize, Serialize)]
/// pub struct MyStruct {
///     inner: Arc<String>,
/// }
///
/// impl MyStruct {
///     pub fn new(data: &str) -> Self {
///         Self {
///             inner: Arc::new(String::from(data)),
///         }
///     }
/// }
///
/// let my_struct = MyStruct::new("Hello, world!");
/// 
/// // Serialize with JSON serializer.
/// let serialized = serde_json::to_string(&my_struct).unwrap();
/// dbg!(&serialized);
///
/// // Deserialize from JSON string.
/// let deserialized: MyStruct = serde_json::from_str(&serialized).unwrap();
/// dbg!(&deserialized);
/// 
/// ```
pub struct Arc<T>(std::sync::Arc<T>);

unsafe impl<T> Send for Arc<T> {}
unsafe impl<T> Sync for Arc<T> {}

impl<T: fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.0)
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> From<std::sync::Arc<T>> for Arc<T> {
    fn from(value: std::sync::Arc<T>) -> Self {
        Self(value)
    }
}

impl<T> From<T> for Arc<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = T::deserialize(deserializer)?;
        Ok(Self::from(data))
    }
}

impl<T: Serialize> Serialize for Arc<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = self.0.as_ref();
        Ok(data.serialize(serializer)?)
    }
}

impl<T> Arc<T> {
    pub fn new(data: T) -> Self {
        Self(std::sync::Arc::new(data))
    }
}

/// A wrapper around [tokio::sync::Mutex] that support:
/// - transparent deserialization & serialization of inner type.
/// - prevention of double-locking within the same thread.
/// 
/// Mutex can safely be shared across different execution contexts.
/// 
/// # Examples
/// 
/// ```
/// let shared_data = Arc::new(Mutex::new(String::default()));
/// 
/// tokio::task::spawn_blocking({
///     let data = shared_data.clone();
///     move || {
///         let mut lock = data.blocking_lock();
///         lock.push_str("Hello, ");
///     }
/// });
/// 
/// let async_task = tokio::spawn({
///     let data = shared_data.clone();
///     async move {
///         let mut lock = data.lock().await;
///         lock.push_str("world!");
///     }
/// });
/// 
/// ```
pub struct Mutex<T> {
    inner: tokio::sync::Mutex<T>,
    thread_id: Arc<AtomicCell<Option<ThreadId>>>,
}

/// # Safety
/// 
/// It is safe to implement Send
unsafe impl<T> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Mutex<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = T::deserialize(deserializer)?;
        Ok(Self::new(data))
    }
}

impl<T: Serialize> Serialize for Mutex<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data = &*self.blocking_lock();
        Ok(data.serialize(serializer)?)
    }
}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(data),
            thread_id: Arc::new(AtomicCell::new(None)),
        }
    }

    fn duplicate_lock_panic(&self) {
        if let Some(thread_id) = self.thread_id.load() {
            if thread_id == thread::current().id() {
                panic!("Duplicate Lock Error");
            }
        }
    }

    /// Blockingly locks the mutex and returns the [`MutexGuard`].
    /// 
    /// # Panics
    /// 
    /// The function panics in any of the following contexts:
    /// - calling it within an asynchronous execution context.
    /// - calling it a second time on the same object within the same thread without
    /// dropping the previous MutexGuard object.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use primitives::sync::Mutex;
    /// 
    /// let data = Mutex::new(String::default());
    /// 
    /// let mut first_lock = data.blocking_lock();
    /// first_lock.push_str("Hello, ");
    /// drop(first_lock); // If you comment out this line, the program panics.
    /// 
    /// let mut second_lock = data.blocking_lock();
    /// second_lock.push_str("world!");
    /// dbg!(&**second_lock);
    /// 
    /// ```
    #[track_caller]
    pub fn blocking_lock(&self) -> MutexGuard<'_, T> {
        self.duplicate_lock_panic();
        let mutex_guard = self.inner.blocking_lock();
        MutexGuard::new(mutex_guard, self.thread_id.clone())
    }

    /// Asynchronously locks the mutex and returns the [`MutexGuard`].
    /// 
    /// # Panics
    /// 
    /// The function panics if called a second time on the same thread without
    /// dropping the previous MutexGuard object.
    /// 
    /// # Example
    /// 
    /// ```
    /// use primitives::sync::Mutex;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let data = Mutex::new(String::default());
    ///     
    ///     let mut first_lock = data.lock().await;
    ///     first_lock.push_str("Hello, ");
    ///     drop(first_lock); // If you comment out this line, the program panics.
    ///     
    ///     let mut second_lock = data.lock().await;
    ///     second_lock.push_str("world!");
    ///     dbg!(&**second_lock);
    /// }
    /// 
    /// ```
    pub async fn lock(&self) -> MutexGuard<'_, T> {
        self.duplicate_lock_panic();
        let mutex_guard = self.inner.lock().await;
        MutexGuard::new(mutex_guard, self.thread_id.clone())
    }

    /// Attempts to require a lock. It returns an error if the lock is held 
    /// held in another thread. If called within a loop in which the loop
    /// continues on error, it works like [`blocking_lock()`] function.
    /// 
    /// # Panics
    /// 
    /// The function panics if called a second time on the same thread without
    /// dropping the previous MutexGuard object.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use primitives::sync::{Arc, Mutex};
    /// use std::thread;
    /// 
    /// let shared_data = Arc::new(Mutex::new(String::default()));
    /// 
    /// let data = shared_data.clone();
    /// let worker = thread::spawn(move || {
    ///     loop {
    ///         match data.try_lock() {
    ///             Ok(mut lock) => {
    ///                 lock.push_str("Hello, world!");
    ///                 break;
    ///             },
    ///             Err(_) => continue,
    ///         }
    ///     }
    /// });
    /// worker.join().unwrap();
    /// 
    /// let lock = shared_data.try_lock().unwrap();
    /// dbg!(&**lock);
    /// 
    /// ```
    #[track_caller]
    pub fn try_lock(&self) -> Result<MutexGuard<'_, T>, PrimitiveError> {
        self.duplicate_lock_panic();
        let mutex_guard = self.inner.try_lock()?;
        Ok(MutexGuard::new(mutex_guard, self.thread_id.clone()))
    }
}

pub struct MutexGuard<'a, T> {
    inner: tokio::sync::MutexGuard<'a, T>,
    thread_id: Arc<AtomicCell<Option<ThreadId>>>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = tokio::sync::MutexGuard<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.thread_id.store(None);
    }
}

impl<'a, T> MutexGuard<'a, T> {
    fn new(
        mutex_guard: tokio::sync::MutexGuard<'a, T>,
        thread_id: Arc<AtomicCell<Option<ThreadId>>>,
    ) -> Self {
        let current_id = thread::current().id();
        thread_id.store(Some(current_id));
        Self {
            inner: mutex_guard,
            thread_id,
        }
    }
}
