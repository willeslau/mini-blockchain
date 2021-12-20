# Trie
This is a study repo for trie implementation.

## Notes On Rust
The initial implementation of Trie is something like this:
```rust
impl <'db, DB: HashDB> Trie<'db, DB> {
    // Other methods
    // ...
    
	/// Try to update the key with provided value
    pub fn try_update(&mut self, key: &[u8], val: &[u8]) -> Result<(), Error> {
		ensure!(val.len() > 0, Error::ValueCannotBeEmpty)?;

        self.unhashed += 1;
		let k = key_bytes_to_hex(key);
		let db = Rc::new(RefCell::new(self.db));
		let node = Rc::new(RefCell::new(&self.root));
		Self::insert(
			db.clone(),
			Rc::clone(&node),
			&vec![],
			&k,
			Node::ValueNode(val.clone().into())
		)
    }

	fn insert(db: Rc<RefCell<&'db DB>>, node: Rc<RefCell<&Node>>, prefix: &[u8], key: &[u8], value: Node) -> Result<(), Error> {
		if key.len() == 0 {
			return Err(Error::KeyCannotBeEmpty);
		}

		match *node.borrow() {
			Node::Empty => {
				let n = Node::ShortNode {
					key: (key.clone()).to_owned(),
					val: Box::new(value),
					flags: NodeFlag::new(true)
				};
                
                // The problem is here!
                // We cannot pass node as Rc<RefCell<Node>> because it requires a move.
                // But we cannot replace here with reference either, because who 
                // would own `n` here?
				node.replace(&n);
				return Ok(())
			},
			_ => {}
		}
		Ok(())
	}
}
```
We would encounter an ownership problem in the above. This is different compared to other programming language.
There are a few ways to handling this. One way is to use `node` itself and pass in the DB object for performing insertion.
Something like `node.insert(db, ...)`. But this way would make the interface look ugly.
As a resolution, we need something to hold the ownership of the nodes. This way is inspired by Parity's Trie implementation.

But there are other problems as well:
```rust
type NodeRef<'a> = Rc<RefCell<Option<&'a mut Node>>>;

pub struct Trie<'a, H: HashDB> {
    db: &'a mut H,
    root: NodeRef<'a>,
    cache: &'a mut Cache,
    unhashed: u32,
}

impl<'a, H: HashDB> Trie<'a, H> {
    //... other code

    fn insert(
        &mut self,
        mut node: NodeRef<'a>,
        prefix: &mut [u8],
        key: Hash,
        value: Node,
    ) -> Result<(), Error> {
        if key.len() == 0 {
            return Err(Error::KeyCannotBeEmpty);
        }

        match &*node.borrow_mut() {
            None => {
                let n = Node::ShortNode {
                    key,
                    val: Box::new(value),
                    flags: NodeFlag::new(true),
                };
                self.cache.insert(key, n)?;
                // >>>> PROBLEM HERE <<<<<
                // This line cannot compile as get_mut's lifetime is not 
                // that of 'a, which from the declaration, they should be ok?
                node.replace(self.cache.get_mut(key));
            }
            Some(n) => {
                println!("{:?}", n);
            }
        }
        Ok(())
    }
}
```
In the end, use `NodeLocation`. Maybe there are better ways to do this? Should come back and do it again.