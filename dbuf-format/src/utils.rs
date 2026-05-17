use dbuf_core::cst::Child;
use dbuf_core::cst::Token;
use dbuf_core::cst::Tree;
use dbuf_core::cst::TreeKind;

use dbuf_core::location::Location;
use dbuf_core::location::Offset;

use pretty::{DocAllocator, DocBuilder};

/// Events during Tree visiting.
pub enum Event<'a> {
    /// Next token in stream.
    NextToken(&'a Token, &'a Location<Offset>),
    /// Entering new scope.
    NewScope(&'a TreeKind),
    /// Exiting scope to.
    ExitScope(&'a TreeKind),
}

pub trait PrettyStrategy<'a, D>: Sized {
    fn next(&mut self, event: Event<'a>, allocator: &'a D) -> DocBuilder<'a, D>
    where
        D: DocAllocator<'a>,
        D::Doc: Clone;
}

pub fn run<'a, D, S>(t: &'a Tree, mut s: S, allocator: &'a D) -> (S, DocBuilder<'a, D>)
where
    D: DocAllocator<'a>,
    D::Doc: Clone,
    S: PrettyStrategy<'a, D>,
{
    let mut doc = allocator.nil();
    for child in &t.children {
        match child {
            Child::Token(token, location) => {
                let new_d = s.next(Event::NextToken(token, location), allocator);
                doc = doc.append(new_d);
            }
            Child::Tree(ch_tree) => {
                let new_d = s.next(Event::NewScope(&ch_tree.kind), allocator);
                doc = doc.append(new_d);

                let (new_s, new_d) = run(ch_tree, s, allocator);
                s = new_s;
                doc = doc.append(new_d);

                let new_d = s.next(Event::ExitScope(&t.kind), allocator);
                doc = doc.append(new_d);
            }
        }
    }

    (s, doc)
}
