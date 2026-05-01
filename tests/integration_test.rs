use std::fs;

use brain_fucker::run_interpreter;

#[test]
fn test_bf_interpreter_outputs() {
    for i in 1..=7 {
        let bf_path = format!("tests/data/{}.bf", i);
        let in_path = format!("tests/data/{}.in", i);
        let ans_path = format!("tests/data/{}.ans", i);

        let code = fs::read(&bf_path).unwrap();

        // Read input as a byte array
        let input_data = fs::read(&in_path).unwrap();
        // Create an empty Vec to capture output in memory instead of writing to a file
        let mut output_data = Vec::new();

        run_interpreter(code, input_data.as_slice(), &mut output_data)
            .unwrap_or_else(|_| panic!("Test case {} failed to execute", i));

        let actual_out = String::from_utf8_lossy(&output_data).replace("\r\n", "\n");
        let expected_ans = fs::read_to_string(&ans_path).unwrap().replace("\r\n", "\n");

        assert_eq!(
            actual_out.trim_end(),
            expected_ans.trim_end(),
            "Output for test case {} does not match the expected result!",
            i
        );

        println!("Test case {} passed!", i);
    }
}
