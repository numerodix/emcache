#[derive(Debug)]
pub struct Node<T> {
    pub data: T,
    prev: Option<*mut Node<T>>,
    next: Option<*mut Node<T>>,
}

impl<T: Eq> Node<T> {
    pub fn new(t: T) -> Node<T> {
        Node {
            data: t,
            prev: None,
            next: None,
        }
    }

    fn eq(&self, other: Node<T>) -> bool {
        // Equality is purely based on the contained value
        self.data == other.data
    }
}

// is this useful?
type NodePtr<T> = *mut Node<T>;


#[derive(Debug)]
pub struct DList<T> {
    front: Option<*mut Node<T>>,
    back: Option<*mut Node<T>>,
    size: u64,
}

impl<T: Eq> DList<T> {
    pub fn new() -> DList<T> {
        DList {
            front: None,
            back: None,
            size: 0,
        }
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn front(&self) -> Option<*mut Node<T>> {
        self.front
    }

    pub fn back(&self) -> Option<*mut Node<T>> {
        self.back
    }

    pub fn push_front(&mut self, t: T) -> *mut Node<T> {
        let mut node = Node::new(t);

        // If there is an existing front node make it and the new front node
        // point at each other
        match self.front {
            Some(prev_front) => unsafe {
                node.next = Some(prev_front);
                (*prev_front).prev = Some(&mut node);
            },
            None => (),
        }
        // Point front to the new front node
        self.front = Some(&mut node);

        // Only point back to new front node if there is no back node yet
        match self.back {
            Some(n) => (),
            None => {
                self.back = Some(&mut node);
            }
        }

        self.size += 1;

        &mut node
    }

    // api:
    //
    // len() -> u64
    //
    // front() -> node
    // back() -> node
    //
    // push_front(T) -> node
    // pop_front() -> T
    // push_back(T) -> node
    // pop_back() -> T
    //
    // insert_after(node, T) -> node
    // insert_before(node, T) -> node
    // pop_after(node) -> T
    // pop_before(node) -> T
    //
    // pop(node) -> T
}


#[test]
fn test_api() {
    let mut dl: DList<u64> = DList::new();
    assert_eq!(dl.len(), 0);

    let node_1 = dl.push_front(1);
    assert_eq!(dl.len(), 1);
    unsafe {
        // Data value is correct
        assert_eq!((*node_1).data, 1);

        // Front points to it
        assert_eq!(dl.front().unwrap(), node_1);
        // Back points to it
        assert_eq!(dl.back().unwrap(), node_1);
    }

    let node_2 = dl.push_front(2);
    assert_eq!(dl.len(), 2);
    unsafe {
        // Data value is correct
        assert_eq!((*node_2).data, 2);

        // Front points to it
        assert_eq!(dl.front().unwrap(), node_2);
        // Back points to it
        assert_eq!(dl.back().unwrap(), node_1);

        // Previous front points to new front
        assert_eq!((*node_1).prev.unwrap(), node_2);
        println!("n1 {:?}", *node_1);
        println!("n2 {:?}", *node_2);
    }
}
