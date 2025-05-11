use std::collections::BTreeMap;

use crate::core::ast_access::ElaboratedAst;
use crate::core::ast_access::ParsedAst;
use crate::core::default_ast::default_parsed_ast;

use super::Visit;
use super::VisitResult;
use super::Visitor;
use super::safe_skip::safe_skip;
use super::visit_ast;

fn get_ast() -> ParsedAst {
    default_parsed_ast()
}

#[derive(Clone, Copy)]
struct SkipMask {
    mask: u32,
    size: u32,
}

struct TestVisitor {
    skip_mask: SkipMask,
    stop_after: u32,
    step: u32,
    stopped: bool,
}

impl SkipMask {
    fn new(size: u32) -> SkipMask {
        assert!(size < 32);
        SkipMask { mask: 0, size }
    }
    fn set(&mut self, mask: u32) {
        assert!(mask < (1 << self.size));
        self.mask = mask
    }
    fn need_skip(&self, step: u32) -> bool {
        step < self.size && (self.mask & (1 << step)) != 0
    }
}
impl Iterator for SkipMask {
    type Item = SkipMask;

    fn next(&mut self) -> Option<Self::Item> {
        let ans = *self;

        self.mask += 1;
        if self.mask > (1 << self.size) {
            None
        } else {
            Some(ans)
        }
    }
}

impl TestVisitor {
    fn new(skip_mask: SkipMask, stop_after: u32) -> TestVisitor {
        TestVisitor {
            skip_mask,
            stop_after,
            step: 0,
            stopped: false,
        }
    }
}

impl<'a> Visitor<'a> for TestVisitor {
    type StopResult = u32;

    fn visit(&mut self, visit: Visit<'a>) -> VisitResult<Self::StopResult> {
        assert!(!self.stopped, "execution stops after stop signal");

        let result = if self.step == self.stop_after {
            self.stopped = true;
            VisitResult::Stop(self.step + 1)
        } else if self.skip_mask.need_skip(self.step) {
            safe_skip(&visit)
        } else {
            VisitResult::Continue
        };

        self.step += 1;
        result
    }
}

fn check_skip_mask(mask: u32, skip_at: &[u32]) {
    let mut skip_mask = SkipMask::new(3);
    skip_mask.set(mask);
    let mut visitor = TestVisitor::new(skip_mask, 3);

    let mut step = 0;
    while step < 3 {
        let result = visitor.visit(Visit::Branch);
        match result {
            VisitResult::Continue => assert!(
                !skip_at.contains(&step),
                "bad continue at step {}, mask {}",
                step,
                mask
            ),
            VisitResult::Skip => assert!(
                skip_at.contains(&step),
                "bad skip at step {}, mask {}",
                step,
                mask
            ),
            VisitResult::Stop(step) => panic!("unexpected stop signal at mask {mask}, step {step}"),
        }
        step += 1;
    }
    assert!(!visitor.stopped);
}

#[test]
fn test_skip_mask() {
    check_skip_mask(0b000, &[]);

    check_skip_mask(0b001, &[0]);
    check_skip_mask(0b010, &[1]);
    check_skip_mask(0b100, &[2]);

    check_skip_mask(0b011, &[0, 1]);
    check_skip_mask(0b101, &[0, 2]);
    check_skip_mask(0b110, &[1, 2]);

    check_skip_mask(0b111, &[0, 1, 2]);
}

#[test]
fn test_skip_mask_iterator() {
    let skip_mask = SkipMask::new(2);

    let mut expect = 0;
    for mask in skip_mask {
        assert!(mask.mask == expect, "bad skip mask iterator");
        expect += 1;
    }
    assert!(expect == 4, "bad skip mask count")
}

#[test]
fn test_stop_after_signal() {
    let ast = get_ast();
    let tempo_elaborated = ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    };

    for stop_after in 0.. {
        let skip_mask = SkipMask::new(0);
        let mut visitor = TestVisitor::new(skip_mask, stop_after);
        let res = visit_ast(&ast, &mut visitor, &tempo_elaborated);
        if visitor.stopped {
            assert!(res.is_some());
            assert!(res.unwrap() == visitor.step)
        }
        if !visitor.stopped {
            assert!(res.is_none());
            break;
        }
    }
}

#[test]
fn test_skip_correctness() {
    let ast = get_ast();
    let tempo_elaborated = ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    };

    let skip_mask = SkipMask::new(18);

    for mask in skip_mask {
        let mut visitor = TestVisitor::new(mask, 1e9 as u32);
        let res = visit_ast(&ast, &mut visitor, &tempo_elaborated);
        assert!(!visitor.stopped, "all steps done");
        assert!(res.is_none());
    }
}

#[test]
fn test_skip_stop_correctness() {
    let ast = get_ast();
    let tempo_elaborated = ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    };

    let skip_mask = SkipMask::new(13);
    for mask in skip_mask {
        for stop_after in 0.. {
            let mut visitor = TestVisitor::new(mask, stop_after);
            let res = visit_ast(&ast, &mut visitor, &tempo_elaborated);
            if visitor.stopped {
                assert!(res.is_some());
                assert!(res.unwrap() == visitor.step)
            }
            if !visitor.stopped {
                assert!(res.is_none());
                break;
            }
        }
    }
}
