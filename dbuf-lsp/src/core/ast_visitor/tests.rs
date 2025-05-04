use std::collections::BTreeMap;

use crate::core::ast_access::ElaboratedAst;
use crate::core::ast_access::ParsedAst;
use crate::core::default_ast::default_parsed_ast;

use super::visit_ast;
use super::Visit;
use super::VisitResult;
use super::Visitor;

fn get_ast() -> ParsedAst {
    default_parsed_ast()
}

fn correct_skip(visit: &Visit<'_>) -> VisitResult {
    match visit {
        Visit::Keyword(_, _location) => VisitResult::Skip,
        Visit::Type(_loc_string, _locationn) => VisitResult::Skip,
        Visit::Dependency(_loc_string, _location) => VisitResult::Skip,
        Visit::Branch => VisitResult::Skip,
        Visit::PatternAlias(_loc_string) => VisitResult::Continue,
        Visit::PatternCall(_loc_string, _location) => VisitResult::Skip,
        Visit::PatternCallArgument(_loc_string) => VisitResult::Skip,
        Visit::PatternCallStop => VisitResult::Continue,
        Visit::PatternLiteral(_literal, _locationn) => VisitResult::Continue,
        Visit::PatternUnderscore(_location) => VisitResult::Continue,
        Visit::Constructor(_constructor) => VisitResult::Skip,
        Visit::Filed(_loc_string, _locationn) => VisitResult::Skip,
        Visit::TypeExpression(_loc_string, _locationn) => VisitResult::Skip,
        Visit::Expression(_location) => VisitResult::Skip,
        Visit::AccessChainStart => VisitResult::Skip,
        Visit::AccessChain(_loc_string) => VisitResult::Continue,
        Visit::AccessDot(_location) => VisitResult::Continue,
        Visit::AccessChainLast(_loc_string) => VisitResult::Continue,
        Visit::ConstructorExpr(_loc_string) => VisitResult::Skip,
        Visit::ConstructorExprArgument(_loc_string) => VisitResult::Skip,
        Visit::ConstructorExprStop => VisitResult::Continue,
        Visit::VarAccess(_loc_string) => VisitResult::Continue,
        Visit::Operator(_, _location) => VisitResult::Continue,
        Visit::Literal(_literal, _locationn) => VisitResult::Continue,
    }
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
        SkipMask { mask: 0, size }
    }
    fn set(&mut self, mask: u32) {
        assert!(mask < (1 << self.size));
        self.mask = mask
    }
    fn next(&mut self) -> bool {
        self.mask += 1;
        if self.mask >= (1 << self.size) {
            self.mask = 0;
            false
        } else {
            true
        }
    }
    fn need_skip(&self, step: u32) -> bool {
        step < self.size && (self.mask & (1 << step)) != 0
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
    fn visit(&mut self, visit: Visit<'a>) -> VisitResult {
        assert!(!self.stopped, "execution stops after stop signal");

        let result = if self.step == self.stop_after {
            self.stopped = true;
            VisitResult::Stop
        } else if self.skip_mask.need_skip(self.step) {
            correct_skip(&visit)
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
            VisitResult::Stop => panic!("unexpected stop signal at mask {}", mask),
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
fn test_stop_after_signal() {
    let ast = get_ast();
    let tempo_elaborated = ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    };

    for stop_after in 0.. {
        let skip_mask = SkipMask::new(0);
        let mut visitor = TestVisitor::new(skip_mask, stop_after);
        visit_ast(&ast, &mut visitor, &tempo_elaborated);
        if !visitor.stopped {
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

    let mut skip_mask = SkipMask::new(18);
    loop {
        let mut visitor = TestVisitor::new(skip_mask, 1e9 as u32);
        visit_ast(&ast, &mut visitor, &tempo_elaborated);
        assert!(!visitor.stopped, "all steps done");
        if !skip_mask.next() {
            break;
        }
    }
}

#[test]
fn test_skip_stop_correctness() {
    let ast = get_ast();
    let tempo_elaborated = ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    };

    let mut skip_mask = SkipMask::new(13);
    loop {
        for stop_after in 0.. {
            let mut visitor = TestVisitor::new(skip_mask, stop_after);
            visit_ast(&ast, &mut visitor, &tempo_elaborated);
            if !visitor.stopped {
                break;
            }
        }
        if !skip_mask.next() {
            break;
        }
    }
}
