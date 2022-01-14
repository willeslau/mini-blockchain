/// The generic trait for key-value pair database storage
pub trait DBStorage: Send + Sync {
    /// Look up a given hash into the bytes that hash to it, returning None if the
    /// hash is not known.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Check for the existance of a hash-key.
    fn contains(&self, key: &[u8]) -> bool;

    /// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
    /// are counted and the equivalent number of `remove()`s must be performed before the data
    /// is considered dead.
    fn insert(&mut self, key: Vec<u8>, value: Vec<u8>);

    /// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
    /// happen without the data being eventually being inserted into the DB. It can be "owed" more than once.
    fn remove(&mut self, key: &[u8]);
}
