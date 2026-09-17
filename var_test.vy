class Main {
    void run() {
        var x = 10;
        {
            var y = 20;
            x = x + y;
        }
        print(x);
    }
}
