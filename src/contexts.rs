use std::{collections::HashMap, ops::Deref};

use leptos::*;
use uuid::Uuid;
use web_sys::Element;

use crate::timer::MultiTimer;

/// Reactively stores the current fullscreen element.
///
/// Derefs to a `ReadSignal<Option<Element>>`.
///
/// Should be provided as a context by the top-level component.
/// Retrieve using `expect_context::<FullscreenElement>()`
#[derive(Debug, Clone, Copy)]
pub struct FullscreenElement(ReadSignal<Option<Element>>);

impl Deref for FullscreenElement {
    type Target = ReadSignal<Option<Element>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FullscreenElement {
    /// Create a new `FullscreenElement` from a signal.
    pub const fn new(elem: ReadSignal<Option<Element>>) -> Self {
        Self(elem)
    }
}

/// Stores and sets icons used into local storage.
///
/// Should be provided as a context by the top-level component.
/// Retrieve using `expect_context::<Icons>()`
#[derive(Debug, Clone, Copy)]
pub struct Icons(StoredValue<HashMap<String, String>>);

impl Icons {
    /// The key used to fetch the icons from local storage.
    pub const STORAGE_KEY: &'static str = "icons";

    /// Tries to retrieve the specified icon.
    ///
    /// Returns [`None`] if it has not been stored.
    pub fn get(&self, icon_name: &str) -> Option<String> {
        self.0.with_value(|v| v.get(icon_name).cloned())
    }

    /// Adds a new icon to the `HashMap` and stores it in local storage.
    ///
    /// If the icon already exists, it is overwritten.
    pub fn add(&self, icon_name: &str, svg: &str) {
        self.0.update_value(|map| {
            map.insert(icon_name.to_string(), svg.to_string());
        });

        let string = self.0.with_value(|map| {
            map.ser()
                .expect("should be able to serialize `HashMap<String, String>`")
        });

        window()
            .local_storage()
            .unwrap()
            .unwrap()
            .set_item(Self::STORAGE_KEY, &string)
            .unwrap();
    }

    /// Constructs a new `Icons` map from the local storage "icons" key.
    ///
    /// Returns an empty map if it could not be deserialized.
    pub fn from_local_storage() -> Self {
        let string = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get_item(Self::STORAGE_KEY)
            .unwrap()
            .unwrap_or_default();

        let map: HashMap<String, String> = serde_json::from_str(&string).unwrap_or_default();

        Self(store_value(map))
    }

    /// Returns a clone of the icons hashmap.
    pub fn get_map(&self) -> HashMap<String, String> {
        self.0.get_value()
    }
}

/// A list of timers.
///
/// There will always be at least one timer. A new one is pushed
/// if the vector is empty.
///
/// Should be provided as a context by the top-level component.
/// Retrieve using `expect_context::<TimerList>()`
#[derive(Debug, Clone, Copy)]
pub struct TimerList {
    vec: RwSignal<Vec<MultiTimer>>,
}

impl TimerList {
    /// Creates a new `TimerList` with one timer.
    pub fn new() -> Self {
        Self {
            vec: create_rw_signal(vec![MultiTimer::new()]),
        }
    }

    /// Sets the stored `RwSignal` to the given list.
    ///
    /// Adds a new timer if `timers` is empty.
    pub fn set(&self, timers: Vec<MultiTimer>) {
        batch(|| {
            self.vec.set(timers);

            if self.is_empty() {
                self.push_new();
            }
        });
    }

    /// Constructs a new `TimerList` from a list of timers.
    ///
    /// If the vec is empty, a timer is created.
    pub fn from_timers(timers: Vec<MultiTimer>) -> Self {
        if timers.is_empty() {
            Self::new()
        } else {
            Self {
                vec: create_rw_signal(timers),
            }
        }
    }

    /// Adds a new timer to the list.
    pub fn push_new(&self) {
        self.vec.update(|v| v.push(MultiTimer::new()));
    }

    /// Removes the timer at a certain index from the list.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove_index(&self, index: usize) {
        batch(|| {
            self.vec.update(|v| {
                let timer = v.remove(index);
                // stop ongoing timers
                timer.reset();
            });
            if self.is_empty() {
                self.push_new();
            };
        });
    }

    /// Removes the timer with the specified id.
    ///
    /// # Panics
    /// Panics if no timer with the given id is found.
    pub fn remove_id(&self, id: Uuid) {
        let index = self.vec.with_untracked(|v| {
            v.iter()
                .position(|t| t.id() == id)
                .expect("Could not find timer with specified id.")
        });
        self.remove_index(index);
    }

    /// Clears the timer list and adds one new timer.
    pub fn clear(&self) {
        batch(|| {
            // don't use `Vec::clear`, need to reset all timers stored
            for i in (0..self.len()).rev() {
                self.remove_index(i);
            }
        });
    }

    /// Gets a copy of timer list.
    pub fn to_vec(&self) -> Vec<MultiTimer> {
        self.vec.get_untracked()
    }

    pub fn vec_signal(&self) -> ReadSignal<Vec<MultiTimer>> {
        self.vec.read_only()
    }

    /// Returns the number of timers stored.
    #[expect(clippy::len_without_is_empty, reason = "should not be empty")]
    pub fn len(&self) -> usize {
        self.vec.with_untracked(Vec::len)
    }

    /// Whether the timer list is empty.
    ///
    /// This should never be the case publically.
    fn is_empty(&self) -> bool {
        self.vec.with_untracked(Vec::is_empty)
    }

    /// Returns whether this timer list is unchanged from initialisation.
    ///
    /// Checks that there is 1 timer with no input.
    pub fn is_initial(&self) -> bool {
        self.len() == 1
            && self
                .vec
                .with_untracked(|v| v[0].input().get_untracked().is_empty())
    }
}

impl IntoIterator for TimerList {
    type Item = MultiTimer;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.get().into_iter()
    }
}

impl Default for TimerList {
    fn default() -> Self {
        Self::new()
    }
}
