use std::rc::Rc;

struct Atom {
    name: String,
}

struct List {
    head: Item,
    rest: Option<Item>,
}

enum Item {
    IAtom(Atom),
    IList(Rc<List>),
}
