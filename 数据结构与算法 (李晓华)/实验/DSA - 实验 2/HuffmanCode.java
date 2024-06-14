import java.io.*;
import java.util.*;

class HuffmanCode {
    private String[] charCode; // 字符的 01 编码
    private int[] weight; // 字符的权重
    private StringBuilder path; // DFS 过程中的编码路径

    public static void main(String[] args) {
        HuffmanCode huffmanCode = new HuffmanCode();
        huffmanCode.run();
    }

    private void run() {
        Scanner in = new Scanner(System.in);
        String input;
        HuffmanTreeNode root = null;
        File srcFile = null, result, recon, ccOutput;
        boolean stop = false;
        while (!stop) {
            System.out.print("+——————————————+\n0 退出\n1 编码(基础)\n2 打印树(特色功能)\n3 压缩到文件(中级)\n4 从文件解码(高级)\n" +
                                     "+——————————————+\n选择操作:" + " ");
            input = in.nextLine();
            switch (input) {
                case "0": {
                    stop = true;
                    break;
                }
                case "1": {
                    // 读取文件
                    System.out.print("文本文件路径: ");
                    srcFile = new File(in.nextLine());
                    if (!srcFile.exists()) {
                        System.out.println("路径无效, 请检查!");
                        break;
                    }
                    // 统计字符频率, 建立哈夫曼树
                    int[] freq = stat(srcFile);
                    root = build(freq);
                    // 编码字符, 按格式打印
                    charCode = new String[128];
                    weight = new int[128];
                    path = new StringBuilder();
                    Arrays.fill(charCode, "");
                    getCode(root);
                    System.out.println("字符 | ASCII | 频率 | 编码");
                    for (int i = 0; i < 128; i++) {
                        if (!charCode[i].isEmpty()) {
                            if (i == 13)
                                System.out.println("'\\n'    13      " + weight[i] + "    " + charCode[i]); // 处理换行符 \n
                            else
                                System.out.println("'" + (char) i + "'     " + i + "      " + weight[i] + "    " + charCode[i]);
                        }
                    }
                    System.out.print("编码输出文件路径: ");
                    ccOutput = new File(in.nextLine());
                    writeCharCode(ccOutput);
                    break;
                }
                case "2": {
                    // 当宽度较小时, 打印哈夫曼树
                    if (root == null) {
                        System.out.println("未进行编码操作, 无法打印树!");
                        break;
                    }
                    PrintTree pt = new PrintTree();
                    List<List<String>> list = pt.printTree(root);
                    boolean p = list.get(0).size() <= 80;
                    if (!p) System.out.print("树过宽, 不建议打印, 真的要打印吗(y/n): ");
                    if (p || in.nextLine().equals("y")) {
                        for (List<String> row : list) {
                            StringBuilder r = new StringBuilder();
                            for (String s : row) r.append(s);
                            System.out.println(r);
                        }
                    }
                    break;
                }
                case "3": {
                    // 输出二进制文件
                    if (srcFile == null || !srcFile.exists()) {
                        System.out.println("未进行编码操作, 无法压缩到文件!");
                        break;
                    }
                    System.out.print("压缩文件路径(如 f1.huf): ");
                    result = new File(in.nextLine());
                    encode(srcFile, result);
                    String rate = String.format("%.3g", (double) srcFile.length() / result.length());
                    System.out.println("压缩完成, 压缩比: " + rate);
                    break;
                }
                case "4": {
                    // 解码二进制文件
                    if (srcFile == null || !srcFile.exists()) {
                        System.out.println("未进行编码操作, 无法从文件解码!");
                        break;
                    }
                    System.out.print("压缩文件路径: ");
                    result = new File(in.nextLine());
                    if (!result.exists()) {
                        System.out.println("路径无效, 请检查!");
                        break;
                    }
                    System.out.print("编码集文件路径: ");
                    ccOutput = new File(in.nextLine());
                    decodeCharCode(ccOutput);
                    System.out.print("重构文件路径: ");
                    recon = new File(in.nextLine());
                    System.out.print("选择解码依据(1 哈夫曼树 / 2 字符编码集): ");
                    switch (in.nextLine()) {
                        case "1": {
                            decode(result, recon, root);
                            System.out.println("解码完成");
                            break;
                        }
                        case "2": {
                            Map<String, Character> codes = new HashMap<>();
                            for (int i = 0; i < 128; i++) { // 预处理字符编码集到哈希表, 实现 O(1) 的检查
                                if (!charCode[i].isEmpty()) codes.put(charCode[i], (char) i);
                            }
                            decode(result, recon, codes);
                            System.out.println("解码完成");
                            break;
                        }
                        default:
                            System.out.println("重新输入!");
                    }
                    break;
                }
                default:
                    System.out.println("重新输入!");
            }
        }
    }

    // 统计字符频率
    private int[] stat(File file) {
        int[] freq = new int[128];
        try (BufferedReader reader = new BufferedReader(new FileReader(file))) {
            String line;
            while ((line = reader.readLine()) != null) {
                for (char c : line.toCharArray()) freq[c]++;
                freq[13]++;
            }
        }
        catch (IOException e) {
            throw new RuntimeException(e);
        }
        return freq;
    }

    // 建立哈夫曼树
    private HuffmanTreeNode build(int[] freq) {
        Queue<HuffmanTreeNode> pq = new PriorityQueue<>(Comparator.comparing(a -> a.weight)); // 优先队列, 权重小者优先
        for (int i = 0; i < 128; i++) {
            if (freq[i] > 0) pq.add(new HuffmanTreeNode((char) (i), freq[i]));
        }
        while (pq.size() > 1) { // 循环合并结点, 直到还剩最后一个
            HuffmanTreeNode n1 = pq.poll(), n2 = pq.poll();
            HuffmanTreeNode node = new HuffmanTreeNode(n1, n2);
            pq.add(node);
        }
        return pq.poll();
    }

    // DFS 遍历哈夫曼树, 编码字符
    private void getCode(HuffmanTreeNode root) {
        if (root == null) return; // 空结点
        if (root.left == null && root.right == null) { // 叶结点, 即字符结点
            charCode[root.c] = path.toString(); // 编码
            weight[root.c] = root.weight;
        }
        path.append('0'); // 当前编码添 '0'
        getCode(root.left); // DFS 左子树
        path.setCharAt(path.length() - 1, '1'); // 将 '0' 改为 '1'
        getCode(root.right); // DFS 右子树
        path.deleteCharAt(path.length() - 1); // 回退, 删除本次递归添加的尾码字
    }

    // 编码二进制文件
    private void encode(File input, File result) {
        byte[] buffer = new byte[1];
        int bitBuffer = 0; // 用于存储当前位
        int bitBufferIndex = 0; // 当前位的位置
        try (FileInputStream in = new FileInputStream(input);
             BufferedOutputStream out = new BufferedOutputStream(new FileOutputStream(result))) {
            int cur;
            while ((cur = in.read()) != -1) {
                String code = charCode[cur];
                for (char bit : code.toCharArray()) {
                    // 将编码字符串中的每一位按位地写入缓冲区
                    if (bit == '1') {
                        bitBuffer |= (1 << (7 - bitBufferIndex));
                    }
                    bitBufferIndex++;
                    // 当缓冲区满时, 将其写入输出流
                    if (bitBufferIndex == 8) {
                        buffer[0] = (byte) bitBuffer;
                        out.write(buffer);
                        bitBuffer = 0;
                        bitBufferIndex = 0;
                    }
                }
            }
            // 处理最后不足一字节的位
            if (bitBufferIndex > 0) {
                buffer[0] = (byte) bitBuffer;
                out.write(buffer);
            }
        }
        catch (IOException e) {
            e.printStackTrace();
        }
    }

    // 解码二进制文件, 依据哈夫曼树
    private void decode(File result, File recon, HuffmanTreeNode root) {
        try (FileInputStream in = new FileInputStream(result); FileOutputStream out = new FileOutputStream(recon)) {
            int bit;
            HuffmanTreeNode node = root; // 当前结点
            while ((bit = in.read()) != -1) {
                for (int i = 7; i >= 0; i--) {
                    int currentBit = (bit >> i) & 1; // 逐位读取压缩文件中的数据
                    // 根据读取的位移动至相应子结点
                    if (currentBit == 0) node = node.left; // 0, 向左子结点
                    else node = node.right; // 1, 向右子结点
                    if (node.c > 0) { // 成员字符存在，到达叶结点，解出一个字符
                        out.write(node.c); // 输出解出的字符
                        node = root; // 重置为根结点
                    }
                }
            }
        }
        catch (IOException e) {
            e.printStackTrace();
        }
    }

    // 解码二进制文件, 依据字符编码集
    private void decode(File result, File recon, Map<String, Character> codes) {
        try (FileInputStream in = new FileInputStream(result); FileOutputStream out = new FileOutputStream(recon)) {
            int bit;
            StringBuilder cur = new StringBuilder(); // 当前编码
            while ((bit = in.read()) != -1) {
                for (int i = 7; i >= 0; i--) {
                    // 逐位读取压缩文件中的数据
                    int currentBit = (bit >> i) & 1;
                    cur.append(currentBit);

                    String k = cur.toString(); // 当前编码序列
                    if (codes.containsKey(k)) { // 如果在编码表中找到匹配的编码
                        out.write(codes.get(k)); // 输出解码后的字符
                        cur = new StringBuilder(); // 重置编码序列
                    }
                }
            }
        }
        catch (IOException e) {
            e.printStackTrace();
        }
    }

    // 将编码集输出到 txt 文件
    private void writeCharCode(File ccOutput) {
        try (FileWriter writer = new FileWriter(ccOutput)) {
            for (int i = 0; i < 128; i++) {
                if (!charCode[i].isEmpty()) {
                    writer.write(i + " " + weight[i] + " " + charCode[i] + "\n");
                }
            }
        }
        catch (IOException e) {
            e.printStackTrace();
        }
    }

    // 从 txt 文件读取编码集
    private void decodeCharCode(File ccOutput) {
        Arrays.fill(charCode, "");
        List<String> cc = new LinkedList<>();
        try (BufferedReader reader = new BufferedReader(new FileReader(ccOutput))) {
            String line;
            while ((line = reader.readLine()) != null) cc.add(line);
        }
        catch (IOException e) {
            e.printStackTrace();
        }
        for (String s : cc) {
            String[] split = s.split(" ");
            charCode[Integer.parseInt(split[0])] = split[2];
        }
    }
}

// 哈夫曼树结点
class HuffmanTreeNode {
    int weight; // 权重
    char c; // 字符, 只有叶结点有字符
    HuffmanTreeNode left, right; // 左右子结点

    // 由字符和权重构造
    public HuffmanTreeNode(char c, int weight) {
        this.c = c;
        this.weight = weight;
    }

    // 由左右子结点构造
    public HuffmanTreeNode(HuffmanTreeNode left, HuffmanTreeNode right) {
        this.left = left;
        this.right = right;
        weight = left.weight + right.weight;
    }
}

// 打印哈夫曼树
class PrintTree {
    private List<List<String>> ans;
    private int height;

    public List<List<String>> printTree(HuffmanTreeNode root) {
        height = getHeight(root);
        ans = new ArrayList<>(height);
        int n = (1 << height) - 1;
        for (int i = 0; i < height; i++) {
            List<String> list = new ArrayList<>(n);
            for (int j = 0; j < n; j++) list.add(" ");
            ans.add(list);
        }
        dfs(root, 0, (n - 1) >> 1);
        return ans;
    }

    private int getHeight(HuffmanTreeNode node) {
        return node != null ? Math.max(getHeight(node.left), getHeight(node.right)) + 1 : 0;
    }

    private void dfs(HuffmanTreeNode node, int r, int c) {
        ans.get(r).set(c, node.weight + "");
        int diff = 1 << height - 2 - r;
        if (node.left != null) dfs(node.left, r + 1, c - diff);
        if (node.right != null) dfs(node.right, r + 1, c + diff);
    }
}
