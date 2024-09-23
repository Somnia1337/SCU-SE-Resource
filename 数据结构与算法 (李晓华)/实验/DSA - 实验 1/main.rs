use std::collections::VecDeque;
use std::io::{self, Write};

const DIGITS: &str = "0123456789.";
const OPS: &str = "+-*/%^&()=";
const RANK: [[u8; 2]; 10] = [
    [3, 2],
    [3, 2],
    [5, 4],
    [5, 4],
    [5, 4],
    [7, 6],
    [7, 6],
    [1, 8],
    [8, 1],
    [0, 0],
];

enum InvalidInputError {
    EmptyInput,
    InvalidCharacter(char),
    InvalidEnding(String),
}

fn main() {
    println!("有效的字符: {}{}", DIGITS, OPS);
    println!("请以 '=' 结尾");
    println!("--------------------");

    'outer: loop {
        let mut stop = false;
        let mut input = String::new();
        while !stop {
            input = read_input("输入表达式: ");
            if input.eq("exit") {
                break 'outer;
            }
            match check(&input) {
                Ok(_) => stop = true,
                Err(e) => {
                    let error_message = match e {
                        InvalidInputError::EmptyInput => String::from("输入为空"),
                        InvalidInputError::InvalidCharacter(c) => format!("非法字符 '{}'", c),
                        InvalidInputError::InvalidEnding(s) => s,
                    };
                    println!("非法输入: {}", error_message);
                }
            }
        }
        println!("计算结果: {}", solve(&input));
        println!("--------------------");
    }
}

fn check(s: &str) -> Result<(), InvalidInputError> {
    if s.is_empty() {
        return Err(InvalidInputError::EmptyInput);
    }
    for c in s.as_bytes() {
        let c = *c as char;
        if c != ' ' && !DIGITS.contains(c) && !OPS.contains(c) {
            return Err(InvalidInputError::InvalidCharacter(c));
        }
    }
    if !s.ends_with("=") {
        return Err(InvalidInputError::InvalidEnding(String::from(
            "必须以 '=' 结尾",
        )));
    }
    Ok(())
}

fn solve(s: &str) -> f64 {
    let mut op_stk: VecDeque<char> = VecDeque::new();
    let mut num_stk: VecDeque<f64> = VecDeque::new();
    let s_vec: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < s_vec.len() {
        let c = s_vec[i];

        if c == ' ' {
            i += 1;
            continue;
        }

        if c.is_ascii_digit() || (c == '-' && (i == 0 || !s_vec[i - 1].is_ascii_digit())) {
            let mut end = i + 1;
            while end < s_vec.len() && DIGITS.contains(s_vec[end]) {
                end += 1;
            }
            let value_str: String = s_vec[i..end].iter().collect();
            let value = value_str.parse().unwrap();
            num_stk.push_front(value);
            i = end;
        } else {
            loop {
                let rank_out = get_rank_out(c);
                let rank_in = op_stk.front().map_or(-1, |&top| get_rank_in(top));

                match rank_in.cmp(&rank_out) {
                    std::cmp::Ordering::Less => {
                        op_stk.push_front(c);
                        break;
                    }
                    std::cmp::Ordering::Greater => {
                        let op = op_stk.pop_front().unwrap();
                        if op != '-' || num_stk.len() >= 2 {
                            let b = num_stk.pop_front().unwrap();
                            let a = num_stk.pop_front().unwrap();
                            num_stk.push_front(cal(a, b, op));
                        } else {
                            let p = -num_stk.pop_front().unwrap();
                            num_stk.push_front(p);
                        }
                    }
                    std::cmp::Ordering::Equal => {
                        op_stk.pop_front();
                        break;
                    }
                }
            }
            i += 1;
        }
    }

    num_stk.pop_front().unwrap()
}

fn cal(a: f64, b: f64, op: char) -> f64 {
    match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => a / b,
        '%' => a % b,
        '^' => a.powf(b),
        '&' => a.powf(1.0 / b),
        _ => unreachable!(),
    }
}

fn get_rank_out(op: char) -> i32 {
    RANK[OPS.find(op).unwrap()][1] as i32
}

fn get_rank_in(op: char) -> i32 {
    RANK[OPS.find(op).unwrap()][0] as i32
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("failed to flush stdout");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read input");

    input.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(result: f64, expected: f64) {
        assert!(
            (result - expected).abs() < 0.0001,
            "Expected {}, but got {}",
            expected,
            result
        );
    }

    #[test]
    fn test_case_1() {
        let result = solve("3+6=");
        assert_close(result, 9.0);
    }

    #[test]
    fn test_case_2() {
        let result = solve("2.4+17.8=");
        assert_close(result, 20.2);
    }

    #[test]
    fn test_case_3() {
        let result = solve("107.6+13-26.57=");
        assert_close(result, 94.03);
    }

    #[test]
    fn test_case_4() {
        let result = solve("6.2*70.5=");
        assert_close(result, 437.1);
    }

    #[test]
    fn test_case_5() {
        let result = solve("104.2/23.78=");
        assert_close(result, 4.3818);
    }

    #[test]
    fn test_case_6() {
        let result = solve("4/5=");
        assert_close(result, 0.8);
    }

    #[test]
    fn test_case_7() {
        let result = solve("20^3=");
        assert_close(result, 8000.0);
    }

    #[test]
    fn test_case_8() {
        let result = solve("12.4^3=");
        assert_close(result, 1906.624);
    }

    #[test]
    fn test_case_9() {
        let result = solve("5.6&3=");
        assert_close(result, 1.7758);
    }

    #[test]
    fn test_case_10() {
        let result = solve("37%4=");
        assert_close(result, 1.0);
    }

    #[test]
    fn test_case_11() {
        let result = solve("37.5%12=");
        assert_close(result, 1.5);
    }

    #[test]
    fn test_case_12() {
        let result = solve("6.2*70.5/18.3=");
        assert_close(result, 23.8852);
    }

    #[test]
    fn test_case_13() {
        let result = solve("14.2+5^3=");
        assert_close(result, 139.2);
    }

    #[test]
    fn test_case_14() {
        let result = solve("14.2+27.5*5^3=");
        assert_close(result, 3451.7);
    }

    #[test]
    fn test_case_15() {
        let result = solve("-32.76+(8-5)^2*102.67/(8%3)=");
        assert_close(result, 429.255);
    }

    #[test]
    fn test_case_16() {
        let result = solve("-3.89*(18-2.0)&2+1023.6^3%4=");
        assert_close(result, -15.30399);
    }

    #[test]
    fn test_case_17() {
        let result = solve("-32.76+102.67*78.934/(8.2-5.3)^2=");
        assert_close(result, 930.87302);
    }

    #[test]
    fn test_case_18() {
        let result = solve("-3*((121.35-0.35)&2)/34.567+102.36^(6/9)=");
        assert_close(result, 20.92732);
    }
}
