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
