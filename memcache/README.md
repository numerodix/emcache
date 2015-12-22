Datastructures:
    Hashtable HT
        - stores: Item { key -> (value, atime) }

    Access list AL (doubly linked list)
        - stores (atime, ptr-to-Item)


InsertNewItem
    - insert into HT - O(1)
    - prepend to AL - O(1)

InsertNewItemWhenFull
    - pop last item in AL - O(1)
    - pop that item from HT - O(1)
    - goto InsertNewItem

UpdateExistingItem / AccessItem:
    - locate item in AL - O(n)  XXX <======
    - lookup / write in HT - O(1)
