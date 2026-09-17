class Math {
    int add(int a, int b) {
        return a + b;
    }

    int subtract(int a, int b) {
        return a - b;
    }
}

class Main {
    void run() {
        var m = new Math();
        var r1 = m.add(10, 5);
        var r2 = m.subtract(10, 5);
        print(r1); // 15
        print(r2); // 5
    }
}
