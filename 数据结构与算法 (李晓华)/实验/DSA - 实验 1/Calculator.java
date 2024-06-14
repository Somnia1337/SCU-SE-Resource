import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Scanner;
import java.util.Stack;

class Calculator {
    private final String digits = "0123456789."; // 合法的数字, 包括小数点
    private final String ops = "+-*/%^&()="; // 合法的运算符
    private final int[][] rank = {{3, 2}, {3, 2}, {5, 4}, {5, 4}, {5, 4}, {7, 6}, {7, 6}, {1, 8}, {8, 1}, {0, 0}};
    // 运算符的栈内栈外优先级

    public static void main(String[] args) {
        Calculator calculator = new Calculator();
        calculator.run();
    }

    private void run() {
        System.out.println("有效的字符: " + digits + ops);
        System.out.println("请以 '=' 结尾");
        System.out.println("--------------------");

        Scanner in = new Scanner(System.in);
        while (true) {
            boolean stop = false;
            String input;
            do {
                System.out.print("输入表达式: ");
                input = in.nextLine().strip();
                if (input.equals("exit")) stop = true;
                if (input.equals("test")) test();
            } while (!check(input) && !stop);
            if (input.equals("test")) continue;
            if (!stop) {
                System.out.println("计算结果: " + solve(input));
                System.out.println("--------------------");
            }
            else break;
        }
    }

    private boolean check(String s) {
        if (s.equals("exit") || s.equals("test")) return true;
        if (s.isEmpty()) {
            System.out.println("非法输入: 输入为空");
            return false;
        }
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c != ' ' && digits.indexOf(c) == -1 && ops.indexOf(c) == -1) {
                System.out.println("非法输入: 在 " + i + " 位置有非法字符 '" + c + "'");
                return false;
            }
        }
        if (!s.endsWith("=")) {
            System.out.println("非法输入: 必须以 '=' 结尾");
            return false;
        }
        return true;
    }

    private double solve(String s) {
        Stack<Character> opStk = new Stack<>(); // opStk: 运算符栈
        Stack<Double> numStk = new Stack<>(); // numStk: 数值栈
        for (int i = 0; i < s.length(); i++) { // 遍历 s
            char c = s.charAt(i); // 当前字符 c
            if (c == ' ') continue;
            if (Character.isDigit(c) || c == '-' && (i == 0 || !Character.isDigit(s.charAt(i - 1)))) { // c 为数字
                int end = i + 1;
                while (digits.indexOf(s.charAt(end)) >= 0) end++; // 寻找当前数值的尾字符
                numStk.push(Double.parseDouble(s.substring(i, end))); // 转换, 压栈
                i = end - 1;
            }
            else { // c 为运算符
                while (true) {
                    int rankOut = getRankOut(c); // 当前操作符的栈外优先级
                    int rankIn = !opStk.isEmpty() ? getRankIn(opStk.peek()) : -1; // 栈顶操作符的栈内优先级
                    // 比较两个优先级
                    if (rankIn < rankOut) {
                        opStk.push(c);
                        break; // 不再运算
                    }
                    else if (rankIn > rankOut) {
                        char op = opStk.pop();
                        if (op != '-' || numStk.size() >= 2) {
                            double b = numStk.pop(), a = numStk.pop();
                            numStk.push(cal(a, b, op));
                        }
                        else numStk.push(-numStk.pop());
                    }
                    else {
                        opStk.pop();
                        break; // 不再运算
                    }
                }
            }
        }
        return numStk.pop();
    }

    private double cal(double a, double b, char op) {
        if (op == '+') return a + b;
        if (op == '-') return a - b;
        if (op == '*') return a * b;
        if (op == '/') return a / b;
        if (op == '%') return a % b;
        if (op == '^') return Math.pow(a, b);
        if (op == '&') return Math.pow(a, 1 / b);
        return Integer.MIN_VALUE;
    }

    private int getRankOut(char op) {
        return rank[ops.indexOf(op)][1];
    }

    private int getRankIn(char op) {
        return rank[ops.indexOf(op)][0];
    }

    private void test() {
        List<String> tests = new ArrayList<>();
        List<Double> ans = new ArrayList<>();
        try (BufferedReader reader = new BufferedReader(new FileReader("tests.txt"))) {
            String line;
            while ((line = reader.readLine()) != null) {
                tests.add(line);
                line = reader.readLine();
                ans.add(Double.parseDouble(line));
            }
        }
        catch (IOException e) {
            e.printStackTrace();
        }
        System.out.println("----------开始测试----------");
        for (int i = 0; i < tests.size(); i++) {
            double myAns = solve(tests.get(i));
            boolean valid = Math.abs(myAns - ans.get(i)) < 0.0001;
            System.out.println("测试结果: " + valid + ", " + "测试用例: " + tests.get(i) + ", 计算结果=" + myAns + ", 正确答案=" + ans.get(i));
        }
        System.out.println("----------测试结束----------");
    }
}
