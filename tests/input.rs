use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use potoo_kumir_interpreter::{Host, Interpreter, Profile, parse_with_profile};

#[derive(Default)]
struct TestHost {
    input: VecDeque<String>,
    output: Rc<RefCell<String>>,
}

impl Host for TestHost {
    fn input(&mut self) -> Result<String, String> {
        self.input
            .pop_front()
            .ok_or_else(|| "no input remains".to_owned())
    }

    fn output(&mut self, text: &str) -> Result<(), String> {
        self.output.borrow_mut().push_str(text);
        Ok(())
    }
}

#[test]
fn input_initializes_an_unassigned_variable() {
    let source = "алг\nцел x\nнач\nввод x\nвывод x\nкон";
    let program = parse_with_profile(source, Profile::Classic).expect("valid source");
    let output = Rc::new(RefCell::new(String::new()));
    let host = TestHost {
        input: VecDeque::from(["42".to_owned()]),
        output: output.clone(),
    };
    let mut interpreter = Interpreter::new(host);

    interpreter.run(&program).expect("input should assign x");

    assert_eq!(output.borrow().as_str(), "42");
}

fn execute_and_capture_output(source: &str) -> String {
    let program = parse_with_profile(source, Profile::Classic).expect("valid Kumir source");
    let output = Rc::new(RefCell::new(String::new()));
    let host = TestHost {
        input: VecDeque::new(),
        output: output.clone(),
    };
    let mut interpreter = Interpreter::new(host);

    interpreter.run(&program).expect("program should execute");

    output.borrow().clone()
}

#[test]
fn grouped_argument_type_and_result_parameter_execute() {
    let source = "алг\nнач\n  вещ x, y, z\n  x := 2\n  y := 3\n  гипотенуза(x, y, z)\n  вывод z\nкон\nалг гипотенуза (вещ a, b, рез вещ c)\nнач\n  c := a * 10 + b\nкон";

    assert_eq!(execute_and_capture_output(source), "23");
}

#[test]
fn grouped_parameter_modes_and_types_reset_between_segments() {
    let source = "алг\nнач\n  цел x, y, m, n\n  вещ c, d\n  x := 2\n  y := 3\n  m := 5\n  n := 7\n  c := 0\n  d := 0\n  transform(x, y, c, d, m, n)\n  вывод x, \":\", y, \":\", c, \":\", d, \":\", m, \":\", n\nкон\nалг transform (арг цел a, b, рез вещ c, d, аргрез цел m, n)\nнач\n  c := a + b\n  d := a * b\n  m := m + a\n  n := n + b\nкон";

    assert_eq!(execute_and_capture_output(source), "2:3:5:6:7:10");
}

#[test]
fn fully_typed_ungrouped_parameters_remain_valid() {
    let source = "алг\nнач\n  вещ x, result\n  x := 4\n  result := 0\n  increment(x, result)\n  вывод result\nкон\nалг increment (арг вещ source, рез вещ target)\nнач\n  target := source + 1\nкон";

    assert_eq!(execute_and_capture_output(source), "5");
}

#[test]
fn a_new_parameter_mode_requires_a_new_type() {
    let source = "алг\nнач\nкон\nалг malformed (вещ a, рез b)\nнач\nкон";
    let diagnostics = parse_with_profile(source, Profile::Classic)
        .expect_err("a new mode segment without a type should be rejected");
    let messages = diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        messages.contains("parameter `рез b` needs a declared type"),
        "unexpected diagnostic: {messages}"
    );
}

#[test]
fn a_typed_parameter_group_requires_at_least_one_name() {
    let source = "алг\nнач\nкон\nалг f (вещ, b)\nнач\nкон";
    let diagnostics = parse_with_profile(source, Profile::Classic)
        .expect_err("a type-only parameter group should be rejected");
    let messages = diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        messages.contains("parameter group `вещ` needs at least one name"),
        "unexpected diagnostic: {messages}"
    );
}

#[test]
fn result_mode_persists_across_a_later_type_group() {
    let source = r#"алг
нач
  цел x, y
  лит message
  вещ input
  x := 0
  y := 0
  message := ""
  input := 2.5
  test(x, y, message, input)
  вывод x, ":", y, ":", message, ":", input
кон
алг test (рез цел m, n, лит t, арг вещ source)
нач
  m := 1
  n := 2
  t := "ok"
кон"#;

    assert_eq!(execute_and_capture_output(source), "1:2:ok:2.5");
}
